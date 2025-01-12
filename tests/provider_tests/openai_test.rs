use std::{ptr::read, sync::Arc, time::Duration};

use futures::future::join_all;
use kairei::{
    config::{self, ProviderConfig, SecretConfig, SystemConfig},
    event_bus::{Event, Value},
    parse_root,
    system::System,
};
use tokio::time::sleep;
use tracing::debug;
use uuid::Uuid;

use crate::should_run_external_api_tests;

const TEST_CONFIG_PATH: &str = "tests/provider_tests/test_config.json";
const TEST_SECRET_PATH: &str = "tests/provider_tests/test_secret.json";

const PROVIDER_NAME: &str = "openai_travel_expert";

// テスト用のヘルパー関数
async fn setup_openai_provider() -> (SystemConfig, SecretConfig) {
    let config: SystemConfig = config::from_file(TEST_CONFIG_PATH).unwrap();
    let secret_config: SecretConfig = config::from_file(TEST_SECRET_PATH).unwrap();

    (config, secret_config)
}

async fn setup_system(
    config: SystemConfig,
    secret: SecretConfig,
    uncheck: bool,
) -> (System, ProviderConfig, SecretConfig) {
    let provider_config = config
        .provider_configs
        .providers
        .get(PROVIDER_NAME)
        .unwrap();
    debug!("{:?}", provider_config);

    // テスト用のDSL
    let dsl = r#"
        micro TravelAgent {
            answer {
                on request PlanTrip() -> Result<String, Error> {
                    return think("Tokyo")
                }
            }
        }
    "#;
    let root = parse_root(dsl).unwrap().1;
    let mut system = System::new(&config, &secret).await;
    sleep(Duration::from_secs(1)).await;

    let ret = system.initialize(root).await;
    if !uncheck {
        ret.unwrap();
    }
    sleep(Duration::from_millis(100)).await;
    (system, provider_config.clone(), secret)
}

fn create_request(request_id: &Uuid, requests: Vec<(&str, &str)>, timeout: Option<u64>) -> Event {
    let mut builder = Event::request_buidler()
        .request_type("PlanTrip")
        .requester("test")
        .responder("TravelAgent")
        .request_id(request_id.to_string().as_str());

    for request in requests.clone() {
        builder = builder
            .clone()
            .parameter(request.0, &Value::String(request.1.to_string()));
    }
    if let Some(timeout) = timeout {
        builder = builder.parameter("timeout", &Value::Duration(Duration::from_secs(timeout)));
    }

    builder.build().unwrap()
}

#[test]
fn test_create_request() {
    let request_id = uuid::Uuid::new_v4();
    let request = create_request(
        &request_id,
        vec![("destination", "Tokyo"), ("1", "test")],
        None,
    );

    assert_eq!(request.parameters.len(), 2);
}

#[tokio::test]
async fn test_openai_assistant_basic_interaction() {
    if !should_run_external_api_tests() {
        return;
    }

    let (system_config, secret_config) = setup_openai_provider().await;

    let (system, provider_config, _) = setup_system(system_config, secret_config, false).await;

    let provider = system
        .get_provider(PROVIDER_NAME)
        .await
        .unwrap()
        .provider
        .clone();

    let thread_id = provider.create_thread().await.unwrap();

    // 基本的な質問でテスト
    let response = provider
        .send_message(
            &thread_id,
            provider_config
                .provider_specific
                .get("assistant_id")
                .unwrap()
                .as_str()
                .unwrap(),
            "What is the capital of France?",
        )
        .await
        .unwrap();

    let _ = provider.delete_thread(&thread_id).await;

    assert!(response.contains("Paris"));
}

#[tokio::test]
async fn test_openai_assistant_error_handling() {
    if !should_run_external_api_tests() {
        return;
    }

    let (mut system_config, secret_config) = setup_openai_provider().await;

    // 意図的に無効な設定を作成
    if let Some(provider_config) = system_config
        .provider_configs
        .providers
        .get_mut(PROVIDER_NAME)
    {
        provider_config.common_config.model = "invalid-model".to_string();
    }

    let (system, _, _) = setup_system(system_config, secret_config, true).await;

    let provider = system.get_provider(PROVIDER_NAME).await;
    assert!(provider.is_err());
}

#[tokio::test]
async fn test_travel_agent_with_assistant() {
    if !should_run_external_api_tests() {
        return;
    }

    let (system_config, secret_config) = setup_openai_provider().await;
    let (system, _, _) = setup_system(system_config, secret_config, false).await;

    // Start system
    system.start().await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let request_id = uuid::Uuid::new_v4();

    let request = create_request(&request_id, vec![("destination", "Tokyo")], None);

    let response = system.send_request(request).await.unwrap();

    if let Value::String(content) = response {
        debug!("response: {:?}", content);
        assert!(content.contains("Tokyo"));
    } else {
        panic!("Invalid response type");
    }
}

#[tokio::test]
async fn test_travel_agent_detailed_request() {
    if !should_run_external_api_tests() {
        return;
    }

    let (system_config, secret_config) = setup_openai_provider().await;
    let (system, _, _) = setup_system(system_config, secret_config, false).await;

    // Start system
    system.start().await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let request_id = uuid::Uuid::new_v4();

    let request = create_request(
        &request_id,
        vec![
            (
                "1",
                "I want to visit Tokyo for 3 days focusing on traditional culture and temples",
            ),
            ("destination", "Tokyo"),
        ],
        None,
    );

    let response = system.send_request(request).await.unwrap();

    if let Value::String(content) = response {
        debug!("response: {:?}", content);
        let content = content.to_lowercase();
        assert!(content.contains("temple"));
        assert!(content.contains("traditional"));
    } else {
        panic!("Invalid response type");
    }
}

#[tokio::test]
async fn test_travel_agent_concurrent_requests() {
    if !should_run_external_api_tests() {
        return;
    }

    let (system_config, secret_config) = setup_openai_provider().await;
    let (system, _, _) = setup_system(system_config, secret_config, false).await;

    // Start system
    system.start().await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let requests = vec![
        "Tell me about Tokyo",
        "What to see in Kyoto",
        "Recommend places in Osaka",
    ];

    let system_arc = Arc::new(system);
    // 複数リクエストを同時実行
    let handles: Vec<_> = requests
        .into_iter()
        .map(|req| {
            let system = system_arc.clone();
            tokio::spawn(async move {
                let request_id = uuid::Uuid::new_v4();
                let request = create_request(&request_id, vec![("1", req)], Some(60));
                system.send_request(request).await
            })
        })
        .collect();

    let results = join_all(handles).await;
    for result in results {
        let response = result.unwrap().unwrap();
        if let Value::String(content) = response {
            assert!(!content.is_empty());
        } else {
            panic!("Invalid response type");
        }
    }
}

#[tokio::test]
async fn test_travel_agent_timeout() {
    if !should_run_external_api_tests() {
        return;
    }

    let (system_config, secret_config) = setup_openai_provider().await;
    let (system, _, _) = setup_system(system_config, secret_config, false).await;

    // Start system
    system.start().await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let request_id = uuid::Uuid::new_v4();

    let request = create_request(
        &request_id,
        vec![
            (
                "1",
                "I want to visit Tokyo for 3 days focusing on traditional culture and temples",
            ),
            ("destination", "Tokyo"),
        ],
        Some(1),
    );

    let response = system.send_request(request).await;
    debug!("{:?}", response);
    assert!(response.is_err());
}

#[tokio::test]
async fn test_join_all_behavior() {
    use futures::future::join_all;
    use std::time::Instant;

    // 異なる待ち時間を持つタスクを生成
    let tasks: Vec<_> = vec![100, 50, 150, 120, 80, 140, 100, 100, 100, 100] // ミリ秒
        .into_iter()
        .map(|delay| {
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                delay
            })
        })
        .collect();

    // 開始時間を記録
    let start = Instant::now();

    // 全タスクを並列実行
    let results = join_all(tasks).await;

    // 完了までの時間を計測
    let elapsed = start.elapsed();

    // 結果の確認
    let delays: Vec<u64> = results.into_iter().map(|r| r.unwrap()).collect();

    debug!("Tasks completed in {:?}", elapsed);
    debug!("Results: {:?}", delays);

    // 最も長い遅延（150ms）より少し長い時間で全て完了しているはず
    assert!(elapsed.as_millis() >= 150);
    // でも、全タスクの合計時間（300ms）より大幅に短いはず
    assert!(elapsed.as_millis() < 250);
}

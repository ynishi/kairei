/*
use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::future::join_all;
use kairei_core::{
    config::{self, ProviderConfig, SecretConfig, SystemConfig},
    context::AgentInfo,
    event_bus::{Event, Value},
    expression,
    provider::{
        llm::ProviderLLM,
        llms::{openai_assistant::OpenAIAssistantProviderLLM, openai_chat::OpenAIChatProviderLLM},
        provider::{Provider, ProviderSecret},
        providers::standard::StandardProvider,
        request::{ExecutionState, ProviderContext, ProviderRequest, RequestInput},
    },
    system::System,
    timestamp::Timestamp,
};
use tokio::time::sleep;
use tracing::debug;
use uuid::Uuid;

use crate::should_run_external_api_tests;

use super::setup_config;

const PROVIDER_NAME: &str = "openai_travel_expert";

// テスト用のヘルパー関数
async fn setup_openai_provider() -> (SystemConfig, SecretConfig) {
    setup_config()
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
    let mut builder = Event::request_builder()
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

fn create_test_request(
    query: &str,
    agent_name: &str,
    session_id: Option<String>,
) -> ProviderRequest {
    ProviderRequest {
        input: RequestInput {
            query: expression::Value::String(query.to_string()),
            parameters: HashMap::new(),
        },
        state: ExecutionState {
            session_id: session_id.unwrap_or_else(|| "test-session".to_string()),
            timestamp: Timestamp::now(),
            agent_name: agent_name.to_string(),
            agent_info: AgentInfo::default(),
            policies: vec![],
            trace_id: "test-trace".to_string(),
        },
        config: ProviderConfig::default(),
    }
}

async fn setup_standard_provider(provider_name: &str) -> (StandardProvider, ProviderContext) {
    let (system_config, secret_config) = setup_openai_provider().await;

    let (config, secret) = setup_openai_provider().await;
    let mut llm = OpenAIAssistantProviderLLM::new(provider_name);

    // 初期化
    llm.initialize(
        &config
            .provider_configs
            .providers
            .get(provider_name)
            .unwrap(),
        &kairei_core::provider::provider::ProviderSecret::from(
            secret.providers.get(provider_name).unwrap().clone(),
        ),
    )
    .await
    .unwrap();

    // StandardProviderの構築
    let provider = StandardProvider::new(llm, vec![]);

    let context = ProviderContext {
        config: system_config
            .provider_configs
            .providers
            .get(provider_name)
            .unwrap()
            .clone(),
        secret: ProviderSecret::from(secret_config.providers.get(provider_name).unwrap().clone()),
    };

    (provider, context)
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
async fn test_openai_chat_provider() {
    if !should_run_external_api_tests() {
        return;
    }

    let (config, secret) = setup_openai_provider().await;
    let mut provider = OpenAIChatProviderLLM::new("test");

    // 初期化
    provider
        .initialize(
            &config
                .provider_configs
                .providers
                .get("openai_chat_4mini")
                .unwrap(),
            &kairei_core::provider::provider::ProviderSecret::from(
                secret.providers.get("openai_chat_4mini").unwrap().clone(),
            ),
        )
        .await
        .unwrap();

    // メッセージ送信テスト
    let response = provider
        .send_message(
            "Test message",
            &config
                .provider_configs
                .providers
                .get("openai_chat_4mini")
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(!response.content.is_empty());
}

#[tokio::test]
async fn test_assistant_provider() {
    if !should_run_external_api_tests() {
        return;
    }

    let (config, secret) = setup_openai_provider().await;
    let mut provider = OpenAIAssistantProviderLLM::new("openai_travel_expert");

    // 初期化
    provider
        .initialize(
            &config
                .provider_configs
                .providers
                .get("openai_travel_expert")
                .unwrap(),
            &kairei_core::provider::provider::ProviderSecret::from(
                secret
                    .providers
                    .get("openai_travel_expert")
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .unwrap();

    // メッセージ送信テスト
    let response = provider
        .send_message(
            "What is the capital of France?",
            &config
                .provider_configs
                .providers
                .get("openai_travel_expert")
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(!response.content.is_empty());

    // クリーンアップ
    provider.stop().await.unwrap();
}

#[tokio::test]
async fn test_openai_assistant_basic_interaction() {
    if !should_run_external_api_tests() {
        return;
    }

    let (mut provider, context) = setup_standard_provider(PROVIDER_NAME).await;

    provider
        .initialize(&context.config, &context.secret)
        .await
        .unwrap();

    // リクエストの作成と実行
    let request = create_test_request("What is the capital of France?", "test-agent", None);

    let response = provider.execute(&context, &request).await.unwrap();

    // レスポンスの検証
    let content = response.output;
    assert!(content.contains("Paris"));

    // クリーンアップ
    provider.shutdown().await.unwrap();
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

    if let Value::Map(content_map) = response {
        if let Some(Value::String(s)) = content_map.get("output") {
            debug!("response: {:?}", s);
            assert!(s.contains("Tokyo"));
        } else {
            panic!("Output content not found");
        }
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

    if let Value::Map(content_map) = response {
        if let Some(Value::String(s)) = content_map.get("output") {
            debug!("response: {:?}", s);
            let content = s.to_lowercase();
            assert!(content.contains("temple"));
            assert!(content.contains("traditional"));
        } else {
            panic!("Output content not found");
        }
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
        if let Value::Map(content_map) = response {
            if let Some(Value::String(s)) = content_map.get("output") {
                assert!(!s.is_empty());
            } else {
                panic!("Output content not found");
            }
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
*/

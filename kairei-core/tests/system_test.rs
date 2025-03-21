use std::{collections::HashMap, time::Duration};

use kairei_core::analyzer::Parser;
use kairei_core::config::{ProviderConfig, ProviderConfigs, ProviderSecretConfig, SecretConfig};
use kairei_core::preprocessor::Preprocessor;
use kairei_core::provider::provider::ProviderType;
use kairei_core::system::SystemResult;
use kairei_core::tokenizer::token::Token;
use kairei_core::type_checker::run_type_checker;
use kairei_core::{
    MicroAgentDef, config::SystemConfig, event_bus::Event, event_registry::EventType,
    system::System,
};
use tokio::{self, time::sleep};
use tracing::debug;

pub fn setup_non_api_config() -> (SystemConfig, SecretConfig) {
    let default_name = "default";
    let mut system_config = SystemConfig::default();
    let provider_configs = ProviderConfigs {
        primary_provider: Some(default_name.to_string()),
        providers: {
            let mut map = HashMap::new();
            map.insert(
                default_name.to_string(),
                ProviderConfig {
                    name: default_name.to_string(),
                    provider_type: ProviderType::SimpleExpert,
                    provider_specific: {
                        let mut map = HashMap::new();
                        map.insert(
                            "type".to_string(),
                            serde_json::Value::String("simple_expert".to_string()),
                        );
                        map
                    },
                    ..Default::default()
                },
            );
            map
        },
    };
    system_config.provider_configs = provider_configs;

    let mut secret_config = SecretConfig::default();
    secret_config
        .providers
        .insert(default_name.to_string(), ProviderSecretConfig::default());
    (system_config, secret_config)
}

#[tokio::test]
async fn test_system_lifecycle() -> SystemResult<()> {
    let (system_config, secret_config) = setup_non_api_config();
    // システムの初期化
    let mut system = System::new(&system_config, &secret_config).await;

    let root = system
        .parse_dsl(
            r#"
            micro TestAgent {
                answer {
                    on request Test() -> Result<String, Error> {
                        return Ok("test")
                    }
                }
            }
        "#,
        )
        .await?;
    system.initialize(root).await?;

    let agent_name = "test-agent";

    // AST登録
    let test_ast = MicroAgentDef {
        name: agent_name.to_string(),
        ..Default::default()
    };
    system.register_agent_ast(agent_name, &test_ast).await?;

    // スケールアップテスト
    //
    let initial_status = system.get_system_status().await?;
    let agents = system.scale_up(agent_name, 3, HashMap::new()).await?;
    debug!("Agents: {:?}", agents);
    assert_eq!(agents.len(), 3);
    assert!(agents.iter().all(|name| name.starts_with(agent_name)));

    // エージェントの起動確認
    for agent in &agents {
        system.start_agent(agent).await?;
    }

    // システム状態の確認
    let status = system.get_system_status().await?;
    assert_eq!(status.agent_count, 3 + initial_status.agent_count);
    assert_eq!(
        status.running_agent_count,
        3 + initial_status.running_agent_count
    );
    assert!(status.running);

    // イベントの送信テスト
    let test_event = Event {
        event_type: EventType::Custom("test-event".to_string()),
        ..Default::default()
    };
    system.send_event(test_event).await?;

    // イベントの購読テスト
    let _ = system
        .subscribe_events(vec![EventType::Custom("test-event".to_string())])
        .await?;

    // スケールダウンテスト
    system.scale_down(agent_name, 2, HashMap::new()).await?;

    sleep(Duration::from_millis(100)).await;

    // 残りのエージェント数確認
    let status = system.get_system_status().await?;
    assert_eq!(status.agent_count, 3 + initial_status.agent_count);
    assert_eq!(
        status.running_agent_count,
        1 + initial_status.running_agent_count
    );

    // 特定のエージェントの状態確認
    let agent_status = system.get_agent_status(&agents[0]).await?;
    assert_eq!(agent_status.name, agents[0]);

    Ok(())
}

#[tokio::test]
async fn test_event_handling() -> SystemResult<()> {
    let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;

    // イベント送受信のテスト
    let test_event = Event {
        event_type: EventType::Custom("test-event".to_string()),
        ..Default::default()
    };

    // イベントの購読設定
    let mut events = system
        .subscribe_events(vec![EventType::Custom("test-event".to_string())])
        .await?;

    // イベント送信
    system.send_event(test_event.clone()).await?;

    sleep(Duration::from_millis(100)).await;

    let received = events.recv().await.expect("No event received");
    assert_eq!(
        received.event_type,
        EventType::Custom("test-event".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> SystemResult<()> {
    let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;

    // 存在しないエージェントへのアクセス
    let result = system.get_agent_status("non-existent").await;
    assert!(result.is_err());

    // 存在しないテンプレートでのスケールアップ
    let result = system.scale_up("non-existent", 1, HashMap::new()).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_request_response() -> SystemResult<()> {
    let (system_config, secret_config) = setup_non_api_config();
    let mut system = System::new(&system_config, &secret_config).await;
    debug!("System created {:?}", SystemConfig::default());

    let test_agent_dsl = r#"
        micro Responder {
            answer {
                on request GetCount() -> Result<Int, Error> {
                    return Ok(1)
                }
            }
        }
    "#;
    let result = kairei_core::tokenizer::token::Tokenizer::new()
        .tokenize(test_agent_dsl)
        .unwrap();
    let preprocessor = kairei_core::preprocessor::TokenPreprocessor::default();
    let tokens: Vec<Token> = preprocessor
        .process(result)
        .iter()
        .map(|e| e.token.clone())
        .collect();
    let (_, mut ast) = kairei_core::analyzer::parsers::world::parse_root()
        .parse(tokens.as_slice(), 0)
        .unwrap();

    run_type_checker(&mut ast).unwrap();

    system.initialize(ast).await?;
    let (_, test_ast) = kairei_core::analyzer::parsers::agent::parse_agent_def()
        .parse(tokens.as_slice(), 0)
        .unwrap();

    system.register_agent_ast("Responder", &test_ast).await?;

    let agent_name = system.scale_up("Responder", 1, HashMap::new()).await?[0].clone();
    system.start_agent(&agent_name).await?;

    sleep(Duration::from_millis(100)).await;

    Ok(())
}

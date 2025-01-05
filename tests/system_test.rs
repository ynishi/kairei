use std::{collections::HashMap, time::Duration};

use kairei::{
    error::RuntimeError,
    event_bus::{Event, Value},
    event_registry::EventType,
    parse_micro_agent,
    system::System,
    AnswerDef, ExecutionError, MicroAgentDef, RequestHandler, RuntimeResult, StateDef,
};
use tokio::{self, time::sleep};
use uuid::Uuid;

#[tokio::test]
async fn test_system_lifecycle() -> RuntimeResult<()> {
    // システムの初期化
    let system = System::new(100).await;

    let agent_name = "test-agent";

    // AST登録
    let test_ast = MicroAgentDef {
        name: agent_name.to_string(),
        ..Default::default()
    };
    system.register_agent_ast(&agent_name, &test_ast).await?;

    // スケールアップテスト
    let agents = system.scale_up(agent_name, 3, HashMap::new()).await?;
    assert_eq!(agents.len(), 3);
    assert!(agents.iter().all(|name| name.starts_with(agent_name)));

    // エージェントの起動確認
    for agent in &agents {
        system.start_agent(agent).await?;
    }

    // システム状態の確認
    let status = system.get_system_status().await?;
    assert_eq!(status.agent_count, 3);
    assert_eq!(status.running_agent_count, 3);
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
    assert_eq!(status.agent_count, 3);
    assert_eq!(status.running_agent_count, 1);

    // 特定のエージェントの状態確認
    let agent_status = system.get_agent_status(&agents[0]).await?;
    assert_eq!(agent_status.name, agents[0]);

    Ok(())
}

#[tokio::test]
async fn test_event_handling() -> RuntimeResult<()> {
    let system = System::new(100).await;

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
async fn test_error_handling() -> RuntimeResult<()> {
    let system = System::new(100).await;

    // 存在しないエージェントへのアクセス
    let result = system.get_agent_status("non-existent").await;
    assert!(matches!(
        result,
        Err(RuntimeError::Execution(
            ExecutionError::AgentNotFound { .. }
        ))
    ));

    // 存在しないテンプレートでのスケールアップ
    let result = system.scale_up("non-existent", 1, HashMap::new()).await;
    assert!(matches!(
        result,
        Err(RuntimeError::Execution(ExecutionError::ASTNotFound(_)))
    ));

    panic!("abc");
    Ok(())
}

#[tokio::test]
async fn test_request_response() -> RuntimeResult<()> {
    let system = System::new(100).await;

    let test_agent_dsl = r#"
        micro Responder {
            answer {
                on request GetCount() -> Result<Int, Error> {
                    return Ok(1)
                }
            }
        }
    "#;
    let test_ast = parse_micro_agent(test_agent_dsl).unwrap().1;

    system.register_agent_ast("Responder", &test_ast).await?;

    let agent_name = system.scale_up("Responder", 1, HashMap::new()).await?[0].clone();
    system.start_agent(&agent_name).await?;

    sleep(Duration::from_millis(100)).await;

    let request_id = Uuid::new_v4().to_string();

    // リクエスト送信
    let request = Event {
        event_type: EventType::Request {
            request_type: "GetName".to_string(),
            requester: "test_agent".to_string(),
            responder: "Responder".to_string(),
            request_id: request_id.clone(),
        },
        parameters: HashMap::new(),
    };

    /*
    // レスポンス待機
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        system.send_request(request),
    )
    .await
    .expect("Timeout waiting for response")?;

    assert_eq!(response, Value::String("Responder".to_string()));
    */

    Ok(())
}

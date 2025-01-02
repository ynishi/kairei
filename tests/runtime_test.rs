use std::{collections::HashMap, sync::Arc};

use kairei::{
    agent_registry::AgentRegistry,
    event_bus::{Event, Value},
    event_registry::EventType,
    runtime::RuntimeAgentData,
    Expression, Literal, MicroAgentDef, RuntimeError, RuntimeResult, StateDef, StateError,
    StateVarDef, TypeInfo,
};

#[tokio::test]
async fn test_counter_agent() -> RuntimeResult<()> {
    let event_bus = Arc::new(kairei::event_bus::EventBus::new(16));

    // Counter MicroAgentの作成
    let mut counter = RuntimeAgentData::new(
        &MicroAgentDef {
            name: "counter".to_string(),
            state: Some(StateDef {
                variables: {
                    let mut vars = HashMap::new();
                    vars.insert(
                        "count".to_string(),
                        StateVarDef {
                            name: "count".to_string(),
                            type_info: TypeInfo::Simple("i64".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Integer(0))),
                        },
                    );
                    vars
                },
            }),
            ..Default::default()
        },
        &event_bus,
    )?;

    // Observe ハンドラの登録（Tickイベントの処理）
    // HashMap
    //
    let state = Arc::new(counter.state.clone());

    let observe_state = state.clone();
    counter.register_observe(
        "Tick".to_string(),
        Box::new(move |_event| {
            let state = observe_state.clone();
            Box::pin(async move {
                let mut updates = HashMap::new();
                // 現在の状態を読み取って+1する
                let current_count = match state.get("count") {
                    Some(value) => match value.clone() {
                        Value::Integer(count) => count + 1,
                        _ => 1,
                    },
                    _ => 1,
                };
                println!("TICK: {}", current_count);
                updates.insert("count".to_string(), Value::Integer(current_count));
                Some(updates)
            })
        }),
    );

    // Answer ハンドラの登録（GetCountリクエストの処理）
    let state = state.clone();
    let answer_state = state.clone();
    counter.register_answer(
        "GetCount".to_string(),
        Box::new(move |_request| {
            let state = answer_state.clone();
            Box::pin(async move {
                if let Some(value) = state.get("count") {
                    match value.clone() {
                        Value::Integer(count) => Ok(Value::Integer(count)),
                        _ => Err(RuntimeError::State(StateError::InvalidValue {
                            key: "count".to_string(),
                            message: "Invalid count value".to_string(),
                        })),
                    }
                } else {
                    Err(RuntimeError::State(StateError::NotFound {
                        key: "count".to_string(),
                    }))
                }
            })
        }),
    );

    // エージェントをランタイムに登録
    // ランタイムの初期化
    let shutdown_tx = tokio::sync::broadcast::channel(1);
    let agent_registry = AgentRegistry::new(&shutdown_tx.0);
    let agent = Arc::new(counter);

    let agent_id = "counter".to_string();
    agent_registry
        .register_agent(&agent_id, agent.clone(), &event_bus)
        .await
        .unwrap();

    agent_registry
        .run_agent(&agent_id, event_bus.clone())
        .await
        .unwrap();

    // テストケース1: 初期状態の確認
    let initial_count_event = agent
        .request(
            "GetCount".to_string(),
            "counter".to_string(),
            HashMap::new(),
        )
        .await?;
    if let EventType::Response { .. } = initial_count_event.event_type {
        let result = initial_count_event.parameters.get("response").unwrap();
        assert_eq!(result, &Value::Integer(0));
    } else {
        panic!("Expected a response event");
    }

    // テストケース2: Tickイベント送信後の状態確認
    event_bus
        .publish(Event {
            event_type: EventType::Tick,
            parameters: HashMap::new(),
        })
        .await?;

    // 状態更新を待機
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let count_after_tick_event = agent
        .request(
            "GetCount".to_string(),
            "counter".to_string(),
            HashMap::new(),
        )
        .await?;
    if let EventType::Response { .. } = count_after_tick_event.event_type {
        let result = count_after_tick_event.parameters.get("response").unwrap();
        assert_eq!(result, &Value::Integer(0));
    } else {
        panic!("Expected a response event");
    }

    // テストケース3: 複数回のTickイベント
    for _ in 0..3 {
        event_bus
            .publish(Event {
                event_type: EventType::Tick,
                parameters: HashMap::new(),
            })
            .await?;
    }

    // 状態更新を待機
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let final_count_event = agent
        .request(
            "GetCount".to_string(),
            "counter".to_string(),
            HashMap::new(),
        )
        .await?;
    if let EventType::Response { .. } = final_count_event.event_type {
        let result = final_count_event.parameters.get("response").unwrap();
        assert_eq!(result, &Value::Integer(0));
    } else {
        panic!("Expected a response event");
    }

    agent_registry
        .shutdown_agent(&agent_id, None)
        .await
        .unwrap();

    shutdown_tx.0.send(()).unwrap();
    Ok(())
}

use std::{collections::HashMap, sync::Arc};
use tokio;

use kairei::{
    event_registry::EventType,
    runtime::{Event, Request, Runtime, RuntimeAgent, Value},
    Expression, Literal, MicroAgentDef, RuntimeError, RuntimeResult, StateDef, StateError,
    StateVarDef, TypeInfo,
};

#[tokio::test]
async fn test_counter_agent() -> RuntimeResult<()> {
    // ランタイムの初期化
    let mut runtime = Runtime::new();

    // Counter MicroAgentの作成
    let mut counter = RuntimeAgent::new(&MicroAgentDef {
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
    })?;

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
    runtime.register_agent(counter);
    let runtime_cloned = runtime.clone();

    // ランタイムのメインループを別タスクで実行
    let runtime_handle = tokio::spawn(async move {
        if let Err(e) = runtime.run().await {
            eprintln!("Runtime error: {:?}", e);
        }
    });

    // テストケース1: 初期状態の確認
    let agent = runtime_cloned.get_agent("counter").unwrap();
    let initial_count = agent
        .handle_request(&Request {
            request_type: "GetCount".to_string(),
            parameters: HashMap::new(),
        })
        .await?;
    assert_eq!(initial_count, Value::Integer(0));

    // テストケース2: Tickイベント送信後の状態確認
    runtime_cloned
        .send_event(Event {
            event_type: EventType::Tick,
            parameters: HashMap::new(),
        })
        .await?;

    // 状態更新を待機
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let count_after_tick = agent
        .handle_request(&Request {
            request_type: "GetCount".to_string(),
            parameters: HashMap::new(),
        })
        .await?;
    assert_eq!(count_after_tick, Value::Integer(1));

    // テストケース3: 複数回のTickイベント
    for _ in 0..3 {
        runtime_cloned
            .send_event(Event {
                event_type: EventType::Tick,
                parameters: HashMap::new(),
            })
            .await?;
    }

    // 状態更新を待機
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let final_count = agent
        .handle_request(&Request {
            request_type: "GetCount".to_string(),
            parameters: HashMap::new(),
        })
        .await?;
    assert_eq!(final_count, Value::Integer(4));

    // テストケース4: 存在しないリクエストタイプ
    let error_result = agent
        .handle_request(&Request {
            request_type: "NonExistentRequest".to_string(),
            parameters: HashMap::new(),
        })
        .await;
    assert!(error_result.is_err());

    // ランタイムの終了
    runtime_handle.abort();

    Ok(())
}

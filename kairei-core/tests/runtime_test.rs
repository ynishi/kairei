/*

// テスト用のヘルパーAgent
struct TestAgent {
    pub name: String,
    pub responses: Arc<Mutex<Vec<Event>>>,
}

impl TestAgent {
    fn new(name: &str, event_bus: &Arc<EventBus>) -> Self {
        let (mut event_rx, _) = event_bus.subscribe();
        let responses = Arc::new(Mutex::new(vec![]));
        let response_ref = responses.clone();
        // 非同期にイベントを取得して、responsesに格納する処理を開始
        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                response_ref.lock().unwrap().push(event);
            }
        });
        Self {
            name: name.to_string(),
            responses,
        }
    }

    fn get_response(&self, request_id: &str) -> Value {
        let want_request_id = request_id.to_string();
        let lock = self.responses.lock().unwrap();
        let filtered = lock.iter().filter(|e| match &e.event_type {
            EventType::ResponseSuccess { request_id, .. } => request_id == &want_request_id,
            _ => false,
        });
        let res = filtered.last().unwrap();
        res.parameters.get("response").unwrap().clone()
    }
}
*/

/*
#[tokio::test]
async fn test_counter_agent() -> RuntimeResult<()> {
    let event_bus = Arc::new(kairei_core::event_bus::EventBus::new(16));

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
    )
    .await?;

    // Observe ハンドラの登録（Tickイベントの処理）

    let observe_state = state.clone();
    counter.register_observe(
        "Tick",
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
                Ok(())
            })
        }),
    );

    // Answer ハンドラの登録（GetCountリクエストの処理）
    let state = state.clone();
    let answer_state = state.clone();
    counter.register_answer(
        "GetCount",
        Box::new(move |_request| {
            let state = answer_state.clone();
            Box::pin(async move {
                if let Some(value) = state.get("count") {
                    match value.clone() {
                        Value::Integer(count) => Ok(()),
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

    sleep(Duration::from_millis(100)).await;

    // テストエージェントのセットアップ
    let test_agent = TestAgent::new("test_agent", &event_bus);

    // テストケース1: 初期状態の確認
    let request_id = Uuid::new_v4().to_string();
    event_bus
        .publish(Event {
            event_type: EventType::Request {
                request_type: "GetCount".to_string(),
                requester: test_agent.name.clone(),
                responder: "counter".to_string(),
                request_id: request_id.clone(),
            },
            parameters: HashMap::new(),
        })
        .await?;

    // レスポンスの待機と確認
    sleep(Duration::from_millis(100)).await;

    let count = test_agent.get_response(&request_id);
    assert_eq!(count, Value::Integer(0));

    // テストケース2: Tickイベント送信後の状態確認
    event_bus
        .publish(Event {
            event_type: EventType::Tick,
            parameters: HashMap::new(),
        })
        .await?;

    // 状態更新を待機
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let request_id = Uuid::new_v4().to_string();
    event_bus
        .publish(Event {
            event_type: EventType::Request {
                request_type: "GetCount".to_string(),
                requester: test_agent.name.clone(),
                responder: "counter".to_string(),
                request_id: request_id.clone(),
            },
            parameters: HashMap::new(),
        })
        .await?;

    // レスポンスの待機と確認
    sleep(Duration::from_millis(100)).await;

    let count = test_agent.get_response(&request_id);
    assert_eq!(count, Value::Integer(1));

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

    let request_id = Uuid::new_v4().to_string();
    event_bus
        .publish(Event {
            event_type: EventType::Request {
                request_type: "GetCount".to_string(),
                requester: test_agent.name.clone(),
                responder: "counter".to_string(),
                request_id: request_id.clone(),
            },
            parameters: HashMap::new(),
        })
        .await?;

    // レスポンスの待機と確認
    sleep(Duration::from_millis(100)).await;

    let count = test_agent.get_response(&request_id);
    assert_eq!(count, Value::Integer(4));

    agent_registry
        .shutdown_agent(&agent_id, None)
        .await
        .unwrap();

    shutdown_tx.0.send(()).unwrap();
    Ok(())
}
*/

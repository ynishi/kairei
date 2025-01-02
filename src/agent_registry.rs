use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::timeout;

use crate::event_bus::{ErrorEvent, Event, EventBus, Value};
use crate::event_registry::EventType;
use crate::runtime::RuntimeAgent;
use crate::{ExecutionError, RuntimeError, RuntimeResult};

pub struct AgentRegistry {
    agents: Arc<DashMap<String, Arc<dyn RuntimeAgent>>>,
    running_agents: Arc<DashMap<String, tokio::task::JoinHandle<()>>>,
    shutdown_tx: broadcast::Sender<()>, // Systemから渡される
}

impl Clone for AgentRegistry {
    fn clone(&self) -> Self {
        Self {
            agents: self.agents.clone(),
            running_agents: self.running_agents.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }
}

impl AgentRegistry {
    pub fn new(shutdown_tx: &broadcast::Sender<()>) -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            running_agents: Arc::new(DashMap::new()),
            shutdown_tx: shutdown_tx.clone(), // Systemから渡される
        }
    }
    pub async fn run(&self) -> RuntimeResult<()> {
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        loop {
            tokio::select! {
                Ok(_) = shutdown_rx.recv() => {
                    tracing::info!("AgentRegistry received shutdown signal");
                    // shutdown_all を呼び出して全エージェントを停止
                    match self.shutdown_all(10).await {  // タイムアウトは適切な値に調整
                        Ok(_) => {
                            tracing::info!("AgentRegistry shutdown completed successfully");
                            break;
                        },
                        Err(e) => {
                            tracing::error!("Error during AgentRegistry shutdown: {:?}", e);
                            // エラーが発生しても全体の停止は継続
                            break;
                        }
                    }
                }
                else => {
                    // 何もしない
                }
            }
        }

        // 最終クリーンアップ
        self.cleanup().await;
        Ok(())
    }

    async fn cleanup(&self) {
        // ここで必要な最終クリーンアップ処理を実行
        // 例: メトリクスの保存、状態の永続化など
        self.running_agents.clear();
        self.agents.clear();
        tracing::info!("AgentRegistry cleanup completed");
    }

    pub async fn register_agent(
        &self,
        id: &str,
        agent: Arc<dyn RuntimeAgent>,
        event_bus: &EventBus,
    ) -> RuntimeResult<()> {
        if self.agents.contains_key(id) {
            return Err(RuntimeError::Execution(
                ExecutionError::AgentAlreadyExists { id: id.to_string() },
            ));
        }

        let agent_name = agent.name();
        // Agentの登録
        self.agents.insert(id.to_string(), agent);

        // AgentAddedイベントの発行
        event_bus
            .publish(Event {
                event_type: EventType::AgentAdded,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("agent_id".to_string(), Value::String(id.to_string()));
                    params.insert("agent_name".to_string(), Value::String(agent_name));
                    params
                },
            })
            .await?;

        Ok(())
    }

    pub async fn unregister_agent(&self, id: &str, event_bus: &EventBus) -> RuntimeResult<()> {
        // まず実行を停止
        if self.is_agent_running(id) {
            self.shutdown_agent(id, None).await?;
        }

        if self.agents.remove(id).is_none() {
            return Err(RuntimeError::Execution(ExecutionError::AgentNotFound {
                id: id.to_string(),
            }));
        }

        // AgentRemovedイベントの発行
        event_bus
            .publish(Event {
                event_type: EventType::AgentRemoved,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("agent_id".to_string(), Value::String(id.to_string()));
                    params
                },
            })
            .await?;

        Ok(())
    }

    pub async fn run_agent(&self, id: &str, event_bus: Arc<EventBus>) -> RuntimeResult<()> {
        let agent = self
            .agents
            .get(id)
            .ok_or_else(|| {
                RuntimeError::Execution(ExecutionError::AgentNotFound { id: id.to_string() })
            })?
            .clone();

        // すでに実行中のエージェントは終了
        if let Some(handle) = self.running_agents.get(id) {
            handle.abort();
        }

        // AgentStartedイベントの発行
        event_bus
            .publish(Event {
                event_type: EventType::AgentStarted,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("agent_id".to_string(), Value::String(id.to_string()));
                    params
                },
            })
            .await?;

        let cloned_id = id.to_string();
        let shutdown_rx = self.shutdown_tx.subscribe();
        let handle = tokio::spawn(async move {
            if let Err(e) = agent.run(shutdown_rx).await {
                // エラー発生時もイベントを発行
                let _ = event_bus
                    .publish_error(ErrorEvent {
                        error_type: "AgentError".to_string(),
                        message: e.to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("agent_id".to_string(), Value::String(cloned_id));
                            params
                        },
                    })
                    .await;
            }
        });

        self.running_agents.insert(id.to_string(), handle);
        Ok(())
    }

    pub async fn shutdown_agent(&self, id: &str, timeout_secs: Option<u64>) -> RuntimeResult<()> {
        let timeout_secs = timeout_secs.unwrap_or(30);
        let agent = self.agents.get(id).ok_or_else(|| {
            RuntimeError::Execution(ExecutionError::AgentNotFound { id: id.to_string() })
        })?;

        // シャットダウンの開始をログ
        tracing::info!("Initiating shutdown for agent: {}", id);

        match timeout(Duration::from_secs(timeout_secs), agent.shutdown()).await {
            Ok(_) => {
                tracing::info!("Agent {} shutdown completed", id);
                self.running_agents.remove(id);
            }
            Err(_) => {
                tracing::warn!("Agent {} shutdown timed out", id);
                return Err(RuntimeError::Execution(ExecutionError::ShutdownTimeout {
                    agent_id: id.to_string(),
                    timeout_secs,
                }));
            }
        }

        Ok(())
    }

    // エージェントの強制停止
    pub async fn kill_agent(&self, id: &str) -> RuntimeResult<()> {
        if let Some((_, handle)) = self.running_agents.remove(id) {
            handle.abort();
            tracing::info!("Agent {} forcefully killed", id);
            self.agents.remove(id);
        }
        Ok(())
    }

    // 全エージェントのシャットダウン
    pub async fn shutdown_all(&self, timeout_secs: u64) -> RuntimeResult<()> {
        tracing::info!("Initiating shutdown for all agents");

        let running_agent_ids: Vec<_> = self
            .running_agents
            .iter()
            .map(|entry| (entry.key().clone()))
            .collect();

        // 並行してシャットダウンを実行
        let shutdown_futures = running_agent_ids.iter().map(|id| {
            let self_ref = self.clone();

            async move {
                // 通常のシャットダウンを試みる
                match timeout(
                    Duration::from_secs(timeout_secs),
                    self_ref.shutdown_agent(id, Some(timeout_secs + 1)),
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("Agent {} shutdown completed", id);
                        let ok: RuntimeResult<()> = Ok(());
                        ok
                    }
                    Err(_) => {
                        tracing::warn!("Agent {} shutdown timed out, executing kill", id);
                        self_ref.kill_agent(id).await?;
                        Ok(())
                    }
                }
            }
        });

        // 全てのシャットダウン処理を待機
        futures::future::join_all(shutdown_futures).await;

        // クリーンアップ
        self.running_agents.clear();
        self.agents.clear();

        tracing::info!("All agents shutdown completed");
        Ok(())
    }

    // エージェントの状態確認
    pub fn is_agent_running(&self, id: &str) -> bool {
        self.running_agents.contains_key(id)
    }

    // 実行中のエージェント数を取得
    pub fn running_agent_count(&self) -> usize {
        self.running_agents.len()
    }
}

// テスト用のヘルパー関数
#[cfg(test)]
mod tests {
    use futures::{stream::SelectAll, Stream};
    use std::{pin::Pin, sync::atomic::AtomicBool, time::Duration};
    use tokio::{sync::Mutex, task::JoinHandle, time::sleep};
    use tokio_stream::{wrappers::BroadcastStream, StreamExt};

    use crate::{
        event_registry::{EventType, LifecycleEvent},
        runtime::StreamMessage,
    };

    use super::*;
    use tokio::test;

    // イベントを収集するヘルパー構造体
    struct EventCollector {
        events: Arc<Mutex<Vec<Event>>>,
        _task: JoinHandle<()>, // タスクを保持して中断を防ぐ
    }

    impl EventCollector {
        fn new(event_bus: &EventBus) -> Self {
            let events = Arc::new(Mutex::new(Vec::new()));
            let events_clone = events.clone();
            let (rx, _) = event_bus.subscribe();

            // イベントを収集するタスクを起動
            let task = tokio::spawn(async move {
                let mut rx = rx;
                while let Ok(event) = rx.recv().await {
                    let mut events = events_clone.lock().await; // tokioのMutexを使う
                    events.push(event);
                }
            });

            Self {
                events,
                _task: task,
            }
        }

        async fn get_events(&self) -> Vec<Event> {
            self.events.lock().await.clone()
        }
    }

    // テスト用のMockAgent実装
    struct MockAgent {
        event_bus: Arc<EventBus>,
        pub received: Arc<AtomicBool>,
        name: String,
        private_shutdown_tx: broadcast::Sender<()>,
    }

    impl MockAgent {
        pub fn new(name: &str, event_bus: &Arc<EventBus>) -> Self {
            Self {
                event_bus: event_bus.clone(),
                received: Arc::new(AtomicBool::new(false)),
                name: name.to_string(),
                private_shutdown_tx: broadcast::channel(1).0,
            }
        }
    }

    #[async_trait::async_trait]
    impl RuntimeAgent for MockAgent {
        fn name(&self) -> String {
            self.name.clone()
        }
        async fn run(&self, shutdown_rx: broadcast::Receiver<()>) -> RuntimeResult<()> {
            let (event_rx, _) = self.event_bus.subscribe();
            let private_shutdown_rx = self.private_shutdown_tx.subscribe();

            // イベントストリームの変換
            let event_stream = BroadcastStream::new(event_rx.receiver).map(|e| {
                tracing::debug!("Event received");
                match e {
                    Ok(event) => Ok(StreamMessage::Event(event)),
                    Err(_) => Err(()),
                }
            });

            // システムシャットダウンストリームの変換
            let system_shutdown_stream = BroadcastStream::new(shutdown_rx).map(|e| {
                tracing::debug!("System shutdown received");
                match e {
                    Ok(_) => Ok(StreamMessage::SystemShutdown),
                    Err(_) => Err(()),
                }
            });

            // プライベートシャットダウンストリームの変換
            let private_shutdown_stream = BroadcastStream::new(private_shutdown_rx).map(|e| {
                tracing::debug!("Private shutdown received");
                match e {
                    Ok(_) => Ok(StreamMessage::PrivateShutdown),
                    Err(_) => Err(()),
                }
            });

            // ストリームの統合
            let mut streams: SelectAll<
                Pin<Box<dyn Stream<Item = Result<StreamMessage, ()>> + Send>>,
            > = SelectAll::new();
            streams.push(Box::pin(event_stream));
            streams.push(Box::pin(system_shutdown_stream));
            streams.push(Box::pin(private_shutdown_stream));

            while let Some(Ok(message)) = streams.next().await {
                match message {
                    StreamMessage::Event(event) => {
                        if event.event_type == EventType::Custom("test".to_string()) {
                            self.received
                                .store(true, std::sync::atomic::Ordering::SeqCst);
                        }
                    }
                    StreamMessage::SystemShutdown | StreamMessage::PrivateShutdown => {
                        break;
                    }
                    _ => {}
                }
            }
            self.event_bus
                .publish(Event {
                    event_type: EventType::AgentStopped,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("agent_id".to_string(), Value::String(self.name.clone()));
                        params
                    },
                })
                .await?;
            Ok(())
        }
        async fn shutdown(&self) -> RuntimeResult<()> {
            self.private_shutdown_tx.send(()).map_err(|e| {
                RuntimeError::Execution(ExecutionError::SendShutdownFailed {
                    agent_name: self.name.clone(),
                    message: e.to_string(),
                })
            })?;
            tokio::time::sleep(Duration::from_secs(2)).await; // 終了処理をシミュレート

            Ok(())
        }
        async fn handle_lifecycle_event(
            &self,
            _event: &LifecycleEvent,
        ) -> RuntimeResult<(Vec<Event>, HashMap<String, Value>)> {
            Ok((vec![], HashMap::new()))
        }
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let agent_registry = AgentRegistry::new(&broadcast::channel(1).0);
        let event_bus = Arc::new(EventBus::new(16));
        let collector = EventCollector::new(&event_bus);

        // エージェントの登録
        let agent = Arc::new(MockAgent::new("test1", &event_bus));
        agent_registry
            .register_agent("test1id", agent, &event_bus)
            .await
            .unwrap();

        // 少し待機してイベントを収集
        sleep(Duration::from_millis(100)).await;
        let events = collector.get_events().await;

        // AgentAddedイベントの確認
        let event = events
            .iter()
            .find(|e| e.event_type == EventType::AgentAdded)
            .unwrap();
        assert_eq!(
            event.parameters.get("agent_id").unwrap(),
            &Value::String("test1id".to_string())
        );
        assert_eq!(
            event.parameters.get("agent_name").unwrap(),
            &Value::String("test1".to_string())
        );
    }

    #[tokio::test]
    async fn test_agent_lifecycle() {
        let event_bus = Arc::new(EventBus::new(16));
        let collector = EventCollector::new(&event_bus);
        let shutdown_tx = broadcast::channel(1);
        let agent_registry = AgentRegistry::new(&shutdown_tx.0);
        let agent_registry_ref = Arc::new(agent_registry.clone());
        tokio::spawn(async move {
            agent_registry_ref.run().await.unwrap();
        });

        // 登録 -> 起動 -> 停止 -> 登録解除のライフサイクルテスト
        let agent = Arc::new(MockAgent::new("test2", &event_bus));
        agent_registry
            .register_agent("test2", agent, &event_bus)
            .await
            .unwrap();
        agent_registry
            .run_agent("test2", event_bus.clone())
            .await
            .unwrap();
        sleep(Duration::from_millis(100)).await;
        agent_registry.shutdown_agent("test2", None).await.unwrap();

        agent_registry
            .unregister_agent("test2", &event_bus)
            .await
            .unwrap();

        shutdown_tx.0.send(()).unwrap();

        sleep(Duration::from_millis(100)).await;

        let events = collector.get_events().await;

        // 各ライフサイクルイベントの確認
        let event_types: Vec<_> = events.iter().map(|e| e.event_type.clone()).collect();

        assert!(event_types.contains(&EventType::AgentAdded));
        assert!(event_types.contains(&EventType::AgentStarted));
        assert!(event_types.contains(&EventType::AgentStopped));
        assert!(event_types.contains(&EventType::AgentRemoved));
    }
    #[tokio::test]
    async fn test_unregister_nonexistent_agent() {
        let agent_registry = AgentRegistry::new(&broadcast::channel(1).0);
        let event_bus = Arc::new(EventBus::new(16));
        let result = agent_registry
            .unregister_agent("nonexistent", &event_bus)
            .await;

        assert!(matches!(
            result,
            Err(RuntimeError::Execution(
                ExecutionError::AgentNotFound { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn test_multiple_agents() {
        let shutdonw_tx = broadcast::channel(1);
        let agent_registry = AgentRegistry::new(&shutdonw_tx.0);
        let agent_registry_ref = Arc::new(agent_registry.clone());
        tokio::spawn(async move {
            agent_registry_ref.run().await.unwrap();
        });

        let event_bus = Arc::new(EventBus::new(16));
        let collector = EventCollector::new(&event_bus);

        // 複数のエージェントを登録して実行
        for i in 0..3 {
            let agent = Arc::new(MockAgent::new(&format!("test{}", i), &event_bus));
            agent_registry
                .register_agent(&format!("test{}", i), agent.clone(), &event_bus)
                .await
                .unwrap();
            agent_registry
                .run_agent(&format!("test{}", i), event_bus.clone())
                .await
                .unwrap();
        }

        sleep(Duration::from_millis(100)).await;

        let events = collector.get_events().await;
        let added_events = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::AgentAdded { .. }))
            .count();
        let started_events = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::AgentStarted { .. }))
            .count();

        assert_eq!(added_events, 3);
        assert_eq!(started_events, 3);

        shutdonw_tx.0.send(()).unwrap();
    }

    #[test]
    async fn test_simple_agent() {
        let shutdown_tx = broadcast::channel(1);
        let agent_registry = AgentRegistry::new(&shutdown_tx.0);
        let agent_registry_ref = Arc::new(agent_registry.clone());
        tokio::spawn(async move {
            agent_registry_ref.run().await.unwrap();
        });

        let event_bus = Arc::new(EventBus::new(100));

        let agent = Arc::new(MockAgent::new("test", &event_bus));
        let event_bus = Arc::new(EventBus::new(100));
        let id = "test";
        agent_registry
            .register_agent(id, agent, &event_bus)
            .await
            .unwrap();

        sleep(Duration::from_millis(1000)).await;

        // テストイベントの送信
        event_bus
            .publish(Event {
                event_type: EventType::Custom("test".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        // 非同期処理のテストなので、少し待機
        sleep(Duration::from_millis(100)).await;

        shutdown_tx.0.send(()).unwrap();
        sleep(Duration::from_millis(100)).await;
    }
}

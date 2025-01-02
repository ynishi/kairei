use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

use crate::event_bus::{ErrorEvent, Event, EventBus, Value};
use crate::event_registry::EventType;
use crate::runtime::RuntimeAgent;
use crate::{ExecutionError, RuntimeError, RuntimeResult};

pub struct AgentRegistry {
    agents: Arc<DashMap<String, Arc<dyn RuntimeAgent>>>,
    running_agents: Arc<DashMap<String, tokio::task::JoinHandle<()>>>,
}

impl Clone for AgentRegistry {
    fn clone(&self) -> Self {
        Self {
            agents: self.agents.clone(),
            running_agents: self.running_agents.clone(),
        }
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            running_agents: Arc::new(DashMap::new()),
        }
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
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    pub async fn unregister_agent(&self, id: &str, event_bus: &EventBus) -> RuntimeResult<()> {
        // まず実行を停止
        self.stop_agent(id, event_bus).await?;

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
                ..Default::default()
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
                ..Default::default()
            })
            .await?;

        let cloned_id = id.to_string();
        let handle = tokio::spawn(async move {
            if let Err(e) = agent.run().await {
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

    pub async fn stop_agent(&self, id: &str, event_bus: &EventBus) -> RuntimeResult<()> {
        if let Some((_, handle)) = self.running_agents.remove(id) {
            handle.abort();

            // AgentStoppedイベントの発行
            event_bus
                .publish(Event {
                    event_type: EventType::AgentStopped,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("agent_id".to_string(), Value::String(id.to_string()));
                        params
                    },
                    ..Default::default()
                })
                .await?;
        }
        Ok(())
    }
}

// テスト用のヘルパー関数
#[cfg(test)]
mod tests {
    use std::{sync::atomic::AtomicBool, time::Duration};
    use tokio::{sync::Mutex, task::JoinHandle, time::sleep};

    use crate::event_registry::EventType;

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
    }

    impl MockAgent {
        pub fn new(name: &str, event_bus: &Arc<EventBus>) -> Self {
            Self {
                event_bus: event_bus.clone(),
                received: Arc::new(AtomicBool::new(false)),
                name: name.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl RuntimeAgent for MockAgent {
        async fn run(&self) -> RuntimeResult<()> {
            let (mut rx, _) = self.event_bus.subscribe();

            // イベントを待ち続ける
            while let Ok(event) = rx.recv().await {
                if event.event_type == EventType::Custom("test".to_string()) {
                    self.received
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
            Ok(())
        }
        fn name(&self) -> String {
            self.name.clone()
        }
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let event_bus = Arc::new(EventBus::new(16));
        let agent_registry = AgentRegistry::new();
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
        let agent_registry = AgentRegistry::new();
        let collector = EventCollector::new(&event_bus);

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
        agent_registry
            .stop_agent("test2", &event_bus)
            .await
            .unwrap();
        agent_registry
            .unregister_agent("test2", &event_bus)
            .await
            .unwrap();

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
        let event_bus = Arc::new(EventBus::new(16));
        let agent_registry = AgentRegistry::new();

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
        let event_bus = Arc::new(EventBus::new(16));
        let agent_registry = AgentRegistry::new();
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
    }

    #[test]
    async fn test_simple_agent() {
        let runtime = AgentRegistry::new();

        let event_bus = Arc::new(EventBus::new(100));

        let agent = Arc::new(MockAgent {
            event_bus: event_bus.clone(),
            received: Arc::new(AtomicBool::new(false)),
            name: "test".to_string(),
        });

        let event_bus = Arc::new(EventBus::new(100));
        let id = "test";
        runtime.register_agent(id, agent, &event_bus).await.unwrap();

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
    }
}

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::{
    agent_registry::AgentRegistry, event_bus::EventBus, event_registry::EventRegistry,
    RuntimeResult,
};

pub struct System {
    event_registry: Arc<RwLock<EventRegistry>>,
    event_bus: Arc<EventBus>,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    shutdown_tx: broadcast::Sender<()>, // Systemがシャットダウンシグナルを送信
}

impl System {
    pub async fn new(capacity: usize) -> Self {
        let event_registry = Arc::new(RwLock::new(EventRegistry::new()));
        let event_bus = Arc::new(EventBus::new(capacity));
        let (shutdown_tx, _) = broadcast::channel(1); // 容量は1で十分
        let agent_registry = Arc::new(tokio::sync::RwLock::new(AgentRegistry::new(&shutdown_tx)));

        Self {
            event_registry,
            event_bus,
            agent_registry,
            shutdown_tx,
        }
    }

    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    pub fn event_registry(&self) -> &Arc<RwLock<EventRegistry>> {
        &self.event_registry
    }

    pub fn runtime(&self) -> &Arc<RwLock<AgentRegistry>> {
        &self.agent_registry
    }

    pub async fn shutdown(&self) -> RuntimeResult<()> {
        // シャットダウンシグナルを送信
        self.shutdown_tx
            .send(())
            .expect("Failed to send shutdown signal");
        Ok(())
    }
}

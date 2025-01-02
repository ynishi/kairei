use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{agent_registry::AgentRegistry, event_bus::EventBus, event_registry::EventRegistry};

pub struct System {
    event_registry: Arc<RwLock<EventRegistry>>,
    event_bus: Arc<EventBus>,
    agent_registry: Arc<RwLock<AgentRegistry>>,
}

impl System {
    pub async fn new(capacity: usize) -> Self {
        let event_registry = Arc::new(RwLock::new(EventRegistry::new()));
        let event_bus = Arc::new(EventBus::new(capacity));
        let agent_registry = Arc::new(tokio::sync::RwLock::new(AgentRegistry::new()));

        Self {
            event_registry,
            event_bus,
            agent_registry,
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
}

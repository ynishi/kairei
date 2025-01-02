use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::{broadcast, RwLock};

use crate::{
    agent_registry::AgentRegistry,
    ast_registry::AstRegistry,
    event_bus::EventBus,
    event_registry::{EventInfo, EventRegistry, EventType, ParameterType},
    EventError, MicroAgentDef, RuntimeError, RuntimeResult,
};

pub struct System {
    event_registry: Arc<RwLock<EventRegistry>>,
    event_bus: Arc<EventBus>,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    ast_registry: Arc<RwLock<AstRegistry>>,
    shutdown_tx: broadcast::Sender<()>, // Systemがシャットダウンシグナルを送信
    _shutdown_rx: broadcast::Receiver<()>, // シャットダウンシグナルを受信
}

impl System {
    // System Lifecycles
    pub async fn new(capacity: usize) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1); // 容量は1で十分
        let event_registry = Arc::new(RwLock::new(EventRegistry::new()));
        let event_bus = Arc::new(EventBus::new(capacity));
        let agent_registry = Arc::new(tokio::sync::RwLock::new(AgentRegistry::new(&shutdown_tx)));
        let ast_registry = Arc::new(RwLock::new(AstRegistry::default()));
        let _shutdown_rx = shutdown_tx.subscribe();

        Self {
            event_registry,
            event_bus,
            agent_registry,
            ast_registry,
            shutdown_tx,
            _shutdown_rx,
        }
    }

    pub async fn start(&self) -> RuntimeResult<()> {
        let agent_names = {
            let registry = self.agent_registry.read().await;
            registry.agent_names().clone()
        };

        for agent_name in agent_names {
            let registry = self.agent_registry.write().await;
            registry
                .run_agent(&agent_name, self.event_bus.clone())
                .await?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> RuntimeResult<()> {
        // シャットダウンシグナルを送信
        self.shutdown_tx
            .send(())
            .expect("Failed to send shutdown signal");
        // TODO: シャットダウン処理完了を受けて、システムを停止する
        Ok(())
    }

    pub async fn emergency_shutdown(&self) -> RuntimeResult<()> {
        // シャットダウンシグナルを送信
        self.shutdown_tx
            .send(())
            .expect("Failed to send shutdown signal");
        self.agent_registry.write().await.shutdown_all(1).await?;
        Ok(())
    }

    /// AST management
    pub async fn register_agent_ast(
        &self,
        agent_name: &str,
        ast: &MicroAgentDef,
    ) -> RuntimeResult<()> {
        self.ast_registry
            .write()
            .await
            .register_agent_ast(agent_name, ast)
            .await
    }

    pub async fn get_agent_ast(&self, _agent_name: &str) -> RuntimeResult<Arc<MicroAgentDef>> {
        self.ast_registry
            .read()
            .await
            .get_agent_ast(_agent_name)
            .await
    }

    pub async fn register_event_ast(
        &self,
        name: &str,
        parameters: HashMap<String, ParameterType>,
    ) -> RuntimeResult<()> {
        let mut registry = self.event_registry.write().await;
        registry.register_custom_event(name.to_string(), parameters)
    }

    pub async fn get_event(&self, name: &str) -> RuntimeResult<EventInfo> {
        let event_type = if let Ok(event_type) = EventType::from_str(name) {
            event_type
        } else {
            EventType::Custom(name.to_string())
        };
        let registry = self.event_registry.read().await;
        registry
            .get_event_info(&event_type)
            .ok_or(RuntimeError::Event(EventError::NotFound(name.to_string())))
    }

    /// Agent management
    pub async fn start_agent(&self, agent_name: &str) -> RuntimeResult<()> {
        let registry = self.agent_registry.read().await;
        registry.run_agent(agent_name, self.event_bus.clone()).await
    }

    pub async fn stop_agent(&self, agent_name: &str) -> RuntimeResult<()> {
        let registry = self.agent_registry.read().await;
        registry.shutdown_agent(agent_name, None).await
    }

    pub async fn restart_agent(&self, agent_name: &str) -> RuntimeResult<()> {
        let registry = self.agent_registry.read().await;
        registry.shutdown_agent(agent_name, None).await?;
        registry.run_agent(agent_name, self.event_bus.clone()).await
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use tokio::{test, time::sleep};

    #[test]
    async fn test_system_creation() {
        System::new(1000).await;
    }

    #[test]
    async fn test_system_shutdown() {
        let system = System::new(1000).await;
        let result = system.shutdown().await;
        sleep(Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    #[test]
    async fn test_system_emergency_shutdown() {
        let system = System::new(1000).await;
        let result = system.emergency_shutdown().await;
        sleep(Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }
}

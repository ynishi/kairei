use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::{
    agent_registry::AgentRegistry,
    ast_registry::AstRegistry,
    event_bus::{EventBus, Value},
    event_registry::{EventInfo, EventRegistry, EventType, ParameterType},
    runtime::RuntimeAgentData,
    EventError, ExecutionError, MicroAgentDef, RuntimeError, RuntimeResult,
};

type AgentName = String;

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

    // システムの初期化処理(builtin agent, eventの登録)
    pub async fn setup_builtin(&self) -> RuntimeResult<()> {
        todo!()
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

    pub async fn scale_up(
        &self,
        name: &str,
        count: usize,
        _metadata: HashMap<String, Value>,
    ) -> RuntimeResult<Vec<String>> {
        let request_id = Uuid::new_v4().to_string();

        // ASTの存在確認
        let registry = self.ast_registry.read().await;
        let ast_def = registry.get_agent_ast(name).await?;

        let mut created_agents = Vec::with_capacity(count);

        // TODO: ScaleManagerAgent へのリクエストを送信してオプションを取得する

        // 指定された数だけエージェントを作成
        for i in 0..count {
            let agent_name = format!("{}-{}-{}", name, request_id, i);
            let agent_def = ast_def.clone();

            let agent_data = Arc::new(RuntimeAgentData::new(&agent_def, &self.event_bus())?);

            let registry = self.agent_registry.write().await;
            registry
                .register_agent(&agent_name, agent_data, &self.event_bus().clone())
                .await?;
            registry
                .run_agent(&agent_name, self.event_bus().clone())
                .await?;

            created_agents.push(agent_name);
        }

        Ok(created_agents)
    }

    /// スケールダウン
    /// * name - スケール対象のAST名
    /// * count - 削除するインスタンス数
    /// * selector - 削除対象の選択方法（オプション）
    pub async fn scale_down(
        &self,
        name: &str,
        count: usize,
        _metadata: HashMap<String, Value>,
    ) -> RuntimeResult<()> {
        let target_agent_names = self.find_agents_by_base_name(name).await;
        // 削除対象が足りない場合はエラー
        if target_agent_names.len() < count {
            return Err(RuntimeError::Execution(
                ExecutionError::ScalingNotEnoughAgents {
                    base_name: name.to_string(),
                    required: count,
                    current: target_agent_names.len(),
                },
            ));
        }

        // TODO: ScaleManagerAgent へのリクエストを送信して削除対象を取得する

        let agent_names_to_remove = target_agent_names.iter().take(count);

        // 対象エージェントの停止と削除
        for agent_name in agent_names_to_remove {
            let registry = self.agent_registry.write().await;
            registry.shutdown_agent(agent_name, None).await?;
        }

        Ok(())
    }

    /// 現在のスケール状態を取得
    pub async fn get_scale_status(&self, name: &str) -> RuntimeResult<ScaleStatus> {
        let agent_names = self.find_agents_by_base_name(name).await;

        let registory = self.agent_registry.read().await;

        Ok(ScaleStatus {
            base_name: name.to_string(),
            total_count: agent_names.len(),
            running_count: agent_names
                .iter()
                .filter(|name| registory.is_agent_running(name))
                .count(),
            agent_names,
        })
    }

    async fn find_agents_by_base_name(&self, name: &str) -> Vec<AgentName> {
        let registry = self.agent_registry.read().await;
        registry
            .agent_names()
            .iter()
            .filter(|n| n.starts_with(name))
            .cloned()
            .collect::<Vec<String>>()
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

#[derive(Debug, Clone)]
pub struct ScaleStatus {
    pub base_name: String,
    pub total_count: usize,
    pub running_count: usize,
    pub agent_names: Vec<AgentName>,
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

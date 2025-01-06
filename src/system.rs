use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::str::FromStr;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{broadcast, oneshot, RwLock},
    time::{sleep, timeout},
};
use tracing::debug;
use uuid::Uuid;

use crate::{
    agent_registry::AgentRegistry,
    ast_registry::AstRegistry,
    config::{AgentConfig, SystemConfig},
    eval::{context::AgentType, expression},
    event_bus::{Event, EventBus, EventReceiver, LastStatus, Value},
    event_registry::{EventInfo, EventRegistry, EventType, ParameterType},
    native_feature::{native_registry::NativeFeatureRegistry, types::NativeFeatureContext},
    runtime::RuntimeAgentData,
    CustomEventDef, EventError, EventsDef, ExecutionError, MicroAgentDef, RuntimeError,
    RuntimeResult, WorldDef,
};

type AgentName = String;

pub struct System {
    event_registry: Arc<RwLock<EventRegistry>>,
    event_bus: Arc<EventBus>,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    ast_registry: Arc<RwLock<AstRegistry>>,
    feature_registry: Arc<RwLock<NativeFeatureRegistry>>,
    shutdown_tx: broadcast::Sender<AgentType>, // Systemがシャットダウンシグナルを送信
    _shutdown_rx: broadcast::Receiver<AgentType>, // シャットダウンシグナルを受信
    // event request/response
    pending_requests: Arc<DashMap<String, oneshot::Sender<Value>>>,
    filtered_subscriptions: Arc<DashMap<Vec<EventType>, broadcast::Sender<Event>>>, // Vec<EventType>は Sorted　である必要がある
    // metrics
    started_at: DateTime<Utc>,
    uptime_instant: Instant,
    last_status: Arc<RwLock<LastStatus>>,
    config: Arc<RwLock<SystemConfig>>,
}

impl System {
    // System Lifecycles
    pub async fn new(config: &SystemConfig) -> Self {
        let capacity = config.event_buffer_size;
        let (shutdown_tx, _) = broadcast::channel::<AgentType>(1); // 容量は1で十分
        let event_registry = Arc::new(RwLock::new(EventRegistry::new()));
        let event_bus = Arc::new(EventBus::new(capacity));
        let agent_registry = Arc::new(tokio::sync::RwLock::new(AgentRegistry::new(
            &config.agent_config,
            &shutdown_tx,
        )));
        let ast_registry = Arc::new(RwLock::new(AstRegistry::default()));
        let native_context = Arc::new(NativeFeatureContext::new(event_bus.clone()));

        let feature_registry = Arc::new(RwLock::new(NativeFeatureRegistry::new(
            native_context.clone(),
            config.native_feature_config.clone(),
        )));
        let _shutdown_rx = shutdown_tx.subscribe();
        let pending_requests: Arc<DashMap<String, oneshot::Sender<Value>>> =
            Arc::new(DashMap::new());
        let pending_requests_ref = pending_requests.clone();
        let mut event_rx = event_bus.subscribe().0;
        let filtered_subscriptions = Arc::new(DashMap::new());

        // Receive response.
        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                debug!("event: {:?}", event);
                if let Some(request_id) = event.event_type.request_id() {
                    debug!("request_id: {}", request_id);
                    if let Some((_, sender)) = pending_requests_ref.remove(request_id) {
                        if let Some(value) = event.parameters.get("response") {
                            let _ = sender.send(value.clone());
                        } else {
                            let _ = sender.send(Value::Null);
                        }
                    }
                }
            }
        });

        let started_at = Utc::now();
        let uptime_instant = Instant::now();

        let last_status = Arc::new(RwLock::new(LastStatus {
            last_event_type: EventType::SystemCreated,
            last_event_time: Utc::now(),
        }));

        Self {
            event_registry,
            event_bus,
            agent_registry,
            ast_registry,
            feature_registry,
            shutdown_tx,
            _shutdown_rx,
            pending_requests,
            filtered_subscriptions,
            started_at,
            uptime_instant,
            last_status,
            config: Arc::new(RwLock::new(config.clone())),
        }
    }

    pub async fn register_native_features(&mut self) -> RuntimeResult<()> {
        let complete_state = EventType::SystemNativeFeaturesRegistered;
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            complete_state.clone(),
        )?;

        let registry = self.feature_registry.write().await;
        registry.register().await?;

        self.update_system_status(complete_state).await;
        Ok(())
    }

    pub async fn register_world(&self, world_def: WorldDef) -> RuntimeResult<()> {
        let complete_state = EventType::SystemWorldRegistered;
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            complete_state.clone(),
        )?;

        let (agent_def, event_defs): (MicroAgentDef, EventsDef) = world_def.into();
        let name = AgentType::World.to_string();
        self.register_agent_ast(&name, &agent_def).await?;
        self.register_agent(&name).await?;

        for event_def in event_defs.events {
            self.register_event_ast(event_def).await?;
        }

        self.update_system_status(complete_state).await;
        Ok(())
    }

    // ビルトインエージェントの登録処理
    pub async fn register_builtin_agents(&self) -> RuntimeResult<()> {
        let complete_state = EventType::SystemBuiltinAgentsRegistered;
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            complete_state.clone(),
        )?;
        let registry = self.agent_registry().read().await;
        let builtin_defs = registry.get_builtin_agent_asts().await?;

        for builtin in builtin_defs {
            self.register_agent_ast(&builtin.name, &builtin).await?;
            self.register_agent(&builtin.name).await?;
        }

        self.update_system_status(complete_state).await;
        Ok(())
    }

    pub async fn register_initial_user_agents(
        &self,
        agent_asts: Vec<MicroAgentDef>,
    ) -> RuntimeResult<()> {
        let complete_state = EventType::SystemUserAgentsRegistered;
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            complete_state.clone(),
        )?;

        for agent_ast in agent_asts {
            self.register_agent_ast(&agent_ast.name, &agent_ast).await?;
            self.register_agent(&agent_ast.name).await?;
        }
        self.update_system_status(complete_state).await;
        Ok(())
    }

    pub async fn start(&self) -> RuntimeResult<()> {
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            EventType::SystemStarting,
        )?;
        self.update_system_status(EventType::SystemStarting).await;

        self.start_native_features().await?;

        self.start_world().await?;
        self.start_builtin_agents().await?;
        self.start_users_agents().await?;

        self.update_system_status(EventType::SystemStarted).await;
        Ok(())
    }

    async fn start_native_features(&self) -> RuntimeResult<()> {
        let registry = self.feature_registry.write().await;
        registry.start().await?;
        Ok(())
    }

    async fn start_world(&self) -> RuntimeResult<()> {
        self.start_agent(&AgentType::World.to_string()).await?;
        Ok(())
    }

    async fn start_builtin_agents(&self) -> RuntimeResult<()> {
        let registry = self.agent_registry().read().await;
        let agent_names = registry
            .get_enabled_builtin_agent_types()
            .iter()
            .map(|e| e.clone().to_string())
            .collect::<Vec<String>>();
        drop(registry);

        for agent_name in agent_names {
            self.start_agent(&agent_name).await?;
        }
        Ok(())
    }

    async fn start_users_agents(&self) -> RuntimeResult<()> {
        let registry = self.agent_registry().read().await;
        let agent_names =
            registry.agent_names_by_types(vec![AgentType::Custom("user".to_string())]);
        drop(registry);

        for agent_name in agent_names {
            self.start_agent(&agent_name).await?;
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> RuntimeResult<()> {
        let shutdown_started = Instant::now();
        self.update_system_status(EventType::SystemStopping).await;
        let shutdown_sequence = AgentRegistry::agent_shutdown_sequence();
        let config = self.config.read().await;
        let timeout = config.shutdown_timeout;
        drop(config);
        // シャットダウンシグナルを送信
        self.shutdown_tx
            .send(AgentType::Custom("All".to_string()))
            .expect("Failed to send shutdown signal");
        let registry = self.feature_registry.write().await;
        registry.shutdown().await?;
        for agent_type in shutdown_sequence {
            self.shutdown_tx
                .send(agent_type.clone())
                .expect("Failed to send shutdown signal");
            loop {
                sleep(Duration::from_secs(10)).await;
                let registry = self.agent_registry.read().await;
                let status = registry.agent_status_by_agent_type(&agent_type).await;
                let not_stopped = status
                    .iter()
                    .filter(|e| {
                        e.last_event_type != EventType::AgentStopped
                            || e.last_event_type != EventType::AgentRemoved
                    })
                    .count();
                if not_stopped == 0 {
                    break;
                }
                // shutdown 開始から 60 秒経過したら即終了する。
                if self.check_shutdown_timeout(shutdown_started, timeout) {
                    break;
                }
            }
            if self.check_shutdown_timeout(shutdown_started, timeout) {
                break;
            }
        }

        // TODO: シャットダウン処理完了を受けて、システムを停止する
        self.update_system_status(EventType::SystemStopped).await;
        Ok(())
    }

    fn check_shutdown_timeout(&self, shutdown_started: Instant, timeout: Duration) -> bool {
        shutdown_started.elapsed() > timeout
    }

    pub async fn emergency_shutdown(&self) -> RuntimeResult<()> {
        // シャットダウンシグナルを送信
        self.shutdown_tx
            .send(AgentType::World)
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

    pub async fn list_agent_asts(&self) -> RuntimeResult<Vec<String>> {
        Ok(self.ast_registry.read().await.list_agent_asts().await)
    }

    pub async fn register_event_ast(&self, event_def: CustomEventDef) -> RuntimeResult<()> {
        let name = event_def.name.to_string();
        let parameters: HashMap<String, ParameterType> = event_def
            .parameters
            .iter()
            .map(|p| (p.name.clone(), ParameterType::from(p.type_info.clone())))
            .collect();
        let mut registry = self.event_registry.write().await;
        registry.register_custom_event(name, parameters)
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

            let agent_data = Arc::new(
                RuntimeAgentData::new(&agent_def, &self.event_bus(), AgentConfig::default())
                    .await?,
            );

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
    pub async fn register_agent(&self, agent_name: &str) -> RuntimeResult<()> {
        let ast_registry = self.ast_registry.read().await;
        let agent_def = ast_registry.get_agent_ast(agent_name).await?;
        drop(ast_registry);
        let runtime = Arc::new(
            RuntimeAgentData::new(&agent_def, &self.event_bus, AgentConfig::default()).await?,
        );
        let agent_registry = self.agent_registry.write().await;
        agent_registry
            .register_agent(agent_name, runtime, &self.event_bus)
            .await?;
        Ok(())
    }
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

    /// Send/Receive events
    pub async fn send_event(&self, event: Event) -> RuntimeResult<()> {
        self.event_bus.publish(event).await
    }

    pub async fn send_request(&self, event: Event) -> RuntimeResult<Value> {
        let request_id = match event.event_type.clone() {
            EventType::Request { request_id, .. } => request_id,
            _ => {
                return Err(RuntimeError::Event(EventError::UnsupportedRequest {
                    request_type: event.event_type.to_string(),
                }));
            }
        };
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending_requests.insert(request_id.clone(), tx);
        let event_bus = self.event_bus.clone();
        event_bus.publish(event).await?;
        let timeout_secs = 30;
        match timeout(Duration::from_secs(timeout_secs), rx).await {
            Ok(value) => value.map_err(|e| {
                RuntimeError::Event(EventError::RecieveResponseFailed {
                    request_id: request_id.to_string(),
                    message: e.to_string(),
                })
            }),
            Err(e) => Err(RuntimeError::Event(EventError::RecieveResponseTimeout {
                request_id: request_id.to_string(),
                timeout_secs,
                message: e.to_string(),
            })),
        }
    }

    pub async fn get_agent_state(
        &self,
        agent_name: &str,
        key: &str,
    ) -> RuntimeResult<expression::Value> {
        let registry = self.agent_registry.read().await;
        registry
            .agent_state(agent_name, key)
            .await
            .ok_or(RuntimeError::Execution(ExecutionError::AgentNotFound {
                id: agent_name.to_string(),
            }))
    }

    /// イベントの購読
    pub async fn subscribe_events(
        &self,
        event_types: Vec<EventType>,
    ) -> RuntimeResult<EventReceiver> {
        let key = self.get_filtered_subscription_key(&event_types);
        if let Some(sender) = self.filtered_subscriptions.get(&key) {
            return Ok(EventReceiver::new(sender.subscribe()));
        }

        let (tx, rx) = broadcast::channel(100);
        self.filtered_subscriptions.insert(key.clone(), tx.clone());

        // イベントバスからサブスクライバーを取得
        let mut subscriber = self.event_bus.subscribe().0;

        let event_types = event_types.clone();
        tokio::spawn(async move {
            while let Ok(event) = subscriber.recv().await {
                if event_types.contains(&event.event_type) {
                    // エラーは無視（受信側がすべて切断された場合など）
                    let _ = tx.send(event);
                }
            }
        });

        Ok(EventReceiver::new(rx))
    }

    fn get_filtered_subscription_key(&self, event_types: &[EventType]) -> Vec<EventType> {
        let mut sorted = event_types.to_vec();
        sorted.sort();
        sorted
    }

    pub fn cleanup_unused_subscriptions(&self) {
        self.filtered_subscriptions
            .retain(|_, sender| sender.receiver_count() > 0)
    }

    /// Status and Metrics
    pub async fn get_system_status(&self) -> RuntimeResult<SystemStatus> {
        let registry = self.agent_registry.read().await;
        let event_bus = &self.event_bus;

        Ok(SystemStatus {
            started_at: self.started_at,
            running: self.last_status.read().await.last_event_type != EventType::SystemStopped,
            uptime: self.uptime_instant.elapsed(),
            agent_count: registry.agent_names().len(),
            running_agent_count: registry.running_agent_count(),
            event_queue_size: event_bus.queue_size(),
            event_subscribers: event_bus.subscribers_size(),
            event_capacity: event_bus.capacity(),
        })
    }

    /// 特定のエージェントの状態取得
    pub async fn get_agent_status(&self, agent_name: &str) -> RuntimeResult<AgentStatus> {
        let registry = self.agent_registry.read().await;
        let agent_status = registry.agent_status(agent_name).await.ok_or_else(|| {
            RuntimeError::Execution(ExecutionError::AgentNotFound {
                id: agent_name.to_string(),
            })
        })?;

        Ok(AgentStatus {
            name: agent_name.to_string(),
            state: agent_status.last_event_type.to_string(),
            last_lifecycle_updated: agent_status.last_event_time,
        })
    }

    async fn update_system_status(&self, event_type: EventType) {
        let mut lock = self.last_status.write().await;
        lock.last_event_type = event_type.clone();
        lock.last_event_time = Utc::now();
        let last_status = lock.clone();
        drop(lock);
        // ignore error
        let _ = self
            .event_bus
            .publish(Event::from(last_status.clone()))
            .await;
    }

    /// basic accessors
    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    pub fn event_registry(&self) -> &Arc<RwLock<EventRegistry>> {
        &self.event_registry
    }

    pub fn agent_registry(&self) -> &Arc<RwLock<AgentRegistry>> {
        &self.agent_registry
    }

    fn check_start_transition(currnt: EventType, next: EventType) -> RuntimeResult<()> {
        let err = Err(RuntimeError::Execution(ExecutionError::InvalidOperation(
            format!(
                "Invalid System State Transition: Current: {}, Next: {}",
                currnt, next
            ),
        )));
        match (currnt.clone(), next.clone()) {
            (EventType::SystemCreated, EventType::SystemNativeFeaturesRegistered) => Ok(()),
            (_, EventType::SystemNativeFeaturesRegistered) => err,
            (EventType::SystemNativeFeaturesRegistered, EventType::SystemWorldRegistered) => Ok(()),
            (_, EventType::SystemWorldRegistered) => err,
            (EventType::SystemWorldRegistered, EventType::SystemBuiltinAgentsRegistered) => Ok(()),
            (_, EventType::SystemBuiltinAgentsRegistered) => err,
            (EventType::SystemBuiltinAgentsRegistered, EventType::SystemUserAgentsRegistered) => {
                Ok(())
            }
            (_, EventType::SystemUserAgentsRegistered) => err,
            (EventType::SystemUserAgentsRegistered, EventType::SystemStarting) => Ok(()),
            (_, EventType::SystemStarting) => err,
            (EventType::SystemStarting, EventType::SystemStarted) => Ok(()),
            (_, EventType::SystemStarted) => err,
            _ => err,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScaleStatus {
    pub base_name: String,
    pub total_count: usize,
    pub running_count: usize,
    pub agent_names: Vec<AgentName>,
}

// システム全体の状態
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub started_at: DateTime<Utc>,
    pub running: bool,
    pub uptime: Duration,
    pub agent_count: usize,
    pub running_agent_count: usize,
    pub event_queue_size: usize,
    pub event_subscribers: usize,
    pub event_capacity: usize,
}

#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub name: String,
    pub state: String,
    pub last_lifecycle_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        ast, event_registry::EventType, AnswerDef, EventHandler, Expression, HandlerBlock, Literal,
        ReactDef, RequestHandler, StateAccessPath, StateDef, StateVarDef, Statement, TypeInfo,
    };

    use super::*;
    use tokio::{test, time::sleep};

    #[test]
    async fn test_system_creation() {
        System::new(&SystemConfig::default()).await;
    }

    #[test]
    async fn test_system_shutdown() {
        let system = System::new(&SystemConfig::default()).await;
        let result = system.shutdown().await;
        sleep(Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    #[test]
    async fn test_system_emergency_shutdown() {
        let system = System::new(&SystemConfig::default()).await;
        let result = system.emergency_shutdown().await;
        sleep(Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    fn create_ping_pong_asts() -> (MicroAgentDef, MicroAgentDef) {
        let ping_ast = MicroAgentDef {
            name: "ping".to_string(),
            state: Some(StateDef {
                variables: {
                    let mut vars = HashMap::new();
                    vars.insert(
                        "received_pong".to_string(),
                        StateVarDef {
                            name: "received_pong".to_string(),
                            type_info: TypeInfo::Simple("bool".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Boolean(false))),
                        },
                    );
                    vars
                },
            }),
            react: Some(ReactDef {
                handlers: vec![EventHandler {
                    event_type: ast::EventType::Message {
                        content_type: "Start".into(),
                    },
                    parameters: vec![],
                    block: HandlerBlock {
                        statements: vec![
                            Statement::Request {
                                agent: "pong".to_string(),
                                request_type: "ping".into(),
                                parameters: vec![],
                                options: None,
                            },
                            Statement::Assignment {
                                target: Expression::StateAccess(StateAccessPath(vec![
                                    "received_pong".to_string(),
                                ])),
                                value: Expression::Literal(Literal::Boolean(true)),
                            },
                        ],
                    },
                }],
            }),
            ..Default::default()
        };

        let pong_ast = MicroAgentDef {
            name: "pong".to_string(),
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: "ping".into(),
                    parameters: vec![],
                    return_type: "bool".into(),
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::Literal(Literal::Boolean(
                            true,
                        )))],
                    },
                }],
            }),
            ..Default::default()
        };

        (ping_ast, pong_ast)
    }

    #[tokio::test]
    async fn test_system_integration() {
        let system = System::new(&SystemConfig::default()).await;

        // Ping-Pong AgentのAST作成
        let (ping_ast, pong_ast) = create_ping_pong_asts();

        // ASTの登録
        system.register_agent_ast("ping", &ping_ast).await.unwrap();
        system.register_agent_ast("pong", &pong_ast).await.unwrap();

        // エージェントのスケールアップ（各1インスタンス）
        let ping_instances = system.scale_up("ping", 1, HashMap::new()).await.unwrap();
        system.register_agent("pong").await.unwrap();
        system.start_agent("pong").await.unwrap();

        // 起動待機
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Pingエージェントに開始イベントを送信
        system
            .send_event(Event {
                event_type: EventType::Message {
                    content_type: "Start".into(),
                },
                ..Default::default()
            })
            .await
            .unwrap();

        // 結果の確認（適切な待機時間を入れる）
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 最初のPingインスタンスの状態を確認
        let state = system
            .get_agent_state(&ping_instances[0], "received_pong")
            .await
            .unwrap();
        assert_eq!(state, Value::Boolean(true).into());
    }
}

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::str::FromStr;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::{
    sync::{broadcast, RwLock},
    time::sleep,
};
use tracing::debug;
use uuid::Uuid;

use crate::agent_registry::AgentError;
use crate::config::{ProviderConfig, SecretConfig};
use crate::context::AGENT_TYPE_CUSTOM_ALL;
use crate::event_bus::EventError;
use crate::native_feature::types::FeatureError;
use crate::provider::provider::{ProviderSecret, ProviderType};
use crate::provider::provider_registry::{ProviderInstance, ProviderRegistry};
use crate::provider::types::ProviderError;
use crate::request_manager::{RequestError, RequestManager};
use crate::runtime::RuntimeError;
use crate::{
    agent_registry::AgentRegistry,
    ast_registry::AstRegistry,
    config::{AgentConfig, SystemConfig},
    eval::{context::AgentType, expression},
    event_bus::{Event, EventBus, EventReceiver, LastStatus, Value},
    event_registry::{EventInfo, EventRegistry, EventType, ParameterType},
    native_feature::{native_registry::NativeFeatureRegistry, types::NativeFeatureContext},
    runtime::RuntimeAgentData,
    ASTError, CustomEventDef, EventsDef, MicroAgentDef,
};
use crate::{ast, WorldDef};

type AgentName = String;

pub struct System {
    event_registry: Arc<RwLock<EventRegistry>>,
    event_bus: Arc<EventBus>,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    ast_registry: Arc<RwLock<AstRegistry>>,
    feature_registry: Arc<RwLock<NativeFeatureRegistry>>,
    provider_registry: Arc<RwLock<ProviderRegistry>>,
    shutdown_tx: broadcast::Sender<AgentType>, // Systemがシャットダウンシグナルを送信
    _shutdown_rx: broadcast::Receiver<AgentType>, // シャットダウンシグナルを受信
    // event request/response
    request_manager: Arc<RequestManager>,
    filtered_subscriptions: Arc<DashMap<Vec<EventType>, broadcast::Sender<Event>>>, // Vec<EventType>は Sorted　である必要がある
    // metrics
    started_at: DateTime<Utc>,
    uptime_instant: Instant,
    last_status: Arc<RwLock<LastStatus>>,
    config: Arc<RwLock<SystemConfig>>,
}

impl System {
    // System Lifecycles
    pub async fn new(config: &SystemConfig, secret_config: &SecretConfig) -> Self {
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
        let request_manager = Arc::new(RequestManager::new(
            event_bus.clone(),
            config.request_timeout,
        ));
        let request_manager_ref = request_manager.clone();
        let mut event_rx = event_bus.subscribe().0;
        let filtered_subscriptions = Arc::new(DashMap::new());
        let provider_registry = Arc::new(RwLock::new(
            ProviderRegistry::new(
                config.provider_configs.clone(),
                secret_config.clone(),
                event_bus.clone(),
            )
            .await,
        ));

        // Receive response.
        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                if event.event_type.is_response() {
                    debug!(
                        "Recv system response: request_id: {:?}",
                        event.event_type.request_id()
                    );
                    let _ = request_manager_ref.handle_event(&event);
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
            provider_registry,
            shutdown_tx,
            _shutdown_rx,
            request_manager,
            filtered_subscriptions,
            started_at,
            uptime_instant,
            last_status,
            config: Arc::new(RwLock::new(config.clone())),
        }
    }

    pub async fn parse_dsl(&self, dsl: &str) -> SystemResult<ast::Root> {
        self.ast_registry
            .read()
            .await
            .create_ast_from_dsl(dsl)
            .await
            .map_err(SystemError::from)
    }

    #[tracing::instrument(skip(self, root))]
    pub async fn initialize(&mut self, root: ast::Root) -> SystemResult<()> {
        // call all registration methods
        self.register_native_features().await?;
        self.register_providers().await?;
        self.register_world(&root.world_def).await?;
        self.register_builtin_agents().await?;
        self.register_initial_user_agents(root.micro_agent_defs)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn register_native_features(&mut self) -> SystemResult<()> {
        debug!("started");

        let complete_state = EventType::SystemNativeFeaturesRegistered;
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            complete_state.clone(),
        )?;

        let registry = self.feature_registry.write().await;
        registry.register().await?;
        self.update_system_status(complete_state).await;
        debug!("ended");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn register_providers(&self) -> SystemResult<()> {
        let complete_state = EventType::SystemProvidersRegistered;
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            complete_state.clone(),
        )?;

        let registry = self.provider_registry.write().await;
        registry.register_providers().await?;
        self.update_system_status(complete_state).await;
        Ok(())
    }

    #[tracing::instrument(skip(self, world_def))]
    pub async fn register_world(&self, world_def: &Option<WorldDef>) -> SystemResult<()> {
        debug!("started");
        let complete_state = EventType::SystemWorldRegistered;
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            complete_state.clone(),
        )?;

        let world_def = if let Some(def) = world_def {
            def.clone()
        } else {
            let register = self.ast_registry.read().await;
            let def = register.create_world_ast();
            drop(register);
            def
        };

        let (agent_def, event_defs): (MicroAgentDef, EventsDef) = world_def.into();
        let name = AgentType::World.to_string();
        self.register_agent_ast(&name, &agent_def).await?;
        self.register_agent(&name).await?;

        for event_def in event_defs.events {
            self.register_event_ast(event_def).await?;
        }

        self.update_system_status(complete_state).await;
        debug!("ended");
        Ok(())
    }

    // ビルトインエージェントの登録処理
    #[tracing::instrument(skip(self))]
    pub async fn register_builtin_agents(&self) -> SystemResult<()> {
        debug!("started");

        let complete_state = EventType::SystemBuiltinAgentsRegistered;
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            complete_state.clone(),
        )?;
        let registry = self.ast_registry().read().await;
        let builtin_defs = registry
            .create_builtin_agent_asts(&self.config.read().await.agent_config)
            .await?;
        drop(registry);

        for builtin in builtin_defs {
            debug!("builtin started: {}", builtin.name);
            self.register_agent_ast(&builtin.name, &builtin).await?;
            self.register_agent(&builtin.name).await?;
            debug!("builtin ended: {}", builtin.name);
        }

        self.update_system_status(complete_state).await;
        debug!("ended");
        Ok(())
    }

    #[tracing::instrument(skip(self, micro_agent_defs))]
    pub async fn register_initial_user_agents(
        &self,
        micro_agent_defs: Vec<MicroAgentDef>,
    ) -> SystemResult<()> {
        debug!("started");
        let complete_state = EventType::SystemUserAgentsRegistered;
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            complete_state.clone(),
        )?;

        for agent_def in micro_agent_defs {
            self.register_agent_ast(&agent_def.name, &agent_def).await?;
            self.register_agent(&agent_def.name).await?;
        }
        self.update_system_status(complete_state).await;
        debug!("ended");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn start(&self) -> SystemResult<()> {
        Self::check_start_transition(
            self.last_status.read().await.last_event_type.clone(),
            EventType::SystemStarting,
        )?;
        self.update_system_status(EventType::SystemStarting).await;

        self.start_native_features().await?;

        self.start_providers().await?;

        self.start_world().await?;
        self.start_builtin_agents().await?;
        self.start_users_agents().await?;

        self.update_system_status(EventType::SystemStarted).await;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn start_native_features(&self) -> SystemResult<()> {
        let registry = self.feature_registry.write().await;
        registry.start().await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn start_providers(&self) -> SystemResult<()> {
        let registry = self.provider_registry.write().await;
        // No need to start, just check health
        registry.check_providers_health().await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn start_world(&self) -> SystemResult<()> {
        self.start_agent(&AgentType::World.to_string()).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn start_builtin_agents(&self) -> SystemResult<()> {
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

    #[tracing::instrument(skip(self))]
    async fn start_users_agents(&self) -> SystemResult<()> {
        let registry = self.agent_registry().read().await;
        let agent_names = registry
            .agent_names_by_types(vec![AgentType::User, AgentType::Custom("All".to_string())]);
        drop(registry);

        for agent_name in agent_names {
            self.start_agent(&agent_name).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn shutdown(&self) -> SystemResult<()> {
        let shutdown_started = Instant::now();
        self.update_system_status(EventType::SystemStopping).await;
        let shutdown_sequence = AgentRegistry::agent_shutdown_sequence();
        let config = self.config.read().await;
        let timeout = config.shutdown_timeout;
        drop(config);
        // シャットダウンシグナルを送信
        self.shutdown_tx
            .send(AgentType::Custom(AGENT_TYPE_CUSTOM_ALL.to_string()))
            .expect("Failed to send shutdown signal");
        // Agent のシャットダウン
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
        // Provider のシャットダウン
        let registry = self.provider_registry.write().await;
        registry.shutdown().await?;

        // Native Feature のシャットダウン
        let registry = self.feature_registry.write().await;
        registry.shutdown().await?;

        self.update_system_status(EventType::SystemStopped).await;
        Ok(())
    }

    fn check_shutdown_timeout(&self, shutdown_started: Instant, timeout: Duration) -> bool {
        shutdown_started.elapsed() > timeout
    }

    pub async fn emergency_shutdown(&self) -> SystemResult<()> {
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
    ) -> SystemResult<()> {
        self.ast_registry
            .write()
            .await
            .register_agent_ast(agent_name, ast)
            .await
            .map_err(SystemError::from)
    }

    pub async fn get_agent_ast(&self, _agent_name: &str) -> SystemResult<Arc<MicroAgentDef>> {
        self.ast_registry
            .read()
            .await
            .get_agent_ast(_agent_name)
            .await
            .map_err(SystemError::from)
    }

    pub async fn list_agent_asts(&self) -> SystemResult<Vec<String>> {
        Ok(self.ast_registry.read().await.list_agent_asts().await)
    }

    pub async fn register_event_ast(&self, event_def: CustomEventDef) -> SystemResult<()> {
        let name = event_def.name.to_string();
        let parameters: HashMap<String, ParameterType> = event_def
            .parameters
            .iter()
            .map(|p| (p.name.clone(), ParameterType::from(p.type_info.clone())))
            .collect();
        let mut registry = self.event_registry.write().await;
        registry
            .register_custom_event(name, parameters)
            .map_err(SystemError::from)
    }

    pub async fn get_event(&self, name: &str) -> SystemResult<EventInfo> {
        let event_type = if let Ok(event_type) = EventType::from_str(name) {
            event_type
        } else {
            EventType::Custom(name.to_string())
        };
        let registry = self.event_registry.read().await;
        registry
            .get_event_info(&event_type)
            .ok_or(EventError::NotFound(name.to_string()))
            .map_err(SystemError::from)
    }

    pub async fn scale_up(
        &self,
        name: &str,
        count: usize,
        _metadata: HashMap<String, Value>,
    ) -> SystemResult<Vec<String>> {
        let request_id = Uuid::new_v4().to_string();

        // ASTの存在確認
        let registry = self.ast_registry.read().await;
        let ast_def = registry.get_agent_ast(name).await?;

        let mut created_agents = Vec::with_capacity(count);

        // ScaleManagerAgent へのリクエストを送信
        let count = if count == 0 {
            let got = self
                .agent_registry
                .read()
                .await
                .agent_state("scale_manager", "max_instances_per_agent")
                .await
                .ok_or(SystemError::ScaleManagerNotFound {
                    agent_name: "scale_manager".to_string(),
                })?;
            match got {
                expression::Value::Integer(i) => i as usize,
                _ => 0,
            }
        } else {
            count
        };

        let primary = self
            .provider_registry
            .read()
            .await
            .get_primary_provider()
            .await
            .map_err(SystemError::from)?;

        let providers = self.provider_registry.read().await.get_providers().clone();

        // 指定された数だけエージェントを作成
        for i in 0..count {
            let agent_name = format!("{}-{}-{}", name, request_id, i);
            let agent_def = ast_def.clone();

            let agent_data = Arc::new(
                RuntimeAgentData::new(
                    &agent_def,
                    &self.event_bus(),
                    AgentConfig::default(),
                    primary.clone(),
                    providers.clone(),
                )
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
    ) -> SystemResult<()> {
        let target_agent_names = self.find_agents_by_base_name(name).await;
        // 削除対象が足りない場合はエラー
        if target_agent_names.len() < count {
            Err(SystemError::ScalingNotEnoughAgents {
                base_name: name.to_string(),
                required: count,
                current: target_agent_names.len(),
            })?;
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
    pub async fn get_scale_status(&self, name: &str) -> SystemResult<ScaleStatus> {
        let agent_names = self.find_agents_by_base_name(name).await;

        let registry = self.agent_registry.read().await;

        Ok(ScaleStatus {
            base_name: name.to_string(),
            total_count: agent_names.len(),
            running_count: agent_names
                .iter()
                .filter(|name| registry.is_agent_running(name))
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
    pub async fn register_agent(&self, agent_name: &str) -> SystemResult<()> {
        debug!("register_agent: {}", agent_name);
        let ast_registry = self.ast_registry.read().await;
        let agent_def = ast_registry.get_agent_ast(agent_name).await?;
        drop(ast_registry);

        let providers = self.provider_registry.read().await.get_providers().clone();

        // プライマリプロバイダーの取得
        let primary = self
            .provider_registry
            .read()
            .await
            .get_primary_provider()
            .await
            .map_err(SystemError::from)?;

        let runtime = Arc::new(
            RuntimeAgentData::new(
                &agent_def,
                &self.event_bus,
                AgentConfig::default(),
                primary,
                providers,
            )
            .await?,
        );
        let agent_registry = self.agent_registry.write().await;
        agent_registry
            .register_agent(agent_name, runtime, &self.event_bus)
            .await?;
        drop(agent_registry);
        Ok(())
    }
    pub async fn start_agent(&self, agent_name: &str) -> SystemResult<()> {
        let registry = self.agent_registry.read().await;
        registry
            .run_agent(agent_name, self.event_bus.clone())
            .await
            .map_err(SystemError::from)
    }

    pub async fn stop_agent(&self, agent_name: &str) -> SystemResult<()> {
        let registry = self.agent_registry.read().await;
        registry
            .shutdown_agent(agent_name, None)
            .await
            .map_err(SystemError::from)
    }

    pub async fn restart_agent(&self, agent_name: &str) -> SystemResult<()> {
        let registry = self.agent_registry.read().await;
        registry.shutdown_agent(agent_name, None).await?;
        registry
            .run_agent(agent_name, self.event_bus.clone())
            .await
            .map_err(SystemError::from)
    }

    /// Send/Receive events
    pub async fn send_event(&self, event: Event) -> SystemResult<()> {
        self.event_bus
            .publish(event)
            .await
            .map_err(SystemError::from)
    }

    pub async fn send_request(&self, event: Event) -> SystemResult<Value> {
        let request_id = match event.event_type.clone() {
            EventType::Request { request_id, .. } => request_id,
            _ => {
                return Err(SystemError::UnsupportedRequest {
                    request_type: event.event_type.to_string(),
                });
            }
        };
        debug!("request_id: {}", request_id);
        let event = self
            .request_manager
            .request(&event)
            .await
            .map_err(SystemError::from)?;
        Ok(event.response_value())
    }

    pub async fn get_agent_state(
        &self,
        agent_name: &str,
        key: &str,
    ) -> SystemResult<expression::Value> {
        let registry = self.agent_registry.read().await;
        registry
            .agent_state(agent_name, key)
            .await
            .ok_or(AgentError::AgentNotFound {
                agent_id: agent_name.to_string(),
            })
            .map_err(SystemError::from)
    }

    /// イベントの購読
    pub async fn subscribe_events(
        &self,
        event_types: Vec<EventType>,
    ) -> SystemResult<EventReceiver> {
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
    pub async fn get_system_status(&self) -> SystemResult<SystemStatus> {
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
    pub async fn get_agent_status(&self, agent_name: &str) -> SystemResult<AgentStatus> {
        let registry = self.agent_registry.read().await;
        let agent_status =
            registry
                .agent_status(agent_name)
                .await
                .ok_or_else(|| AgentError::AgentNotFound {
                    agent_id: agent_name.to_string(),
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

    /// provider management
    pub async fn register_provider(
        &self,
        name: &str,
        provider_type: ProviderType,
        config: ProviderConfig,
        secret: ProviderSecret,
    ) -> SystemResult<()> {
        let registry = self.provider_registry.write().await;
        let provider = registry.create_provider(&provider_type).await?;
        registry
            .register_provider_with_config(name, provider, &config, &secret)
            .await
            .map_err(SystemError::from)
    }

    pub async fn get_provider(&self, name: &str) -> SystemResult<Arc<ProviderInstance>> {
        let registry = self.provider_registry.read().await;
        registry.get_provider(name).await.map_err(SystemError::from)
    }

    pub async fn set_primary_provider(&self, name: &str) -> SystemResult<()> {
        let registry = self.provider_registry.write().await;
        registry.set_default_provider(name).await?;
        Ok(())
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

    pub fn ast_registry(&self) -> &Arc<RwLock<AstRegistry>> {
        &self.ast_registry
    }

    fn check_start_transition(current: EventType, next: EventType) -> SystemResult<()> {
        let err = Err(SystemError::InvalidStateTransition {
            current: current.to_string(),
            wanted: next.to_string(),
        });
        match (current.clone(), next.clone()) {
            (EventType::SystemCreated, EventType::SystemNativeFeaturesRegistered) => Ok(()),
            (_, EventType::SystemNativeFeaturesRegistered) => err,
            (EventType::SystemNativeFeaturesRegistered, EventType::SystemProvidersRegistered) => {
                Ok(())
            }
            (_, EventType::SystemProvidersRegistered) => err,
            (EventType::SystemProvidersRegistered, EventType::SystemWorldRegistered) => Ok(()),
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

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Parse error: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("Event error: {0}")]
    Event(#[from] EventError),
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    #[error("AST error: {0}")]
    Ast(#[from] ASTError),
    #[error("Feature error: {0}")]
    Feature(#[from] FeatureError),
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    #[error("Request error: {0}")]
    Request(#[from] RequestError),
    #[error("Scaling not enough agents: {base_name}, required: {required}, current: {current}")]
    ScalingNotEnoughAgents {
        base_name: String,
        required: usize,
        current: usize,
    },
    #[error("ScaleManager not found: {agent_name}")]
    ScaleManagerNotFound { agent_name: String },
    #[error("Invalid state transition: {current:?} -> {wanted:?}")]
    InvalidStateTransition { current: String, wanted: String },
    #[error("Unsupported request: {request_type}")]
    UnsupportedRequest { request_type: String },

    #[error("Event Receive response failed: {message}")]
    ReceiveResponseFailed { request_id: String, message: String },

    #[error("Event Receive response timeout: {request_id}")]
    ReceiveResponseTimeout {
        request_id: String,
        timeout_secs: u64,
        message: String,
    },
}

pub type SystemResult<T> = Result<T, SystemError>;

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
        System::new(&SystemConfig::default(), &SecretConfig::default()).await;
    }

    #[test]
    async fn test_system_shutdown() {
        let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;
        let result = system.shutdown().await;
        sleep(Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    #[test]
    async fn test_system_emergency_shutdown() {
        let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;
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
        let default_name = "default";
        let mut system_config = SystemConfig::default();
        system_config.provider_configs.primary_provider = Some(default_name.to_string());

        let secret_config = SecretConfig::default();
        let system = System::new(&system_config, &secret_config).await;

        let provider_config = ProviderConfig {
            provider_type: ProviderType::OpenAIAssistant,
            name: default_name.to_string(),
            ..Default::default()
        };

        system
            .register_provider(
                default_name,
                ProviderType::OpenAIAssistant,
                provider_config,
                ProviderSecret::default(),
            )
            .await
            .unwrap();

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

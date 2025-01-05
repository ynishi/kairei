use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use futures::future::BoxFuture;
use futures::{stream::SelectAll, Stream};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::debug;

use crate::config::AgentConfig;
use crate::eval::context::{AgentInfo, AgentType, ExecutionContext, StateAccessMode};
use crate::eval::evaluator::Evaluator;
use crate::eval::expression;
use crate::event_bus::{ErrorEvent, Event, EventBus, LastStatus, Value};
use crate::event_registry::{EventType, LifecycleEvent};
use crate::{
    EventHandler, ExecutionError, HandlerBlock, HandlerError, MicroAgentDef, RequestHandler,
    RuntimeError, RuntimeResult,
};

// ハンドラの型
type ObserveHandler = Box<dyn Fn(&Event) -> BoxFuture<'static, RuntimeResult<()>> + Send + Sync>;
type AnswerHandler = Box<dyn Fn(&Event) -> BoxFuture<'static, RuntimeResult<()>> + Send + Sync>;
type ReactHandler = Box<dyn Fn(&Event) -> BoxFuture<'static, RuntimeResult<()>> + Send + Sync>;
type LifecycleHandler = Box<dyn Fn() -> BoxFuture<'static, RuntimeResult<()>> + Send + Sync>;

// 並行処理のためのTrait
#[async_trait]
pub trait RuntimeAgent: Send + Sync {
    fn name(&self) -> String;
    fn agent_type(&self) -> AgentType;
    async fn status(&self) -> LastStatus;
    async fn state(&self, key: &str) -> Option<expression::Value>;
    async fn run(&self, shutdown_rx: broadcast::Receiver<AgentType>) -> RuntimeResult<()>;
    async fn shutdown(&self) -> RuntimeResult<()> {
        // simply cleanup
        if let Err(e) = self.cleanup().await {
            self.handle_runtime_error(e).await;
            return Ok(());
        }
        Ok(())
    }
    async fn cleanup(&self) -> RuntimeResult<()> {
        self.handle_lifecycle_event(&LifecycleEvent::OnDestroy)
            .await?;
        Ok(())
    }
    async fn handle_lifecycle_event(&self, event: &LifecycleEvent) -> RuntimeResult<()>;

    async fn handle_runtime_error(&self, error: RuntimeError);
}

// MicroAgentの実行時表現
pub struct RuntimeAgentData {
    name: String,
    ast: MicroAgentDef,
    observe_handlers: DashMap<String, ObserveHandler>,
    answer_handlers: DashMap<String, AnswerHandler>,
    react_handlers: DashMap<String, ReactHandler>,
    lifecycle_handlers: DashMap<LifecycleEvent, LifecycleHandler>,
    pub evaluator: Arc<Evaluator>,
    pub base_context: Arc<ExecutionContext>, // 基本コンテキスト
    event_bus: Arc<EventBus>,
    private_shutdown_start_tx: broadcast::Sender<()>, // 個別シャットダウン開始用
    private_shutdown_end_tx: broadcast::Sender<()>,   // 個別シャットダウン完了用
    last_status: RwLock<LastStatus>,
}

#[derive(Debug)]
pub enum StreamMessage {
    Event(Event),
    ErrorEvent(ErrorEvent),
    SystemShutdown,
    PrivateShutdown,
}

#[async_trait]
impl RuntimeAgent for RuntimeAgentData {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn agent_type(&self) -> AgentType {
        self.base_context.agent_info().agent_type.clone()
    }
    async fn status(&self) -> LastStatus {
        self.last_status.read().await.clone()
    }

    async fn state(&self, key: &str) -> Option<expression::Value> {
        self.base_context.get_state(key).await.ok()
    }
    async fn run(&self, shutdown_rx: broadcast::Receiver<AgentType>) -> RuntimeResult<()> {
        self.update_last_status(EventType::AgentStarting).await?;

        // state blockを評価して初期値を設定
        if let Some(state_def) = &self.ast.state {
            for (name, var_def) in &state_def.variables {
                if let Some(initial) = &var_def.initial_value {
                    let value = self
                        .evaluator
                        .eval_expression(initial, self.base_context.clone())
                        .await
                        .map_err(|e| {
                            RuntimeError::Execution(ExecutionError::EvaluationFailed(format!(
                                "Failed to evaluate initial value for variable {}: {}",
                                name, e
                            )))
                        })?;
                    self.base_context
                        .set_state(name.as_str(), value)
                        .map_err(|e| {
                            RuntimeError::Execution(ExecutionError::EvaluationFailed(format!(
                                "Failed to set initial value for variable {}: {}",
                                name, e
                            )))
                        })?;
                }
            }
        }

        let (event_rx, error_rx) = self.event_bus.subscribe();
        let private_shutdown_rx = self.private_shutdown_start_tx.subscribe();

        self.handle_lifecycle_event(&LifecycleEvent::OnInit).await?;

        // イベントストリームの変換
        let event_stream = BroadcastStream::new(event_rx.receiver).map(|e| {
            tracing::debug!("Event received");
            match e {
                Ok(event) => Ok(StreamMessage::Event(event)),
                Err(_) => Err(()),
            }
        });

        // エラーストリームの変換
        let error_stream = BroadcastStream::new(error_rx.receiver).map(|e| {
            tracing::debug!("Error event received");
            match e {
                Ok(error) => Ok(StreamMessage::ErrorEvent(error)),
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
        let mut streams: SelectAll<Pin<Box<dyn Stream<Item = Result<StreamMessage, ()>> + Send>>> =
            SelectAll::new();
        streams.push(Box::pin(event_stream));
        streams.push(Box::pin(error_stream));
        streams.push(Box::pin(system_shutdown_stream));
        streams.push(Box::pin(private_shutdown_stream));

        self.update_last_status(EventType::AgentStarted).await?;

        while let Some(Ok(message)) = streams.next().await {
            match message {
                StreamMessage::Event(event) => {
                    debug!("Event received: {:?}", event);
                    self.handle_event(&event).await?;
                }
                StreamMessage::ErrorEvent(error) => {
                    tracing::error!("Error received in agent {}: {:?}", self.name, error);
                }

                StreamMessage::SystemShutdown => {
                    tracing::info!("Agent {} received shutdown signal", self.name);
                    break;
                }
                StreamMessage::PrivateShutdown => {
                    tracing::info!("Agent {} received single shutdown signal", self.name);
                    break;
                }
            }
        }

        self.update_last_status(EventType::AgentStopping).await?;

        // クリーンアップ処理
        self.cleanup().await?;

        self.private_shutdown_end_tx.send(()).map_err(|e| {
            RuntimeError::Execution(ExecutionError::ShutdownFailed {
                agent_name: self.name.clone(),
                message: e.to_string(),
            })
        })?;

        // AgentStoppedイベントを発行
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
        self.update_last_status(EventType::AgentStopped).await?;
        Ok(())
    }

    async fn shutdown(&self) -> RuntimeResult<()> {
        self.private_shutdown_start_tx.send(()).map_err(|e| {
            RuntimeError::Execution(ExecutionError::ShutdownFailed {
                agent_name: self.name.clone(),
                message: e.to_string(),
            })
        })?;
        self.private_shutdown_end_tx
            .subscribe()
            .recv()
            .await
            .map_err(|e| {
                RuntimeError::Execution(ExecutionError::ShutdownFailed {
                    agent_name: self.name.clone(),
                    message: e.to_string(),
                })
            })?;
        Ok(())
    }

    // シャットダウン時のクリーンアップ処理
    async fn cleanup(&self) -> RuntimeResult<()> {
        // 1. OnDestroy処理の実行
        self.handle_lifecycle_event(&LifecycleEvent::OnDestroy)
            .await?;

        // 2. 待機中のリクエストをキャンセル
        self.base_context
            .cancel_pending_requests()
            .await
            .map_err(|e| {
                RuntimeError::Execution(ExecutionError::CleanUpFailed {
                    agent_name: self.name.clone(),
                    message: e.to_string(),
                })
            })?;

        // 3. 状態の保存や他のリソースのクリーンアップ
        // TODO: 必要に応じて実装

        Ok(())
    }

    async fn handle_lifecycle_event(&self, event: &LifecycleEvent) -> RuntimeResult<()> {
        if let Some(handler) = self.lifecycle_handlers.get(event) {
            return handler().await;
        }
        Ok(())
    }

    async fn handle_runtime_error(&self, _error: RuntimeError) {
        todo!()
    }
}

impl RuntimeAgentData {
    pub async fn new(
        agent_def: &MicroAgentDef,
        event_bus: &Arc<EventBus>,
        config: AgentConfig,
    ) -> RuntimeResult<Self> {
        let agent_name = agent_def.name.clone();
        let agent_info = AgentInfo {
            agent_name: agent_name.clone(),
            agent_type: AgentType::Custom(agent_name),
            created_at: Utc::now(),
        };

        let evaluator = Arc::new(Evaluator::new());

        let base_context = Arc::new(ExecutionContext::new(
            event_bus.clone(),
            agent_info,
            StateAccessMode::ReadWrite,
            config.context,
        ));

        let last_status = RwLock::new(LastStatus {
            last_event_type: EventType::AgentCreated,
            last_event_time: Utc::now(),
        });

        let mut new_self = Self {
            name: agent_def.name.clone(),
            ast: agent_def.clone(),
            observe_handlers: DashMap::new(),
            answer_handlers: DashMap::new(),
            react_handlers: DashMap::new(),
            lifecycle_handlers: DashMap::new(),
            base_context,
            evaluator,
            event_bus: event_bus.clone(),
            private_shutdown_start_tx: broadcast::channel(1).0,
            private_shutdown_end_tx: broadcast::channel(1).0,
            last_status,
        };

        new_self.register_handlers_from_ast(agent_def)?;
        Ok(new_self)
    }

    pub fn register_handlers_from_ast(&mut self, agent_def: &MicroAgentDef) -> RuntimeResult<()> {
        if let Some(observe_def) = &agent_def.observe {
            for handler in observe_def.handlers.iter() {
                let created = Self::create_observe_handler(
                    self.evaluator.clone(),
                    Arc::new(handler.clone()),
                    self.base_context.clone(),
                );
                self.register_observe(&handler.event_type.to_string(), created);
            }
        }
        if let Some(answer_def) = &agent_def.answer {
            for handler in answer_def.handlers.iter() {
                let created = Self::create_answer_handler(
                    self.evaluator.clone(),
                    Arc::new(handler.clone()),
                    self.base_context.clone(),
                );
                self.register_answer(&handler.request_type.to_string(), created);
            }
        }
        if let Some(react_def) = &agent_def.react {
            for handler in react_def.handlers.iter() {
                let created = Self::create_react_handler(
                    self.evaluator.clone(),
                    Arc::new(handler.clone()),
                    self.base_context.clone(),
                );
                self.register_react(&handler.event_type.to_string(), created);
            }
        }
        if let Some(lifecycle_def) = &agent_def.lifecycle {
            if let Some(on_init) = &lifecycle_def.on_init {
                let created = Self::create_lifecycle_handler(
                    self.evaluator.clone(),
                    Arc::new(on_init.clone()),
                    self.base_context.clone(),
                );
                self.register_lifecycle(LifecycleEvent::OnInit, created);
            }
            if let Some(on_destroy) = &lifecycle_def.on_destroy {
                let created = Self::create_lifecycle_handler(
                    self.evaluator.clone(),
                    Arc::new(on_destroy.clone()),
                    self.base_context.clone(),
                );
                self.register_lifecycle(LifecycleEvent::OnDestroy, created);
            }
        }
        Ok(())
    }

    // observe ハンドラの登録
    pub fn register_observe(&mut self, event_type: &str, handler: ObserveHandler) {
        self.observe_handlers
            .insert(event_type.to_string(), handler);
    }

    pub fn create_observe_handler(
        evaluator: Arc<Evaluator>,
        event_handler: Arc<EventHandler>,
        base_context: Arc<ExecutionContext>,
    ) -> ObserveHandler {
        Box::new(move |event| {
            let evaluator = evaluator.clone();
            let handler = event_handler.clone();
            let base = base_context.clone();
            let event = event.clone();

            Box::pin(async move {
                let context = base.fork(Some(StateAccessMode::ReadWrite)).await;
                let context_ref = Arc::new(context);

                for param in &handler.parameters {
                    if let Some(value) = event.parameters.get(&param.name) {
                        context_ref
                            .set_variable(&param.name, expression::Value::from(value.clone()))
                            .await
                            .unwrap();
                    }
                }

                evaluator
                    .eval_handler_block(&handler.block, context_ref)
                    .await
                    .map(|_| ())
                    .map_err(|e| {
                        RuntimeError::Execution(ExecutionError::EvaluationFailed(format!(
                            "Failed to evaluate observe handler: {}",
                            e
                        )))
                    })
            })
        })
    }

    // answer ハンドラの登録
    pub fn register_answer(&mut self, request_type: &str, handler: AnswerHandler) {
        self.answer_handlers
            .insert(request_type.to_string(), handler);
    }

    pub fn create_answer_handler(
        evaluator: Arc<Evaluator>,
        event_handler: Arc<RequestHandler>,
        base_context: Arc<ExecutionContext>,
    ) -> AnswerHandler {
        Box::new(move |event| {
            let evaluator = evaluator.clone();
            let handler = event_handler.clone();
            let base = base_context.clone();
            let event = event.clone();
            let event_type = event.event_type.clone();

            Box::pin(async move {
                let context = base.fork(Some(StateAccessMode::ReadOnly)).await;
                let context_ref = Arc::new(context);

                for param in &handler.parameters {
                    if let Some(value) = event.parameters.get(&param.name) {
                        context_ref
                            .set_variable(&param.name, expression::Value::from(value.clone()))
                            .await
                            .unwrap();
                    }
                }

                evaluator
                    .eval_answer_handler_block(&handler.block, context_ref, event_type)
                    .await
                    .map(|_| ())
                    .map_err(|e| {
                        RuntimeError::Execution(ExecutionError::EvaluationFailed(format!(
                            "Failed to evaluate answer handler: {}",
                            e
                        )))
                    })
            })
        })
    }

    // react ハンドラの登録
    pub fn register_react(&mut self, event_type: &str, handler: ReactHandler) {
        self.react_handlers.insert(event_type.to_string(), handler);
    }

    pub fn create_react_handler(
        evaluator: Arc<Evaluator>,
        event_handler: Arc<EventHandler>,
        base_context: Arc<ExecutionContext>,
    ) -> ReactHandler {
        Box::new(move |event| {
            let evaluator = evaluator.clone();
            let handler = event_handler.clone();
            let base = base_context.clone();
            let event = event.clone();

            Box::pin(async move {
                let context = base.fork(Some(StateAccessMode::ReadWrite)).await;
                let context_ref = Arc::new(context);

                for param in &handler.parameters {
                    if let Some(value) = event.parameters.get(&param.name) {
                        context_ref
                            .set_variable(&param.name, expression::Value::from(value.clone()))
                            .await
                            .unwrap();
                    }
                }

                evaluator
                    .eval_handler_block(&handler.block, context_ref)
                    .await
                    .map(|_| ())
                    .map_err(|e| {
                        RuntimeError::Execution(ExecutionError::EvaluationFailed(format!(
                            "Failed to evaluate react handler: {}",
                            e
                        )))
                    })
            })
        })
    }

    pub fn register_lifecycle(&mut self, event: LifecycleEvent, handler: LifecycleHandler) {
        self.lifecycle_handlers.insert(event, handler);
    }

    pub fn create_lifecycle_handler(
        evaluator: Arc<Evaluator>,
        handler_block: Arc<HandlerBlock>,
        base_context: Arc<ExecutionContext>,
    ) -> LifecycleHandler {
        Box::new(move || {
            let evaluator = evaluator.clone();
            let handler_block = handler_block.clone();
            let base = base_context.clone();

            Box::pin(async move {
                let context = base.fork(Some(StateAccessMode::ReadWrite)).await;
                let context_ref = Arc::new(context);

                evaluator
                    .eval_handler_block(&handler_block, context_ref)
                    .await
                    .map(|_| ())
                    .map_err(|e| {
                        RuntimeError::Execution(ExecutionError::EvaluationFailed(format!(
                            "Failed to evaluate lifecycle handler {}",
                            e
                        )))
                    })
            })
        })
    }

    // イベントの処理
    async fn handle_event(&self, event: &Event) -> RuntimeResult<()> {
        debug!("Handle event in Runtime: {:?}", event);
        match &event.event_type {
            EventType::Request {
                request_type,
                responder,
                ..
            } => {
                if responder == &self.name {
                    if let Some(handler) = self.answer_handlers.get(request_type) {
                        handler(event).await
                    } else {
                        Err(RuntimeError::Handler(HandlerError::NotFound {
                            handler_type: "answer".to_string(),
                            name: request_type.clone(),
                        }))
                    }
                } else {
                    // not for me
                    Ok(())
                }
            }

            // 通常のメッセージ(Ok, Err)とカスタムイベント
            EventType::Message { .. } | EventType::Failure { .. } | EventType::Custom(_) => {
                self.handle_normal_event(event).await
            }

            // システムイベント
            EventType::Tick
            | EventType::StateUpdated { .. }
            | EventType::AgentCreated
            | EventType::AgentAdded
            | EventType::AgentRemoved
            | EventType::AgentStarting
            | EventType::AgentStarted
            | EventType::AgentStopping
            | EventType::AgentStopped
            | EventType::SystemCreated
            | EventType::SystemWorldRegistered
            | EventType::SystemBuiltinAgentsRegistered
            | EventType::SystemUserAgentsRegistered
            | EventType::SystemStarting
            | EventType::SystemStarted
            | EventType::SystemStopping
            | EventType::SystemStopped => self.handle_system_event(event).await,
            // レスポンスは直接evaluatorで処理する
            EventType::ResponseSuccess { .. } => Ok(()),
            EventType::ResponseFailure { .. } => Ok(()),
            // 確実にバリアントを処理するため _ => Ok(()) は使用しない
        }
    }

    async fn handle_normal_event(&self, event: &Event) -> RuntimeResult<()> {
        debug!(
            "Normal event received: {:?}, {}, {:?}",
            event,
            &event.event_type.to_string(),
            self.react_handlers
                .iter()
                .map(|e| e.key().clone())
                .collect::<Vec<String>>()
        );
        // Observe処理
        if let Some(handler) = self.observe_handlers.get(&event.event_type.to_string()) {
            handler(event).await?;
        }

        // React処理
        if let Some(handler) = self.react_handlers.get(&event.event_type.to_string()) {
            handler(event).await?;
        }

        Ok(())
    }

    async fn handle_system_event(&self, event: &Event) -> RuntimeResult<()> {
        // システムイベントは主にObserveで処理
        if let Some(handler) = self.observe_handlers.get(&event.event_type.to_string()) {
            handler(event).await?;
        }
        Ok(())
    }

    async fn update_last_status(&self, event_type: EventType) -> RuntimeResult<()> {
        let mut lock = self.last_status.write().await;
        lock.last_event_type = event_type.clone();
        lock.last_event_time = Utc::now();
        let last_status = lock.clone(); // clone to avoid lifetime issue in the closure
        drop(lock); // release the lock before calling publish to avoid deadlocks
        let mut event = Event::from(last_status);
        event
            .parameters
            .insert("agent_id".to_string(), Value::String(self.name.clone()));
        self.event_bus.publish(event).await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::{sync::Mutex, time::Duration};

    use uuid::Uuid;

    use crate::{
        ast, AnswerDef, BinaryOperator, Expression, Literal, ObserveDef, Parameter, ReactDef,
        RequestType, StateAccessPath, StateDef, StateVarDef, Statement, TypeInfo,
    };

    use super::*;

    #[tokio::test]
    async fn test_runtime_agent() {
        let event_bus = Arc::new(EventBus::new(20));

        // Counter AgentのASTを作成
        let counter_def = &MicroAgentDef {
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
            observe: Some(ObserveDef {
                handlers: vec![EventHandler {
                    event_type: ast::EventType::Tick,
                    parameters: vec![],
                    block: HandlerBlock {
                        statements: vec![Statement::Assignment {
                            target: Expression::StateAccess(StateAccessPath(vec![
                                "count".to_string()
                            ])),
                            value: Expression::BinaryOp {
                                op: BinaryOperator::Add,
                                left: Box::new(Expression::StateAccess(StateAccessPath(vec![
                                    "count".to_string(),
                                ]))),
                                right: Box::new(Expression::Literal(Literal::Integer(1))),
                            },
                        }],
                    },
                }],
            }),
            ..Default::default()
        };

        // RuntimeAgentを生成
        let agent = RuntimeAgentData::new(&counter_def, &event_bus, AgentConfig::default())
            .await
            .unwrap();
        let context = agent.base_context.clone();

        // 初期状態を確認
        let result = context.get_state("count").await;
        assert!(result.is_err());

        // エージェントを起動
        let shutdown_rx = broadcast::channel(1).1;
        tokio::spawn(async move {
            agent.run(shutdown_rx).await.unwrap();
        });

        // 起動直後を確認
        tokio::time::sleep(Duration::from_millis(100)).await;
        let state = context.get_state("count").await.unwrap();
        assert_eq!(state, expression::Value::Integer(0));

        // Tickイベントを送信
        event_bus
            .publish(Event {
                event_type: EventType::Tick,
                ..Default::default()
            })
            .await
            .unwrap();

        // イベント処理後を確認
        tokio::time::sleep(Duration::from_millis(100)).await;
        let state = context.get_state("count").await.unwrap();
        assert_eq!(state, expression::Value::Integer(1));
    }

    struct TestAgent {
        pub _name: String,
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
                _name: name.to_string(),
                responses,
            }
        }

        fn get_response(&self, request_id: &str) -> Value {
            let want_request_id = request_id.to_string();
            let lock = self.responses.lock().unwrap();
            let filtered = lock
                .iter()
                .filter(|e| e.event_type.request_id() == Some(&want_request_id));
            let res = filtered.last().unwrap();
            // failure will be handled by unwrap
            res.parameters.get("response").unwrap().clone()
        }
    }

    #[tokio::test]
    async fn test_answer_handler() {
        let event_bus = Arc::new(EventBus::new(20));
        let answer_def = &MicroAgentDef {
            name: "calculator".to_string(),
            state: Some(StateDef {
                variables: {
                    let mut vars = HashMap::new();
                    vars.insert(
                        "self.x".to_string(),
                        StateVarDef {
                            name: "self.x".to_string(),
                            type_info: TypeInfo::Simple("i64".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Integer(2))),
                        },
                    );
                    vars
                },
            }),
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: RequestType::Custom("add".to_string()),
                    parameters: vec![
                        Parameter {
                            name: "a".to_string(),
                            type_info: TypeInfo::Simple("i64".to_string()),
                        },
                        Parameter {
                            name: "b".to_string(),
                            type_info: TypeInfo::Simple("i64".to_string()),
                        },
                    ],
                    return_type: TypeInfo::Simple("i64".to_string()),
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![
                            Statement::Assignment {
                                target: Expression::Variable("last_result".into()),
                                value: Expression::BinaryOp {
                                    op: BinaryOperator::Add,
                                    left: Box::new(Expression::Variable("a".into())),
                                    right: Box::new(Expression::Variable("b".into())),
                                },
                            },
                            Statement::Assignment {
                                target: Expression::Variable("last_result".into()),
                                value: Expression::BinaryOp {
                                    op: BinaryOperator::Add,
                                    left: Box::new(Expression::StateAccess(StateAccessPath(vec![
                                        "self".into(),
                                        "x".into(),
                                    ]))),
                                    right: Box::new(Expression::Variable("last_result".into())),
                                },
                            },
                            Statement::Return(Expression::Variable("last_result".into())),
                        ],
                    },
                }],
            }),
            ..Default::default()
        };

        let agent = RuntimeAgentData::new(&answer_def, &event_bus, AgentConfig::default())
            .await
            .unwrap();
        let context = agent.base_context.clone();
        let shutdown_rx = broadcast::channel(1).1;
        let sender_agent = TestAgent::new("test", &event_bus);

        tokio::spawn(async move {
            agent.run(shutdown_rx).await.unwrap();
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // リクエストを送信
        let request_id = Uuid::new_v4().to_string();
        event_bus
            .publish(Event {
                event_type: EventType::Request {
                    request_type: "add".into(),
                    requester: "test".into(),
                    responder: "calculator".into(),
                    request_id: request_id.clone(),
                },
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("a".to_string(), Value::Integer(3));
                    params.insert("b".to_string(), Value::Integer(5));
                    params
                },
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let state = context.get_state("self.x").await.unwrap();
        assert_eq!(state, expression::Value::Integer(2));
        let response = sender_agent.get_response(&request_id);
        assert_eq!(response, Value::Integer(10));
    }

    #[tokio::test]
    async fn test_react_handler() {
        let event_bus = Arc::new(EventBus::new(20));
        let react_def = &MicroAgentDef {
            name: "reactor".to_string(),
            state: Some(StateDef {
                variables: {
                    let mut vars = HashMap::new();
                    vars.insert(
                        "event_count".to_string(),
                        StateVarDef {
                            name: "event_count".to_string(),
                            type_info: TypeInfo::Simple("i64".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Integer(0))),
                        },
                    );
                    vars.insert(
                        "last_value".to_string(),
                        StateVarDef {
                            name: "last_value".to_string(),
                            type_info: TypeInfo::Simple("i64".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Integer(0))),
                        },
                    );
                    vars
                },
            }),
            react: Some(ReactDef {
                handlers: vec![EventHandler {
                    event_type: ast::EventType::Message {
                        content_type: "update".into(),
                    },
                    parameters: vec![Parameter {
                        name: "value".to_string(),
                        type_info: TypeInfo::Simple("i64".to_string()),
                    }],
                    block: HandlerBlock {
                        statements: vec![
                            Statement::Assignment {
                                target: Expression::StateAccess(StateAccessPath(vec![
                                    "event_count".into(),
                                ])),
                                value: Expression::BinaryOp {
                                    op: BinaryOperator::Add,
                                    left: Box::new(Expression::StateAccess(StateAccessPath(vec![
                                        "event_count".into(),
                                    ]))),
                                    right: Box::new(Expression::Literal(Literal::Integer(1))),
                                },
                            },
                            Statement::Assignment {
                                target: Expression::StateAccess(StateAccessPath(vec![
                                    "last_value".into(),
                                ])),
                                value: Expression::Variable("value".into()),
                            },
                        ],
                    },
                }],
            }),
            ..Default::default()
        };

        let agent = RuntimeAgentData::new(&react_def, &event_bus, AgentConfig::default())
            .await
            .unwrap();
        let context = agent.base_context.clone();
        let shutdown_rx = broadcast::channel(1).1;

        tokio::spawn(async move {
            agent.run(shutdown_rx).await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // カスタムイベントを送信
        event_bus
            .publish(Event {
                event_type: EventType::Message {
                    content_type: "update".into(),
                },
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("value".to_string(), Value::Integer(42));
                    params
                },
                ..Default::default()
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let last_value = context.get_state("last_value").await.unwrap();
        assert_eq!(last_value, expression::Value::Integer(42));

        let event_count = context.get_state("event_count").await.unwrap();
        assert_eq!(event_count, expression::Value::Integer(1));
    }
}

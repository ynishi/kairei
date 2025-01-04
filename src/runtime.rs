use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use futures::future::BoxFuture;
use futures::{stream::SelectAll, Stream};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::debug;
use uuid::Uuid;

use crate::eval::context::{AgentInfo, ExecutionContext, StateAccessMode};
use crate::eval::evaluator::Evaluator;
use crate::eval::expression;
use crate::eval::statement::StatementResult;
use crate::event_bus::{self, ErrorEvent, Event, EventBus, LastStatus, Value};
use crate::event_registry::{EventType, LifecycleEvent};
use crate::{
    EventHandler, ExecutionError, HandlerBlock, HandlerError, MicroAgentDef, RequestHandler,
    RuntimeError, RuntimeResult,
};

// ハンドラの型
type ObserveHandler =
    Box<dyn Fn(&Event) -> BoxFuture<'static, Option<HashMap<String, Value>>> + Send + Sync>;
type AnswerHandler = Box<dyn Fn(&Event) -> BoxFuture<'static, RuntimeResult<Value>> + Send + Sync>;
type ReactHandler = Box<dyn Fn(&Event) -> BoxFuture<'static, Option<Vec<Event>>> + Send + Sync>;
type LifecycleHandler = Box<
    dyn Fn() -> BoxFuture<'static, RuntimeResult<(Vec<Event>, HashMap<String, Value>)>>
        + Send
        + Sync,
>;

// 並行処理のためのTrait
#[async_trait]
pub trait RuntimeAgent: Send + Sync {
    fn name(&self) -> String;
    async fn status(&self) -> LastStatus;
    async fn run(&self, shutdown_rx: broadcast::Receiver<()>) -> RuntimeResult<()>;
    async fn shutdown(&self) -> RuntimeResult<()> {
        // simply cleanup
        self.cleanup().await?;
        Ok(())
    }
    async fn cleanup(&self) -> RuntimeResult<()> {
        self.handle_lifecycle_event(&LifecycleEvent::OnDestroy)
            .await?;
        Ok(())
    }
    async fn handle_lifecycle_event(
        &self,
        event: &LifecycleEvent,
    ) -> RuntimeResult<(Vec<Event>, HashMap<String, Value>)>;
}

// MicroAgentの実行時表現
pub struct RuntimeAgentData {
    name: String,
    pub state: DashMap<String, Value>,
    observe_handlers: DashMap<String, ObserveHandler>,
    answer_handlers: DashMap<String, AnswerHandler>,
    react_handlers: DashMap<String, ReactHandler>,
    lifecycle_handlers: DashMap<LifecycleEvent, LifecycleHandler>,
    pub evaluator: Arc<Evaluator>,
    pub base_context: Arc<ExecutionContext>, // 基本コンテキスト
    event_bus: Arc<EventBus>,
    pending_requests: Arc<DashMap<String, oneshot::Sender<Event>>>,
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
    async fn status(&self) -> LastStatus {
        self.last_status.read().await.clone()
    }
    async fn run(&self, shutdown_rx: broadcast::Receiver<()>) -> RuntimeResult<()> {
        self.update_last_status(EventType::AgentStarting).await?;
        let (event_rx, error_rx) = self.event_bus.subscribe();
        let private_shutdown_rx = self.private_shutdown_start_tx.subscribe();

        let on_init = self.handle_lifecycle_event(&LifecycleEvent::OnInit).await?;
        for event in on_init.0 {
            self.event_bus.publish(event).await?;
        }
        for (key, value) in on_init.1 {
            self.state.insert(key, value);
        }

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
                    let new_events = self.handle_event(&event).await?;
                    for event in new_events {
                        self.event_bus.publish(event).await?;
                    }
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
        let (events, states) = self
            .handle_lifecycle_event(&LifecycleEvent::OnDestroy)
            .await?;
        for event in events {
            self.event_bus.publish(event).await?;
        }
        for (key, value) in states {
            self.state.insert(key, value);
        }

        // 2. 待機中のリクエストをキャンセル
        for entry in self.pending_requests.iter() {
            let request_id = entry.key();
            let (_, tx) = self.pending_requests.remove(request_id).ok_or_else(|| {
                RuntimeError::Execution(ExecutionError::InvalidPendingRequest {
                    request_id: request_id.clone(),
                })
            })?;
            let error_response = Event {
                event_type: EventType::Response {
                    request_type: "cancelled".to_string(),
                    requester: "unknown".to_string(),
                    responder: self.name.clone(),
                    request_id: request_id.clone(),
                },
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "error".to_string(),
                        Value::String("Agent shutdown".to_string()),
                    );
                    params
                },
            };
            let _ = tx.send(error_response);
        }
        self.pending_requests.clear();

        // 3. 状態の保存や他のリソースのクリーンアップ
        // TODO: 必要に応じて実装

        Ok(())
    }

    async fn handle_lifecycle_event(
        &self,
        event: &LifecycleEvent,
    ) -> RuntimeResult<(Vec<Event>, HashMap<String, Value>)> {
        if let Some(handler) = self.lifecycle_handlers.get(event) {
            return handler().await;
        }
        Ok((vec![], HashMap::new()))
    }
}

impl RuntimeAgentData {
    pub async fn new(agent_def: &MicroAgentDef, event_bus: &Arc<EventBus>) -> RuntimeResult<Self> {
        let agent_info = AgentInfo {
            agent_name: agent_def.name.clone(),
            agent_type: "".to_string(),
            created_at: Utc::now(),
        };

        let evaluator = Arc::new(Evaluator::new());

        let base_context = Arc::new(ExecutionContext::new(event_bus.clone(), agent_info));

        if let Some(state_def) = &agent_def.state {
            for (name, var_def) in &state_def.variables {
                if let Some(initial) = &var_def.initial_value {
                    let value = evaluator
                        .eval_expression(initial, base_context.clone())
                        .await
                        .map_err(|e| {
                            RuntimeError::Execution(ExecutionError::EvaluationFailed(format!(
                                "Failed to evaluate initial value for variable {}: {}",
                                name, e
                            )))
                        })?;
                    base_context.set_state(name.as_str(), value).map_err(|e| {
                        RuntimeError::Execution(ExecutionError::EvaluationFailed(format!(
                            "Failed to set initial value for variable {}: {}",
                            name, e
                        )))
                    })?;
                }
            }
        }

        let mut parameters = HashMap::new();
        parameters.insert(
            "agent_id".to_string(),
            Value::String(agent_def.name.clone()),
        );

        let last_status = RwLock::new(LastStatus {
            last_event_type: EventType::AgentCreated,
            last_event_time: Utc::now(),
        });
        let state = DashMap::new();

        Ok(Self {
            name: agent_def.name.clone(),
            state,
            observe_handlers: DashMap::new(),
            answer_handlers: DashMap::new(),
            react_handlers: DashMap::new(),
            lifecycle_handlers: DashMap::new(),
            base_context,
            evaluator,
            event_bus: event_bus.clone(),
            pending_requests: Arc::new(DashMap::new()),
            private_shutdown_start_tx: broadcast::channel(1).0,
            private_shutdown_end_tx: broadcast::channel(1).0,
            last_status,
        })
    }

    pub fn register_handlers_from_ast(&mut self, agent_def: MicroAgentDef) -> RuntimeResult<()> {
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

                let result = evaluator
                    .eval_handler_block(&handler.block, context_ref)
                    .await;

                match result {
                    Ok(statement_result) => match statement_result {
                        StatementResult::Value(value) => {
                            let event_value = event_bus::Value::from(value);
                            let mut updates = HashMap::new();
                            updates.insert("value".to_string(), event_value);
                            Some(updates)
                        }
                        _ => None,
                    },
                    Err(_) => None,
                }
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

                let result = evaluator
                    .eval_handler_block(&handler.block, context_ref)
                    .await;

                match result {
                    Ok(statement_result) => match statement_result {
                        StatementResult::Value(value) => Ok(event_bus::Value::from(value)),
                        _ => Ok(event_bus::Value::Null),
                    },
                    Err(_) => Err(RuntimeError::Execution(ExecutionError::EvaluationFailed(
                        "Failed to evaluate answer handler".to_string(),
                    ))),
                }
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

                let result = evaluator
                    .eval_handler_block(&handler.block, context_ref)
                    .await;

                match result {
                    Ok(_statement_result) => None,
                    Err(_) => None,
                }
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

                let _result = evaluator
                    .eval_handler_block(&handler_block, context_ref)
                    .await;
                Err(RuntimeError::Execution(ExecutionError::EvaluationFailed(
                    "Failed to evaluate lifecycle handler".to_string(),
                )))
            })
        })
    }

    // イベントの処理
    async fn handle_event(&self, event: &Event) -> RuntimeResult<Vec<Event>> {
        // TODO: use evaluator to evaluate expressions in event parameters
        match &event.event_type {
            EventType::Request {
                request_type,
                requester,
                responder,
                request_id,
            } => {
                if responder == &self.name {
                    if let Some(handler) = self.answer_handlers.get(request_type) {
                        // レスポンスイベントを生成して返す
                        let response = handler(event).await?;
                        let mut parameters = HashMap::new();
                        parameters.insert("response".to_string(), response);
                        let event = Event {
                            event_type: EventType::Response {
                                request_id: request_id.clone(),
                                responder: self.name.clone(),
                                requester: requester.clone(),
                                request_type: request_type.clone(),
                            },
                            parameters,
                        };
                        Ok(vec![event])
                    } else {
                        Err(RuntimeError::Handler(HandlerError::NotFound {
                            handler_type: "answer".to_string(),
                            name: request_type.clone(),
                        }))
                    }
                } else {
                    // not for me
                    Ok(vec![])
                }
            }

            EventType::Response { request_id, .. } => {
                // 待機中のリクエストがあれば、レスポンスを送信
                if let Some((_, tx)) = self.pending_requests.remove(request_id) {
                    let _ = tx.send(event.clone());
                }
                Ok(vec![])
            }

            // 通常のメッセージとカスタムイベント
            EventType::Message { .. } | EventType::Custom(_) => {
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
            | EventType::SystemStarting
            | EventType::SystemStarted
            | EventType::SystemStopping
            | EventType::SystemStopped => self.handle_system_event(event).await,
        }
    }

    // リクエスト送信と応答待ち
    pub async fn request(
        &self,
        request_type: String,
        responder: String,
        parameters: HashMap<String, Value>,
    ) -> RuntimeResult<Event> {
        let request_id = Uuid::new_v4().to_string();

        // 応答待ち用のチャネルを作成
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(request_id.clone(), tx);

        // リクエストを送信
        self.event_bus
            .publish(Event {
                event_type: EventType::Request {
                    request_type,
                    requester: self.name.clone(),
                    responder,
                    request_id: request_id.clone(),
                },
                parameters,
            })
            .await?;

        // レスポンスを待機
        match rx.await {
            Ok(response) => Ok(response),
            Err(_) => Err(RuntimeError::Execution(ExecutionError::RequestTimeout {
                request_id,
            })),
        }
    }

    async fn handle_normal_event(&self, event: &Event) -> RuntimeResult<Vec<Event>> {
        let mut new_events = Vec::new();

        // Observe処理
        if let Some(handler) = self.observe_handlers.get(&event.event_type.to_string()) {
            if let Some(updates) = handler(event).await {
                for (key, value) in updates {
                    self.state.insert(key, value);
                }
            }
        }

        // React処理
        if let Some(handler) = self.react_handlers.get(&event.event_type.to_string()) {
            if let Some(events) = handler(event).await {
                new_events.extend(events);
            }
        }

        Ok(new_events)
    }

    async fn handle_system_event(&self, event: &Event) -> RuntimeResult<Vec<Event>> {
        // システムイベントは主にObserveで処理
        if let Some(handler) = self.observe_handlers.get(&event.event_type.to_string()) {
            if let Some(updates) = handler(event).await {
                for (key, value) in updates {
                    self.state.insert(key, value);
                }
            }
        }
        Ok(vec![])
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

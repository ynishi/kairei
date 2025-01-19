use std::{collections::HashMap, time::Duration};

use crate::{eval::expression, event_registry::EventType, RetryDelay};
use chrono::{DateTime, Utc};
use thiserror::Error;
use tokio::sync::broadcast;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Event {
    pub event_type: EventType,
    pub parameters: HashMap<String, Value>,
}

impl Event {
    pub fn new(event_type: &EventType, parameters: &HashMap<String, Value>) -> Self {
        Self {
            event_type: event_type.clone(),
            parameters: parameters.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventCategory {
    System,
    Request { request_type: String },
    Response,
    Agent,
    Component,
    // 必要に応じて追加
}

impl Event {
    pub fn category(&self) -> EventCategory {
        match &self.event_type {
            EventType::Tick => EventCategory::System,
            EventType::StateUpdated { .. } => EventCategory::Agent,
            EventType::Message { .. } => EventCategory::Agent,
            EventType::Failure { .. } => EventCategory::Agent,
            EventType::Request { request_type, .. } => EventCategory::Request {
                request_type: request_type.clone(),
            },
            EventType::ResponseSuccess { .. } => EventCategory::Response,
            EventType::ResponseFailure { .. } => EventCategory::Response,
            EventType::AgentCreated => EventCategory::Agent,
            EventType::AgentAdded => EventCategory::Agent,
            EventType::AgentRemoved => EventCategory::Agent,
            EventType::AgentStarting => EventCategory::Agent,
            EventType::AgentStarted => EventCategory::Agent,
            EventType::AgentStopping => EventCategory::Agent,
            EventType::AgentStopped => EventCategory::Agent,
            EventType::SystemCreated => EventCategory::System,
            EventType::SystemNativeFeaturesRegistered => EventCategory::System,
            EventType::SystemProvidersRegistered => EventCategory::System,
            EventType::SystemWorldRegistered => EventCategory::System,
            EventType::SystemBuiltinAgentsRegistered => EventCategory::System,
            EventType::SystemUserAgentsRegistered => EventCategory::System,
            EventType::SystemStarting => EventCategory::System,
            EventType::SystemStarted => EventCategory::System,
            EventType::SystemStopping => EventCategory::System,
            EventType::SystemStopped => EventCategory::System,
            EventType::FeatureStatusUpdated { .. } => EventCategory::Component,
            EventType::FeatureFailure { .. } => EventCategory::Component,
            EventType::ProviderRegistered => EventCategory::Component,
            EventType::ProviderStatusUpdated => EventCategory::Component,
            EventType::ProviderShutdown => EventCategory::Component,
            EventType::ProviderPrimarySet => EventCategory::Component,
            EventType::Custom(_) => EventCategory::Agent,
        }
    }

    pub fn request_buidler() -> RequestBuilder {
        RequestBuilder::new()
    }

    pub fn response_builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    pub fn response_value(&self) -> Value {
        match &self.event_type {
            EventType::ResponseSuccess { .. } => self
                .parameters
                .get("response")
                .cloned()
                .unwrap_or(Value::Null),
            EventType::ResponseFailure { .. } => {
                self.parameters.get("error").cloned().unwrap_or(Value::Null)
            }
            _ => Value::Null,
        }
    }
}

#[derive(Default, Clone)]
pub struct RequestBuilder {
    request_type: Option<String>,
    requester: Option<String>,
    responder: Option<String>,
    request_id: Option<String>,
    parameters: HashMap<String, Value>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn request_type(mut self, request_type: &str) -> Self {
        self.request_type = Some(request_type.to_string());
        self
    }

    pub fn requester(mut self, requester: &str) -> Self {
        self.requester = Some(requester.to_string());
        self
    }

    pub fn responder(mut self, responder: &str) -> Self {
        self.responder = Some(responder.to_string());
        self
    }

    pub fn request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }

    pub fn parameters(mut self, parameters: HashMap<String, Value>) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn parameter(mut self, key: &str, value: &Value) -> Self {
        let key = key.to_string();
        self.parameters.insert(key, value.to_owned());
        self
    }

    pub fn build(self) -> EventResult<Event> {
        Ok(Event {
            event_type: EventType::Request {
                request_type: self.request_type.ok_or(EventError::RequestBuilderFailed(
                    "request_type is required".to_string(),
                ))?,
                requester: self.requester.ok_or(EventError::RequestBuilderFailed(
                    "requester is required".to_string(),
                ))?,
                responder: self.responder.ok_or(EventError::RequestBuilderFailed(
                    "responder is required".to_string(),
                ))?,
                request_id: self.request_id.ok_or(EventError::RequestBuilderFailed(
                    "request_id is required".to_string(),
                ))?,
            },
            parameters: self.parameters,
        })
    }
}

// response builder
#[derive(Default)]
pub struct ResponseBuilder {
    is_success: Option<bool>,
    request_type: Option<String>,
    requester: Option<String>,
    responder: Option<String>,
    request_id: Option<String>,
    response: Option<Value>,
    error: Option<String>,
    parameters: HashMap<String, Value>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn success(mut self) -> Self {
        self.is_success = Some(true);
        self
    }

    pub fn failure(mut self) -> Self {
        self.is_success = Some(false);
        self
    }

    pub fn request_type(mut self, request_type: &str) -> Self {
        self.request_type = Some(request_type.to_string());
        self
    }

    pub fn requester(mut self, requester: &str) -> Self {
        self.requester = Some(requester.to_string());
        self
    }

    pub fn responder(mut self, responder: &str) -> Self {
        self.responder = Some(responder.to_string());
        self
    }

    pub fn request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }

    pub fn response(mut self, response: Value) -> Self {
        self.response = Some(response);
        self
    }

    pub fn error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }

    pub fn parameters(mut self, parameters: HashMap<String, Value>) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn build(self) -> EventResult<Event> {
        match self.is_success {
            Some(true) => self.build_success(),
            Some(false) => self.build_failure(),
            None => Err(EventError::ResponseBuilderFailed(
                "is_success is required".to_string(),
            )),
        }
    }

    fn build_success(self) -> EventResult<Event> {
        let parameters = if let Some(response) = self.response {
            let mut params = self.parameters;
            params.insert("response".to_string(), response);
            params
        } else {
            self.parameters
        };

        Ok(Event {
            event_type: EventType::ResponseSuccess {
                request_type: self.request_type.ok_or(EventError::ResponseBuilderFailed(
                    "request_type is required".to_string(),
                ))?,
                requester: self.requester.ok_or(EventError::ResponseBuilderFailed(
                    "requester is required".to_string(),
                ))?,
                responder: self.responder.ok_or(EventError::ResponseBuilderFailed(
                    "responder is required".to_string(),
                ))?,
                request_id: self.request_id.ok_or(EventError::ResponseBuilderFailed(
                    "request_id is required".to_string(),
                ))?,
            },
            parameters,
        })
    }

    fn build_failure(self) -> EventResult<Event> {
        let parameters = if let Some(error) = self.error {
            let mut params = self.parameters;
            params.insert("error".to_string(), Value::String(error));
            params
        } else {
            self.parameters
        };
        Ok(Event {
            event_type: EventType::ResponseFailure {
                request_type: self.request_type.ok_or(EventError::ResponseBuilderFailed(
                    "request_type is required".to_string(),
                ))?,
                requester: self.requester.ok_or(EventError::ResponseBuilderFailed(
                    "requester is required".to_string(),
                ))?,
                responder: self.responder.ok_or(EventError::ResponseBuilderFailed(
                    "responder is required".to_string(),
                ))?,
                request_id: self.request_id.ok_or(EventError::ResponseBuilderFailed(
                    "request_id is required".to_string(),
                ))?,
            },
            parameters,
        })
    }
}

// eum などを event_type string に変換する
pub trait ToEventType {
    fn to_event_type(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ErrorEvent {
    pub error_type: String,
    pub message: String,
    pub severity: ErrorSeverity,
    pub parameters: HashMap<String, Value>,
    // pub agent_id: Option<String>,      // エラー発生元のエージェント
    // pub component: String,             // エラー発生元のコンポーネント
    // pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ErrorSeverity {
    #[default]
    Warning, // 通知のみ
    Error,    // 処理中断
    Critical, // システム停止
}

// 値の型
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Duration(Duration),
    Map(HashMap<String, Value>),
    Null,
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<expression::Value> for Value {
    fn from(value: expression::Value) -> Self {
        match value {
            expression::Value::Integer(i) => Value::Integer(i),
            expression::Value::UInteger(u) => Value::Integer(u as i64),
            expression::Value::Float(f) => Value::Float(f),
            expression::Value::String(s) => Value::String(s),
            expression::Value::Boolean(b) => Value::Boolean(b),
            expression::Value::List(l) => Value::List(l.into_iter().map(Value::from).collect()),
            expression::Value::Null => Value::Null,
            expression::Value::Duration(d) => Value::Duration(d),
            expression::Value::Unit => Value::Null,
            expression::Value::Tuple(t) => Value::List(t.into_iter().map(Value::from).collect()),
            expression::Value::Map(m) => Value::Map(
                m.into_iter()
                    .map(|(k, v)| (k, Value::from(v)))
                    .collect::<HashMap<String, Value>>(),
            ),
            expression::Value::Error(s) => Value::String(s),
            expression::Value::Delay(retry) => {
                let mut map = HashMap::new();
                map.insert("type".to_string(), Value::String("retry".to_string()));
                match retry {
                    RetryDelay::Fixed(d) => {
                        map.insert(
                            "delay".to_string(),
                            Value::Duration(Duration::from_millis(d)),
                        );
                    }
                    RetryDelay::Exponential { initial, max } => {
                        map.insert(
                            "initial_delay".to_string(),
                            Value::Duration(Duration::from_millis(initial)),
                        );
                        map.insert("multiplier".to_string(), Value::Integer(max as i64));
                    }
                }
                Value::Map(map)
            }
            expression::Value::Ok(value) => Value::from(*value),
            expression::Value::Err(value) => Value::from(*value),
        }
    }
}

impl From<Value> for expression::Value {
    fn from(val: Value) -> Self {
        match val {
            Value::Integer(i) => expression::Value::Integer(i),
            Value::Float(f) => expression::Value::Float(f),
            Value::String(s) => expression::Value::String(s),
            Value::Boolean(b) => expression::Value::Boolean(b),
            Value::List(l) => expression::Value::List(l.into_iter().map(Into::into).collect()),
            Value::Null => expression::Value::Null,
            Value::Duration(d) => expression::Value::Duration(d),
            Value::Map(m) => {
                expression::Value::Map(m.into_iter().map(|(k, v)| (k, v.into())).collect::<HashMap<
                    String,
                    expression::Value,
                >>(
                ))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LastStatus {
    pub last_event_type: EventType,
    pub last_event_time: DateTime<Utc>,
}

impl From<LastStatus> for Event {
    fn from(status: LastStatus) -> Self {
        Event {
            event_type: status.last_event_type,
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "last_event_time".to_string(),
                    Value::String(status.last_event_time.to_rfc3339()),
                );
                params
            },
        }
    }
}

pub struct EventBus {
    event_sender: broadcast::Sender<Event>,
    error_sender: broadcast::Sender<ErrorEvent>,
    capacity: usize,
    _internal_receiver: broadcast::Receiver<Event>, // EventBusをアクティブに保つための内部Receiver
    _internal_error_receiver: broadcast::Receiver<ErrorEvent>,
}

impl EventBus {
    /// Create a new EventBus with the given capacity.
    /// The capacity is the maximum number of events that can be buffered.
    /// EventBus will initiate a broadcast channel with the given capacity.
    /// The internal receiver is used to keep the EventBus active.
    pub fn new(capacity: usize) -> Self {
        let (event_sender, event_receiver) = broadcast::channel(capacity);
        let (error_sender, error_reciever) = broadcast::channel(capacity);
        Self {
            event_sender,
            error_sender,
            capacity,
            _internal_receiver: event_receiver,
            _internal_error_receiver: error_reciever,
        }
    }

    pub fn subscribe(&self) -> (EventReceiver, ErrorReceiver) {
        let event_rx = self.event_sender.subscribe();
        let error_rx = self.error_sender.subscribe();
        (EventReceiver::new(event_rx), ErrorReceiver::new(error_rx))
    }

    pub async fn publish(&self, event: Event) -> EventResult<()> {
        debug!("Publishing event: {:?}", event);
        self.event_sender
            .send(event)
            .map_err(|e| EventError::SendFailed {
                message: e.to_string(),
            })?;
        Ok(())
    }

    pub fn sync_publish(&self, event: Event) -> EventResult<()> {
        debug!("Publishing event: {:?}", event);
        self.event_sender
            .send(event)
            .map_err(|e| EventError::SendFailed {
                message: e.to_string(),
            })?;
        Ok(())
    }

    pub async fn publish_error(&self, error: ErrorEvent) -> EventResult<()> {
        self.error_sender
            .send(error)
            .map_err(|e| EventError::SendFailed {
                message: e.to_string(),
            })?;
        Ok(())
    }

    pub fn queue_size(&self) -> usize {
        self.event_sender.len()
    }

    pub fn error_queue_size(&self) -> usize {
        self.error_sender.len()
    }

    pub fn subscribers_size(&self) -> usize {
        self.event_sender.receiver_count()
    }

    pub fn error_subscribers_size(&self) -> usize {
        self.error_sender.receiver_count()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

pub struct EventReceiver {
    pub receiver: broadcast::Receiver<Event>,
}

impl EventReceiver {
    pub fn new(receiver: broadcast::Receiver<Event>) -> Self {
        Self { receiver }
    }

    /// イベントを受信する。Laggedエラーが発生した場合はresubscribeを試みて、エラーを返す。
    /// 利用側で、Laggedなどが発生しないようできるだけすぐに次のrecvを呼ぶようにする。
    pub async fn recv(&mut self) -> EventResult<Event> {
        match self.receiver.recv().await {
            Ok(event) => Ok(event),
            Err(broadcast::error::RecvError::Lagged(n)) => {
                // n個のメッセージをスキップ
                self.receiver = self.receiver.resubscribe();
                Err(EventError::Lagged { count: n })
            }
            Err(e) => Err(EventError::ReceiveFailed {
                message: e.to_string(),
            }),
        }
    }
}

pub struct ErrorReceiver {
    pub receiver: broadcast::Receiver<ErrorEvent>,
}

impl ErrorReceiver {
    fn new(receiver: broadcast::Receiver<ErrorEvent>) -> Self {
        Self { receiver }
    }

    pub async fn recv(&mut self) -> EventResult<ErrorEvent> {
        self.receiver
            .recv()
            .await
            .map_err(|e| EventError::ReceiveFailed {
                message: e.to_string(),
            })
    }
}

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Event type not supported: {event_type}")]
    UnsupportedType { event_type: String },

    #[error("Unsupported request event: {request_type}")]
    UnsupportedRequest { request_type: String },

    #[error("Invalid event parameters: {message}")]
    InvalidParameters { message: String },

    #[error("Event parameters length not matched: {event_type}, expected {expected}, got {got}")]
    ParametersLengthNotMatched {
        event_type: String,
        expected: usize,
        got: usize,
    },

    #[error("Event parameter type mismatch: {event_type}, expected {expected}, got {got}")]
    TypeMismatch {
        event_type: String,
        expected: String,
        got: String,
    },

    #[error("Event Send failed: {message}")]
    SendFailed { message: String },

    #[error("Event Receive failed: {message}")]
    ReceiveFailed { message: String },

    #[error("Event Receive response failed: {message}")]
    ReceiveResponseFailed { request_id: String, message: String },

    #[error("Event Receive response timeout: {request_id}")]
    ReceiveResponseTimeout {
        request_id: String,
        timeout_secs: u64,
        message: String,
    },

    #[error("Event lagged: {count}")]
    Lagged { count: u64 },

    #[error("Event already registered: {event_type}")]
    AlreadyRegistered { event_type: String },

    #[error("Built-in event already registered")]
    BuiltInAlreadyRegistered,

    #[error("Event not found: {0}")]
    NotFound(String),

    #[error("request_type is required")]
    RequestTypeRequired(String),

    #[error("request builder failed: {0}")]
    RequestBuilderFailed(String),

    #[error("response builder failed: {0}")]
    ResponseBuilderFailed(String),
}

pub type EventResult<T> = Result<T, EventError>;

#[cfg(test)]
mod tests {
    use crate::event_registry::EventType;

    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_initial_publish_success() {
        let bus = EventBus::new(16);
        let test_event = Event {
            event_type: EventType::Custom("test".to_string()),
            ..Default::default()
        };
        assert!(bus.publish(test_event.clone()).await.is_ok());
    }

    #[tokio::test]
    async fn test_basic_publish_subscribe() {
        let bus = EventBus::new(16);
        let (mut event_rx, _) = bus.subscribe();

        let test_event = Event {
            event_type: EventType::Custom("test".to_string()),
            ..Default::default()
        };

        bus.publish(test_event.clone()).await.unwrap();

        let received = event_rx.recv().await.unwrap();
        assert_eq!(received.event_type, EventType::Custom("test".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new(16);
        let (mut rx1, _) = bus.subscribe();
        let (mut rx2, _) = bus.subscribe();

        let test_event = Event {
            event_type: EventType::Custom("test".to_string()),
            ..Default::default()
        };

        bus.publish(test_event.clone()).await.unwrap();

        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();

        assert_eq!(received1.event_type, EventType::Custom("test".to_string()));
        assert_eq!(received2.event_type, EventType::Custom("test".to_string()));
    }

    #[tokio::test]
    async fn test_error_channel() {
        let bus = EventBus::new(16);
        let (_, mut error_rx) = bus.subscribe();

        let test_error = ErrorEvent {
            error_type: "test_error".to_string(),
            message: "test message".to_string(),
            ..Default::default()
        };

        bus.publish_error(test_error.clone()).await.unwrap();

        let received = error_rx.recv().await.unwrap();
        assert_eq!(received.error_type, "test_error");
    }
}

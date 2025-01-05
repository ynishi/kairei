use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use tracing::debug;

use crate::{eval::expression, event_registry::EventType, EventError, RuntimeError, RuntimeResult};

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

    pub async fn publish(&self, event: Event) -> RuntimeResult<()> {
        debug!("Publishing event: {:?}", event);
        self.event_sender.send(event).map_err(|e| {
            RuntimeError::Event(EventError::SendFailed {
                message: e.to_string(),
            })
        })?;
        Ok(())
    }

    pub fn sync_publish(&self, event: Event) -> RuntimeResult<()> {
        debug!("Publishing event: {:?}", event);
        self.event_sender.send(event).map_err(|e| {
            RuntimeError::Event(EventError::SendFailed {
                message: e.to_string(),
            })
        })?;
        Ok(())
    }

    pub async fn publish_error(&self, error: ErrorEvent) -> RuntimeResult<()> {
        self.error_sender.send(error).map_err(|e| {
            RuntimeError::Event(EventError::SendFailed {
                message: e.to_string(),
            })
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
    pub async fn recv(&mut self) -> RuntimeResult<Event> {
        match self.receiver.recv().await {
            Ok(event) => Ok(event),
            Err(broadcast::error::RecvError::Lagged(n)) => {
                // n個のメッセージをスキップ
                self.receiver = self.receiver.resubscribe();
                Err(RuntimeError::Event(EventError::Lagged { count: n }))
            }
            Err(e) => Err(RuntimeError::Event(EventError::RecieveFailed {
                message: e.to_string(),
            })),
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

    pub async fn recv(&mut self) -> RuntimeResult<ErrorEvent> {
        self.receiver.recv().await.map_err(|e| {
            RuntimeError::Event(EventError::RecieveFailed {
                message: e.to_string(),
            })
        })
    }
}

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

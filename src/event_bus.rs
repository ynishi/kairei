use tokio::sync::broadcast;

use crate::{
    runtime::{ErrorEvent, Event},
    EventError, RuntimeError, RuntimeResult,
};

pub struct EventBus {
    event_sender: broadcast::Sender<Event>,
    error_sender: broadcast::Sender<ErrorEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (event_sender, _) = broadcast::channel(capacity);
        let (error_sender, _) = broadcast::channel(capacity);
        Self {
            event_sender,
            error_sender,
        }
    }

    pub fn subscribe(&self) -> (EventReceiver, ErrorReceiver) {
        let event_rx = self.event_sender.subscribe();
        let error_rx = self.error_sender.subscribe();
        (EventReceiver::new(event_rx), ErrorReceiver::new(error_rx))
    }

    pub async fn publish(&self, event: Event) -> RuntimeResult<()> {
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
}

pub struct EventReceiver {
    receiver: broadcast::Receiver<Event>,
}

impl EventReceiver {
    fn new(receiver: broadcast::Receiver<Event>) -> Self {
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
    receiver: broadcast::Receiver<ErrorEvent>,
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
    async fn test_basic_publish_subscribe() {
        let bus = EventBus::new(16);
        let (mut event_rx, _) = bus.subscribe();

        let test_event = Event {
            event_type: EventType::Custom("test".to_string()),
            parameters: Default::default(),
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
            parameters: Default::default(),
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
        };

        bus.publish_error(test_error.clone()).await.unwrap();

        let received = error_rx.recv().await.unwrap();
        assert_eq!(received.error_type, "test_error");
    }
}

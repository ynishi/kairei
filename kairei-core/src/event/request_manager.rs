//! # Request Manager
//!
//! The RequestManager provides a synchronous request-response pattern on top of the
//! asynchronous event bus. It allows components to send a request and wait for a
//! matching response, with timeout handling and correlation.
//!
//! ## Key Features
//!
//! - **Request-Response Correlation**: Tracks pending requests and matches responses
//! - **Timeout Handling**: Automatically times out requests that don't receive responses
//! - **Response Awaiting**: Provides a Future that resolves when a response is received
//! - **Cancellation**: Supports cancelling pending requests when a component shuts down
//!
//! ## Implementation Details
//!
//! The RequestManager uses Tokio oneshot channels to bridge the gap between the
//! asynchronous event bus and the synchronous request-response pattern. When a request
//! is made, a oneshot receiver is registered, and the corresponding sender is stored
//! with the request ID. When a matching response arrives, it's forwarded through
//! the oneshot channel to awaken the waiting task.

use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use thiserror::Error;
use tokio::sync::oneshot;
use tracing::instrument;

use super::{
    event_bus::{Event, EventBus, EventError, Value},
    event_registry::EventType,
};

/// Type alias for request correlation identifiers
type RequestId = String;

/// Represents a pending request awaiting a response
///
/// Each pending request consists of a oneshot sender for delivering the response
/// and the original request event type for correlation and error handling.
pub struct PendingRequest {
    /// Channel for delivering the response back to the requester
    sender: oneshot::Sender<Event>,
    /// The original request event type (for error handling and cancellation)
    request_event_type: EventType,
}

/// # Request Manager
///
/// Coordinates the request-response pattern on top of the event bus.
///
/// The RequestManager maintains a registry of pending requests and handles
/// matching responses to their original requests. It also manages timeouts
/// for requests that don't receive timely responses.
pub struct RequestManager {
    /// Reference to the event bus for publishing requests
    event_bus: Arc<EventBus>,
    /// Map of pending requests indexed by request ID
    pending_requests: Arc<DashMap<RequestId, PendingRequest>>,
    /// Default timeout duration for requests that don't specify one
    default_timeout: Duration,
}

impl RequestManager {
    /// Creates a new RequestManager with the given event bus and timeout.
    ///
    /// # Parameters
    ///
    /// * `event_bus` - Shared reference to the event bus for publishing requests
    /// * `timeout` - Default timeout duration for requests
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use kairei_core::event_bus::EventBus;
    /// use kairei_core::request_manager::RequestManager;
    /// use std::time::Duration;
    /// let event_bus = Arc::new(EventBus::new(100));
    /// let request_manager = RequestManager::new(
    ///     event_bus.clone(),
    ///     Duration::from_secs(5) // 5 second default timeout
    /// );
    /// ```
    pub fn new(event_bus: Arc<EventBus>, timeout: Duration) -> Self {
        Self {
            event_bus,
            pending_requests: Arc::new(DashMap::new()),
            default_timeout: timeout,
        }
    }

    /// Sends a request event and waits for a matching response.
    ///
    /// This method provides a synchronous request-response pattern by:
    /// 1. Creating a oneshot channel for the response
    /// 2. Registering the pending request with its ID
    /// 3. Publishing the request to the event bus
    /// 4. Awaiting a matching response or timeout
    ///
    /// # Parameters
    ///
    /// * `request` - The request event to send
    ///
    /// # Returns
    ///
    /// * `RequestResult<Event>` - The response event or an error
    ///
    /// # Errors
    ///
    /// * `RequestError::InvalidRequest` - If the request is missing required fields
    /// * `RequestError::Timeout` - If no response is received within the timeout period
    /// * `RequestError::EventBus` - If publishing the request fails
    /// * `RequestError::ChannelClosed` - If the response channel unexpectedly closes
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use kairei_core::event_bus::{EventBus, Event};
    /// # use kairei_core::event_registry::EventType;
    /// # use kairei_core::event::request_manager::{RequestManager, RequestError};
    /// # use std::sync::Arc;
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let event_bus = Arc::new(EventBus::new(10));
    /// # let request_manager = Arc::new(RequestManager::new(event_bus.clone(), Duration::from_secs(5)));
    /// # let request = Event::request_builder()
    /// #     .request_type("get_data")
    /// #     .requester("client")
    /// #     .responder("data_service")
    /// #     .request_id("test-id")
    /// #     .build()?;
    /// #
    /// match request_manager.request(&request).await {
    ///     Ok(response) => {
    ///         println!("Got response: {:?}", response);
    ///     },
    ///     Err(RequestError::Timeout(_)) => {
    ///         println!("Request timed out");
    ///     },
    ///     Err(e) => {
    ///         println!("Request failed: {}", e);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn request(&self, request: &Event) -> RequestResult<Event> {
        let (tx, rx) = oneshot::channel();
        let request_id = request
            .event_type
            .request_id()
            .ok_or(RequestError::InvalidRequest(
                "Request ID not found in event".to_string(),
            ))?
            .to_string();

        self.pending_requests.insert(
            request_id.clone(),
            PendingRequest {
                sender: tx,
                request_event_type: request.event_type.clone(),
            },
        );

        let timeout = self.timeout(request);
        self.event_bus.publish(request.clone()).await?;

        // Wait for the response
        self.await_response(request_id, timeout, rx).await
    }

    #[instrument(skip(self, rx))]
    async fn await_response(
        &self,
        request_id: RequestId,
        timeout: Duration,
        mut rx: oneshot::Receiver<Event>,
    ) -> RequestResult<Event> {
        // タイムアウト用のスリープを作成
        let sleep = tokio::time::sleep(timeout);
        tokio::pin!(sleep);

        loop {
            tokio::select! {
                // タイムアウト
                _ = &mut sleep => {
                    return Err(RequestError::Timeout(request_id));
                }
                // レスポンス受信
                result = &mut rx => {
                    match result {
                        Ok(event) => {
                            if event.event_type.is_response_to(&request_id) {
                                return Ok(event);
                            }
                            // 対象外のイベントは無視して継続
                        },
                        Err(_) => return Err(RequestError::ChannelClosed),
                    }
                }
            }
        }
    }

    /// EventカテゴリーResponseのみを処理。それ以外はエラーを返す
    #[instrument(skip(self))]
    pub fn handle_event(&self, event: &Event) -> RequestResult<()> {
        if event.event_type.is_response() {
            event
                .event_type
                .request_id()
                .map(|id| {
                    if let Some(pending) = self.pending_requests.remove(id) {
                        let _ = pending.1.sender.send(event.clone());
                    }
                })
                .ok_or(RequestError::InvalidRequest(
                    "Request ID not found in event".to_string(),
                ))
        } else {
            Err(RequestError::InvalidRequest(
                "Invalid event type".to_string(),
            ))
        }
    }

    pub async fn cancel_waiting_requests(
        &self,
        failure_message: &str,
    ) -> RequestResult<Vec<Event>> {
        let mut ret = vec![];
        for entry in self.pending_requests.iter() {
            let request_id = entry.key();
            if let Some((_, pending_request)) = self.pending_requests.remove(request_id) {
                match &pending_request.request_event_type {
                    EventType::Request {
                        requester,
                        responder,
                        request_type,
                        ..
                    } => {
                        if let Ok(response_failure) = Event::response_builder()
                            .failure()
                            .request_id(request_id)
                            .requester(requester)
                            .responder(responder)
                            .request_type(request_type)
                            .error(format!("request_cancelled: {}", failure_message).as_str())
                            .build()
                        {
                            ret.push(response_failure.clone());
                            let _ = pending_request.sender.send(response_failure);
                        } else {
                            return Err(RequestError::InvalidRequest(
                                "Failed to create response".to_string(),
                            ));
                        }
                    }
                    _ => {
                        return Err(RequestError::InvalidRequest(
                            "EventType not supported".to_string(),
                        ));
                    }
                }
            }
        }
        self.pending_requests.clear();
        Ok(ret)
    }

    fn timeout(&self, event: &Event) -> Duration {
        match event.parameters.get("timeout") {
            Some(Value::Duration(d)) if *d > Duration::from_secs(1) => *d,
            _ => self.default_timeout,
        }
    }
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Request timed out: {0}")]
    Timeout(RequestId),
    #[error("Response channel closed")]
    ChannelClosed,
    #[error("Event bus error: {0}")]
    EventBus(#[from] EventError),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Request not found: {0}")]
    NotFound(EventError),
}

type RequestResult<T> = Result<T, RequestError>;

#[cfg(test)]
mod tests {

    use super::*;

    use crate::event_bus::{self};

    use tokio::time::Duration;

    // テスト用のヘルパー関数
    async fn setup() -> (Arc<EventBus>, Arc<RequestManager>) {
        let event_bus = Arc::new(EventBus::new(10));
        let manager = Arc::new(RequestManager::new(
            event_bus.clone(),
            Duration::from_secs(5), // テスト用のタイムアウト
        ));
        (event_bus, manager)
    }

    fn create_events(prefix: &str) -> (Event, Event) {
        let target_name = format!("{}target", prefix);
        let request_id = format!("{}request_id", prefix);
        let requester_name = format!("{}responder", prefix);
        let request_type = format!("{}request", prefix);
        let response = event_bus::Value::String(format!("{}response", prefix));

        (
            Event::request_builder()
                .request_type(&request_type)
                .requester(&requester_name)
                .responder(&target_name)
                .request_id(&request_id)
                .build()
                .unwrap(),
            Event::response_builder()
                .success()
                .request_type(&request_type)
                .requester(&requester_name)
                .responder(&target_name)
                .request_id(&request_id)
                .response(response)
                .build()
                .unwrap(),
        )
    }

    #[tokio::test]
    async fn test_request_response_success() {
        let (event_bus, manager) = setup().await;

        let (request_event, response_event) = create_events("test");

        // ハンドラーをSpawnする
        let manager_ref = manager.clone();
        let (mut event_rx, _) = event_bus.subscribe();
        let handler_task = tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                // 任意のEventを入れても問題ない（handle_eventではErrを返すので処理する)
                let _ = manager_ref.handle_event(&event);
            }
        });

        // リクエストを発行
        let request_task = tokio::spawn({
            let manager = manager.clone();
            async move { manager.request(&request_event).await }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // レスポンスを送信
        event_bus.publish(response_event.clone()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // 結果の確認
        let result = request_task.await.unwrap().unwrap();
        assert_eq!(result, response_event);

        // ハンドラーのタスクをクリーンアップ
        handler_task.abort();
        let _ = handler_task.await; // エラーは無視
    }

    #[tokio::test]
    async fn test_request_timeout() {
        let (event_bus, _) = setup().await;

        // 短いタイムアウトで設定
        let manager = RequestManager::new(event_bus.clone(), Duration::from_millis(100));

        let (request_event, _) = create_events("test");

        // タイムアウトするまで待機
        let result = manager.request(&request_event).await;

        assert!(matches!(result, Err(RequestError::Timeout(_))));
    }

    #[tokio::test]
    async fn test_multiple_requests() {
        let (event_bus, manager) = setup().await;

        let (request_event1, response_event1) = create_events("test1");
        let (request_event2, response_event2) = create_events("test2");

        // ハンドラーをSpawnする
        let manager_ref = manager.clone();
        let (mut event_rx, _) = event_bus.subscribe();
        let handler_task = tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                // 任意のEventを入れても問題ない（handle_eventではErrを返すので処理する)
                let _ = manager_ref.handle_event(&event);
            }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // 複数のリクエストを同時に発行
        let request1 = manager.request(&request_event1);
        let request2 = manager.request(&request_event2);

        // 両方のリクエストにレスポンスを送信
        tokio::time::sleep(Duration::from_millis(100)).await;

        event_bus.publish(response_event1.clone()).await.unwrap();
        event_bus.publish(response_event2.clone()).await.unwrap();

        // 両方のレスポンスを確認
        let (result1, result2) = tokio::join!(request1, request2);
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // ハンドラーのタスクをクリーンアップ
        handler_task.abort();
        let _ = handler_task.await; // エラーは無視
    }

    #[tokio::test]
    async fn test_cancelled_request() {
        let (_, manager) = setup().await;

        let (request_event, _) = create_events("test");

        // リクエストタスクを作成して即ドロップ
        let task = { manager.request(&request_event) };
        drop(task);

        // pending_requestsからクリーンアップされていることを確認
        assert!(manager.pending_requests.is_empty());
    }
}

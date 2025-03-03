//! Event API for kairei-core
//!
//! This module defines the EventApi trait for event operations.

use async_trait::async_trait;
use serde_json::Value as JsonValue;

use crate::event::event_bus::{Event, EventReceiver, Value};
use crate::event::event_registry::EventType;
use crate::system::SystemResult;

/// API for event operations
#[async_trait]
pub trait EventApi: Send + Sync {
    /// Send an event to the system
    async fn send_event(&self, event: Event) -> SystemResult<()>;

    /// Send a request and wait for response
    async fn send_request(&self, event: Event) -> SystemResult<Value>;

    /// Subscribe to specific event types
    async fn subscribe_events(&self, event_types: Vec<EventType>) -> SystemResult<EventReceiver>;

    /// Send an event with the given type and payload
    async fn send_typed_event(
        &self,
        event_type: String,
        payload: JsonValue,
        target_agents: Vec<String>,
    ) -> SystemResult<String>;

    /// Send a request to a specific agent
    async fn send_agent_request(
        &self,
        agent_id: &str,
        request_type: String,
        parameters: JsonValue,
    ) -> SystemResult<JsonValue>;
}

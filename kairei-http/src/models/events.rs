use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Event submission request model
#[derive(Debug, Deserialize)]
pub struct EventRequest {
    /// Type of the event
    pub event_type: String,

    /// Event payload data
    pub payload: Value,

    /// Optional list of target agent IDs
    #[serde(default)]
    pub target_agents: Vec<String>,
}

/// Event submission response model
#[derive(Debug, Serialize)]
pub struct EventResponse {
    /// Unique identifier for the event
    pub event_id: String,

    /// Status of the event
    pub status: EventStatus,

    /// Number of agents that received the event
    pub delivered_to: usize,
}

/// Event status enum
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    /// Event has been queued for delivery
    Queued,

    /// Event has been delivered to all target agents
    Delivered,

    /// Event delivery failed
    Failed,
}

/// Agent request model
#[derive(Debug, Deserialize)]
pub struct AgentRequestPayload {
    /// Type of the request
    pub request_type: String,

    /// Request parameters
    pub parameters: Value,
}

/// Agent request response model
#[derive(Debug, Serialize)]
pub struct AgentRequestResponse {
    /// Unique identifier for the request
    pub request_id: String,

    /// Status of the request
    pub status: RequestStatus,

    /// Result of the request, if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error message, if the request failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request status enum
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    /// Request is pending processing
    Pending,

    /// Request is being processed
    Processing,

    /// Request has been completed successfully
    Completed,

    /// Request failed
    Failed,
}

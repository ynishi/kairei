use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Agent creation request model
#[derive(Debug, Deserialize)]
pub struct AgentCreationRequest {
    /// Name of the agent
    pub name: String,

    /// DSL code defining the agent
    pub dsl_code: String,

    /// Optional agent creation options
    #[serde(default)]
    pub options: AgentCreationOptions,
}

/// Agent creation options
#[derive(Debug, Deserialize, Default)]
pub struct AgentCreationOptions {
    /// Whether to automatically start the agent after creation
    #[serde(default)]
    pub auto_start: bool,
}

/// Agent creation response model
#[derive(Debug, Serialize)]
pub struct AgentCreationResponse {
    /// Unique identifier for the agent
    pub agent_id: String,

    /// Current status of the agent
    pub status: AgentStatus,

    /// Result of agent validation
    pub validation_result: ValidationResult,
}

/// Agent details response model
#[derive(Debug, Serialize)]
pub struct AgentDetails {
    /// Unique identifier for the agent
    pub agent_id: String,

    /// Name of the agent
    pub name: String,

    /// Current status of the agent
    pub status: AgentStatus,

    /// When the agent was created
    pub created_at: String,

    /// Statistics about the agent
    pub statistics: AgentStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAgentResponse {
    pub agent_id: String,

    pub status: kairei_core::system::AgentStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAgentsResponse {
    pub agents: Vec<GetAgentResponse>,
}

/// Agent creation request model
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScaleUpAgentRequest {
    /// Number of instances to scale up by
    pub instances: usize,

    /// Optional agent scaling options
    pub options: HashMap<String, serde_json::Value>,
}

/// Agent creation request model
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScaleDownAgentRequest {
    /// Number of instances to scale up by
    pub instances: usize,

    /// Optional agent scaling options
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendRequestAgentRequest {
    pub request_type: String,
    pub payload: Value,
}

#[derive(Debug, Serialize)]
pub struct SendRequestAgentResponse {
    pub value: Value,
}

/// Agent status enum
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Agent has been created but not started
    Created,

    /// Agent is running
    Running,

    /// Agent is paused
    Paused,

    /// Agent has been stopped
    Stopped,

    /// Agent has encountered an error
    Error,
}

/// Validation result model
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    /// Whether validation was successful
    pub success: bool,

    /// Any warnings generated during validation
    pub warnings: Vec<String>,
}

/// Agent statistics model
#[derive(Debug, Serialize)]
pub struct AgentStatistics {
    /// Number of events processed by the agent
    pub events_processed: usize,

    /// Number of requests handled by the agent
    pub requests_handled: usize,

    /// Agent uptime in seconds
    pub uptime_seconds: u64,
}

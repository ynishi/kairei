use serde::{Deserialize, Serialize};

/// Agent creation request model
#[derive(Debug, Deserialize)]
pub struct AgentCreationRequest {
    /// Name of the agent
    pub name: String,

    /// DSL code defining the agent
    pub dsl_code: String,

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

//! API data models for kairei-core
//!
//! This module defines API-friendly versions of core data structures.

use serde::{Deserialize, Serialize};

use crate::system::{AgentStatus as CoreAgentStatus, SystemStatus as CoreSystemStatus};

/// API-friendly system status information
#[derive(Debug, Clone, Serialize)]
pub struct SystemStatusDto {
    /// System version
    pub version: String,

    /// Current system status (running/stopped)
    pub status: String,

    /// System start time
    pub started_at: String,

    /// System uptime in seconds
    pub uptime_seconds: u64,

    /// Total number of agents in the system
    pub agent_count: usize,

    /// Number of currently running agents
    pub running_agent_count: usize,

    /// Current size of the event queue
    pub event_queue_size: usize,

    /// Number of event subscribers
    pub event_subscribers: usize,
}

impl From<CoreSystemStatus> for SystemStatusDto {
    fn from(status: CoreSystemStatus) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            status: if status.running { "running" } else { "stopped" }.to_string(),
            started_at: status.started_at.to_rfc3339(),
            uptime_seconds: status.uptime.as_secs(),
            agent_count: status.agent_count,
            running_agent_count: status.running_agent_count,
            event_queue_size: status.event_queue_size,
            event_subscribers: status.event_subscribers,
        }
    }
}

/// API-friendly agent status information
#[derive(Debug, Clone, Serialize)]
pub struct AgentStatusDto {
    /// Agent name
    pub name: String,

    /// Current agent state
    pub state: String,

    /// Last time the agent's lifecycle was updated
    pub last_updated: String,
}

impl From<CoreAgentStatus> for AgentStatusDto {
    fn from(status: CoreAgentStatus) -> Self {
        Self {
            name: status.name,
            state: status.state,
            last_updated: status.last_lifecycle_updated.to_rfc3339(),
        }
    }
}

/// Agent creation request
#[derive(Debug, Clone, Deserialize)]
pub struct AgentCreationRequest {
    /// Agent name
    pub name: String,

    /// DSL code for the agent
    pub dsl_code: String,

    /// Optional configuration options
    #[serde(default)]
    pub options: AgentCreationOptions,
}

/// Agent creation options
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AgentCreationOptions {
    /// Whether to automatically start the agent after creation
    #[serde(default)]
    pub auto_start: bool,
}

/// Agent creation response
#[derive(Debug, Clone, Serialize)]
pub struct AgentCreationResponse {
    /// Unique identifier for the agent
    pub agent_id: String,

    /// Current status of the agent
    pub status: String,

    /// Result of validation
    pub validation_result: ValidationResult,
}

/// Validation result
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    /// Whether validation was successful
    pub success: bool,

    /// Any warnings generated during validation
    pub warnings: Vec<String>,
}

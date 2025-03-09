use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Requests
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct CreateSystemRequest {
    /// System name
    pub name: String,

    /// System description
    pub description: Option<String>,

    /// System config
    pub config: kairei_core::config::SystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct SystemConfig {
    /// System name
    pub name: String,

    /// System description
    pub description: Option<String>,
}

impl From<SystemConfig> for kairei_core::config::SystemConfig {
    fn from(_config: SystemConfig) -> Self {
        Self::default()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSystemResponse {
    /// System ID
    pub system_id: String,

    /// Session ID
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListSystemsResponse {
    pub system_statuses: HashMap<String, kairei_core::system::SystemStatus>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StartSystemRequest {
    pub dsl: Option<String>,
}

/// System information response model
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SystemInfo {
    /// API version
    pub version: String,

    /// Current system status
    pub status: SystemStatus,

    /// List of available system capabilities
    pub capabilities: Vec<String>,

    /// System statistics
    pub statistics: SystemStatistics,
}

/// System status enum
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemStatus {
    /// System is running normally
    Running,

    /// System is starting up
    Starting,

    /// System is shutting down
    ShuttingDown,

    /// System is in maintenance mode
    Maintenance,
}

/// System statistics model
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SystemStatistics {
    /// Number of agents in the system
    pub agent_count: usize,

    /// Number of events processed
    pub event_count: usize,

    /// System uptime in seconds
    pub uptime_seconds: u64,
}

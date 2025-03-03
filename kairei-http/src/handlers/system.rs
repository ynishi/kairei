use crate::models::system::{SystemInfo, SystemStatistics, SystemStatus};
use axum::response::Json;

/// Get system information
///
/// Returns information about the current state of the system.
pub async fn get_system_info() -> Json<SystemInfo> {
    // In a real implementation, this would fetch actual system information
    // from kairei-core. For now, we'll return mock data.

    let info = SystemInfo {
        version: "0.1.0".to_string(),
        status: SystemStatus::Running,
        capabilities: vec![
            "agent_management".to_string(),
            "event_processing".to_string(),
        ],
        statistics: SystemStatistics {
            agent_count: 5,
            event_count: 120,
            uptime_seconds: 3600,
        },
    };

    Json(info)
}

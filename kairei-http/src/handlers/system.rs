use crate::models::system::{SystemInfo, SystemStatistics, SystemStatus};
use crate::session::manager::SessionManager;
use axum::{extract::State, response::Json};

/// Get system information
///
/// Returns information about the current state of the system.
pub async fn get_system_info(State(session_manager): State<SessionManager>) -> Json<SystemInfo> {
    // Get the number of active sessions
    let session_count = session_manager.session_count();

    // In a real implementation, this would fetch more actual system information
    // from kairei-core. For now, we'll return mostly mock data but include the
    // real session count.
    let info = SystemInfo {
        version: "0.1.0".to_string(),
        status: SystemStatus::Running,
        capabilities: vec![
            "agent_management".to_string(),
            "event_processing".to_string(),
            "session_management".to_string(),
        ],
        statistics: SystemStatistics {
            agent_count: 5,
            event_count: 120,
            uptime_seconds: 3600,
        },
    };

    Json(info)
}

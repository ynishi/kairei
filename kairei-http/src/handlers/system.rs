use axum::{extract::State, response::Json};
use std::sync::Arc;

use crate::{
    error::AppError,
    integration::KaireiSystem,
    models::system::{SystemInfo, SystemStatistics, SystemStatus},
};

/// Get system information
///
/// Returns information about the current state of the system.
pub async fn get_system_info(
    State(kairei_system): State<Arc<KaireiSystem>>,
) -> Result<Json<SystemInfo>, AppError> {
    // Get system status from kairei-core
    let system_status = kairei_system.system_api.get_system_status().await?;

    // Convert to HTTP API model
    let info = SystemInfo {
        version: system_status.version,
        status: if system_status.status == "running" {
            SystemStatus::Running
        } else {
            SystemStatus::Stopped
        },
        capabilities: vec![
            "agent_management".to_string(),
            "event_processing".to_string(),
        ],
        statistics: SystemStatistics {
            agent_count: system_status.agent_count,
            event_count: system_status.event_queue_size,
            uptime_seconds: system_status.uptime_seconds,
        },
    };

    Ok(Json(info))
}

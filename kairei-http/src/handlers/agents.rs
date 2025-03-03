use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

use crate::{
    error::AppError,
    integration::KaireiSystem,
    models::agents::{
        AgentCreationRequest as HttpAgentCreationRequest, AgentCreationResponse, AgentDetails,
        AgentStatistics, AgentStatus, ValidationResult,
    },
};
use kairei_core::api::models::AgentCreationRequest as CoreAgentCreationRequest;

/// Create a new agent
///
/// Creates a new agent from the provided DSL code.
pub async fn create_agent(
    State(kairei_system): State<Arc<KaireiSystem>>,
    Json(payload): Json<HttpAgentCreationRequest>,
) -> Result<(StatusCode, Json<AgentCreationResponse>), AppError> {
    // Convert HTTP request to core request
    let core_request = CoreAgentCreationRequest {
        name: payload.name.clone(),
        dsl_code: payload.dsl_code.clone(),
        options: kairei_core::api::models::AgentCreationOptions {
            auto_start: payload.auto_start,
        },
    };

    // Register the agent using the core API
    let result = kairei_system
        .agent_api
        .register_agent_from_dsl(core_request)
        .await?;

    // Convert core response to HTTP response
    let status_code = if result.validation_result.success {
        StatusCode::CREATED
    } else {
        StatusCode::BAD_REQUEST
    };

    let status = match result.status.as_str() {
        "running" => AgentStatus::Running,
        "created" => AgentStatus::Created,
        "failed" => AgentStatus::Error,
        _ => AgentStatus::Created,
    };

    let response = AgentCreationResponse {
        agent_id: result.agent_id,
        status,
        validation_result: ValidationResult {
            success: result.validation_result.success,
            warnings: result.validation_result.warnings,
        },
    };

    Ok((status_code, Json(response)))
}

/// Get agent details
///
/// Returns details about a specific agent.
pub async fn get_agent_details(
    State(kairei_system): State<Arc<KaireiSystem>>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetails>, AppError> {
    // Get agent status from core API
    let agent_status = kairei_system.agent_api.get_agent_status(&agent_id).await?;

    // Convert to HTTP API model
    let status = match agent_status.state.as_str() {
        "running" => AgentStatus::Running,
        "stopped" => AgentStatus::Stopped,
        "failed" => AgentStatus::Error,
        _ => AgentStatus::Created,
    };

    let details = AgentDetails {
        agent_id: agent_id.clone(),
        name: agent_status.name,
        status,
        created_at: agent_status.last_updated,
        statistics: AgentStatistics {
            events_processed: 0, // Not available in core API yet
            requests_handled: 0, // Not available in core API yet
            uptime_seconds: 0,   // Not available in core API yet
        },
    };

    Ok(Json(details))
}

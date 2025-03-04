use crate::auth::{AuthAdmin, AuthUser};
use crate::models::agents::{
    AgentCreationRequest, AgentCreationResponse, AgentDetails, AgentStatistics, AgentStatus,
    ValidationResult,
};
use crate::models::user::UserRole;
use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

/// Create a new agent
///
/// Creates a new agent from the provided DSL code.
/// Requires authentication with admin role.
#[axum::debug_handler]
pub async fn create_agent(
    State(_state): State<AppState>,
    auth: AuthAdmin,
    Path(user_id): Path<String>,
    Json(payload): Json<AgentCreationRequest>,
) -> Result<(StatusCode, Json<AgentCreationResponse>), StatusCode> {
    let user = auth.user();

    // Verify that the user_id in the path matches the authenticated user's ID
    // or that the user is an admin (admins can create agents for any user)
    if user.user_id != user_id && user.role != UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }
    // In a real implementation, this would create an actual agent
    // using kairei-core with the session manager. For now, we'll return mock data.
    let agent_id = format!("{}-001", payload.name.to_lowercase());

    let response = AgentCreationResponse {
        agent_id,
        status: AgentStatus::Created,
        validation_result: ValidationResult {
            success: true,
            warnings: vec![],
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get agent details
///
/// Returns details about a specific agent.
/// Requires authentication.
#[axum::debug_handler]
pub async fn get_agent_details(
    State(_state): State<AppState>,
    auth: AuthUser,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetails>, StatusCode> {
    let user = auth.user();

    // In a real implementation, we would check if the user has access to this agent
    // For now, we'll just check if the agent_id starts with the user's ID
    // or if the user is an admin (admins can access any agent)
    if !agent_id.starts_with(&user.user_id) && !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    // In a real implementation, this would fetch the agent from
    // kairei-core using the session manager. For now, we'll return mock data.

    // Simulate agent not found
    if agent_id.contains("not-found") {
        return Err(StatusCode::NOT_FOUND);
    }

    let details = AgentDetails {
        agent_id: agent_id.clone(),
        name: "WeatherAgent".to_string(),
        status: AgentStatus::Running,
        created_at: "2025-03-03T12:00:00Z".to_string(),
        statistics: AgentStatistics {
            events_processed: 42,
            requests_handled: 15,
            uptime_seconds: 1800,
        },
    };

    Ok(Json(details))
}

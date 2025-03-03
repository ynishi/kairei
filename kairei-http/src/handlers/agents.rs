use crate::models::agents::{
    AgentCreationRequest, AgentCreationResponse, AgentDetails, AgentStatistics, AgentStatus,
    ValidationResult,
};
use crate::session::manager::SessionManager;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

/// Create a new agent
///
/// Creates a new agent from the provided DSL code.
pub async fn create_agent(
    State(session_manager): State<SessionManager>,
    Json(payload): Json<AgentCreationRequest>,
) -> (StatusCode, Json<AgentCreationResponse>) {
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

    (StatusCode::CREATED, Json(response))
}

/// Get agent details
///
/// Returns details about a specific agent.
pub async fn get_agent_details(
    State(session_manager): State<SessionManager>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetails>, StatusCode> {
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

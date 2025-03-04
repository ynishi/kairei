use crate::auth::AuthUser;
use crate::models::events::{
    AgentRequestPayload, AgentRequestResponse, EventRequest, EventResponse, EventStatus,
    RequestStatus,
};
use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use uuid::Uuid;

/// Send an event
///
/// Sends an event to one or more agents.
/// Requires authentication.
#[axum::debug_handler]
pub async fn send_event(
    State(_state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<EventRequest>,
) -> Json<EventResponse> {
    // We could use the authenticated user for additional checks or logging
    let _user = auth.user();
    // In a real implementation, this would send the event to the
    // specified agents using kairei-core with the session manager.
    // For now, we'll return mock data.

    let event_id = format!(
        "evt-{}",
        Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    let response = EventResponse {
        event_id,
        status: EventStatus::Delivered,
        delivered_to: payload.target_agents.len().max(1), // If no targets specified, assume broadcast
    };

    Json(response)
}

/// Send a request to an agent
///
/// Sends a request to a specific agent and returns the result.
/// Requires authentication.
#[axum::debug_handler]
pub async fn send_agent_request(
    State(_state): State<AppState>,
    auth: AuthUser,
    Path(agent_id): Path<String>,
    Json(payload): Json<AgentRequestPayload>,
) -> Result<Json<AgentRequestResponse>, StatusCode> {
    let user = auth.user();

    // In a real implementation, we would check if the user has access to this agent
    // For now, we'll just check if the agent_id starts with the user's ID
    // or if the user is an admin (admins can access any agent)
    if !agent_id.starts_with(&user.user_id) && !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    // In a real implementation, this would send the request to the
    // specified agent using kairei-core with the session manager.
    // For now, we'll return mock data.

    // Simulate agent not found
    if agent_id.contains("not-found") {
        return Err(StatusCode::NOT_FOUND);
    }

    let request_id = format!(
        "req-{}",
        Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    // Mock response for GetWeather request
    let result = if payload.request_type == "GetWeather" {
        let location = payload
            .parameters
            .get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        Some(json!({
            "temperature": 25.5,
            "conditions": "Sunny",
            "humidity": 60,
            "location": location
        }))
    } else {
        Some(json!({
            "message": "Request processed successfully"
        }))
    };

    let response = AgentRequestResponse {
        request_id,
        status: RequestStatus::Completed,
        result,
        error: None,
    };

    Ok(Json(response))
}

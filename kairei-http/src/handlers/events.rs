use crate::auth::AuthUser;
use crate::models::events::{AgentRequestResponse, EventRequest, EventResponse};
use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use tracing::debug;

/// List events
///
/// Lists all events for a system.
/// Requires authentication.
// OpenAPI documentation removed
#[axum::debug_handler]
pub async fn list_events(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Path(system_id): Path<String>,
) -> Result<Json<Vec<EventResponse>>, StatusCode> {
    debug!("list_events, system_id: {}", system_id);
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Send an event
///
/// Sends an event to one or more agents.
/// Requires authentication.
// OpenAPI documentation removed
#[axum::debug_handler]
pub async fn emit_event(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Path((system_id, event_id)): Path<(String, String)>,
    Json(_payload): Json<EventRequest>,
) -> Result<Json<EventResponse>, StatusCode> {
    debug!(
        "emit_event, system_id: {}, event_id: {}",
        system_id, event_id
    );
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Send a request to an agent
///
/// Sends a request to a specific agent and returns the result.
/// Requires authentication.
// OpenAPI documentation removed
#[axum::debug_handler]
pub async fn subscribe_event(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Path((system_id, event_id)): Path<(String, String)>,
) -> Result<Json<AgentRequestResponse>, StatusCode> {
    debug!(
        "subscribe_event, system_id: {}, event_id: {}",
        system_id, event_id
    );
    Err(StatusCode::NOT_IMPLEMENTED)
}

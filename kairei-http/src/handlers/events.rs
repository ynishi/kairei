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
#[utoipa::path(
    get,
    path = "/systems/{system_id}/events",
    responses(
        (status = 200, description = "Events listed successfully", body = Vec<EventResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "System not found"),
        (status = 500, description = "Internal server error"),
        (status = 501, description = "Not implemented")
    ),
    params(
        ("system_id" = String, Path, description = "System identifier")
    )
)]
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
#[utoipa::path(
    post,
    path = "/systems/{system_id}/events/{event_id}",
    request_body = EventRequest,
    responses(
        (status = 200, description = "Event emitted successfully", body = EventResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "System not found"),
        (status = 500, description = "Internal server error"),
        (status = 501, description = "Not implemented")
    ),
    params(
        ("system_id" = String, Path, description = "System identifier"),
        ("event_id" = String, Path, description = "Event identifier")
    )
)]
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
#[utoipa::path(
    get,
    path = "/systems/{system_id}/events/{event_id}/subscribe",
    responses(
        (status = 200, description = "Subscribed to event successfully", body = AgentRequestResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "System or event not found"),
        (status = 500, description = "Internal server error"),
        (status = 501, description = "Not implemented")
    ),
    params(
        ("system_id" = String, Path, description = "System identifier"),
        ("event_id" = String, Path, description = "Event identifier")
    )
)]
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

use crate::handlers::events::{send_agent_request, send_event};
use crate::server::AppState;
use axum::{Router, routing::post};

/// Create the events routes with state
pub fn routes() -> Router<AppState> {
    Router::new().route("/events", post(send_event)).route(
        "/events/agents/{agent_id}/request",
        post(send_agent_request),
    )
}

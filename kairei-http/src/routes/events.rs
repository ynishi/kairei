use crate::handlers::events::{send_agent_request, send_event};
use axum::{Router, routing::post};

/// Create the events routes
pub fn routes() -> Router {
    Router::new().route("/events", post(send_event)).route(
        "/events/agents/{agent_id}/request",
        post(send_agent_request),
    )
}

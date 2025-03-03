use axum::{Router, routing::post};
use std::sync::Arc;

use crate::handlers::events::{send_agent_request, send_event};
use crate::integration::KaireiSystem;

/// Create the events routes
pub fn routes(kairei_system: Arc<KaireiSystem>) -> Router {
    Router::new()
        .route(
            "/events",
            post(send_event).with_state(kairei_system.clone()),
        )
        .route(
            "/events/agents/{agent_id}/request",
            post(send_agent_request).with_state(kairei_system),
        )
}

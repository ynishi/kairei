use crate::handlers::agents::get_agent;
use crate::handlers::{
    list_agents, request_agent, scale_down_agent, scale_up_agent, start_agent, stop_agent,
};
use crate::server::AppState;
use axum::{
    Router,
    routing::{get, post},
};

/// Create the agents routes with state
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_agents))
        .route("/{agent_id}", get(get_agent))
        .route("/{agent_id}/start", post(start_agent))
        .route("/{agent_id}/stop", post(stop_agent))
        .route("/{agent_id}/scaleup", post(scale_up_agent))
        .route("/{agent_id}/scaledown", post(scale_down_agent))
        .route("/{agent_id}/request", post(request_agent))
}

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
        .route("/agents", get(list_agents))
        .route("/agents/{agent_id}", get(get_agent))
        .route("/agents/{agent_id}/start", post(start_agent))
        .route("/agents/{agent_id}/stop", post(stop_agent))
        .route("/agents/{agent_id}/scale_up", post(scale_up_agent))
        .route("/agents/{agent_id}/scale_down", post(scale_down_agent))
        .route("/agents/{agent_id}/request", post(request_agent))
}

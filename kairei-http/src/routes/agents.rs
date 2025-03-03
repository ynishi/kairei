use crate::handlers::agents::{create_agent, get_agent_details};
use crate::server::AppState;
use axum::{
    Router,
    routing::{get, post},
};

/// Create the agents routes with state
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/agents", post(create_agent))
        .route("/agents/{agent_id}", get(get_agent_details))
}

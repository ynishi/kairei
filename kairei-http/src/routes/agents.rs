use crate::handlers::agents::{create_agent, get_agent_details};
use axum::{
    Router,
    routing::{get, post},
};

/// Create the agents routes
pub fn routes() -> Router {
    Router::new()
        .route("/agents", post(create_agent))
        .route("/agents/{agent_id}", get(get_agent_details))
}

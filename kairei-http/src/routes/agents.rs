use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use crate::handlers::agents::{create_agent, get_agent_details};
use crate::integration::KaireiSystem;

/// Create the agents routes
pub fn routes(kairei_system: Arc<KaireiSystem>) -> Router {
    Router::new()
        .route(
            "/agents",
            post(create_agent).with_state(kairei_system.clone()),
        )
        .route(
            "/agents/{agent_id}",
            get(get_agent_details).with_state(kairei_system),
        )
}

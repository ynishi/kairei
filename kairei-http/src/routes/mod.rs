pub mod agents;
pub mod events;
pub mod system;

use crate::server::AppState;
use axum::{routing::get, Router, http::StatusCode, response::IntoResponse};

/// Create the main API router with state
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_v1_router())
}

/// Create the v1 API router with state
fn api_v1_router() -> Router<AppState> {
    Router::new().merge(system::routes())
}

/// Health check endpoint for container health monitoring
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

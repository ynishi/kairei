pub mod agents;
pub mod events;
pub mod system;

use axum::Router;

/// Create the main API router
pub fn create_api_router() -> Router {
    Router::new().nest("/api/v1", api_v1_router())
}

/// Create the v1 API router
fn api_v1_router() -> Router {
    Router::new()
        .merge(system::routes())
        .merge(agents::routes())
        .merge(events::routes())
}

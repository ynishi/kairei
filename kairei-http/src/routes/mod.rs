pub mod agents;
pub mod events;
pub mod system;

use axum::Router;
use std::sync::Arc;

use crate::integration::KaireiSystem;

/// Create the main API router
pub fn create_api_router(kairei_system: Arc<KaireiSystem>) -> Router {
    Router::new().nest("/api/v1", api_v1_router(kairei_system))
}

/// Create the v1 API router
fn api_v1_router(kairei_system: Arc<KaireiSystem>) -> Router {
    Router::new()
        .merge(system::routes(kairei_system.clone()))
        .merge(agents::routes(kairei_system.clone()))
        .merge(events::routes(kairei_system))
}

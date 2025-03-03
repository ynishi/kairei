pub mod agents;
pub mod events;
pub mod system;

use crate::session::manager::SessionManager;
use axum::Router;

/// Create the main API router with state
pub fn create_api_router() -> Router<SessionManager> {
    Router::new().nest("/api/v1", api_v1_router())
}

/// Create the v1 API router with state
fn api_v1_router() -> Router<SessionManager> {
    Router::new()
        .merge(system::routes())
        .merge(agents::routes())
        .merge(events::routes())
}

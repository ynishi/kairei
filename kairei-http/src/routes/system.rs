use crate::handlers::{
    create_system, delete_system, get_system, list_systems, shutdown_system, start_system,
};
use crate::server::AppState;
use axum::routing::delete;
use axum::{
    Router,
    routing::{get, post},
};

use super::{agents, events};

/// Create the system routes with state
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/systems", post(create_system))
        .route("/systems", get(list_systems))
        .route("/systems/{system_id}", get(get_system))
        .route("/systems/{system_id}/start", post(start_system))
        .route("/systems/{system_id}/shutdown", post(shutdown_system))
        .route("/systems/{system_id}", delete(delete_system))
        .merge(agents::routes())
        .merge(events::routes())
}

use crate::handlers::{
    compile_system, create_system, delete_system, get_system, list_systems, start_system,
    stop_system,
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
    Router::new().nest("/systems", system_routes())
}

fn system_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_systems))
        .route("/", post(create_system))
        .route("/{system_id}", get(get_system))
        .route("/{system_id}/compile", post(compile_system))
        .route("/{system_id}/start", post(start_system))
        .route("/{system_id}/stop", post(stop_system))
        .route("/{system_id}", delete(delete_system))
        .nest("/{system_id}/agents", agents::routes())
        .nest("/{system_id}/events", events::routes())
}

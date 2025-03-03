use crate::handlers::system::get_system_info;
use crate::server::AppState;
use axum::{Router, routing::get};

/// Create the system routes with state
pub fn routes() -> Router<AppState> {
    Router::new().route("/system/info", get(get_system_info))
}

use crate::handlers::system::get_system_info;
use axum::{Router, routing::get};

/// Create the system routes
pub fn routes() -> Router {
    Router::new().route("/system/info", get(get_system_info))
}

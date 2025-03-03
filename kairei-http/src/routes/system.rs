use crate::handlers::system::get_system_info;
use crate::session::manager::SessionManager;
use axum::{Router, routing::get};

/// Create the system routes with state
pub fn routes() -> Router<SessionManager> {
    Router::new().route("/system/info", get(get_system_info))
}

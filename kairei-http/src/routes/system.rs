use axum::{Router, routing::get};
use std::sync::Arc;

use crate::handlers::system::get_system_info;
use crate::integration::KaireiSystem;

/// Create the system routes
pub fn routes(kairei_system: Arc<KaireiSystem>) -> Router {
    Router::new().route(
        "/system/info",
        get(get_system_info).with_state(kairei_system),
    )
}

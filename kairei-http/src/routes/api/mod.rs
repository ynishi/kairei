use axum::Router;

use crate::server::AppState;

pub mod v1;

/// Create the v1 API router with state
pub fn api_v1_router() -> Router<AppState> {
    Router::new().merge(v1::system::routes())
}

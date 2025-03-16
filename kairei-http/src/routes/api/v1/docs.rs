//! API routes for DSL documentation.

use crate::handlers::{
    get_all_documentation, get_category_documentation, get_parser_documentation,
};
use crate::server::AppState;
use axum::{Router, routing::get};

/// Create the documentation routes with state
pub fn routes() -> Router<AppState> {
    Router::new().nest("/docs", docs_routes())
}

/// Create the documentation routes
fn docs_routes() -> Router<AppState> {
    Router::new()
        .route("/dsl", get(get_all_documentation))
        .route("/dsl/{category}", get(get_category_documentation))
        .route("/dsl/{category}/{name}", get(get_parser_documentation))
}

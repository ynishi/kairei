//! API routes for DSL documentation.

use crate::handlers::{
    export_documentation, get_all_documentation, get_category_documentation, get_documentation_map,
    get_parser_documentation,
};
use crate::server::AppState;
use axum::{
    Router,
    routing::{get, post},
};

/// Create the documentation routes with state
pub fn routes() -> Router<AppState> {
    Router::new().nest("/docs", docs_routes())
}

/// Create the documentation routes
fn docs_routes() -> Router<AppState> {
    Router::new()
        .route("/dsl", get(get_all_documentation))
        .route("/dsl/map", get(get_documentation_map))
        .route("/dsl/export", post(export_documentation))
        .route("/dsl/{category}", get(get_category_documentation))
        .route("/dsl/{category}/{name}", get(get_parser_documentation))
}

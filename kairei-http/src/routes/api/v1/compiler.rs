use axum::{Router, routing::post};

use crate::{
    server::AppState,
    services::compiler::handlers::{suggest_fixes, validate_dsl},
};

/// Create the compiler routes with state
pub fn routes() -> Router<AppState> {
    Router::new().nest("/compiler", compiler_router())
}

pub fn compiler_router() -> Router<AppState> {
    Router::new()
        .route("/validate", post(validate_dsl))
        .route("/suggest", post(suggest_fixes))
}

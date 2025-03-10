use axum::{Router, routing::post};

use crate::{
    handlers::{suggest_fixes, validate_dsl},
    server::AppState,
};

/// Create the compiler routes with state
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/validate", post(validate_dsl))
        .route("/suggest", post(suggest_fixes))
}

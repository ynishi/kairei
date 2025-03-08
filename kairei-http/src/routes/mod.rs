pub mod agents;
pub mod events;
pub mod system;

use crate::handlers;
use crate::{models::CreateSystemRequest, server::AppState};
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(handlers::create_system),
    components(schemas(CreateSystemRequest))
)]
struct ApiDoc;

/// Create the main API router with state
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health_check))
        .nest("/api/v1", api_v1_router())
}

/// Create the v1 API router with state
fn api_v1_router() -> Router<AppState> {
    Router::new().merge(system::routes())
}

/// Health check endpoint for container health monitoring
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

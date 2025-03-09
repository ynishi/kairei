pub mod api;
pub mod swagger;

use crate::server::{AppState, ServerConfig};
use api::api_v1_router;
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

use swagger::ApiDoc;
use utoipa::{OpenApi, openapi::Server};
use utoipa_swagger_ui::SwaggerUi;

/// Create the main API router with state
pub fn create_api_router(config: &ServerConfig) -> Router<AppState> {
    let mut doc = ApiDoc::openapi();
    doc.servers = doc.servers.map(|mut servers| {
        let mut currents: Vec<Server> = config
            .servers
            .iter()
            .map(|s| {
                let mut url = s.to_string();
                if !url.ends_with('/') {
                    url.push('/');
                }
                Server::new(url)
            })
            .collect();
        servers.append(&mut currents);
        servers
    });
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", doc))
        .route("/health", get(health_check))
        .nest("/api/v1", api_v1_router())
}

/// Health check endpoint for container health monitoring
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

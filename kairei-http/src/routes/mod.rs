pub mod agents;
pub mod events;
pub mod system;

use crate::handlers;
use crate::models::agents::{
    AgentStatistics, AgentStatus, GetAgentResponse, ListAgentsResponse, ScaleDownAgentRequest,
    ScaleUpAgentRequest, SendRequestAgentRequest, SendRequestAgentResponse, ValidationResult,
};
use crate::models::events::{
    AgentRequestPayload, AgentRequestResponse, EventRequest, EventResponse, EventStatus,
    RequestStatus,
};
use crate::models::{
    CreateSystemRequest, CreateSystemResponse, ListSystemsResponse, StartSystemRequest, SystemInfo,
    SystemStatistics, SystemStatus,
};
use crate::server::AppState;
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::system::create_system,
        handlers::system::get_system,
        handlers::system::list_systems,
        handlers::system::start_system,
        handlers::system::stop_system,
        handlers::system::delete_system,
        handlers::agents::get_agent,
        handlers::agents::list_agents,
        handlers::agents::start_agent,
        handlers::agents::stop_agent,
        handlers::agents::scale_up_agent,
        handlers::agents::scale_down_agent,
        handlers::agents::request_agent,
        handlers::events::list_events,
        handlers::events::emit_event,
        handlers::events::subscribe_event
    ),
    components(schemas(
        CreateSystemRequest,
        CreateSystemResponse,
        ListSystemsResponse,
        StartSystemRequest,
        SystemInfo,
        SystemStatus,
        SystemStatistics,
        GetAgentResponse,
        ListAgentsResponse,
        ScaleUpAgentRequest,
        ScaleDownAgentRequest,
        SendRequestAgentRequest,
        SendRequestAgentResponse,
        AgentStatus,
        ValidationResult,
        AgentStatistics,
        EventRequest,
        EventResponse,
        EventStatus,
        AgentRequestPayload,
        AgentRequestResponse,
        RequestStatus
    ))
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

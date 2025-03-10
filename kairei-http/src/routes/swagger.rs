use crate::handlers::agents;
use crate::handlers::events;
use crate::handlers::system;
use crate::models::CompileSystemRequest;
use crate::models::CompileSystemResponse;
use crate::services::compiler::handlers as compiler;

use utoipa::OpenApi;

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
use crate::services::compiler::models::{
    ErrorLocation, SuggestionRequest, SuggestionResponse, ValidationError, ValidationRequest,
    ValidationResponse, ValidationSuggestion, ValidationWarning,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        system::create_system,
        system::get_system,
        system::list_systems,
        system::compile_system,
        system::start_system,
        system::stop_system,
        system::delete_system,
        agents::get_agent,
        agents::list_agents,
        agents::start_agent,
        agents::stop_agent,
        agents::scale_up_agent,
        agents::scale_down_agent,
        agents::request_agent,
        events::list_events,
        events::emit_event,
        events::subscribe_event,
        compiler::validate_dsl,
        compiler::suggest_fixes
    ),
    components(schemas(
        CreateSystemRequest,
        CreateSystemResponse,
        ListSystemsResponse,
        CompileSystemRequest,
        CompileSystemResponse,
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
        RequestStatus,
        ValidationRequest,
        ValidationResponse,
        ValidationError,
        ValidationWarning,
        ErrorLocation,
        ValidationSuggestion,
        SuggestionRequest,
        SuggestionResponse
    )),
    tags(
        (name = "compiler", description = "Compiler API")
    ),
    servers(
        (url = "http://localhost:3000/api/v1", description = "Local development server"),
    )
)]
pub struct ApiDoc;

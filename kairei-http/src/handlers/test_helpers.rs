use crate::auth::AuthStore;
use crate::models::agents::{
    AgentCreationRequest, AgentCreationResponse, AgentDetails, AgentStatistics, AgentStatus,
    ValidationResult,
};
use crate::models::events::{
    AgentRequestPayload, AgentRequestResponse, EventRequest, EventResponse, EventStatus,
    RequestStatus,
};
use crate::models::system::{SystemInfo, SystemStatistics, SystemStatus};
use crate::server::AppState;
use crate::services::compiler::models::{
    ErrorLocation, SuggestionRequest, SuggestionResponse, ValidationError, ValidationRequest,
    ValidationResponse, ValidationSuggestion, ValidationWarning,
};
use crate::session::manager::SessionManager;
use axum::{extract::Path, http::StatusCode, response::Json};
use serde_json::json;
use uuid::Uuid;

/// Create a test AppState for testing
pub fn create_test_state() -> AppState {
    let session_manager = SessionManager::default();
    let auth_store = AuthStore::default();

    // Ensure the auth store has the default test users and API keys
    // This is already done in AuthStore::default() which calls AuthStore::with_defaults()
    AppState {
        session_manager,
        auth_store,
        ..Default::default()
    }
}

/// Create a test user with the given API key for testing
pub fn create_test_user_with_api_key(
    app_state: &AppState,
    user_id: &str,
    username: &str,
    is_admin: bool,
    api_key: &str,
) {
    let user = if is_admin {
        crate::models::user::User::new_admin(user_id, username)
    } else {
        crate::models::user::User::new_user(user_id, username)
    };

    app_state.auth_store.add_user(user.clone());
    app_state.auth_store.add_api_key(api_key, user.user_id);
}

/// Test version of get_system that doesn't require State
pub async fn test_get_system() -> Json<SystemInfo> {
    let info = SystemInfo {
        version: "0.1.0".to_string(),
        status: SystemStatus::Running,
        capabilities: vec![
            "agent_management".to_string(),
            "event_processing".to_string(),
            "session_management".to_string(),
        ],
        statistics: SystemStatistics {
            agent_count: 5,
            event_count: 120,
            uptime_seconds: 3600,
        },
    };

    Json(info)
}

/// Test version of create_agent that doesn't require State
pub async fn test_create_agent(
    Json(payload): Json<AgentCreationRequest>,
) -> (StatusCode, Json<AgentCreationResponse>) {
    let agent_id = format!("{}-001", payload.name.to_lowercase());

    let response = AgentCreationResponse {
        agent_id,
        status: AgentStatus::Created,
        validation_result: ValidationResult {
            success: true,
            warnings: vec![],
        },
    };

    (StatusCode::CREATED, Json(response))
}

/// Test version of get_agent that doesn't require State
pub async fn test_get_agent(
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetails>, StatusCode> {
    // Simulate agent not found
    if agent_id.contains("not-found") {
        return Err(StatusCode::NOT_FOUND);
    }

    let details = AgentDetails {
        agent_id: agent_id.clone(),
        name: "WeatherAgent".to_string(),
        status: AgentStatus::Running,
        created_at: "2025-03-03T12:00:00Z".to_string(),
        statistics: AgentStatistics {
            events_processed: 42,
            requests_handled: 15,
            uptime_seconds: 1800,
        },
    };

    Ok(Json(details))
}

/// Test version of send_event that doesn't require State
pub async fn test_send_event(Json(payload): Json<EventRequest>) -> Json<EventResponse> {
    let event_id = format!(
        "evt-{}",
        Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    let response = EventResponse {
        event_id,
        status: EventStatus::Delivered,
        delivered_to: payload.target_agents.len().max(1), // If no targets specified, assume broadcast
    };

    Json(response)
}

/// Test version of send_agent_request that doesn't require State
pub async fn test_send_agent_request(
    Path(agent_id): Path<String>,
    Json(payload): Json<AgentRequestPayload>,
) -> Result<Json<AgentRequestResponse>, StatusCode> {
    // Simulate agent not found
    if agent_id.contains("not-found") {
        return Err(StatusCode::NOT_FOUND);
    }

    let request_id = format!(
        "req-{}",
        Uuid::new_v4().to_string().split('-').next().unwrap()
    );

    // Mock response for GetWeather request
    let result = if payload.request_type == "GetWeather" {
        let location = payload
            .parameters
            .get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        Some(json!({
            "temperature": 25.5,
            "conditions": "Sunny",
            "humidity": 60,
            "location": location
        }))
    } else {
        Some(json!({
            "message": "Request processed successfully"
        }))
    };

    let response = AgentRequestResponse {
        request_id,
        status: RequestStatus::Completed,
        result,
        error: None,
    };

    Ok(Json(response))
}

/// Test version of validate_dsl that doesn't require State
pub async fn test_validate_dsl(Json(payload): Json<ValidationRequest>) -> Json<ValidationResponse> {
    // For valid DSL code
    if payload.code.contains("micro") && !payload.code.contains("ERROR") {
        Json(ValidationResponse {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: None,
        })
    } else {
        // For invalid DSL code
        let error = ValidationError {
            message: "Parse error: unexpected token".to_string(),
            location: ErrorLocation {
                line: 1,
                column: payload.code.find("ERROR").unwrap_or(1),
                context: payload.code.clone(),
            },
            error_code: "E1001".to_string(),
            suggestion: "Check syntax for errors".to_string(),
        };

        let warning = if payload.code.contains("WARNING") {
            vec![ValidationWarning {
                message: "Potential performance issue".to_string(),
                location: ErrorLocation {
                    line: 1,
                    column: payload.code.find("WARNING").unwrap_or(1),
                    context: payload.code.clone(),
                },
                warning_code: "W1001".to_string(),
            }]
        } else {
            Vec::new()
        };

        Json(ValidationResponse {
            valid: false,
            errors: vec![error.clone()],
            warnings: warning,
            suggestions: Some(ValidationSuggestion {
                code: payload.code.replace("ERROR", ""),
            }),
        })
    }
}

/// Test version of suggest_fixes that doesn't require State
pub async fn test_suggest_fixes(
    Json(payload): Json<SuggestionRequest>,
) -> Json<SuggestionResponse> {
    // Simple implementation that removes "ERROR" from the code
    let fixed_code = payload.code.replace("ERROR", "");

    Json(SuggestionResponse {
        original_code: payload.code,
        fixed_code,
        explanation: "Removed syntax errors from the code.".to_string(),
    })
}

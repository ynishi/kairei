use crate::models::agents::{
    AgentCreationRequest, AgentCreationResponse, AgentDetails, AgentStatistics, AgentStatus,
    ValidationResult,
};
use crate::models::events::{
    AgentRequestPayload, AgentRequestResponse, EventRequest, EventResponse, EventStatus,
    RequestStatus,
};
use crate::models::system::{SystemInfo, SystemStatistics, SystemStatus};
use axum::{extract::Path, http::StatusCode, response::Json};
use serde_json::json;
use uuid::Uuid;

/// Test version of get_system_info that doesn't require State
pub async fn test_get_system_info() -> Json<SystemInfo> {
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

/// Test version of get_agent_details that doesn't require State
pub async fn test_get_agent_details(
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

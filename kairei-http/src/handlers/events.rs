use axum::{
    extract::{Path, State},
    response::Json,
};
use std::sync::Arc;

use crate::{
    error::AppError,
    integration::KaireiSystem,
    models::events::{
        AgentRequestPayload, AgentRequestResponse, EventRequest, EventResponse, EventStatus,
        RequestStatus,
    },
};

/// Send an event
///
/// Sends an event to one or more agents.
pub async fn send_event(
    State(kairei_system): State<Arc<KaireiSystem>>,
    Json(payload): Json<EventRequest>,
) -> Result<Json<EventResponse>, AppError> {
    // Clone target_agents before using it
    let target_agents = payload.target_agents.clone();
    let target_count = target_agents.len().max(1); // If no targets specified, assume broadcast

    // Send the event using the core API
    let event_id = kairei_system
        .event_api
        .send_typed_event(payload.event_type, payload.payload, target_agents)
        .await?;

    let response = EventResponse {
        event_id,
        status: EventStatus::Delivered,
        delivered_to: target_count,
    };

    Ok(Json(response))
}

/// Send a request to an agent
///
/// Sends a request to a specific agent and returns the result.
pub async fn send_agent_request(
    State(kairei_system): State<Arc<KaireiSystem>>,
    Path(agent_id): Path<String>,
    Json(payload): Json<AgentRequestPayload>,
) -> Result<Json<AgentRequestResponse>, AppError> {
    // Send the request using the core API
    let result = kairei_system
        .event_api
        .send_agent_request(&agent_id, payload.request_type, payload.parameters)
        .await?;

    // Create the response
    let response = AgentRequestResponse {
        request_id: format!(
            "req-{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
        ),
        status: RequestStatus::Completed,
        result: Some(result),
        error: None,
    };

    Ok(Json(response))
}

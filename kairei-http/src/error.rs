//! Error handling for kairei-http
//!
//! This module provides error handling for the HTTP API.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kairei_core::{agent_registry::AgentError, event_bus::EventError, system::SystemError};
use serde_json::json;
use std::cmp::PartialEq;

/// Application error type
#[derive(Debug)]
pub enum AppError {
    /// System error
    System(SystemError),

    /// Agent error
    Agent(AgentError),

    /// Event error
    Event(EventError),

    /// Internal error
    Internal(String),
}

impl From<SystemError> for AppError {
    fn from(err: SystemError) -> Self {
        Self::System(err)
    }
}

impl From<AgentError> for AppError {
    fn from(err: AgentError) -> Self {
        Self::Agent(err)
    }
}

impl From<EventError> for AppError {
    fn from(err: EventError) -> Self {
        Self::Event(err)
    }
}

impl PartialEq<StatusCode> for AppError {
    fn eq(&self, status_code: &StatusCode) -> bool {
        let (error_status, _) = self.status_and_message();
        &error_status == status_code
    }
}

impl AppError {
    /// Get the status code and error message for this error
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            Self::System(SystemError::Agent(AgentError::AgentNotFound { agent_id })) => (
                StatusCode::NOT_FOUND,
                format!("Agent not found: {}", agent_id),
            ),
            Self::System(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            Self::Agent(AgentError::AgentNotFound { agent_id }) => (
                StatusCode::NOT_FOUND,
                format!("Agent not found: {}", agent_id),
            ),
            Self::Agent(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            Self::Event(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = self.status_and_message();

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

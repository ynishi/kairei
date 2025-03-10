use serde::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;

/// Request for validating DSL code
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationRequest {
    /// DSL code to validate
    pub code: String,
}

/// Response for DSL validation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationResponse {
    /// Whether the DSL code is valid
    pub valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Suggestions for fixing errors
    pub suggestions: Option<ValidationSuggestion>,
}

/// Detailed validation error
#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Error location
    pub location: ErrorLocation,
    /// Error code
    pub error_code: String,
    /// Suggestion for fixing the error
    pub suggestion: String,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,
    /// Warning location
    pub location: ErrorLocation,
    /// Warning code
    pub warning_code: String,
}

/// Error location information
#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Context around the error
    pub context: String,
}

/// Suggestions for fixing validation errors
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationSuggestion {
    /// Suggested code with fixes
    pub code: String,
}

/// Request for suggesting fixes for DSL code
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuggestionRequest {
    /// Original DSL code
    pub code: String,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
}

/// Response for suggestion request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuggestionResponse {
    /// Original DSL code
    pub original_code: String,
    /// Fixed DSL code
    pub fixed_code: String,
    /// Explanation of the fixes
    pub explanation: String,
}

// Cloud Logging compatible structures

/// Cloud Logging compatible log structure
#[derive(Debug, Clone, Serialize)]
pub struct CloudLog {
    /// Log severity level
    pub severity: String,
    /// Log message payload
    pub message: LogPayload,
    /// Timestamp in ISO 8601 format
    pub timestamp: String,
    /// Trace information
    pub trace: String,
}

/// Log payload containing request/response information and error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPayload {
    /// Request ID for tracking
    pub x_request_id: Option<String>,
    /// Host information
    pub host: Option<String>,
    /// User agent information
    pub user_agent: Option<String>,
    /// HTTP method
    pub method: Option<String>,
    /// Request URI
    pub uri: Option<String>,
    /// HTTP status code
    pub status: Option<String>,
    /// Request/response latency
    pub latency: Option<Duration>,
    /// Log entry type
    pub kind: LogKind,
    /// Error message details (if applicable)
    pub error_message: Option<LogErrorMessage>,

    /// Content type of the request
    pub content_type: Option<String>,
    /// Referrer information
    pub referrer: Option<String>,
    /// X-Forwarded-For header (client IP when behind proxy)
    pub x_forwarded_for: Option<String>,
    /// Google Cloud Trace context for distributed tracing
    pub x_cloud_trace_context: Option<String>,
}

/// Log entry type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogKind {
    /// Request log
    Request,
    /// Response log
    Response,
    /// Error log
    Err,
}

/// Detailed error message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogErrorMessage {
    /// Error type
    pub r#type: String,
    /// Error title
    pub title: String,
    /// Detailed error message
    pub detail: String,
}

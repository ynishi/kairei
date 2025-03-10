use axum::{extract::State, http::header::HeaderMap, response::Json};
use chrono::Utc;
use kairei_core::system::SystemError;
use tracing::{error, info};

use crate::{
    server::AppState,
    services::compiler::models::{
        CloudLog, ErrorLocation, LogErrorMessage, LogKind, LogPayload, SuggestionRequest,
        SuggestionResponse, ValidationError, ValidationRequest, ValidationResponse,
        ValidationSuggestion,
    },
};

/// Error logger for compiler service
#[derive(Debug, Clone, Default)]
pub struct ErrorLogger {
    // Configuration for cloud logging and Sentry would go here
}

impl ErrorLogger {
    /// Create a new ErrorLogger
    pub fn new() -> Self {
        Self {}
    }

    /// Log validation errors in Cloud Logging format
    #[allow(clippy::too_many_arguments)]
    pub async fn log_validation_error(
        &self,
        request_id: Option<String>,
        host: Option<String>,
        user_agent: Option<String>,
        method: Option<String>,
        uri: Option<String>,
        content_type: Option<String>,
        referrer: Option<String>,
        x_forwarded_for: Option<String>,
        x_cloud_trace_context: Option<String>,
        code_hash: &str,
        errors: &[ValidationError],
    ) {
        // Basic logging for backward compatibility
        info!(
            "Validation error: code_hash={}, error_count={}",
            code_hash,
            errors.len()
        );

        // Cloud Logging compatible structured logging
        for error in errors {
            let log = CloudLog {
                severity: "ERROR".to_string(),
                message: LogPayload {
                    x_request_id: request_id.clone(),
                    host: host.clone(),
                    user_agent: user_agent.clone(),
                    method: method.clone(),
                    uri: uri.clone(),
                    status: None,
                    latency: None,
                    kind: LogKind::Err,
                    error_message: Some(LogErrorMessage {
                        r#type: format!("ValidationError:{}", error.error_code),
                        title: format!("Validation error in code (hash: {})", code_hash),
                        detail: error.message.clone(),
                    }),
                    content_type: content_type.clone(),
                    referrer: referrer.clone(),
                    x_forwarded_for: x_forwarded_for.clone(),
                    x_cloud_trace_context: x_cloud_trace_context.clone(),
                },
                timestamp: Utc::now().format("%Y/%m/%dT%H:%M:%S%z").to_string(),
                trace: format!(
                    "line:{},column:{}",
                    error.location.line, error.location.column
                ),
            };

            // Log the structured error
            if let Ok(log_json) = serde_json::to_string(&log) {
                error!("{}", log_json);
            }
        }
    }

    /// Log system errors in Cloud Logging format
    #[allow(clippy::too_many_arguments)]
    pub async fn log_system_error(
        &self,
        request_id: Option<String>,
        host: Option<String>,
        user_agent: Option<String>,
        method: Option<String>,
        uri: Option<String>,
        content_type: Option<String>,
        referrer: Option<String>,
        x_forwarded_for: Option<String>,
        x_cloud_trace_context: Option<String>,
        error: &SystemError,
    ) {
        let log = CloudLog {
            severity: "ERROR".to_string(),
            message: LogPayload {
                x_request_id: request_id,
                host,
                user_agent,
                method,
                uri,
                status: None,
                latency: None,
                kind: LogKind::Err,
                error_message: Some(LogErrorMessage {
                    r#type: format!("SystemError:{}", error_type_from_system_error(error)),
                    title: "System error during compilation".to_string(),
                    detail: error.to_string(),
                }),
                content_type,
                referrer,
                x_forwarded_for,
                x_cloud_trace_context,
            },
            timestamp: Utc::now().format("%Y/%m/%dT%H:%M:%S%z").to_string(),
            trace: "".to_string(),
        };

        // Log the structured error
        if let Ok(log_json) = serde_json::to_string(&log) {
            error!("{}", log_json);
        }
    }
}

/// Helper function to extract error type from SystemError
fn error_type_from_system_error(error: &SystemError) -> String {
    match error {
        SystemError::Ast(_) => "AstError",
        SystemError::Runtime(_) => "RuntimeError",
        SystemError::Event(_) => "EventError",
        SystemError::Agent(_) => "AgentError",
        SystemError::Feature(_) => "FeatureError",
        SystemError::Provider(_) => "ProviderError",
        SystemError::Request(_) => "RequestError",
        SystemError::Initialization(_) => "InitializationError",
        SystemError::ScalingNotEnoughAgents { .. } => "ScalingError",
        SystemError::ScaleManagerNotFound { .. } => "ScaleManagerError",
        SystemError::InvalidStateTransition { .. } => "StateTransitionError",
        SystemError::UnsupportedRequest { .. } => "UnsupportedRequestError",
        SystemError::ReceiveResponseFailed { .. } => "ResponseFailedError",
        SystemError::ReceiveResponseTimeout { .. } => "ResponseTimeoutError",
    }
    .to_string()
}

/// Validate DSL code
#[utoipa::path(
    post,
    path = "/compiler/validate",
    request_body = ValidationRequest,
    responses(
        (status = 200, description = "DSL validated successfully", body = ValidationResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn validate_dsl(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ValidationRequest>,
) -> Json<ValidationResponse> {
    // Extract headers for logging
    let (
        request_id,
        host,
        user_agent,
        content_type,
        referrer,
        x_forwarded_for,
        x_cloud_trace_context,
    ) = (
        headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("referrer")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("x-cloud-trace-context")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
    );

    // Fixed values for this endpoint
    let method = Some("POST".to_string());
    let uri = Some("/compiler/validate".to_string());

    if state.compiler_system_manager.is_none() {
        return Json(ValidationResponse {
            valid: false,
            errors: vec![ValidationError {
                message: "Compiler system not available".to_string(),
                location: ErrorLocation {
                    line: 1,
                    column: 1,
                    context: "".to_string(),
                },
                error_code: "E1006".to_string(),
                suggestion: "Check system configuration".to_string(),
            }],
            warnings: Vec::new(),
            suggestions: None,
        });
    }

    let manager = state.compiler_system_manager.clone().unwrap();
    let error_logger = ErrorLogger::new();
    if payload.code.is_empty() {
        return Json(ValidationResponse {
            valid: false,
            errors: vec![ValidationError {
                message: "Empty code provided".to_string(),
                location: ErrorLocation {
                    line: 1,
                    column: 1,
                    context: "".to_string(),
                },
                error_code: "E1005".to_string(),
                suggestion: "Provide valid DSL code".to_string(),
            }],
            warnings: Vec::new(),
            suggestions: None,
        });
    }

    // Attempt to parse the DSL using the System
    match manager.validate_dsl(&payload.code).await {
        Ok(_) => {
            // DSL is valid
            Json(ValidationResponse {
                valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                suggestions: None,
            })
        }
        Err(err) => {
            // DSL has errors
            let errors = convert_system_error_to_validation_errors(&err, &payload.code);

            // Log system error first
            error_logger
                .log_system_error(
                    request_id.clone(),
                    host.clone(),
                    user_agent.clone(),
                    method.clone(),
                    uri.clone(),
                    content_type.clone(),
                    referrer.clone(),
                    x_forwarded_for.clone(),
                    x_cloud_trace_context.clone(),
                    &err,
                )
                .await;

            // Log validation errors
            // Using a simple hash for the prototype
            let code_hash = format!("{:x}", payload.code.len());
            error_logger
                .log_validation_error(
                    request_id,
                    host,
                    user_agent,
                    method,
                    uri,
                    content_type,
                    referrer,
                    x_forwarded_for,
                    x_cloud_trace_context,
                    &code_hash,
                    &errors,
                )
                .await;

            // Generate suggestions if there are errors
            let suggestions = if !errors.is_empty() {
                Some(generate_suggestions(&payload.code, &errors))
            } else {
                None
            };

            Json(ValidationResponse {
                valid: false,
                errors,
                warnings: Vec::new(),
                suggestions,
            })
        }
    }
}

/// Suggest fixes for DSL code
#[utoipa::path(
    post,
    path = "/compiler/suggest",
    request_body = SuggestionRequest,
    responses(
        (status = 200, description = "Suggestions generated successfully", body = SuggestionResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn suggest_fixes(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SuggestionRequest>,
) -> Json<SuggestionResponse> {
    // Extract headers for logging
    let (
        request_id,
        host,
        user_agent,
        content_type,
        referrer,
        x_forwarded_for,
        x_cloud_trace_context,
    ) = (
        headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("referrer")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        headers
            .get("x-cloud-trace-context")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
    );

    // Fixed values for this endpoint
    let method = Some("POST".to_string());
    let uri = Some("/compiler/suggest".to_string());

    // In a real implementation, this would use an LLM or rule-based system to generate fixes
    // For now, we'll just provide a simple implementation

    let error_logger = ErrorLogger::new();
    for error in &payload.errors {
        error_logger
            .log_validation_error(
                request_id.clone(),
                host.clone(),
                user_agent.clone(),
                method.clone(),
                uri.clone(),
                content_type.clone(),
                referrer.clone(),
                x_forwarded_for.clone(),
                x_cloud_trace_context.clone(),
                &format!("{:x}", payload.code.len()),
                &[error.clone()],
            )
            .await;
    }

    let fixed_code = payload.code.clone();
    let explanation = "Suggested fixes for the provided errors.".to_string();

    Json(SuggestionResponse {
        original_code: payload.code,
        fixed_code,
        explanation,
    })
}

/// Convert System errors to validation errors
fn convert_system_error_to_validation_errors(
    system_error: &SystemError,
    code: &str,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    match system_error {
        SystemError::Ast(ast_error) => {
            // Convert AST errors
            match ast_error {
                kairei_core::ASTError::ParseError { target, message } => {
                    errors.push(ValidationError {
                        message: format!("Parse error in {}: {}", target, message),
                        location: ErrorLocation {
                            line: 1,
                            column: 1,
                            context: extract_context(code, 1, 1),
                        },
                        error_code: "E1001".to_string(),
                        suggestion: "Check syntax for errors".to_string(),
                    });
                }
                kairei_core::ASTError::TokenizeError(tokenizer_error) => {
                    // Extract location information from tokenizer error
                    match tokenizer_error {
                        kairei_core::tokenizer::token::TokenizerError::ParseError {
                            message,
                            found,
                            span,
                        } => {
                            errors.push(ValidationError {
                                message: message.clone(),
                                location: ErrorLocation {
                                    line: span.line,
                                    column: span.column,
                                    context: extract_context(code, span.line, span.column),
                                },
                                error_code: "E1002".to_string(),
                                suggestion: format!("Unexpected token: {}", found),
                            });
                        }
                    }
                }
                kairei_core::ASTError::TypeCheckError(type_check_error) => {
                    errors.push(ValidationError {
                        message: type_check_error.to_string(),
                        location: ErrorLocation {
                            line: 1,
                            column: 1,
                            context: extract_context(code, 1, 1),
                        },
                        error_code: "E1003".to_string(),
                        suggestion: "Check type compatibility".to_string(),
                    });
                }
                kairei_core::ASTError::ASTNotFound(name) => {
                    errors.push(ValidationError {
                        message: format!("AST not found: {}", name),
                        location: ErrorLocation {
                            line: 1,
                            column: 1,
                            context: extract_context(code, 1, 1),
                        },
                        error_code: "E1004".to_string(),
                        suggestion: "Check agent name".to_string(),
                    });
                }
            }
        }
        // Handle other system error types
        _ => {
            errors.push(ValidationError {
                message: format!("System error: {}", system_error),
                location: ErrorLocation {
                    line: 1,
                    column: 1,
                    context: extract_context(code, 1, 1),
                },
                error_code: "E1000".to_string(),
                suggestion: "Check system configuration".to_string(),
            });
        }
    }

    errors
}

/// Extract context around an error location
fn extract_context(code: &str, line: usize, _column: usize) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let mut context = String::new();

    // Get a few lines before and after the error
    let start_line = line.saturating_sub(2);
    let end_line = (line + 2).min(lines.len());

    for i in start_line..end_line {
        if i < lines.len() {
            context.push_str(lines[i]);
            context.push('\n');
        }
    }

    context
}

/// Generate suggestions for fixing errors
fn generate_suggestions(code: &str, _errors: &[ValidationError]) -> ValidationSuggestion {
    // In a real implementation, this would use an LLM or rule-based system to generate suggestions
    // For now, we'll just return the original code
    ValidationSuggestion {
        code: code.to_string(),
    }
}

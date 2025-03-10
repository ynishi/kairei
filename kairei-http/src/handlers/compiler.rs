use axum::{extract::State, response::Json};
use kairei_core::{ASTError, ast_registry::AstRegistry};
use tracing::info;

use crate::{
    models::{
        ErrorLocation, SuggestionRequest, SuggestionResponse, ValidationError, ValidationRequest,
        ValidationResponse, ValidationSuggestion,
    },
    server::AppState,
};

/// Error logger for compiler service
#[derive(Debug, Clone, Default)]
pub struct ErrorLogger {
    // Configuration for cloud logging and Sentry would go here
}

impl ErrorLogger {
    /// Log validation errors
    pub async fn log_validation_error(&self, code_hash: &str, errors: &[ValidationError]) {
        // Hash the code to avoid storing potentially sensitive information
        info!(
            "Validation error: code_hash={}, error_count={}",
            code_hash,
            errors.len()
        );

        // In a real implementation, this would send to cloud logging or Sentry
    }
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
    State(_state): State<AppState>,
    Json(payload): Json<ValidationRequest>,
) -> Json<ValidationResponse> {
    let registry = AstRegistry::default();
    let error_logger = ErrorLogger::default();

    // Attempt to parse the DSL
    match registry.create_ast_from_dsl(&payload.code).await {
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
            let errors = convert_ast_error_to_validation_errors(&err, &payload.code);

            // Log validation errors
            // Using a simple hash for the prototype
            let code_hash = format!("{:x}", payload.code.len());
            error_logger.log_validation_error(&code_hash, &errors).await;

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
    Json(payload): Json<SuggestionRequest>,
) -> Json<SuggestionResponse> {
    // In a real implementation, this would use an LLM or rule-based system to generate fixes
    // For now, we'll just provide a simple implementation

    let fixed_code = payload.code.clone();
    let explanation = "Suggested fixes for the provided errors.".to_string();

    Json(SuggestionResponse {
        original_code: payload.code,
        fixed_code,
        explanation,
    })
}

/// Convert AST errors to validation errors
fn convert_ast_error_to_validation_errors(
    ast_error: &ASTError,
    code: &str,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    match ast_error {
        ASTError::ParseError { target, message } => {
            // For parse errors, we don't have location information in the current implementation
            // In a real implementation, we would extract location information from the error
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
        ASTError::TokenizeError(tokenizer_error) => {
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
        ASTError::TypeCheckError(type_check_error) => {
            // Extract location information from type check error
            // This would depend on the specific type check error type
            // For now, we'll just use a generic error
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
        ASTError::ASTNotFound(name) => {
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

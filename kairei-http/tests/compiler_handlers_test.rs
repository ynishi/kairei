use std::sync::Arc;

use axum::{extract::State, http::header::HeaderMap, response::Json};
use kairei_http::{
    server::AppState,
    services::compiler::{
        CompilerSystemManager,
        handlers::{suggest_fixes, validate_dsl},
        models::{ErrorLocation, SuggestionRequest, ValidationError, ValidationRequest},
    },
};

#[tokio::test]
async fn test_validate_dsl_handler_integration() {
    // Create a test state
    let mut compiler_manager = CompilerSystemManager::default();
    compiler_manager.initialize(false).await.unwrap();
    let app_state = AppState {
        compiler_system_manager: Some(Arc::new(compiler_manager)),
        ..Default::default()
    };
    // Test with valid DSL code
    let valid_payload = ValidationRequest {
        code: "micro TestAgent { }".to_string(),
    };

    // Call the actual handler
    let response = validate_dsl(
        State(app_state.clone()),
        HeaderMap::new(),
        Json(valid_payload),
    )
    .await;

    // Verify the response
    assert!(response.0.valid);
    assert!(response.0.errors.is_empty());

    // Test with invalid DSL code
    let invalid_payload = ValidationRequest {
        code: "micro TestAgent { invalid syntax }".to_string(),
    };

    // Call the actual handler
    let response = validate_dsl(State(app_state), HeaderMap::new(), Json(invalid_payload)).await;

    // Verify the response
    assert!(!response.0.valid);
    assert!(!response.0.errors.is_empty());

    // Check error details
    let error = &response.0.errors[0];
    assert!(!error.message.is_empty());
    assert!(!error.error_code.is_empty());
    assert!(!error.suggestion.is_empty());
}

#[tokio::test]
async fn test_suggest_fixes_handler_integration() {
    // Create a test state
    let app_state = AppState::default();

    // Create a validation error
    let error = ValidationError {
        message: "Parse error: unexpected token".to_string(),
        location: ErrorLocation {
            line: 1,
            column: 15,
            end_line: None,
            end_column: None,
            start_position: None,
            end_position: None,
            context: "micro TestAgent { invalid syntax }".to_string(),
            token_text: None,
        },
        error_code: "E1001".to_string(),
        suggestion: "Check syntax for errors".to_string(),
    };

    // Create a request payload
    let payload = SuggestionRequest {
        code: "micro TestAgent { invalid syntax }".to_string(),
        errors: vec![error],
    };

    // Call the actual handler
    let response = suggest_fixes(State(app_state), HeaderMap::new(), Json(payload.clone())).await;

    // Verify the response
    assert_eq!(response.0.original_code, payload.code);
    assert!(!response.0.fixed_code.is_empty());
    assert!(!response.0.explanation.is_empty());
}

#[tokio::test]
async fn test_validate_dsl_handler_empty_code() {
    // Create a test state
    let app_state = AppState::default();

    // Test with empty code
    let empty_payload = ValidationRequest {
        code: "".to_string(),
    };

    // Call the actual handler
    let response = validate_dsl(State(app_state), HeaderMap::new(), Json(empty_payload)).await;

    // Verify the response
    assert!(!response.0.valid);
    assert!(!response.0.errors.is_empty());
}

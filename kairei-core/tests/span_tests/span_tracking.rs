//! Integration tests for span tracking through the compilation pipeline
//!
//! These tests verify that location information is properly preserved
//! through the tokenization, parsing, and type checking phases.

use kairei_core::{ast::ASTError, ast_registry::AstRegistry, tokenizer::token::TokenizerError};
use tracing::debug;

/// Test that tokenization errors preserve span information
#[tokio::test]
async fn test_tokenization_error_preserves_span() {
    // Invalid token in the DSL code
    let invalid_dsl = "micro TestAgent { @invalid_token }";

    // Parse the DSL code
    let result = AstRegistry::default()
        .create_ast_from_dsl(invalid_dsl)
        .await;

    // Verify that the error contains span information
    assert!(result.is_err());
    if let Err(ASTError::TokenizeError(TokenizerError::ParseError {
        message,
        found,
        span,
    })) = result
    {
        // Verify that the span information is correct, pointing to the invalid token
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 19);
        assert_eq!(span.start, 18);
        assert_eq!(span.end, 19);

        // The found string might include trailing characters, so we just check it contains @invalid_token
        assert!(found.contains("@invalid_token"));

        // Extract the problematic token from the source using span information
        let token_from_source = &invalid_dsl[span.start..span.end];

        debug!("Tokenization error: {}", message);
        debug!("Found: {}", found);
        debug!(
            "Span: line {}, column {}, start {}, end {}",
            span.line, span.column, span.start, span.end
        );
        debug!("Token from source: {}", token_from_source);
    } else {
        panic!("Expected TokenizeError, got unexpected error");
    }
}

/// Test that parsing errors with multiple lines preserve span information
#[tokio::test]
async fn test_tokenization_error_preserves_span_multiline() {
    // Invalid token in the DSL code
    let invalid_dsl = r#"micro TestAgent {

    @invalid_token
}"#;

    // Parse the DSL code
    let result = AstRegistry::default()
        .create_ast_from_dsl(invalid_dsl)
        .await;

    // Verify that the error contains span information
    assert!(result.is_err());
    if let Err(ASTError::TokenizeError(TokenizerError::ParseError {
        message,
        found,
        span,
    })) = result
    {
        // Verify that the span information is correct, pointing to the invalid token
        assert_eq!(span.line, 3);
        assert_eq!(span.column, 5);
        assert_eq!(span.start, 23);
        assert_eq!(span.end, 24);

        // The found string might include trailing characters, so we just check it contains @invalid_token
        assert!(found.contains("@invalid_token"));

        // Extract the problematic token from the source using span information
        let token_from_source = &invalid_dsl[span.start..span.end];

        debug!("Tokenization error: {}", message);
        debug!("Found: {}", found);
        debug!(
            "Span: line {}, column {}, start {}, end {}",
            span.line, span.column, span.start, span.end
        );
        debug!("Token from source: {}", token_from_source);
    } else {
        panic!("Expected TokenizeError, got unexpected error");
    }
}

/// Test that parsing errors preserve span information
#[tokio::test]
async fn test_parsing_error_preserves_span() {
    // Syntactically invalid DSL code (missing closing brace)
    let invalid_dsl = "micro TestAgent { on request(\"test\"){} }";

    // Parse the DSL code
    let result = AstRegistry::default()
        .create_ast_from_dsl(invalid_dsl)
        .await;

    // Verify that the error contains span information
    assert!(result.is_err());
    if let Err(ASTError::ParseError {
        message,
        token_span,
        error,
    }) = result
    {
        // For now, we're just checking that we get a parse error
        // The span information might not be available in all parse errors yet
        debug!("Parsing error: {}", message);
        debug!("Target: {:?}", token_span);
        debug!("Error: {}", error);
        let span = token_span.unwrap().span;

        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);

        // Extract the problematic token from the source using span information
        let token_from_source = &invalid_dsl[span.start..span.end];

        assert_eq!(token_from_source, "micro");
    } else {
        panic!("Expected ParseError, got unexpected error");
    }
}

/// Test the full compilation pipeline with various error types
#[tokio::test]
async fn test_compilation_pipeline_error_locations() {
    // Test cases with different types of errors
    let test_cases = vec![
        // Tokenization error
        ("micro TestAgent { @invalid_token }", "TokenizeError"),
        // Parsing error - missing closing brace
        ("micro TestAgent { on_event(\"test\") { ", "ParseError"),
        // Parsing error - invalid event handler
        ("micro TestAgent { invalid_handler() }", "ParseError"),
        // Type error - will be caught during parsing in this implementation
        ("micro TestAgent { on_event(123) {} }", "ParseError"),
    ];

    for (dsl_code, expected_error_type) in test_cases {
        debug!("\nTesting DSL code: {}", dsl_code);

        // Parse the DSL code
        let result = AstRegistry::default().create_ast_from_dsl(dsl_code).await;

        // Verify that the error is of the expected type and contains span information
        assert!(result.is_err());
        match result {
            Err(ASTError::TokenizeError(TokenizerError::ParseError {
                message,
                found,
                span,
            })) => {
                assert_eq!(expected_error_type, "TokenizeError");

                // Verify span information
                assert!(span.line > 0);
                assert!(span.column > 0);
                assert!(span.start < span.end);

                // Extract the problematic token from the source
                let token_from_source = &dsl_code[span.start..span.end];

                debug!("TokenizeError: {}", message);
                debug!("Found: {}", found);
                debug!(
                    "Span: line {}, column {}, start {}, end {}",
                    span.line, span.column, span.start, span.end
                );
                debug!("Token from source: {}", token_from_source);
            }
            Err(ASTError::ParseError {
                message,
                token_span,
                error,
            }) => {
                assert_eq!(expected_error_type, "ParseError");

                debug!("ParseError: {}", message);
                debug!("TokenSpan: {:?}", token_span);
                debug!("Error: {}", error);

                // Verify span information if available
                if let Some(token_span) = token_span {
                    let span = token_span.span;
                    assert_eq!(span.line, 1);
                    assert!(span.column > 0);
                    assert!(span.start < span.end);

                    // Extract the problematic token from the source
                    let token_from_source = &dsl_code[span.start..span.end];

                    debug!(
                        "Span: line {}, column {}, start {}, end {}",
                        span.line, span.column, span.start, span.end
                    );
                    debug!("Token from source: {}", token_from_source);
                } else {
                    debug!("No span information available");
                }
            }
            Err(err) => {
                panic!("Unexpected error type: {:?}", err);
            }
            Ok(_) => {
                panic!("Expected error, but parsing succeeded");
            }
        }
    }
}

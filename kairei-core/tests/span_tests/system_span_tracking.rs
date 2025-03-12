//! Integration tests for span tracking through the System interface
//!
//! These tests verify that location information is properly preserved
//! when using the System interface to parse DSL code.

use kairei_core::{
    ast::ASTError,
    config::{SecretConfig, SystemConfig},
    system::{System, SystemError},
    tokenizer,
};

/// Test that tokenization errors preserve span information through the System interface
#[tokio::test]
async fn test_system_tokenization_error_preserves_span() {
    let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;

    let invalid_dsl = "micro TestAgent { @invalid_token }";

    let result = system.parse_dsl(invalid_dsl).await;

    let error = result.unwrap_err();

    match error {
        SystemError::Ast(ASTError::TokenizeError(
            tokenizer::token::TokenizerError::ParseError {
                message,
                found,
                span,
            },
        )) => {
            // Verify that the span information is correct, pointing to the invalid token
            assert_eq!(span.line, 1);
            assert_eq!(span.column, 19);
            assert_eq!(span.start, 18);
            assert_eq!(span.end, 19);

            // The found string might include trailing characters, so we just check it contains @invalid_token
            assert!(found.contains("@invalid_token"));

            // Extract the problematic token from the source using span information
            let token_from_source = &invalid_dsl[span.start..span.end];

            println!("Tokenization error: {}", message);
            println!("Found: {}", found);
            println!(
                "Span: line {}, column {}, start {}, end {}",
                span.line, span.column, span.start, span.end
            );
            println!("Token from source: {}", token_from_source);
        }
        other => {
            panic!("Expected TokenizeError, got unexpected error: {:?}", other);
        }
    }
}

/// Test that parsing errors preserve span information through the System interface
#[tokio::test]
async fn test_system_parsing_error_preserves_span() {
    let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;

    let invalid_dsl = "micro TestAgent { on_event(\"test\") { ";

    let result = system.parse_dsl(invalid_dsl).await;

    assert!(result.is_err());

    match result {
        Err(SystemError::Ast(ASTError::ParseError {
            message,
            token_span,
            error,
        })) => {
            // Verify that the token span information is correct
            let span = token_span.unwrap().span;
            assert_eq!(span.line, 1);
            assert!(span.column > 0);
            assert!(span.start < span.end);

            // Extract the problematic token from the source using span information
            let token_from_source = &invalid_dsl[span.start..span.end];

            println!("Parsing error: {}", message);
            println!("Error: {}", error);
            println!(
                "Span: line {}, column {}, start {}, end {}",
                span.line, span.column, span.start, span.end
            );
            println!("Token from source: {}", token_from_source);
        }
        other => {
            panic!("Expected SystemError::Ast(ParseError), got: {:?}", other);
        }
    }
}

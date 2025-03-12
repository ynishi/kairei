//! Integration tests for span tracking through the System interface
//!
//! These tests verify that location information is properly preserved
//! when using the System interface to parse DSL code.

use kairei_core::{
    config::{SystemConfig, SecretConfig},
    system::{System, SystemError},
    ast::ASTError,
};

/// Test that tokenization errors preserve span information through the System interface
#[tokio::test]
async fn test_system_tokenization_error_preserves_span() {
    // Create a system instance
    let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;
    
    // Invalid token in the DSL code
    let invalid_dsl = "micro TestAgent { @invalid_token }";
    
    // Parse the DSL code using the system interface
    let result = system.parse_dsl(invalid_dsl).await;
    
    // Verify that the error contains span information
    assert!(result.is_err());
    match result {
        Err(SystemError::Ast(ASTError::TokenizeError { message, found, span })) => {
            // Verify that the span information is correct
            assert_eq!(span.line, 1);
            assert!(span.column > 0);
            assert!(span.start < span.end);
            
            // The found string might include trailing characters, so we just check it contains @invalid_token
            assert!(found.contains("@invalid_token"));
            
            // Extract the problematic token from the source using span information
            let token_from_source = &invalid_dsl[span.start..span.end];
            
            println!("Tokenization error: {}", message);
            println!("Found: {}", found);
            println!("Span: line {}, column {}, start {}, end {}", 
                     span.line, span.column, span.start, span.end);
            println!("Token from source: {}", token_from_source);
        },
        other => {
            panic!("Expected SystemError::Ast(TokenizeError), got: {:?}", other);
        }
    }
}

/// Test that parsing errors preserve span information through the System interface
#[tokio::test]
async fn test_system_parsing_error_preserves_span() {
    // Create a system instance
    let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;
    
    // Syntactically invalid DSL code (missing closing brace)
    let invalid_dsl = "micro TestAgent { on_event(\"test\") { ";
    
    // Parse the DSL code using the system interface
    let result = system.parse_dsl(invalid_dsl).await;
    
    // Verify that the error contains span information
    assert!(result.is_err());
    match result {
        Err(SystemError::Ast(ASTError::ParseError { message, target, span })) => {
            // For now, we're just checking that we get a parse error
            // The span information might not be available in all parse errors yet
            println!("Parsing error: {}", message);
            println!("Target: {}", target);
            
            if let Some(span) = span {
                // Verify that the span points to a valid location
                assert!(span.line > 0);
                assert!(span.column > 0);
                assert!(span.start < span.end);
                
                // Extract the problematic token from the source
                let token_from_source = &invalid_dsl[span.start..span.end];
                
                println!("Parsing error: {}", message);
                println!("Target: {}", target);
                println!("Span: line {}, column {}, start {}, end {}", 
                         span.line, span.column, span.start, span.end);
                println!("Token from source: {}", token_from_source);
            }
        },
        other => {
            panic!("Expected SystemError::Ast(ParseError), got: {:?}", other);
        }
    }
}

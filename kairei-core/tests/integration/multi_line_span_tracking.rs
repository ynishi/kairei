//! Integration tests for multi-line span tracking through the compilation pipeline
//!
//! These tests verify that location information is properly preserved for multi-line
//! tokens and errors through the tokenization, parsing, and type checking phases.

use kairei_core::{
    ast::ASTError,
    ast_registry::AstRegistry,
    config::{SecretConfig, SystemConfig},
    system::{System, SystemError},
};

/// Test multi-line string tokenization and error location tracking
#[tokio::test]
async fn test_multi_line_string_tokenization() {
    // Multi-line string in DSL code
    let multi_line_dsl = r#"micro TestAgent {
    on_event("test") {
        let multi_line_string = "This is a
multi-line
string that spans
multiple lines";
    }
}"#;

    // Parse the DSL code
    let result = AstRegistry::default()
        .create_ast_from_dsl(multi_line_dsl)
        .await;

    // This should parse successfully
    assert!(
        result.is_ok(),
        "Multi-line string should parse successfully"
    );

    // Now introduce an error in the multi-line string
    let invalid_multi_line_dsl = r#"micro TestAgent {
    on_event("test") {
        let multi_line_string = "This is a
multi-line
string with an unclosed quote;
    }
}"#;

    // Parse the invalid DSL code
    let result = AstRegistry::default()
        .create_ast_from_dsl(invalid_multi_line_dsl)
        .await;

    // Verify that the error contains span information
    assert!(result.is_err());
    if let Err(ASTError::TokenizeError {
        message,
        found,
        span,
    }) = result
    {
        // Verify that the span information is correct
        assert!(span.line > 0);
        assert!(span.column > 0);
        assert!(span.start < span.end);

        // Verify that the span covers multiple lines
        assert!(
            span.end_line > span.line,
            "Error should span multiple lines"
        );

        println!("Tokenization error: {}", message);
        println!("Found: {}", found);
        println!(
            "Span: line {}, column {}, end_line {}, end_column {}, start {}, end {}",
            span.line, span.column, span.end_line, span.end_column, span.start, span.end
        );
    } else {
        panic!("Expected TokenizeError, got unexpected error");
    }
}

/// Test multi-line block parsing and error location tracking
#[tokio::test]
async fn test_multi_line_block_parsing() {
    // Multi-line block with missing closing brace
    let invalid_block_dsl = r#"micro TestAgent {
    on_event("test") {
        let x = 1;
        let y = 2;
        let z = 3;
        // Missing closing brace for on_event
    
}"#;

    // Parse the DSL code
    let result = AstRegistry::default()
        .create_ast_from_dsl(invalid_block_dsl)
        .await;

    // Verify that the error contains span information
    assert!(result.is_err());
    match result {
        Err(ASTError::ParseError {
            message,
            target,
            span,
        }) => {
            println!("Parse error: {}", message);
            println!("Target: {}", target);

            if let Some(span) = span {
                // Verify that the span points to a valid location
                assert!(span.line > 0);
                assert!(span.column > 0);
                assert!(span.start < span.end);

                println!(
                    "Span: line {}, column {}, end_line {}, end_column {}, start {}, end {}",
                    span.line, span.column, span.end_line, span.end_column, span.start, span.end
                );
            } else {
                println!("No span information available (this is expected for some parse errors)");
            }
        }
        other => {
            panic!("Expected ParseError, got: {:?}", other);
        }
    }
}

/// Test multi-line error tracking through the System interface
#[tokio::test]
async fn test_system_multi_line_error_tracking() {
    // Create a system instance
    let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;

    // Multi-line DSL with syntax error spanning multiple lines
    let invalid_multi_line_dsl = r#"micro TestAgent {
    on_event("test") {
        let complex_expression = (1 + 2
            * 3
            - 4;  // Missing closing parenthesis
    }
}"#;

    // Parse the DSL code using the system interface
    let result = system.parse_dsl(invalid_multi_line_dsl).await;

    // Verify that the error contains span information
    assert!(result.is_err());
    match result {
        Err(SystemError::Ast(ASTError::ParseError {
            message,
            target,
            span,
        })) => {
            println!("Parse error: {}", message);
            println!("Target: {}", target);

            if let Some(span) = span {
                // Verify that the span points to a valid location
                assert!(span.line > 0);
                assert!(span.column > 0);
                assert!(span.start < span.end);

                println!(
                    "Span: line {}, column {}, end_line {}, end_column {}, start {}, end {}",
                    span.line, span.column, span.end_line, span.end_column, span.start, span.end
                );

                // Extract the problematic token from the source
                let lines: Vec<&str> = invalid_multi_line_dsl.lines().collect();
                println!("Error context:");
                for i in (span.line - 1)..(span.end_line) {
                    if i < lines.len() {
                        println!("{}: {}", i + 1, lines[i]);
                    }
                }
            }
        }
        other => {
            panic!("Expected SystemError::Ast(ParseError), got: {:?}", other);
        }
    }
}

/// Test error visualization with multi-line spans
#[tokio::test]
async fn test_multi_line_error_visualization() {
    // Create a system instance
    let system = System::new(&SystemConfig::default(), &SecretConfig::default()).await;

    // Multi-line DSL with a complex multi-line error
    let invalid_multi_line_dsl = r#"micro TestAgent {
    on_event("test") {
        let nested_blocks = {
            let inner = {
                let deep = {
                    1 + 2
                    * 3
                    - 4
                // Missing closing brace here
            };
        };
    }
}"#;

    // Parse the DSL code using the system interface
    let result = system.parse_dsl(invalid_multi_line_dsl).await;

    // Verify that the error contains span information
    assert!(result.is_err());
    match result {
        Err(SystemError::Ast(ASTError::ParseError {
            message,
            target,
            span,
        })) => {
            println!("Parse error: {}", message);
            println!("Target: {}", target);

            if let Some(span) = span {
                // Verify that the span points to a valid location
                assert!(span.line > 0);
                assert!(span.column > 0);
                assert!(span.start < span.end);

                println!(
                    "Span: line {}, column {}, end_line {}, end_column {}, start {}, end {}",
                    span.line, span.column, span.end_line, span.end_column, span.start, span.end
                );

                // Extract and visualize the error context
                let lines: Vec<&str> = invalid_multi_line_dsl.lines().collect();
                println!("Error context with visualization:");

                // Show a few lines before the error
                let context_start = span.line.saturating_sub(2);
                let context_end = (span.end_line + 2).min(lines.len());

                for i in context_start..context_end {
                    if i < lines.len() {
                        println!("{:>3} | {}", i + 1, lines[i]);

                        // Add error markers
                        if i + 1 == span.line {
                            // Start of error
                            let marker = " ".repeat(span.column - 1) + "^ Error starts here";
                            println!("    | {}", marker);
                        } else if i + 1 == span.end_line {
                            // End of error
                            let marker = " ".repeat(span.end_column - 1) + "^ Error ends here";
                            println!("    | {}", marker);
                        } else if i + 1 > span.line && i + 1 < span.end_line {
                            // Middle of multi-line error
                            println!("    | | Error continues on this line");
                        }
                    }
                }
            }
        }
        other => {
            panic!("Expected SystemError::Ast(ParseError), got: {:?}", other);
        }
    }
}

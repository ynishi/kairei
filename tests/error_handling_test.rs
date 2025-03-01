use kairei::{
    analyzer::{
        self,
        core::{ParseError, Parser},
        error_handling::{
            error_collecting_many, error_collecting_optional, format_detailed_error_message,
            ERROR_COLLECTOR,
        },
        prelude::*,
    },
    ast_registry::AstRegistry,
    tokenizer::token::{Token, Tokenizer},
    ASTError,
};

#[test]
fn test_error_collecting_optional() {
    // Create a parser that always fails
    let fail_parser = fail::<Token, i32>("test failure");
    let parser = error_collecting_optional(fail_parser, "test context");

    // Clear any previous errors
    ERROR_COLLECTOR.with(|collector| {
        collector.borrow_mut().clear();
    });

    // Parse should return None but collect the error
    let input = vec![Token::Identifier("test".to_string())];
    let result = parser.parse(&input, 0);
    assert_eq!(result, Ok((0, None)));

    // Check that the error was collected
    ERROR_COLLECTOR.with(|collector| {
        let collector = collector.borrow();
        assert!(collector.has_errors());
        assert_eq!(collector.get_errors().len(), 1);

        let error_info = &collector.get_errors()[0];
        assert_eq!(error_info.context, "test context");
        assert!(error_info.is_optional);
        match &error_info.error {
            ParseError::Fail(msg) => assert_eq!(msg, "test failure"),
            _ => panic!("Unexpected error type"),
        }
    });
}

#[test]
fn test_error_collecting_many() {
    // Create a parser that succeeds once then fails
    let input = vec![
        Token::Identifier("test1".to_string()),
        Token::Identifier("test2".to_string()),
        Token::Keyword("invalid".to_string()),
    ];

    let parser = error_collecting_many(
        satisfy(|token: &Token| match token {
            Token::Identifier(name) => Some(name.clone()),
            _ => None,
        }),
        "test context",
    );

    // Clear any previous errors
    ERROR_COLLECTOR.with(|collector| {
        collector.borrow_mut().clear();
    });

    // Parse should return ["test1", "test2"] and collect an error for the keyword
    let result = parser.parse(&input, 0);
    assert_eq!(
        result,
        Ok((
            2,
            vec!["test1".to_string(), "test2".to_string()]
        ))
    );

    // Check that the error was collected
    ERROR_COLLECTOR.with(|collector| {
        let collector = collector.borrow();
        assert!(collector.has_errors());
        assert_eq!(collector.get_errors().len(), 1);

        let error_info = &collector.get_errors()[0];
        assert_eq!(error_info.context, "test context");
        assert!(!error_info.is_optional);
        assert!(matches!(error_info.error, ParseError::EOF));
    });
}

#[test]
fn test_format_detailed_error_message() {
    // Create a main error and some collected errors
    let main_error = ParseError::Fail("main error".to_string());
    
    // Clear any previous errors
    ERROR_COLLECTOR.with(|collector| {
        collector.borrow_mut().clear();
    });
    
    // Add some errors to the collector
    let parser1 = error_collecting_optional(fail::<Token, i32>("optional error"), "optional context");
    let parser2 = error_collecting_many(
        satisfy(|_: &Token| None::<String>),
        "many context",
    );
    
    let input = vec![Token::Identifier("test".to_string())];
    let _ = parser1.parse(&input, 0);
    let _ = parser2.parse(&input, 0);
    
    // Get the collected errors
    let collected_errors = ERROR_COLLECTOR.with(|collector| {
        let collector = collector.borrow();
        collector.get_errors().to_vec()
    });
    
    // Format the error message
    let message = format_detailed_error_message(&main_error, &collected_errors);
    
    // Check that the message contains all the expected information
    assert!(message.contains("Parse error: main error"));
    assert!(message.contains("Additional parsing issues:"));
    assert!(message.contains("Optional parsing failed in 'optional context'"));
    assert!(message.contains("Repeated parsing failed in 'many context'"));
}

#[tokio::test]
async fn test_ast_registry_error_handling() {
    let registry = AstRegistry::default();
    
    // A DSL with intentional errors in both world and agent definitions
    let dsl = r#"
    world {
        invalid_world_item
    }
    
    micro Agent1 {
        state {
            counter: i64 = 0;
        }
    }
    
    micro Agent2 {
        invalid_agent_item
    }
    "#;
    
    // Try to parse the DSL
    let result = registry.create_ast_from_dsl(dsl).await;
    
    // Check that the error contains detailed information
    assert!(result.is_err());
    if let Err(ASTError::ParseError { message, .. }) = result {
        // The error message should contain information about both the world and agent errors
        assert!(message.contains("world definition"));
        assert!(message.contains("agent definitions"));
        println!("Error message: {}", message);
    } else {
        panic!("Expected ParseError, got: {:?}", result);
    }
}

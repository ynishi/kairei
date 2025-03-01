use crate::analyzer::core::Parser;
use crate::analyzer::parsers::expression::{parse_await_multiple, parse_await_single};
use crate::ast;
use crate::tokenizer::{
    keyword::Keyword,
    literal::{Literal, StringLiteral, StringPart},
    symbol::Delimiter,
    token::Token,
};

#[test]
fn test_parse_await_single() {
    // Test a simple variable
    let input = &[
        Token::Keyword(Keyword::Await),
        Token::Identifier("future".to_string()),
    ];
    let (pos, expr) = parse_await_single().parse(input, 0).unwrap();
    assert_eq!(pos, 2);
    match expr {
        ast::Expression::Await(expressions) => {
            assert_eq!(expressions.len(), 1);
            match &expressions[0] {
                ast::Expression::Variable(name) => assert_eq!(name, "future"),
                _ => panic!("Expected Variable expression"),
            }
        }
        _ => panic!("Expected Await expression"),
    }

    // Test with a function call
    let input = &[
        Token::Keyword(Keyword::Await),
        Token::Identifier("getData".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::Integer(42)),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    let (pos, expr) = parse_await_single().parse(input, 0).unwrap();
    assert_eq!(pos, 5);
    match expr {
        ast::Expression::Await(expressions) => {
            assert_eq!(expressions.len(), 1);
            match &expressions[0] {
                ast::Expression::FunctionCall {
                    function,
                    arguments,
                } => {
                    assert_eq!(function, "getData");
                    assert_eq!(arguments.len(), 1);
                    match &arguments[0] {
                        ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(*n, 42),
                        _ => panic!("Expected Integer argument"),
                    }
                }
                _ => panic!("Expected FunctionCall expression"),
            }
        }
        _ => panic!("Expected Await expression"),
    }

    // Failure case - missing await keyword
    let input = &[Token::Identifier("future".to_string())];
    assert!(parse_await_single().parse(input, 0).is_err());
}

#[test]
fn test_parse_await_multiple() {
    // Test multiple variables
    let input = &[
        Token::Keyword(Keyword::Await),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("future1".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("future2".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    let (pos, expr) = parse_await_multiple().parse(input, 0).unwrap();
    assert_eq!(pos, 6);
    match expr {
        ast::Expression::Await(expressions) => {
            assert_eq!(expressions.len(), 2);
            match &expressions[0] {
                ast::Expression::Variable(name) => assert_eq!(name, "future1"),
                _ => panic!("Expected Variable expression for first argument"),
            }
            match &expressions[1] {
                ast::Expression::Variable(name) => assert_eq!(name, "future2"),
                _ => panic!("Expected Variable expression for second argument"),
            }
        }
        _ => panic!("Expected Await expression"),
    }

    // Test with more complex expressions
    let input = &[
        Token::Keyword(Keyword::Await),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("getData".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::Integer(1)),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("processData".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::Integer(2)),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    let (pos, expr) = parse_await_multiple().parse(input, 0).unwrap();
    assert_eq!(pos, 12);
    match expr {
        ast::Expression::Await(expressions) => {
            assert_eq!(expressions.len(), 2);

            // Check first function call
            match &expressions[0] {
                ast::Expression::FunctionCall {
                    function,
                    arguments,
                } => {
                    assert_eq!(function, "getData");
                    assert_eq!(arguments.len(), 1);
                    match &arguments[0] {
                        ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(*n, 1),
                        _ => panic!("Expected Integer argument for first function"),
                    }
                }
                _ => panic!("Expected FunctionCall expression for first argument"),
            }

            // Check second function call
            match &expressions[1] {
                ast::Expression::FunctionCall {
                    function,
                    arguments,
                } => {
                    assert_eq!(function, "processData");
                    assert_eq!(arguments.len(), 1);
                    match &arguments[0] {
                        ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(*n, 2),
                        _ => panic!("Expected Integer argument for second function"),
                    }
                }
                _ => panic!("Expected FunctionCall expression for second argument"),
            }
        }
        _ => panic!("Expected Await expression"),
    }

    // Test empty arguments
    let input = &[
        Token::Keyword(Keyword::Await),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    let (pos, expr) = parse_await_multiple().parse(input, 0).unwrap();
    assert_eq!(pos, 3);
    match expr {
        ast::Expression::Await(expressions) => {
            assert_eq!(expressions.len(), 0);
        }
        _ => panic!("Expected Await expression"),
    }

    // Failure case - missing await keyword
    let input = &[
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("future1".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    assert!(parse_await_multiple().parse(input, 0).is_err());

    // Failure case - missing closing parenthesis
    let input = &[
        Token::Keyword(Keyword::Await),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("future1".to_string()),
    ];
    assert!(parse_await_multiple().parse(input, 0).is_err());
}

#[test]
fn test_parse_await_mixed_arguments() {
    // Test with mixed argument types (literals, variables, function calls)
    let input = &[
        Token::Keyword(Keyword::Await),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::Integer(123)),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("variable".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("getData".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::Boolean(true)),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::CloseParen),
    ];

    let (pos, expr) = parse_await_multiple().parse(input, 0).unwrap();
    assert_eq!(pos, 11);

    match expr {
        ast::Expression::Await(expressions) => {
            assert_eq!(expressions.len(), 3);

            // Check first argument (integer literal)
            match &expressions[0] {
                ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(*n, 123),
                _ => panic!("Expected Integer literal for first argument"),
            }

            // Check second argument (variable)
            match &expressions[1] {
                ast::Expression::Variable(name) => assert_eq!(name, "variable"),
                _ => panic!("Expected Variable for second argument"),
            }

            // Check third argument (function call)
            match &expressions[2] {
                ast::Expression::FunctionCall {
                    function,
                    arguments,
                } => {
                    assert_eq!(function, "getData");
                    assert_eq!(arguments.len(), 1);
                    match &arguments[0] {
                        ast::Expression::Literal(ast::Literal::Boolean(b)) => assert_eq!(*b, true),
                        _ => panic!("Expected Boolean argument"),
                    }
                }
                _ => panic!("Expected FunctionCall for third argument"),
            }
        }
        _ => panic!("Expected Await expression"),
    }
}

#[test]
fn test_parse_await_with_think_expressions() {
    // Test await with think expressions inside
    let input = &[
        Token::Keyword(Keyword::Await),
        Token::Delimiter(Delimiter::OpenParen),
        // First argument: think expression
        Token::Keyword(Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Analyze data".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::Comma),
        // Second argument: think expression with named params
        Token::Keyword(Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("query".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Find hotels".to_string()),
        ]))),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("location".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("destination".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        // Think with "with" block
        Token::Keyword(Keyword::With),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("provider".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("openai".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseParen),
    ];

    let (_, expr) = parse_await_multiple().parse(input, 0).unwrap();
    match expr {
        ast::Expression::Await(expressions) => {
            assert_eq!(expressions.len(), 2);

            // Check first argument (simple think expression)
            match &expressions[0] {
                ast::Expression::Think { args, with_block } => {
                    assert_eq!(args.len(), 1);
                    match &args[0] {
                        ast::Argument::Positional(ast::Expression::Literal(
                            ast::Literal::String(s),
                        )) => {
                            assert_eq!(s, "Analyze data");
                        }
                        _ => panic!("Expected string literal argument in first think"),
                    }
                    assert!(with_block.is_none());
                }
                _ => panic!("Expected Think expression as first argument"),
            }

            // Check second argument (think with named params and with block)
            match &expressions[1] {
                ast::Expression::Think { args, with_block } => {
                    assert_eq!(args.len(), 2);

                    // Check named parameters
                    match &args[0] {
                        ast::Argument::Named { name, value } => {
                            assert_eq!(name, "query");
                            match value {
                                ast::Expression::Literal(ast::Literal::String(s)) => {
                                    assert_eq!(s, "Find hotels");
                                }
                                _ => panic!("Expected string literal for query parameter"),
                            }
                        }
                        _ => panic!("Expected named argument for first parameter"),
                    }

                    match &args[1] {
                        ast::Argument::Named { name, value } => {
                            assert_eq!(name, "location");
                            match value {
                                ast::Expression::Variable(var) => {
                                    assert_eq!(var, "destination");
                                }
                                _ => panic!("Expected variable for location parameter"),
                            }
                        }
                        _ => panic!("Expected named argument for second parameter"),
                    }

                    // Check with block
                    assert!(with_block.is_some());
                    let with_attrs = with_block.as_ref().unwrap();
                    assert_eq!(with_attrs.provider, Some("openai".to_string()));
                }
                _ => panic!("Expected Think expression as second argument"),
            }
        }
        _ => panic!("Expected Await expression"),
    }
}

#[test]
fn test_parse_await_with_nested_complex_expressions() {
    // Test complex nesting: await(think(...), request(...))
    let input = &[
        Token::Keyword(Keyword::Await),
        Token::Delimiter(Delimiter::OpenParen),
        // First argument: think expression
        Token::Keyword(Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Process data".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::Comma),
        // Second argument: request expression
        Token::Keyword(Keyword::Request),
        Token::Identifier("FindData".to_string()),
        Token::Keyword(Keyword::To),
        Token::Identifier("DataAgent".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("query".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("search term".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::CloseParen),
    ];

    let (_, expr) = parse_await_multiple().parse(input, 0).unwrap();
    match expr {
        ast::Expression::Await(expressions) => {
            assert_eq!(expressions.len(), 2);

            // Check first argument (think expression)
            match &expressions[0] {
                ast::Expression::Think { args, with_block } => {
                    assert_eq!(args.len(), 1);
                    assert!(with_block.is_none());
                }
                _ => panic!("Expected Think expression as first argument"),
            }

            // Check second argument (request expression)
            match &expressions[1] {
                ast::Expression::Request {
                    agent,
                    request_type,
                    parameters,
                    options,
                } => {
                    assert_eq!(agent, "DataAgent");
                    match request_type {
                        ast::RequestType::Custom(req_type) => assert_eq!(req_type, "FindData"),
                        _ => panic!("Expected Custom request type"),
                    }
                    assert_eq!(parameters.len(), 1);
                    assert!(options.is_none());
                }
                _ => panic!("Expected Request expression as second argument"),
            }
        }
        _ => panic!("Expected Await expression"),
    }
}

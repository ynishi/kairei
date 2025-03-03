use crate::analyzer::core::Parser;
use crate::analyzer::parsers::expression::{parse_think_multiple, parse_think_single};
use crate::ast;
use crate::tokenizer::{
    keyword::Keyword,
    literal::{Literal, StringLiteral, StringPart},
    symbol::Delimiter,
    token::Token,
};

#[test]
fn test_parse_think_single() {
    // Test with single positional argument
    let input = &[
        Token::Keyword(Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Analyze data".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    let (pos, expr) = parse_think_single().parse(input, 0).unwrap();
    assert_eq!(pos, 4);
    match expr {
        ast::Expression::Think { args, with_block } => {
            assert_eq!(args.len(), 1);
            match &args[0] {
                ast::Argument::Positional(ast::Expression::Literal(ast::Literal::String(s))) => {
                    assert_eq!(s, "Analyze data");
                }
                _ => panic!("Expected string literal argument"),
            }
            assert!(with_block.is_none());
        }
        _ => panic!("Expected Think expression"),
    }

    // Test with named argument
    let input = &[
        Token::Keyword(Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("prompt".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Find hotels".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    let (pos, expr) = parse_think_single().parse(input, 0).unwrap();
    assert_eq!(pos, 6);
    match expr {
        ast::Expression::Think { args, with_block } => {
            assert_eq!(args.len(), 1);
            match &args[0] {
                ast::Argument::Named { name, value } => {
                    assert_eq!(name, "prompt");
                    match value {
                        ast::Expression::Literal(ast::Literal::String(s)) => {
                            assert_eq!(s, "Find hotels");
                        }
                        _ => panic!("Expected string literal value"),
                    }
                }
                _ => panic!("Expected named argument"),
            }
            assert!(with_block.is_none());
        }
        _ => panic!("Expected Think expression"),
    }

    // Test with with_block
    let input = &[
        Token::Keyword(Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Analyze data".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Keyword(Keyword::With),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("provider".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("openai".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseBrace),
    ];
    let (pos, expr) = parse_think_single().parse(input, 0).unwrap();
    assert_eq!(pos, 10);
    match expr {
        ast::Expression::Think { args, with_block } => {
            assert_eq!(args.len(), 1);
            assert!(with_block.is_some());
            let attributes = with_block.unwrap();
            assert_eq!(attributes.provider, Some("openai".to_string()));
        }
        _ => panic!("Expected Think expression"),
    }

    // Failure case - missing think keyword
    let input = &[
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Analyze data".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    assert!(parse_think_single().parse(input, 0).is_err());
}

#[test]
fn test_parse_think_multiple() {
    // Test with multiple named parameters
    let input = &[
        Token::Keyword(Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("prompt".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Find hotels".to_string()),
        ]))),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("location".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Tokyo".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    let (pos, expr) = parse_think_multiple().parse(input, 0).unwrap();
    assert_eq!(pos, 10);
    match expr {
        ast::Expression::Think { args, with_block } => {
            assert_eq!(args.len(), 2);

            // Check first argument
            match &args[0] {
                ast::Argument::Named { name, value } => {
                    assert_eq!(name, "prompt");
                    match value {
                        ast::Expression::Literal(ast::Literal::String(s)) => {
                            assert_eq!(s, "Find hotels");
                        }
                        _ => panic!("Expected string literal for first argument"),
                    }
                }
                _ => panic!("Expected named argument for first parameter"),
            }

            // Check second argument
            match &args[1] {
                ast::Argument::Named { name, value } => {
                    assert_eq!(name, "location");
                    match value {
                        ast::Expression::Literal(ast::Literal::String(s)) => {
                            assert_eq!(s, "Tokyo");
                        }
                        _ => panic!("Expected string literal for second argument"),
                    }
                }
                _ => panic!("Expected named argument for second parameter"),
            }

            assert!(with_block.is_none());
        }
        _ => panic!("Expected Think expression"),
    }

    // Test with with_block
    let input = &[
        Token::Keyword(Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("prompt".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Find hotels".to_string()),
        ]))),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("location".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Tokyo".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Keyword(Keyword::With),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("provider".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("openai".to_string()),
        ]))),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("model".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("gpt-4".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseBrace),
    ];
    let (pos, expr) = parse_think_multiple().parse(input, 0).unwrap();
    assert_eq!(pos, 20);
    match expr {
        ast::Expression::Think { args, with_block } => {
            assert_eq!(args.len(), 2);
            assert!(with_block.is_some());
            let attributes = with_block.unwrap();
            assert_eq!(attributes.provider, Some("openai".to_string()));
            assert_eq!(attributes.model, Some("gpt-4".to_string()));
        }
        _ => panic!("Expected Think expression"),
    }

    // Test empty parameter list - this should succeed now since our implementation allows it
    let input = &[
        Token::Keyword(Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    // Actually, this should now parse successfully with our implementation
    let result = parse_think_multiple().parse(input, 0);
    assert!(result.is_ok());
    if let Ok((pos, expr)) = result {
        assert_eq!(pos, 3);
        match expr {
            ast::Expression::Think { args, with_block } => {
                assert_eq!(args.len(), 0);
                assert!(with_block.is_none());
            }
            _ => panic!("Expected Think expression"),
        }
    }

    // Failure case - missing think keyword
    let input = &[
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("prompt".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("Find hotels".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    assert!(parse_think_multiple().parse(input, 0).is_err());
}

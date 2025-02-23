use crate::analyzer::parsers::types::*;
use crate::analyzer::Parser;
use crate::ast;
use crate::tokenizer::literal::{StringLiteral, StringPart};
use crate::tokenizer::symbol::{Delimiter, Operator};
use crate::tokenizer::{literal::Literal, token::Token};

#[test]
fn test_parse_type_info() {
    // Result型のテスト
    let input = &[
        Token::Identifier("Result".to_string()),
        Token::Operator(Operator::Less),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("Error".to_string()),
        Token::Operator(Operator::Greater),
    ];
    let (pos, result) = parse_type_info().parse(input, 0).unwrap();
    assert_eq!(pos, 6);
    assert_eq!(
        result,
        ast::TypeInfo::Result {
            ok_type: Box::new(ast::TypeInfo::Simple("String".to_string())),
            err_type: Box::new(ast::TypeInfo::Simple("Error".to_string())),
        }
    );

    // Option型のテスト
    let input = &[
        Token::Identifier("Option".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("Integer".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
    ];
    let (pos, result) = parse_type_info().parse(input, 0).unwrap();
    assert_eq!(pos, 4);
    assert_eq!(
        result,
        ast::TypeInfo::Option(Box::new(ast::TypeInfo::Simple("Integer".to_string())))
    );

    // シンプル型のテスト
    let input = &[Token::Identifier("String".to_string())];
    let (pos, result) = parse_type_info().parse(input, 0).unwrap();
    assert_eq!(pos, 1);
    assert_eq!(result, ast::TypeInfo::Simple("String".to_string()));
}

#[test]
fn test_parse_custom_type_with_type_and_default() {
    let input = &[
        Token::Identifier("Person".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("John".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let (pos, result) = parse_custom_type().parse(input, 0).unwrap();
    assert_eq!(pos, input.len());

    match result {
        ast::TypeInfo::Custom { name, fields } => {
            assert_eq!(name, "Person");
            let field = fields.get("name").unwrap();
            assert_eq!(
                field.type_info,
                Some(ast::TypeInfo::Simple("String".to_string()))
            );
            assert_eq!(
                field.default_value,
                Some(ast::Expression::Literal(ast::Literal::String(
                    "John".to_string()
                )))
            );
        }
        _ => panic!("Expected Custom type"),
    }
}

#[test]
fn test_parse_custom_type_with_type_only() {
    let input = &[
        Token::Identifier("Person".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("age".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("Int".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let (pos, result) = parse_custom_type().parse(input, 0).unwrap();
    assert_eq!(pos, input.len());

    match result {
        ast::TypeInfo::Custom { name, fields } => {
            assert_eq!(name, "Person");
            let field = fields.get("age").unwrap();
            assert_eq!(
                field.type_info,
                Some(ast::TypeInfo::Simple("Int".to_string()))
            );
            assert_eq!(field.default_value, None);
        }
        _ => panic!("Expected Custom type"),
    }
}

#[test]
fn test_parse_custom_type_with_default_only() {
    let input = &[
        Token::Identifier("Person".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("age".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(32)),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let (pos, result) = parse_custom_type().parse(input, 0).unwrap();
    assert_eq!(pos, input.len());

    match result {
        ast::TypeInfo::Custom { name, fields } => {
            assert_eq!(name, "Person");
            let field = fields.get("age").unwrap();
            assert_eq!(field.type_info, None);
            assert_eq!(
                field.default_value,
                Some(ast::Expression::Literal(ast::Literal::Integer(32)))
            );
        }
        _ => panic!("Expected Custom type"),
    }
}

#[test]
fn test_parse_custom_type_multiple_fields() {
    let input = &[
        Token::Identifier("Person".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        // name: String = "John"
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("John".to_string()),
        ]))),
        Token::Delimiter(Delimiter::Comma),
        // age = 32
        Token::Identifier("age".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(32)),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let (pos, result) = parse_custom_type().parse(input, 0).unwrap();
    assert_eq!(pos, input.len());

    match result {
        ast::TypeInfo::Custom { name, fields } => {
            assert_eq!(name, "Person");
            assert_eq!(fields.len(), 2);

            let name_field = fields.get("name").unwrap();
            assert_eq!(
                name_field.type_info,
                Some(ast::TypeInfo::Simple("String".to_string()))
            );
            assert_eq!(
                name_field.default_value,
                Some(ast::Expression::Literal(ast::Literal::String(
                    "John".to_string()
                )))
            );

            let age_field = fields.get("age").unwrap();
            assert_eq!(age_field.type_info, None);
            assert_eq!(
                age_field.default_value,
                Some(ast::Expression::Literal(ast::Literal::Integer(32)))
            );
        }
        _ => panic!("Expected Custom type"),
    }
}

#[test]
fn test_parse_custom_type_empty() {
    let input = &[
        Token::Identifier("Empty".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let (pos, result) = parse_custom_type().parse(input, 0).unwrap();
    assert_eq!(pos, 3);

    match result {
        ast::TypeInfo::Custom { name, fields } => {
            assert_eq!(name, "Empty");
            assert!(fields.is_empty());
        }
        _ => panic!("Expected Custom type"),
    }
}

#[test]
fn test_parse_field_typed_with_default() {
    let input = &[
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("test".to_string()),
        ]))),
    ];
    let (pos, field_info) = parse_field_typed_with_default().parse(input, 0).unwrap();
    assert_eq!(pos, input.len());
    assert_eq!(
        field_info.type_info,
        Some(ast::TypeInfo::Simple("String".to_string()))
    );
    assert_eq!(
        field_info.default_value,
        Some(ast::Expression::Literal(ast::Literal::String(
            "test".to_string()
        )))
    );
}

#[test]
fn test_parse_option_type() {
    let input = &[
        Token::Identifier("Option".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
    ];
    let (pos, result) = parse_option_type().parse(input, 0).unwrap();
    assert_eq!(pos, 4);
    assert_eq!(
        result,
        ast::TypeInfo::Option(Box::new(ast::TypeInfo::Simple("String".to_string())))
    );
}

#[test]
fn test_parse_result_type() {
    let input = &[
        Token::Identifier("Result".to_string()),
        Token::Operator(Operator::Less),
        Token::Identifier("Success".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("Error".to_string()),
        Token::Operator(Operator::Greater),
    ];
    let (pos, result) = parse_result_type().parse(input, 0).unwrap();
    assert_eq!(pos, 6);
    assert_eq!(
        result,
        ast::TypeInfo::Result {
            ok_type: Box::new(ast::TypeInfo::Simple("Success".to_string())),
            err_type: Box::new(ast::TypeInfo::Simple("Error".to_string())),
        }
    );
}

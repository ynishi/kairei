use crate::analyzer::parsers::agent::*;
use crate::analyzer::Parser;
use crate::ast;
use crate::tokenizer::{keyword::Keyword, literal::Literal, symbol::Delimiter, token::Token};
use std::collections::HashMap;

#[test]
fn test_parse_agent_def() {
    let input = vec![
        Token::Keyword(Keyword::Micro),
        Token::Identifier("TestAgent".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        // State block
        Token::Keyword(Keyword::State),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("Integer".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(0)),
        Token::Delimiter(Delimiter::Semicolon),
        Token::Delimiter(Delimiter::CloseBrace),
        // Observe block
        Token::Keyword(Keyword::Observe),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::On),
        Token::Identifier("Tick".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Literal(Literal::Null),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::MicroAgentDef {
        name: "TestAgent".to_string(),
        policies: vec![],
        lifecycle: None,
        state: Some(ast::StateDef {
            variables: {
                let mut vars = HashMap::new();
                vars.insert(
                    "counter".to_string(),
                    ast::StateVarDef {
                        name: "counter".to_string(),
                        type_info: ast::TypeInfo::Simple("Integer".to_string()),
                        initial_value: Some(ast::Expression::Literal(ast::Literal::Integer(0))),
                    },
                );
                vars
            },
        }),
        observe: Some(ast::ObserveDef {
            handlers: vec![ast::EventHandler {
                event_type: ast::EventType::Tick,
                parameters: vec![],
                block: ast::HandlerBlock {
                    statements: vec![ast::Statement::Return(ast::Expression::Literal(ast::Literal::Null))],
                },
            }],
        }),
        answer: None,
        react: None,
    };

    assert_eq!(parse_agent_def().parse(&input, 0), Ok((input.len(), expected)));
}

#[test]
fn test_parse_lifecycle() {
    let input = vec![
        Token::Keyword(Keyword::Lifecycle),
        Token::Delimiter(Delimiter::OpenBrace),
        // init handler
        Token::Keyword(Keyword::OnInit),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Literal(Literal::Null),
        Token::Delimiter(Delimiter::CloseBrace),
        // destroy handler
        Token::Keyword(Keyword::OnDestroy),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Literal(Literal::Null),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::LifecycleDef {
        on_init: Some(ast::HandlerBlock {
            statements: vec![ast::Statement::Return(ast::Expression::Literal(ast::Literal::Null))],
        }),
        on_destroy: Some(ast::HandlerBlock {
            statements: vec![ast::Statement::Return(ast::Expression::Literal(ast::Literal::Null))],
        }),
    };

    assert_eq!(parse_lifecycle().parse(&input, 0), Ok((input.len(), expected)));
}

#[test]
fn test_parse_state() {
    let input = vec![
        Token::Keyword(Keyword::State),
        Token::Delimiter(Delimiter::OpenBrace),
        // 変数1: 初期値あり
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("Integer".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(0)),
        Token::Delimiter(Delimiter::Semicolon),
        // 変数2: 初期値なし
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::Semicolon),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::StateDef {
        variables: {
            let mut vars = HashMap::new();
            vars.insert(
                "counter".to_string(),
                ast::StateVarDef {
                    name: "counter".to_string(),
                    type_info: ast::TypeInfo::Simple("Integer".to_string()),
                    initial_value: Some(ast::Expression::Literal(ast::Literal::Integer(0))),
                },
            );
            vars.insert(
                "name".to_string(),
                ast::StateVarDef {
                    name: "name".to_string(),
                    type_info: ast::TypeInfo::Simple("String".to_string()),
                    initial_value: None,
                },
            );
            vars
        },
    };

    assert_eq!(parse_state().parse(&input, 0), Ok((input.len(), expected)));
}


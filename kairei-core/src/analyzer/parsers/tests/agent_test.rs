use tracing::debug;

use crate::analyzer::Parser;
use crate::analyzer::parsers::agent::*;
use crate::analyzer::parsers::handlers::answer::parse_answer;
use crate::analyzer::parsers::handlers::observe::parse_observe;
use crate::analyzer::parsers::handlers::react::parse_react;
use crate::tokenizer::literal::{StringLiteral, StringPart};
use crate::tokenizer::symbol::Operator;
use crate::tokenizer::{keyword::Keyword, literal::Literal, symbol::Delimiter, token::Token};
use crate::{RequestHandler, ast};
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
                    statements: vec![ast::Statement::Return(ast::Expression::Literal(
                        ast::Literal::Null,
                    ))],
                },
            }],
        }),
        answer: None,
        react: None,
    };

    assert_eq!(
        parse_agent_def().parse(&input, 0),
        Ok((input.len(), expected))
    );
}

#[test]
fn test_parse_lifecycle() {
    let input = vec![
        Token::Keyword(Keyword::Lifecycle),
        Token::Delimiter(Delimiter::OpenBrace),
        // init handler
        Token::Keyword(Keyword::OnInit),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Literal(Literal::Null),
        Token::Delimiter(Delimiter::CloseBrace),
        // destroy handler
        Token::Keyword(Keyword::OnDestroy),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Literal(Literal::Null),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::LifecycleDef {
        on_init: Some(ast::HandlerBlock {
            statements: vec![ast::Statement::Return(ast::Expression::Literal(
                ast::Literal::Null,
            ))],
        }),
        on_destroy: Some(ast::HandlerBlock {
            statements: vec![ast::Statement::Return(ast::Expression::Literal(
                ast::Literal::Null,
            ))],
        }),
    };

    assert_eq!(
        parse_lifecycle().parse(&input, 0),
        Ok((input.len(), expected))
    );
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

#[test]
fn test_parse_full_state_block() {
    let input = vec![
        Token::Keyword(Keyword::State),
        Token::Delimiter(Delimiter::OpenBrace),
        // counter: Int = 0
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("Int".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(0)),
        Token::Delimiter(Delimiter::Semicolon),
        // name: String = "test"
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("test".to_string()),
        ]))),
        Token::Delimiter(Delimiter::Semicolon),
        // active: Bool = true
        Token::Identifier("active".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("Bool".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Boolean(true)),
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
                    type_info: ast::TypeInfo::Simple("Int".to_string()),
                    initial_value: Some(ast::Expression::Literal(ast::Literal::Integer(0))),
                },
            );
            vars.insert(
                "name".to_string(),
                ast::StateVarDef {
                    name: "name".to_string(),
                    type_info: ast::TypeInfo::Simple("String".to_string()),
                    initial_value: Some(ast::Expression::Literal(ast::Literal::String(
                        "test".to_string(),
                    ))),
                },
            );
            vars.insert(
                "active".to_string(),
                ast::StateVarDef {
                    name: "active".to_string(),
                    type_info: ast::TypeInfo::Simple("Bool".to_string()),
                    initial_value: Some(ast::Expression::Literal(ast::Literal::Boolean(true))),
                },
            );
            vars
        },
    };

    assert_eq!(parse_state().parse(&input, 0), Ok((input.len(), expected)));
}

#[test]
fn test_parse_lifecycle_block() {
    let input = vec![
        Token::Keyword(Keyword::Lifecycle),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::OnInit),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(0)),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Keyword(Keyword::OnDestroy),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Emit),
        Token::Identifier("Shutdown".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Keyword(Keyword::To),
        Token::Identifier("manager".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::LifecycleDef {
        on_init: Some(ast::HandlerBlock {
            statements: vec![ast::Statement::Assignment {
                target: vec![ast::Expression::Variable("counter".to_string())],
                value: ast::Expression::Literal(ast::Literal::Integer(0)),
            }],
        }),
        on_destroy: Some(ast::HandlerBlock {
            statements: vec![ast::Statement::Emit {
                event_type: ast::EventType::Custom("Shutdown".to_string()),
                parameters: vec![],
                target: Some("manager".to_string()),
            }],
        }),
    };

    assert_eq!(
        parse_lifecycle().parse(&input, 0),
        Ok((input.len(), expected))
    );
}

#[test]
fn test_parse_observe_block() {
    let input = vec![
        Token::Keyword(Keyword::Observe),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::On),
        Token::Identifier("Tick".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Identifier("counter".to_string()),
        Token::Operator(Operator::Plus),
        Token::Literal(Literal::Integer(1)),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Keyword(Keyword::On),
        Token::Identifier("StateUpdated".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("updated".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::ObserveDef {
        handlers: vec![
            ast::EventHandler {
                event_type: ast::EventType::Tick,
                parameters: vec![],
                block: ast::HandlerBlock {
                    statements: vec![ast::Statement::Assignment {
                        target: vec![ast::Expression::Variable("counter".to_string())],
                        value: ast::Expression::BinaryOp {
                            op: ast::BinaryOperator::Add,
                            left: Box::new(ast::Expression::Variable("counter".to_string())),
                            right: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                        },
                    }],
                },
            },
            ast::EventHandler {
                event_type: ast::EventType::Custom("StateUpdated".to_string()),
                parameters: vec![],
                block: ast::HandlerBlock {
                    statements: vec![ast::Statement::Assignment {
                        target: vec![ast::Expression::Variable("name".to_string())],
                        value: ast::Expression::Literal(ast::Literal::String(
                            "updated".to_string(),
                        )),
                    }],
                },
            },
        ],
    };

    assert_eq!(
        parse_observe().parse(&input, 0),
        Ok((input.len(), expected))
    );
}

#[test]
fn test_parse_answer_block() {
    let input = vec![
        // Answer block
        Token::Keyword(Keyword::Answer),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::On),
        Token::Keyword(Keyword::Request),
        Token::Identifier("GetCount".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Operator(Operator::ThinArrow),
        Token::Identifier("Result".to_string()),
        Token::Operator(Operator::Less),
        Token::Identifier("Int".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("Error".to_string()),
        Token::Operator(Operator::Greater),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Identifier("Ok".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Keyword(Keyword::On),
        Token::Keyword(Keyword::Request),
        Token::Identifier("SetName".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("newName".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Operator(Operator::ThinArrow),
        Token::Identifier("Result".to_string()),
        Token::Operator(Operator::Less),
        Token::Identifier("Bool".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("Error".to_string()),
        Token::Operator(Operator::Greater),
        Token::Keyword(Keyword::With),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("strictness".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::Float(0.9)),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("stability".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::Float(0.95)),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Identifier("newName".to_string()),
        Token::Keyword(Keyword::Return),
        Token::Identifier("Ok".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::Boolean(true)),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::AnswerDef {
        handlers: vec![
            RequestHandler {
                request_type: ast::RequestType::Custom("GetCount".to_string()),
                parameters: vec![],
                return_type: ast::TypeInfo::Result {
                    ok_type: Box::new(ast::TypeInfo::Simple("Int".to_string())),
                    err_type: Box::new(ast::TypeInfo::Simple("Error".to_string())),
                },
                constraints: None,
                block: ast::HandlerBlock {
                    statements: vec![ast::Statement::Return(ast::Expression::Ok(Box::new(
                        ast::Expression::Variable("counter".to_string()),
                    )))],
                },
            },
            RequestHandler {
                request_type: ast::RequestType::Custom("SetName".to_string()),
                parameters: vec![ast::Parameter {
                    name: "newName".to_string(),
                    type_info: ast::TypeInfo::Simple("String".to_string()),
                }],
                return_type: ast::TypeInfo::Result {
                    ok_type: Box::new(ast::TypeInfo::Simple("Bool".to_string())),
                    err_type: Box::new(ast::TypeInfo::Simple("Error".to_string())),
                },
                constraints: Some(ast::Constraints {
                    strictness: Some(0.9),
                    stability: Some(0.95),
                    latency: None,
                }),
                block: ast::HandlerBlock {
                    statements: vec![
                        ast::Statement::Assignment {
                            target: vec![ast::Expression::Variable("name".to_string())],
                            value: ast::Expression::Variable("newName".to_string()),
                        },
                        ast::Statement::Return(ast::Expression::Ok(Box::new(
                            ast::Expression::Literal(ast::Literal::Boolean(true)),
                        ))),
                    ],
                },
            },
        ],
    };

    assert_eq!(parse_answer().parse(&input, 0), Ok((input.len(), expected)));
}

#[test]
fn test_parse_react_block() {
    let input = vec![
        Token::Keyword(Keyword::React),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::On),
        Token::Identifier("Message".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("content".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(0)),
        Token::Keyword(Keyword::Emit),
        Token::Identifier("StateUpdated".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("agent".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("self".to_string()),
        ]))),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("counter".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Keyword(Keyword::To),
        Token::Identifier("manager".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::ReactDef {
        handlers: vec![ast::EventHandler {
            event_type: ast::EventType::Custom("Message".to_string()),
            parameters: vec![ast::Parameter {
                name: "content".to_string(),
                type_info: ast::TypeInfo::Simple("String".to_string()),
            }],
            block: ast::HandlerBlock {
                statements: vec![
                    ast::Statement::Assignment {
                        target: vec![ast::Expression::Variable("counter".to_string())],
                        value: ast::Expression::Literal(ast::Literal::Integer(0)),
                    },
                    ast::Statement::Emit {
                        event_type: ast::EventType::Custom("StateUpdated".to_string()),
                        parameters: vec![
                            ast::Argument::Named {
                                name: "agent".to_string(),
                                value: ast::Expression::Literal(ast::Literal::String(
                                    "self".to_string(),
                                )),
                            },
                            ast::Argument::Named {
                                name: "counter".to_string(),
                                value: ast::Expression::Literal(ast::Literal::String(
                                    "counter".to_string(),
                                )),
                            },
                        ],
                        target: Some("manager".to_string()),
                    },
                ],
            },
        }],
    };

    assert_eq!(parse_react().parse(&input, 0), Ok((input.len(), expected)));
}
#[test]
fn test_parse_agent() {
    let input = vec![
        Token::Keyword(Keyword::Micro),
        Token::Identifier("TestAgent".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Lifecycle),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::OnInit),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(0)),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Keyword(Keyword::OnDestroy),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Emit),
        Token::Identifier("Shutdown".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Keyword(Keyword::To),
        Token::Identifier("manager".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Keyword(Keyword::State),
        Token::Delimiter(Delimiter::OpenBrace),
        // counter: Int = 0
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("Int".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(0)),
        Token::Delimiter(Delimiter::Semicolon),
        // name: String = "test"
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("test".to_string()),
        ]))),
        Token::Delimiter(Delimiter::Semicolon),
        // active: Bool = true
        Token::Identifier("active".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("Bool".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Boolean(true)),
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
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Identifier("counter".to_string()),
        Token::Operator(Operator::Plus),
        Token::Literal(Literal::Integer(1)),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Keyword(Keyword::On),
        Token::Identifier("StateUpdated".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("updated".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Keyword(Keyword::Answer),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::On),
        Token::Keyword(Keyword::Request),
        Token::Identifier("GetCount".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Operator(Operator::ThinArrow),
        Token::Identifier("Result".to_string()),
        Token::Operator(Operator::Less),
        Token::Identifier("Int".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("Error".to_string()),
        Token::Operator(Operator::Greater),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Identifier("Ok".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Keyword(Keyword::On),
        Token::Keyword(Keyword::Request),
        Token::Identifier("SetName".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("newName".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Operator(Operator::ThinArrow),
        Token::Identifier("Result".to_string()),
        Token::Operator(Operator::Less),
        Token::Identifier("Bool".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("Error".to_string()),
        Token::Operator(Operator::Greater),
        Token::Keyword(Keyword::With),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("strictness".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::Float(0.9)),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("stability".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::Float(0.95)),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Identifier("newName".to_string()),
        Token::Keyword(Keyword::Return),
        Token::Identifier("Ok".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::Boolean(true)),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Keyword(Keyword::React),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::On),
        Token::Identifier("Message".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("content".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Equal),
        Token::Literal(Literal::Integer(0)),
        Token::Keyword(Keyword::Emit),
        Token::Identifier("StateUpdated".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("agent".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("self".to_string()),
        ]))),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("counter".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("counter".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Keyword(Keyword::To),
        Token::Identifier("manager".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let result = parse_agent_def().parse(&input, 0);
    debug!("{:?}", result);

    assert!(result.is_ok());
}

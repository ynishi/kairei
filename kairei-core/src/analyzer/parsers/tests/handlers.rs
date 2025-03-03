use crate::analyzer::Parser;
use crate::analyzer::parsers::expression::*;
use crate::analyzer::parsers::handlers::{answer::*, observe::*, react::*, *};
use crate::analyzer::parsers::world::*;
use crate::ast;
use crate::tokenizer::literal::{StringLiteral, StringPart};
use crate::tokenizer::symbol::Operator;
use crate::tokenizer::{keyword::Keyword, literal::Literal, symbol::Delimiter, token::Token};

#[test]
fn test_parse_observe() {
    let input = vec![
        Token::Keyword(Keyword::Observe),
        Token::Delimiter(Delimiter::OpenBrace),
        // イベントハンドラー1: Tickイベント
        Token::Keyword(Keyword::On),
        Token::Identifier("Tick".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Literal(Literal::Null),
        Token::Delimiter(Delimiter::CloseBrace),
        // イベントハンドラー2: カスタムイベント
        Token::Keyword(Keyword::On),
        Token::Identifier("CustomEvent".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("param".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Identifier("param".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::ObserveDef {
        handlers: vec![
            ast::EventHandler {
                event_type: ast::EventType::Tick,
                parameters: vec![],
                block: ast::HandlerBlock {
                    statements: vec![ast::Statement::Return(ast::Expression::Literal(
                        ast::Literal::Null,
                    ))],
                },
            },
            ast::EventHandler {
                event_type: ast::EventType::Custom("CustomEvent".to_string()),
                parameters: vec![ast::Parameter {
                    name: "param".to_string(),
                    type_info: ast::TypeInfo::Simple("String".to_string()),
                }],
                block: ast::HandlerBlock {
                    statements: vec![ast::Statement::Return(ast::Expression::Variable(
                        "param".to_string(),
                    ))],
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
fn test_parse_answer() {
    let input = vec![
        Token::Keyword(Keyword::Answer),
        Token::Delimiter(Delimiter::OpenBrace),
        // リクエストハンドラー1: シンプルなクエリ
        Token::Keyword(Keyword::On),
        Token::Keyword(Keyword::Query),
        Token::Operator(Operator::Dot),
        Token::Identifier("GetData".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Operator(Operator::Arrow),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("data".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseBrace),
        // リクエストハンドラー2: 制約付きアクション
        Token::Keyword(Keyword::On),
        Token::Keyword(Keyword::Action),
        Token::Operator(Operator::Dot),
        Token::Identifier("DoSomething".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("input".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Operator(Operator::Arrow),
        Token::Identifier("Result".to_string()),
        Token::Operator(Operator::Less),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("Error".to_string()),
        Token::Operator(Operator::Greater),
        Token::Keyword(Keyword::With),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("strictness".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::Float(0.8)),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Identifier("Ok".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("input".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::AnswerDef {
        handlers: vec![
            ast::RequestHandler {
                request_type: ast::RequestType::Query {
                    query_type: "GetData".to_string(),
                },
                parameters: vec![],
                return_type: ast::TypeInfo::Simple("String".to_string()),
                constraints: None,
                block: ast::HandlerBlock {
                    statements: vec![ast::Statement::Return(ast::Expression::Literal(
                        ast::Literal::String("data".to_string()),
                    ))],
                },
            },
            ast::RequestHandler {
                request_type: ast::RequestType::Action {
                    action_type: "DoSomething".to_string(),
                },
                parameters: vec![ast::Parameter {
                    name: "input".to_string(),
                    type_info: ast::TypeInfo::Simple("String".to_string()),
                }],
                return_type: ast::TypeInfo::Result {
                    ok_type: Box::new(ast::TypeInfo::Simple("String".to_string())),
                    err_type: Box::new(ast::TypeInfo::Simple("Error".to_string())),
                },
                constraints: Some(ast::Constraints {
                    strictness: Some(0.8),
                    stability: None,
                    latency: None,
                }),
                block: ast::HandlerBlock {
                    statements: vec![ast::Statement::Return(ast::Expression::Ok(Box::new(
                        ast::Expression::Variable("input".to_string()),
                    )))],
                },
            },
        ],
    };

    assert_eq!(parse_answer().parse(&input, 0), Ok((input.len(), expected)));
}

#[test]
fn test_parse_react() {
    let input = vec![
        Token::Keyword(Keyword::React),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::On),
        Token::Identifier("StateUpdated".to_string()),
        Token::Operator(Operator::Dot),
        Token::Identifier("other_agent".to_string()),
        Token::Operator(Operator::Dot),
        Token::Identifier("status".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("new_status".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Identifier("new_status".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];

    let expected = ast::ReactDef {
        handlers: vec![ast::EventHandler {
            event_type: ast::EventType::StateUpdated {
                agent_name: "other_agent".to_string(),
                state_name: "status".to_string(),
            },
            parameters: vec![ast::Parameter {
                name: "new_status".to_string(),
                type_info: ast::TypeInfo::Simple("String".to_string()),
            }],
            block: ast::HandlerBlock {
                statements: vec![ast::Statement::Return(ast::Expression::Variable(
                    "new_status".to_string(),
                ))],
            },
        }],
    };

    assert_eq!(parse_react().parse(&input, 0), Ok((input.len(), expected)));
}

#[test]
fn test_parse_handler() {
    let input = vec![
        Token::Keyword(Keyword::Handlers),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::On),
        Token::Identifier("event".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("param1".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Identifier("param1".to_string()),
        Token::Delimiter(Delimiter::CloseBrace),
        Token::Delimiter(Delimiter::CloseBrace),
    ];
    let expected = ast::HandlersDef {
        handlers: vec![ast::HandlerDef {
            event_name: "event".to_string(),
            parameters: vec![ast::Parameter {
                name: "param1".to_string(),
                type_info: ast::TypeInfo::Simple("String".to_string()),
            }],
            block: ast::HandlerBlock {
                statements: vec![ast::Statement::Return(ast::Expression::Variable(
                    "param1".to_string(),
                ))],
            },
        }],
    };
    assert_eq!(
        parse_handlers().parse(&input, 0),
        Ok((input.len(), expected))
    );
}

#[test]
fn test_parse_parameter() {
    let input = vec![
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
    ];
    let expected = ast::Parameter {
        name: "name".to_string(),
        type_info: ast::TypeInfo::Simple("String".to_string()),
    };
    assert_eq!(parse_parameter().parse(&input, 0), Ok((3, expected)));
}

#[test]
fn test_parse_parameters() {
    let input = vec![
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("param1".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    let expected = vec![ast::Parameter {
        name: "param1".to_string(),
        type_info: ast::TypeInfo::Simple("String".to_string()),
    }];
    assert_eq!(parse_parameters().parse(&input, 0), Ok((5, expected)));
}

#[test]
fn test_parse_request() {
    // 基本的なリクエスト
    let input = &[
        Token::Keyword(Keyword::Request),
        Token::Identifier("FindHotels".to_string()),
        Token::Keyword(Keyword::To),
        Token::Identifier("HotelFinder".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("location".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("destination".to_string()),
        Token::Delimiter(Delimiter::Comma),
        Token::Identifier("budget".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("budget".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
    ];

    let (pos, expr) = parse_request().parse(input, 0).unwrap();
    assert_eq!(pos, input.len());

    match expr {
        ast::Expression::Request {
            agent,
            request_type,
            parameters,
            options,
        } => {
            assert_eq!(agent, "HotelFinder");
            assert_eq!(
                request_type,
                ast::RequestType::Custom("FindHotels".to_string())
            );
            assert_eq!(parameters.len(), 2);

            match parameters[0].clone() {
                ast::Argument::Named { name, value } => {
                    assert_eq!(name, "location");
                    assert_eq!(value, ast::Expression::Variable("destination".to_string()));
                }
                _ => panic!("Expected Named argument"),
            }

            assert!(options.is_none());
        }
        _ => panic!("Expected Request expression"),
    }

    // 複雑な式を含むリクエスト
    let input = &[
        Token::Keyword(Keyword::Request),
        Token::Identifier("FindHotels".to_string()),
        Token::Keyword(Keyword::To),
        Token::Identifier("HotelFinder".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("budget".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("budget".to_string()),
        Token::Operator(Operator::Multiply),
        Token::Literal(Literal::Float(0.4)),
        Token::Delimiter(Delimiter::CloseParen),
    ];

    let (pos, expr) = parse_request().parse(input, 0).unwrap();
    assert_eq!(pos, input.len());

    match expr {
        ast::Expression::Request { parameters, .. } => {
            assert_eq!(parameters.len(), 1);
            match parameters[0].clone() {
                ast::Argument::Named { name, value } => {
                    assert_eq!(name, "budget");
                    match value {
                        ast::Expression::BinaryOp { op, left, right } => {
                            assert_eq!(op, ast::BinaryOperator::Multiply);
                            match (&*left, *right) {
                                (
                                    ast::Expression::Variable(name),
                                    ast::Expression::Literal(ast::Literal::Float(value)),
                                ) => {
                                    assert_eq!(name, "budget");
                                    assert_eq!(value, 0.4);
                                }
                                _ => panic!("Expected multiplication of variable and float"),
                            }
                        }
                        _ => panic!("Expected BinaryOp expression for budget"),
                    }
                }
                _ => panic!("Expected Named argument"),
            }
        }
        _ => panic!("Expected Request expression"),
    }

    // エラーケース
    // request キーワードがない
    let input = &[Token::Identifier("FindHotels".to_string())];
    assert!(parse_request().parse(input, 0).is_err());

    // "to" キーワードがない
    let input = &[
        Token::Identifier("request".to_string()),
        Token::Identifier("FindHotels".to_string()),
        Token::Identifier("HotelFinder".to_string()),
    ];
    assert!(parse_request().parse(input, 0).is_err());
}

use tracing::warn;

use super::{
    super::{core::*, prelude::*},
    *,
};
use crate::ast;
use crate::tokenizer::{keyword::Keyword, symbol::Operator, token::Token};

// Import will action parser
pub mod will;
use std::collections::HashMap;
pub use will::parse_will_action;

pub fn parse_expression() -> impl Parser<Token, ast::Expression> {
    with_context(lazy(parse_binary_expression), "expression")
}

pub fn parse_binary_expression() -> impl Parser<Token, ast::Expression> {
    with_context(parse_logical_or(), "binary expression")
}

fn parse_logical_or() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            tuple2(
                parse_logical_and(),
                many(tuple2(parse_operator_or(), parse_logical_and())),
            ),
            |(first, rest)| {
                rest.into_iter()
                    .fold(first, |left, (op, right)| ast::Expression::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
            },
        ),
        "logical or",
    )
}

fn parse_logical_and() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            tuple2(
                parse_comparison(),
                many(tuple2(parse_operator_and(), parse_comparison())),
            ),
            |(first, rest)| {
                rest.into_iter()
                    .fold(first, |left, (op, right)| ast::Expression::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
            },
        ),
        "logical and",
    )
}

fn parse_comparison() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            tuple2(
                parse_additive(),
                many(tuple2(parse_operator_comparison(), parse_additive())),
            ),
            |(first, rest)| {
                rest.into_iter()
                    .fold(first, |left, (op, right)| ast::Expression::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
            },
        ),
        "comparison",
    )
}

fn parse_operator_comparison() -> impl Parser<Token, ast::BinaryOperator> {
    with_context(
        choice(vec![
            Box::new(parse_comparison_equal()),
            Box::new(parse_comparison_not_equal()),
            Box::new(parse_comparison_greater()),
            Box::new(parse_comparison_greater_equal()),
            Box::new(parse_comparison_less()),
            Box::new(parse_comparison_less_equal()),
        ]),
        "comparison operator",
    )
}

fn parse_operator_or() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::Or)), |_| {
        ast::BinaryOperator::Or
    })
}

fn parse_operator_and() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::And)), |_| {
        ast::BinaryOperator::And
    })
}

fn parse_comparison_equal() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::EqualEqual)), |_| {
        ast::BinaryOperator::Equal
    })
}

fn parse_comparison_not_equal() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::NotEqual)), |_| {
        ast::BinaryOperator::NotEqual
    })
}

fn parse_comparison_greater() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::Greater)), |_| {
        ast::BinaryOperator::GreaterThan
    })
}

fn parse_comparison_greater_equal() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::GreaterEqual)), |_| {
        ast::BinaryOperator::GreaterThanEqual
    })
}

fn parse_comparison_less() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::Less)), |_| {
        ast::BinaryOperator::LessThan
    })
}

fn parse_comparison_less_equal() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::LessEqual)), |_| {
        ast::BinaryOperator::LessThanEqual
    })
}

fn parse_additive() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            tuple2(
                parse_multiplicative(),
                many(tuple2(
                    choice(vec![
                        Box::new(parse_operator_add()),
                        Box::new(parse_operator_subtract()),
                    ]),
                    parse_multiplicative(),
                )),
            ),
            |(first, rest)| {
                rest.into_iter()
                    .fold(first, |left, (op, right)| ast::Expression::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
            },
        ),
        "additive",
    )
}

// 乗除算 (*, /)
fn parse_multiplicative() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            tuple2(
                parse_unary(),
                many(tuple2(
                    choice(vec![
                        Box::new(parse_operator_multiply()),
                        Box::new(parse_operator_divide()),
                    ]),
                    parse_unary(),
                )),
            ),
            |(first, rest)| {
                rest.into_iter()
                    .fold(first, |left, (op, right)| ast::Expression::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
            },
        ),
        "multiplicative",
    )
}

fn parse_unary() -> impl Parser<Token, ast::Expression> {
    with_context(
        choice(vec![
            Box::new(map(
                tuple2(parse_operator_not(), parse_primary()),
                |(op, expr)| ast::Expression::BinaryOp {
                    op,
                    left: Box::new(expr),
                    right: Box::new(ast::Expression::Literal(ast::Literal::String(
                        "OPERATOR_NOT".to_string(),
                    ))),
                },
            )),
            Box::new(map(
                tuple2(parse_operator_minus(), parse_primary()),
                |(op, expr)| ast::Expression::BinaryOp {
                    op,
                    left: Box::new(expr),
                    right: Box::new(ast::Expression::Literal(ast::Literal::String(
                        "OPERATOR_MINUS".to_string(),
                    ))),
                },
            )),
            Box::new(parse_primary()),
        ]),
        "unary",
    )
}

fn parse_operator_not() -> impl Parser<Token, ast::BinaryOperator> {
    with_context(
        map(equal(Token::Operator(Operator::Not)), |_| {
            ast::BinaryOperator::NotEqual
        }),
        "not operator",
    )
}

fn parse_operator_minus() -> impl Parser<Token, ast::BinaryOperator> {
    with_context(
        map(equal(Token::Operator(Operator::Minus)), |_| {
            ast::BinaryOperator::Subtract
        }),
        "minus operator",
    )
}

fn parse_primary() -> impl Parser<Token, ast::Expression> {
    with_context(
        choice(vec![
            Box::new(parse_ok()),
            Box::new(parse_err()),
            Box::new(parse_think()),
            Box::new(parse_function_call()),
            Box::new(map(parse_literal(), ast::Expression::Literal)),
            Box::new(map(parse_identifier(), ast::Expression::Variable)),
            Box::new(map(parse_state_access(), ast::Expression::StateAccess)),
            Box::new(parse_request()),
            Box::new(parse_await()),
            Box::new(will::parse_will_action()),
        ]),
        "primary",
    )
}

fn parse_state_access() -> impl Parser<Token, ast::StateAccessPath> {
    with_context(
        map(
            tuple2(
                parse_identifier(),
                many(preceded(as_unit(parse_dot()), parse_identifier())),
            ),
            |(first, rest)| {
                ast::StateAccessPath(
                    std::iter::once(first)
                        .chain(rest.into_iter().map(|s| s.to_string()))
                        .collect::<Vec<_>>(),
                )
            },
        ),
        "state access",
    )
}

pub fn parse_dot() -> impl Parser<Token, Token> {
    with_context(equal(Token::Operator(Operator::Dot)), "dot")
}

fn parse_function_call() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            tuple2(
                parse_identifier(),
                delimited(
                    as_unit(parse_open_paren()),
                    separated_list(parse_expression(), as_unit(parse_comma())),
                    as_unit(parse_close_paren()),
                ),
            ),
            |(function, arguments)| ast::Expression::FunctionCall {
                function,
                arguments,
            },
        ),
        "function call",
    )
}

fn parse_think() -> impl Parser<Token, ast::Expression> {
    with_context(
        choice(vec![
            Box::new(parse_think_multiple()),
            Box::new(parse_think_single()),
        ]),
        "think",
    )
}

pub fn parse_think_single() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            tuple3(
                as_unit(parse_think_keyword()),
                parse_think_arguments(),
                optional(parse_think_attributes()),
            ),
            |(_, args, with_block)| ast::Expression::Think { args, with_block },
        ),
        "think single",
    )
}

pub fn parse_think_multiple() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            preceded(
                as_unit(parse_think_keyword()),
                tuple2(
                    delimited(
                        as_unit(parse_open_paren()),
                        separated_list(
                            tuple3(
                                parse_identifier(),
                                as_unit(parse_colon()),
                                parse_expression(),
                            ),
                            as_unit(parse_comma()),
                        ),
                        as_unit(parse_close_paren()),
                    ),
                    optional(parse_think_attributes()),
                ),
            ),
            |(params, with_block)| {
                let args = params
                    .into_iter()
                    .map(|(name, _, value)| ast::Argument::Named { name, value })
                    .collect();
                ast::Expression::Think { args, with_block }
            },
        ),
        "think multiple",
    )
}

fn parse_think_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Think)), "think keyword")
}

fn parse_think_arguments() -> impl Parser<Token, Vec<ast::Argument>> {
    with_context(parse_arguments(), "think attributes")
}

pub fn parse_arguments() -> impl Parser<Token, Vec<ast::Argument>> {
    with_context(
        map(
            delimited(
                as_unit(parse_open_paren()),
                separated_list(
                    choice(vec![
                        Box::new(
                            // パターン1: 名前付き引数
                            map(
                                tuple3(
                                    parse_identifier(),
                                    as_unit(parse_colon()),
                                    lazy(parse_expression),
                                ),
                                |(name, _, value)| (Some(name), value),
                            ),
                        ),
                        Box::new(
                            // パターン2: 名前なし引数
                            map(lazy(parse_expression), |value| (None, value)),
                        ),
                    ]),
                    as_unit(parse_comma()),
                ),
                as_unit(parse_close_paren()),
            ),
            |arguments| {
                arguments
                    .into_iter()
                    .map(|(name, value)| match name {
                        Some(name) => ast::Argument::Named { name, value },
                        None => ast::Argument::Positional(value),
                    })
                    .collect()
            },
        ),
        "attributes",
    )
}

fn parse_think_attributes() -> impl Parser<Token, ast::ThinkAttributes> {
    with_context(
        map(
            preceded(
                as_unit(parse_with_keyword()),
                delimited(
                    as_unit(parse_open_brace()),
                    separated_list(parse_think_attribute(), as_unit(parse_comma())),
                    as_unit(parse_close_brace()),
                ),
            ),
            collect_with_settings,
        ),
        "think attributes",
    )
}

pub fn parse_with_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::With)), "with keyword")
}

fn parse_think_attribute() -> impl Parser<Token, ThinkAttributeKV> {
    with_context(
        map(
            tuple3(
                parse_identifier(),
                as_unit(parse_colon()),
                parse_attribute_value(),
            ),
            |(key, _, value)| ThinkAttributeKV { key, value },
        ),
        "think attribute",
    )
}

fn parse_attribute_value() -> impl Parser<Token, ast::Literal> {
    with_context(parse_literal(), "attribute value")
}

fn collect_with_settings(settings: Vec<ThinkAttributeKV>) -> ast::ThinkAttributes {
    let mut block = ast::ThinkAttributes {
        provider: None,
        model: None,
        temperature: None,
        max_tokens: None,
        retry: None,
        policies: vec![],
        prompt_generator_type: None,
        plugins: HashMap::new(),
    };

    for setting in settings {
        match (setting.key.as_str(), setting.value) {
            ("provider", ast::Literal::String(s)) => block.provider = Some(s),
            ("model", ast::Literal::String(s)) => block.model = Some(s),
            ("temperature", ast::Literal::Float(f)) => block.temperature = Some(f),
            ("retry", ast::Literal::Retry(r)) => block.retry = Some(r),
            ("max_tokens", ast::Literal::Integer(n)) => block.max_tokens = Some(n as u32),
            ("policies", ast::Literal::List(policies)) => {
                for policy in policies {
                    if let ast::Literal::String(text) = policy {
                        let policy = ast::Policy {
                            text,
                            scope: ast::PolicyScope::Think,
                            internal_id: ast::PolicyId::new(),
                        };
                        block.policies.push(policy);
                    }
                }
            }
            // プラグイン設定の処理
            (plugin_name, ast::Literal::Map(configs)) => {
                let mut plugin_config = HashMap::new();
                for (key, value) in configs {
                    plugin_config.insert(key, value);
                }
                block.plugins.insert(plugin_name.to_string(), plugin_config);
            }
            (key, value) => {
                warn!("Unknown think attribute: {}={:?}", key, value);
            }
        }
    }

    block
}

#[derive(Debug, Clone)]
struct ThinkAttributeKV {
    key: String,
    value: ast::Literal,
}

pub fn parse_request() -> impl Parser<Token, ast::Expression> {
    map(
        tuple5(
            as_unit(parse_request_keyword()),
            parse_identifier(), // リクエストタイプ
            as_unit(parse_to_keyword()),
            parse_identifier(), // エージェント名
            delimited(
                as_unit(parse_open_paren()),
                separated_list(
                    // パラメータリスト
                    tuple3(
                        parse_identifier(), // パラメータ名
                        as_unit(parse_colon()),
                        parse_expression(), // 値
                    ),
                    as_unit(parse_comma()),
                ),
                as_unit(parse_close_paren()),
            ),
        ),
        |(_, request_type, _, agent, parameters)| {
            let params = parameters
                .into_iter()
                .map(|(name, _, value)| ast::Argument::Named { name, value })
                .collect();

            ast::Expression::Request {
                agent,
                request_type: ast::RequestType::Custom(request_type),
                parameters: params,
                options: None, // オプションの実装は別途必要
            }
        },
    )
}

fn parse_request_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Request)), "request keyword")
}

fn parse_ok() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            preceded(
                as_unit(parse_ok_ident()),
                delimited(
                    as_unit(parse_open_paren()),
                    parse_expression(),
                    as_unit(parse_close_paren()),
                ),
            ),
            |expression| ast::Expression::Ok(Box::new(expression)),
        ),
        "Ok expression",
    )
}

pub fn parse_err() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            preceded(
                as_unit(parse_err_ident()),
                delimited(
                    as_unit(parse_open_paren()),
                    parse_expression(),
                    as_unit(parse_close_paren()),
                ),
            ),
            |expression| ast::Expression::Err(Box::new(expression)),
        ),
        "Err expression",
    )
}

fn parse_await() -> impl Parser<Token, ast::Expression> {
    with_context(
        choice(vec![
            Box::new(parse_await_single()),
            Box::new(parse_await_multiple()),
        ]),
        "await",
    )
}

pub fn parse_await_single() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            preceded(as_unit(parse_await_keyword()), parse_expression()),
            |expression| ast::Expression::Await(vec![expression]),
        ),
        "await single",
    )
}

pub fn parse_await_multiple() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            preceded(
                as_unit(parse_await_keyword()),
                delimited(
                    as_unit(parse_open_paren()),
                    separated_list(parse_expression(), as_unit(parse_comma())),
                    as_unit(parse_close_paren()),
                ),
            ),
            ast::Expression::Await,
        ),
        "await multiple",
    )
}

fn parse_await_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Await)), "await")
}

fn parse_operator_add() -> impl Parser<Token, ast::BinaryOperator> {
    with_context(
        map(equal(Token::Operator(Operator::Plus)), |_| {
            ast::BinaryOperator::Add
        }),
        "add operator",
    )
}

fn parse_operator_subtract() -> impl Parser<Token, ast::BinaryOperator> {
    with_context(
        map(equal(Token::Operator(Operator::Minus)), |_| {
            ast::BinaryOperator::Subtract
        }),
        "subtract operator",
    )
}

fn parse_operator_multiply() -> impl Parser<Token, ast::BinaryOperator> {
    with_context(
        map(equal(Token::Operator(Operator::Multiply)), |_| {
            ast::BinaryOperator::Multiply
        }),
        "multiply operator",
    )
}

fn parse_operator_divide() -> impl Parser<Token, ast::BinaryOperator> {
    with_context(
        map(equal(Token::Operator(Operator::Divide)), |_| {
            ast::BinaryOperator::Divide
        }),
        "divide operator",
    )
}

pub fn parse_literal_expression() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(parse_literal(), ast::Expression::Literal),
        "literal expression",
    )
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::literal::{Literal, StringLiteral, StringPart};
    use crate::tokenizer::symbol::Delimiter;

    use super::*;

    #[test]
    fn test_parse_binary_expression_logical() {
        // a && b || c のテスト
        let input = &[
            Token::Identifier("a".to_string()),
            Token::Operator(Operator::And),
            Token::Identifier("b".to_string()),
            Token::Operator(Operator::Or),
            Token::Identifier("c".to_string()),
        ];
        let (pos, expr) = parse_binary_expression().parse(input, 0).unwrap();
        assert_eq!(pos, 5);

        match expr {
            ast::Expression::BinaryOp { op, left, right } => {
                assert_eq!(op, ast::BinaryOperator::Or);
                // (a && b) の部分を確認
                match *left {
                    ast::Expression::BinaryOp { op, left, right } => {
                        assert_eq!(op, ast::BinaryOperator::And);
                        match *left {
                            ast::Expression::Variable(name) => assert_eq!(name, "a"),
                            _ => panic!("Expected variable 'a'"),
                        }
                        match *right {
                            ast::Expression::Variable(name) => assert_eq!(name, "b"),
                            _ => panic!("Expected variable 'b'"),
                        }
                    }
                    _ => panic!("Expected And operation"),
                }
                // c の部分を確認
                match *right {
                    ast::Expression::Variable(name) => assert_eq!(name, "c"),
                    _ => panic!("Expected variable 'c'"),
                }
            }
            _ => panic!("Expected Or operation"),
        }
    }

    #[test]
    fn test_parse_comparison_with_logical() {
        // a > b && c == d のテスト
        let input = &[
            Token::Identifier("a".to_string()),
            Token::Operator(Operator::Greater),
            Token::Identifier("b".to_string()),
            Token::Operator(Operator::And),
            Token::Identifier("c".to_string()),
            Token::Operator(Operator::EqualEqual),
            Token::Identifier("d".to_string()),
        ];
        let (pos, expr) = parse_binary_expression().parse(input, 0).unwrap();
        assert_eq!(pos, 7);

        match expr {
            ast::Expression::BinaryOp { op, left, right } => {
                assert_eq!(op, ast::BinaryOperator::And);
                match *left {
                    ast::Expression::BinaryOp { op, left, right } => {
                        assert_eq!(op, ast::BinaryOperator::GreaterThan);
                        match *left {
                            ast::Expression::Variable(name) => assert_eq!(name, "a"),
                            _ => panic!("Expected variable 'a'"),
                        }
                        match *right {
                            ast::Expression::Variable(name) => assert_eq!(name, "b"),
                            _ => panic!("Expected variable 'b'"),
                        }
                    }
                    _ => panic!("Expected GreaterThan operation"),
                }
                // c == d の部分を確認
                match *right {
                    ast::Expression::BinaryOp { op, left, right } => {
                        assert_eq!(op, ast::BinaryOperator::Equal);
                        match *left {
                            ast::Expression::Variable(name) => assert_eq!(name, "c"),
                            _ => panic!("Expected variable 'c'"),
                        }
                        match *right {
                            ast::Expression::Variable(name) => assert_eq!(name, "d"),
                            _ => panic!("Expected variable 'd'"),
                        }
                    }
                    _ => panic!("Expected Equal operation"),
                }
            }
            _ => panic!("Expected And operation"),
        }
    }

    #[test]
    fn test_parse_binary_expression_precedence() {
        // a || b && c のテスト（&& が || より優先）
        let input = &[
            Token::Identifier("a".to_string()),
            Token::Operator(Operator::Or),
            Token::Identifier("b".to_string()),
            Token::Operator(Operator::And),
            Token::Identifier("c".to_string()),
        ];
        let (pos, expr) = parse_binary_expression().parse(input, 0).unwrap();
        assert_eq!(pos, 5);

        match expr {
            ast::Expression::BinaryOp { op, left, right } => {
                assert_eq!(op, ast::BinaryOperator::Or);
                // a の部分を確認
                match *left {
                    ast::Expression::Variable(name) => assert_eq!(name, "a"),
                    _ => panic!("Expected variable 'a'"),
                }
                // (b && c) の部分を確認
                match *right {
                    ast::Expression::BinaryOp { op, left, right } => {
                        assert_eq!(op, ast::BinaryOperator::And);
                        match *left {
                            ast::Expression::Variable(name) => assert_eq!(name, "b"),
                            _ => panic!("Expected variable 'b'"),
                        }
                        match *right {
                            ast::Expression::Variable(name) => assert_eq!(name, "c"),
                            _ => panic!("Expected variable 'c'"),
                        }
                    }
                    _ => panic!("Expected And operation"),
                }
            }
            _ => panic!("Expected Or operation"),
        }
    }

    #[test]
    fn test_parse_state_access() {
        // シンプルなアクセス（単一識別子）
        let input = &[Token::Identifier("state".to_string())];
        let (pos, path) = parse_state_access().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(path.0, vec!["state"]);

        // ドット区切りのパス
        let input = &[
            Token::Identifier("state".to_string()),
            Token::Operator(Operator::Dot),
            Token::Identifier("user".to_string()),
            Token::Operator(Operator::Dot),
            Token::Identifier("name".to_string()),
        ];
        let (pos, path) = parse_state_access().parse(input, 0).unwrap();
        assert_eq!(pos, 5);
        assert_eq!(path.0, vec!["state", "user", "name"]);
    }

    #[test]
    fn test_parse_ok_err() {
        // OKのテスト
        let input = &[
            Token::Identifier("Ok".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (pos, expr) = parse_ok().parse(input, 0).unwrap();
        assert_eq!(pos, 4);
        match expr {
            ast::Expression::Ok(expr) => match *expr {
                ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 42),
                _ => panic!("Expected Integer literal inside Ok"),
            },
            _ => panic!("Expected Ok expression"),
        }

        // Errのテスト
        let input = &[
            Token::Identifier("Err".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("error message".to_string()),
            ]))),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (pos, expr) = parse_err().parse(input, 0).unwrap();
        assert_eq!(pos, 4);
        match expr {
            ast::Expression::Err(expr) => match *expr {
                ast::Expression::Literal(ast::Literal::String(ref s)) => {
                    assert_eq!(s, "error message")
                }
                _ => panic!("Expected String literal inside Err"),
            },
            _ => panic!("Expected Err expression"),
        }

        // 複雑な式を含むOkのテスト
        let input = &[
            Token::Identifier("Ok".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(1)),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (pos, expr) = parse_ok().parse(input, 0).unwrap();
        assert_eq!(pos, 7);
        match expr {
            ast::Expression::Ok(expr) => match *expr {
                ast::Expression::FunctionCall {
                    ref function,
                    ref arguments,
                } => {
                    assert_eq!(function, "foo");
                    assert_eq!(arguments.len(), 1);
                    match arguments[0] {
                        ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 1),
                        _ => panic!("Expected Integer argument"),
                    }
                }
                _ => panic!("Expected FunctionCall inside Ok"),
            },
            _ => panic!("Expected Ok expression"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        // 引数なしの関数呼び出し
        let input = &[
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (pos, expr) = parse_function_call().parse(input, 0).unwrap();
        assert_eq!(pos, 3);
        match expr {
            ast::Expression::FunctionCall {
                function,
                arguments,
            } => {
                assert_eq!(function, "foo");
                assert!(arguments.is_empty());
            }
            _ => panic!("Expected FunctionCall"),
        }

        // 複数引数の関数呼び出し
        let input = &[
            Token::Identifier("bar".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::Comma),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("test".to_string()),
            ]))),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (pos, expr) = parse_function_call().parse(input, 0).unwrap();
        assert_eq!(pos, 6);
        match expr {
            ast::Expression::FunctionCall {
                function,
                arguments,
            } => {
                assert_eq!(function, "bar");
                assert_eq!(arguments.len(), 2);
                match &arguments[0] {
                    ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(*n, 42),
                    _ => panic!("Expected Integer literal"),
                }
                match &arguments[1] {
                    ast::Expression::Literal(ast::Literal::String(s)) => assert_eq!(s, "test"),
                    _ => panic!("Expected String literal"),
                }
            }
            _ => panic!("Expected FunctionCall"),
        }

        // ネストした関数呼び出し
        let input = &[
            Token::Identifier("outer".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("inner".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (pos, expr) = parse_function_call().parse(input, 0).unwrap();
        assert_eq!(pos, 6);
        match expr {
            ast::Expression::FunctionCall {
                function,
                arguments,
            } => {
                assert_eq!(function, "outer");
                assert_eq!(arguments.len(), 1);
                match &arguments[0] {
                    ast::Expression::FunctionCall {
                        function,
                        arguments,
                    } => {
                        assert_eq!(function, "inner");
                        assert!(arguments.is_empty());
                    }
                    _ => panic!("Expected nested FunctionCall"),
                }
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_parse_await() {
        // 単一式のawait
        let input = &[
            Token::Keyword(Keyword::Await),
            Token::Identifier("future".to_string()),
        ];
        let (pos, expr) = parse_await().parse(input, 0).unwrap();
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

        // 複数式のawait（カンマ区切り、括弧付き）
        let input = &[
            Token::Keyword(Keyword::Await),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("bar".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (pos, expr) = parse_await().parse(input, 0).unwrap();
        assert_eq!(pos, 6);
        match expr {
            ast::Expression::Await(expressions) => {
                assert_eq!(expressions.len(), 2);
                match &expressions[0] {
                    ast::Expression::Variable(name) => assert_eq!(name, "foo"),
                    _ => panic!("Expected first Variable expression"),
                }
                match &expressions[1] {
                    ast::Expression::Variable(name) => assert_eq!(name, "bar"),
                    _ => panic!("Expected second Variable expression"),
                }
            }
            _ => panic!("Expected Await expression"),
        }

        // 複雑な式を含む単一await
        let input = &[
            Token::Keyword(Keyword::Await),
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (pos, expr) = parse_await().parse(input, 0).unwrap();
        assert_eq!(pos, 5);
        match expr {
            ast::Expression::Await(expressions) => {
                assert_eq!(expressions.len(), 1);
                match &expressions[0] {
                    ast::Expression::FunctionCall {
                        function,
                        arguments,
                    } => {
                        assert_eq!(function, "foo");
                        assert_eq!(arguments.len(), 1);
                        match &arguments[0] {
                            ast::Expression::Literal(ast::Literal::Integer(n)) => {
                                assert_eq!(*n, 42)
                            }
                            _ => panic!("Expected Integer argument"),
                        }
                    }
                    _ => panic!("Expected FunctionCall"),
                }
            }
            _ => panic!("Expected Await expression"),
        }

        // エラーケース: awaitキーワードなし
        let input = &[Token::Identifier("notawait".to_string())];
        assert!(parse_await().parse(input, 0).is_err());

        // エラーケース: 括弧が不完全
        let input = &[
            Token::Keyword(Keyword::Await),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("foo".to_string()),
        ];
        assert!(parse_await().parse(input, 0).is_err());
    }

    #[test]
    fn test_parse_operators() {
        // 論理演算子
        let input = &[Token::Operator(Operator::Or)];
        let (pos, op) = parse_operator_or().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::Or);

        let input = &[Token::Operator(Operator::And)];
        let (pos, op) = parse_operator_and().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::And);

        // 比較演算子
        let input = &[Token::Operator(Operator::EqualEqual)];
        let (pos, op) = parse_comparison_equal().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::Equal);
    }

    #[test]
    fn test_parse_comparison() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Operator(Operator::EqualEqual),
            Token::Literal(Literal::Integer(2)),
        ];
        let (pos, expr) = parse_comparison().parse(input, 0).unwrap();
        assert_eq!(pos, 3);
        match expr {
            ast::Expression::BinaryOp { op, left, right } => {
                assert_eq!(op, ast::BinaryOperator::Equal);
                match *left {
                    ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 1),
                    _ => panic!("Expected Integer literal 1"),
                }
                match *right {
                    ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 2),
                    _ => panic!("Expected Integer literal 2"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_parse_operator_comparison() {
        // すべての比較演算子をテスト
        let comparison_tests = vec![
            (
                Token::Operator(Operator::EqualEqual),
                ast::BinaryOperator::Equal,
            ),
            (
                Token::Operator(Operator::NotEqual),
                ast::BinaryOperator::NotEqual,
            ),
            (
                Token::Operator(Operator::Greater),
                ast::BinaryOperator::GreaterThan,
            ),
            (
                Token::Operator(Operator::GreaterEqual),
                ast::BinaryOperator::GreaterThanEqual,
            ),
            (
                Token::Operator(Operator::Less),
                ast::BinaryOperator::LessThan,
            ),
            (
                Token::Operator(Operator::LessEqual),
                ast::BinaryOperator::LessThanEqual,
            ),
        ];

        for (token, expected_op) in comparison_tests {
            let input = &[token.clone()];
            let result = parse_operator_comparison().parse(input, 0);
            assert!(result.is_ok(), "Failed to parse {:?}", token);
            let (pos, op) = result.unwrap();
            assert_eq!(pos, 1);
            assert_eq!(op, expected_op);
        }

        // 非比較演算子はパースに失敗することを確認
        let non_comparison_tests = vec![
            Token::Operator(Operator::Plus),
            Token::Operator(Operator::Minus),
            Token::Operator(Operator::Multiply),
            Token::Operator(Operator::Divide),
            Token::Operator(Operator::And),
            Token::Operator(Operator::Or),
            Token::Operator(Operator::Not),
        ];

        for token in non_comparison_tests {
            let input = &[token.clone()];
            let result = parse_operator_comparison().parse(input, 0);
            assert!(result.is_err(), "Should not parse {:?}", token);
        }
    }

    #[test]
    fn test_parse_additive() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Operator(Operator::Plus),
            Token::Literal(Literal::Integer(2)),
            Token::Operator(Operator::Minus),
            Token::Literal(Literal::Integer(3)),
        ];
        let (pos, expr) = parse_additive().parse(input, 0).unwrap();
        assert_eq!(pos, 5);
        match expr {
            ast::Expression::BinaryOp { op, left, right } => {
                assert_eq!(op, ast::BinaryOperator::Subtract);
                match *left {
                    ast::Expression::BinaryOp { op, left, right } => {
                        assert_eq!(op, ast::BinaryOperator::Add);
                        match *left {
                            ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 1),
                            _ => panic!("Expected Integer literal 1"),
                        }
                        match *right {
                            ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 2),
                            _ => panic!("Expected Integer literal 2"),
                        }
                    }
                    _ => panic!("Expected BinaryOp"),
                }
                match *right {
                    ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 3),
                    _ => panic!("Expected Integer literal 3"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_parse_multiplicative() {
        let input = &[
            Token::Literal(Literal::Integer(2)),
            Token::Operator(Operator::Multiply),
            Token::Literal(Literal::Integer(3)),
            Token::Operator(Operator::Divide),
            Token::Literal(Literal::Integer(4)),
        ];
        let (pos, expr) = parse_multiplicative().parse(input, 0).unwrap();
        assert_eq!(pos, 5);

        // ((2 * 3) / 4) の構造を確認
        match expr {
            ast::Expression::BinaryOp { op, left, right } => {
                assert_eq!(op, ast::BinaryOperator::Divide);
                match *left {
                    ast::Expression::BinaryOp { op, left, right } => {
                        assert_eq!(op, ast::BinaryOperator::Multiply);
                        match *left {
                            ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 2),
                            _ => panic!("Expected Integer literal 2"),
                        }
                        match *right {
                            ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 3),
                            _ => panic!("Expected Integer literal 3"),
                        }
                    }
                    _ => panic!("Expected BinaryOp"),
                }
                match *right {
                    ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 4),
                    _ => panic!("Expected Integer literal 4"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_parse_operator_add() {
        let input = &[Token::Operator(Operator::Plus)];
        let (pos, op) = parse_operator_add().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::Add);
    }

    #[test]
    fn test_parse_operator_add_fails() {
        let input = &[Token::Operator(Operator::Minus)];
        assert!(parse_operator_add().parse(input, 0).is_err());
    }

    #[test]
    fn test_parse_operator_subtract() {
        let input = &[Token::Operator(Operator::Minus)];
        let (pos, op) = parse_operator_subtract().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::Subtract);
    }

    #[test]
    fn test_parse_operator_multiply() {
        let input = &[Token::Operator(Operator::Multiply)];
        let (pos, op) = parse_operator_multiply().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::Multiply);
    }

    #[test]
    fn test_parse_operator_divide() {
        let input = &[Token::Operator(Operator::Divide)];
        let (pos, op) = parse_operator_divide().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::Divide);
    }

    #[test]
    fn test_parse_unary() {
        let input = &[
            Token::Operator(Operator::Not),
            Token::Literal(Literal::Integer(42)),
        ];
        let (pos, expr) = parse_unary().parse(input, 0).unwrap();
        assert_eq!(pos, 2);
        match expr {
            ast::Expression::BinaryOp { op, left, .. } => {
                assert_eq!(op, ast::BinaryOperator::NotEqual);
                match *left {
                    ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 42),
                    _ => panic!("Expected Integer literal 42"),
                }
            }
            _ => panic!("Expected UnaryOp"),
        }

        let input = &[
            Token::Operator(Operator::Minus),
            Token::Literal(Literal::Integer(42)),
        ];
        let (pos, expr) = parse_unary().parse(input, 0).unwrap();
        assert_eq!(pos, 2);
        match expr {
            ast::Expression::BinaryOp { op, left, .. } => {
                assert_eq!(op, ast::BinaryOperator::Subtract);
                match *left {
                    ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 42),
                    _ => panic!("Expected Integer literal 42"),
                }
            }
            _ => panic!("Expected UnaryOp"),
        }
        // without unary operator
        let input = &[Token::Literal(Literal::Integer(42))];
        let (pos, expr) = parse_unary().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        match expr {
            ast::Expression::Literal(ast::Literal::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Expected Integer literal 42"),
        }
        // without unary operator, null
        let input = &[Token::Literal(Literal::Null)];
        let (pos, expr) = parse_unary().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        match expr {
            ast::Expression::Literal(ast::Literal::Null) => {}
            _ => panic!("Expected Null literal"),
        }
    }

    #[test]
    fn test_parse_operator_not() {
        let input = &[Token::Operator(Operator::Not)];
        let (pos, op) = parse_operator_not().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::NotEqual);
    }

    #[test]
    fn test_parse_operator_minus() {
        let input = &[Token::Operator(Operator::Minus)];
        let (pos, op) = parse_operator_minus().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::Subtract);
    }

    #[test]
    fn test_parse_think() {
        // 基本的なthink式（with blockなし）
        let input = &[
            Token::Keyword(Keyword::Think),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("Find suitable hotels matching criteria".to_string()),
            ]))),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("location".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
        ];

        let (pos, expr) = parse_think().parse(input, 0).unwrap();
        assert_eq!(pos, input.len());

        match expr {
            ast::Expression::Think { args, with_block } => {
                assert_eq!(args.len(), 2);

                // 第1引数の検証（文字列リテラル）
                match &args[0] {
                    ast::Argument::Positional(value) => match value {
                        ast::Expression::Literal(ast::Literal::String(s)) => {
                            assert_eq!(s, "Find suitable hotels matching criteria");
                        }
                        _ => panic!("Expected string literal for first argument"),
                    },
                    _ => panic!("Expected positional argument for first argument"),
                }

                // 第2引数の検証（識別子）
                match &args[1] {
                    ast::Argument::Positional(value) => match value {
                        ast::Expression::Variable(name) => {
                            assert_eq!(name, "location");
                        }
                        _ => panic!("Expected identifier for second argument"),
                    },
                    _ => panic!("Expected positional argument for second argument"),
                }

                assert!(with_block.is_none());
            }
            _ => panic!("Expected Think expression"),
        }

        // 名前付き引数とwithブロックを含むthink式
        let input = &[
            Token::Keyword(Keyword::Think),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("Find suitable hotels matching criteria".to_string()),
            ]))),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("check_in".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Identifier("start_date".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Keyword(Keyword::With),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Identifier("provider".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("openai".to_string()),
            ]))),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("search".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Identifier("filters".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Delimiter(Delimiter::OpenBracket),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("hotels".to_string()),
            ]))),
            Token::Delimiter(Delimiter::CloseBracket),
            // Policy設定
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("policies".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Delimiter(Delimiter::OpenBracket),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("hotelsPolicy".to_string()),
            ]))),
            Token::Delimiter(Delimiter::CloseBracket),
            Token::Delimiter(Delimiter::CloseBrace),
            Token::Delimiter(Delimiter::CloseBrace),
        ];

        let (pos, expr) = parse_think().parse(input, 0).unwrap();
        assert_eq!(pos, input.len());

        match expr {
            ast::Expression::Think { args, with_block } => {
                assert_eq!(args.len(), 2);

                match args[1].clone() {
                    ast::Argument::Named { name, value } => {
                        assert_eq!(name, "check_in");
                        match value {
                            ast::Expression::Variable(name) => assert_eq!(name, "start_date"),
                            _ => panic!("Expected Variable"),
                        }
                    }
                    _ => panic!("Expected String literal"),
                }

                // withブロック
                with_block.expect("Expected with block");
            }
            _ => panic!("Expected Think expression"),
        }
    }

    #[test]
    fn test_parse_think_keyword() {
        let input = &[
            Token::Keyword(Keyword::Think),
            Token::Identifier("think".to_string()),
        ];

        let (pos, token) = parse_think_keyword().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(token, Token::Keyword(Keyword::Think));
    }

    #[test]
    fn test_parse_think_attributes() {
        let parser = parse_think_attributes();

        let tokens = vec![
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
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("temperature".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Literal(Literal::Float(0.7)),
            Token::Delimiter(Delimiter::Comma),
            // search: { filters: ["hotels"] }
            Token::Identifier("search".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Identifier("filters".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Delimiter(Delimiter::OpenBracket),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("hotels".to_string()),
            ]))),
            Token::Delimiter(Delimiter::CloseBracket),
            Token::Delimiter(Delimiter::CloseBrace),
            Token::Delimiter(Delimiter::Comma),
            // key: "value"
            Token::Identifier("key".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("value".to_string()),
            ]))),
            Token::Delimiter(Delimiter::Comma),
            // value: 42
            Token::Identifier("value".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseBrace),
        ];

        let result = parser.parse(&tokens, 0);
        assert!(result.is_ok());
        let (pos, attrs) = result.unwrap();
        assert_eq!(pos, tokens.len());

        // 基本設定の検証
        assert_eq!(attrs.provider, Some("openai".to_string()));
        assert_eq!(attrs.model, Some("gpt-4".to_string()));
        assert_eq!(attrs.temperature, Some(0.7));

        // プラグイン設定の検証
        assert!(attrs.plugins.contains_key("search"));
        if let Some(search_config) = attrs.plugins.get("search") {
            assert!(search_config.contains_key("filters"));
            match &search_config["filters"] {
                ast::Literal::List(arr) => {
                    assert_eq!(arr.len(), 1);
                    assert!(matches!(&arr[0], ast::Literal::String(s) if s == "hotels"));
                }
                _ => panic!("Expected array for filters"),
            }
        }
    }

    #[test]
    fn test_collect_with_settings() {
        let settings = vec![
            ThinkAttributeKV {
                key: "provider".to_string(),
                value: ast::Literal::String("openai".to_string()),
            },
            ThinkAttributeKV {
                key: "model".to_string(),
                value: ast::Literal::String("gpt-4o-mini".to_string()),
            },
            ThinkAttributeKV {
                key: "temperature".to_string(),
                value: ast::Literal::Float(0.7),
            },
            ThinkAttributeKV {
                key: "max_tokens".to_string(),
                value: ast::Literal::Integer(500),
            },
            ThinkAttributeKV {
                key: "retry".to_string(),
                value: ast::Literal::Retry(ast::RetryConfig {
                    max_attempts: 3,
                    delay: ast::RetryDelay::Fixed(5),
                }),
            },
            ThinkAttributeKV {
                key: "policies".to_string(),
                value: ast::Literal::List(vec![
                    ast::Literal::String("Policy 1".to_string()),
                    ast::Literal::String("Policy 2".to_string()),
                ]),
            },
            ThinkAttributeKV {
                key: "plugin".to_string(),
                value: ast::Literal::Map(
                    vec![(
                        "plugin_config".to_string(),
                        ast::Literal::Map(HashMap::from_iter(vec![
                            (
                                "key1".to_string(),
                                ast::Literal::String("value1".to_string()),
                            ),
                            (
                                "key2".to_string(),
                                ast::Literal::String("value2".to_string()),
                            ),
                        ])),
                    )]
                    .into_iter()
                    .collect(),
                ),
            },
        ];

        let block = collect_with_settings(settings);

        assert_eq!(block.provider, Some("openai".to_string()));
        assert_eq!(block.model, Some("gpt-4o-mini".to_string()));
        assert_eq!(block.temperature, Some(0.7));
        assert_eq!(block.max_tokens, Some(500));
        assert_eq!(
            block.retry,
            Some(ast::RetryConfig {
                max_attempts: 3,
                delay: ast::RetryDelay::Fixed(5),
            })
        );
        assert_eq!(block.policies.len(), 2);
        assert_eq!(block.plugins.len(), 1);
    }

    #[test]
    fn test_parse_with_keyword() {
        let input = &[
            Token::Keyword(Keyword::With),
            Token::Identifier("with".to_string()),
        ];

        let (pos, token) = parse_with_keyword().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(token, Token::Keyword(Keyword::With));
    }

    #[test]
    fn test_parse_think_arguments() {
        let parser = parse_think_arguments();

        // テストケース1: 空の引数リスト
        let tokens = vec![
            Token::Delimiter(Delimiter::OpenParen),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let result = parser.parse(&tokens, 0);
        assert!(result.is_ok());
        let (pos, args) = result.unwrap();
        assert_eq!(pos, 2); // 2トークンを消費
        assert_eq!(args.len(), 0);

        // テストケース2: 位置引数のみ
        let tokens = vec![
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let result = parser.parse(&tokens, 0);
        assert!(result.is_ok());
        let (pos, args) = result.unwrap();
        assert_eq!(pos, 3); // 3トークンを消費
        assert_eq!(args.len(), 1);
        match &args[0] {
            ast::Argument::Positional(expr) => {
                assert!(matches!(
                    expr,
                    ast::Expression::Literal(ast::Literal::Integer(42))
                ));
            }
            _ => panic!("Expected positional argument"),
        }

        // テストケース3: 文字列リテラルと識別子
        let tokens = vec![
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("Find suitable hotels".to_string()),
            ]))),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("location".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let result = parser.parse(&tokens, 0);
        assert!(result.is_ok());
        let (pos, args) = result.unwrap();
        assert_eq!(pos, 5); // 5トークンを消費
        assert_eq!(args.len(), 2);
        match &args[0] {
            ast::Argument::Positional(expr) => {
                assert!(
                    matches!(expr,  ast::Expression::Literal(ast::Literal::String(s)) if s == "Find suitable hotels")
                );
            }
            _ => panic!("Expected positional string argument"),
        }
        match &args[1] {
            ast::Argument::Positional(expr) => {
                assert!(matches!(expr, ast::Expression::Variable(id) if id == "location"));
            }
            _ => panic!("Expected positional identifier argument"),
        }

        // テストケース4: 名前付き引数
        let tokens = vec![
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("prompt".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("Search query".to_string()),
            ]))),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let result = parser.parse(&tokens, 0);
        assert!(result.is_ok());
        let (pos, args) = result.unwrap();
        assert_eq!(pos, 5); // 5トークンを消費
        assert_eq!(args.len(), 1);
        match &args[0] {
            ast::Argument::Named { name, value } => {
                assert_eq!(name, "prompt");
                assert!(
                    matches!(value, ast::Expression::Literal(ast::Literal::String(s)) if s == "Search query")
                );
            }
            _ => panic!("Expected named argument"),
        }
    }
}

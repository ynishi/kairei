use std::{collections::HashMap, time::Duration};

use crate::{
    tokenizer::{
        literal::{Literal, StringPart},
        symbol::{Delimiter, Operator},
        token::Token,
    },
    FieldInfo, TypeInfo,
};

use super::{ast, prelude::*, Parser};

fn parse_type_info() -> impl Parser<Token, ast::TypeInfo> {
    lazy(|| {
        choice(vec![
            Box::new(parse_result_type()),
            Box::new(parse_option_type()),
            Box::new(parse_array_type()),
            Box::new(parse_simple_type()),
        ])
    })
}

fn parse_field() -> impl Parser<Token, (String, FieldInfo)> {
    with_context(
        preceded(
            parse_identifier(),
            choice(vec![
                // パターン1: 型指定あり + 初期値あり
                Box::new(parse_field_typed_with_default()),
                // パターン2: 型指定あり + 初期値なし
                Box::new(map(
                    preceded(parse_colon(), parse_type_reference()),
                    |(_, type_info)| FieldInfo {
                        type_info: Some(type_info),
                        default_value: None,
                    },
                )),
                // パターン3: 型推論 + 初期値
                Box::new(map(
                    preceded(parse_equal(), parse_expression()),
                    |(_, value)| FieldInfo {
                        type_info: None,
                        default_value: Some(value),
                    },
                )),
            ]),
        ),
        "field",
    )
}

fn parse_field_typed_with_default() -> impl Parser<Token, FieldInfo> {
    map(
        tuple4(
            parse_colon(),
            parse_identifier(),
            parse_equal(),
            parse_expression(),
        ),
        |(_, type_info, _, value)| FieldInfo {
            type_info: Some(TypeInfo::Simple(type_info)),
            default_value: Some(value),
        },
    )
}

// 型参照のみを許可（インライン定義は不可）
fn parse_type_reference() -> impl Parser<Token, TypeInfo> {
    lazy(|| {
        choice(vec![
            Box::new(parse_simple_type()), // String, Intなど
            Box::new(parse_result_type()), // Result<T, E>
            Box::new(parse_option_type()), // Option<T>
            Box::new(parse_array_type()),  // Array<T>
                                           // カスタム型の参照はOK、定義は不可
        ])
    })
}

fn parse_custom_type() -> impl Parser<Token, TypeInfo> {
    with_context(
        map(
            with_context(
                preceded(
                    with_context(parse_identifier(), "pi"), // 型名
                    with_context(
                        delimited(
                            with_context(as_unit(parse_open_brace()), "op"),
                            separated_list(
                                with_context(lazy(|| with_context(parse_field(), "pf")), "lz"),
                                with_context(as_unit(parse_comma()), "pc"),
                            ),
                            as_unit(parse_close_brace()),
                        ),
                        "del",
                    ),
                ),
                "map",
            ),
            |(name, fields)| {
                let field_map = fields.into_iter().collect::<HashMap<_, _>>();
                TypeInfo::Custom {
                    name,
                    fields: field_map,
                }
            },
        ),
        "custom type",
    )
}

fn parse_option_type() -> impl Parser<Token, ast::TypeInfo> {
    map(parse_generic_single_arg("Option"), |inner_type| {
        ast::TypeInfo::Option(inner_type)
    })
}

fn parse_array_type() -> impl Parser<Token, ast::TypeInfo> {
    map(parse_generic_single_arg("Array"), |element_type| {
        ast::TypeInfo::Array(element_type)
    })
}

fn parse_generic_single_arg(type_name: &'static str) -> impl Parser<Token, Box<TypeInfo>> {
    map(
        preceded(
            expected(parse_identifier(), type_name.to_string()),
            delimited(
                as_unit(parse_open_brace()),
                map(parse_type_info(), Box::new),
                as_unit(parse_close_brace()),
            ),
        ),
        |(_, inner)| inner,
    )
}

fn parse_result_type() -> impl Parser<Token, ast::TypeInfo> {
    map(
        tuple6(
            expected(parse_identifier(), "Result".to_string()),
            parse_open_brace(),
            parse_identifier(),
            parse_comma(),
            parse_identifier(),
            parse_close_brace(),
        ),
        |(_, _, ok, _, err, _)| ast::TypeInfo::Result {
            ok_type: Box::new(ast::TypeInfo::Simple(ok)),
            err_type: Box::new(ast::TypeInfo::Simple(err)),
        },
    )
}

fn parse_simple_type() -> impl Parser<Token, ast::TypeInfo> {
    map(parse_identifier(), ast::TypeInfo::Simple)
}

fn parse_expression() -> impl Parser<Token, ast::Expression> {
    with_context(
        choice(vec![Box::new(lazy(parse_binary_expression))]),
        "expression",
    )
}

fn parse_binary_expression() -> impl Parser<Token, ast::Expression> {
    with_context(parse_logical_or(), "binary expression")
}

fn parse_logical_or() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            preceded(
                parse_logical_and(),
                many(preceded(parse_operator_or(), parse_logical_and())),
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
            preceded(
                parse_comparison(),
                many(preceded(parse_operator_and(), parse_comparison())),
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
            preceded(
                parse_additive(),
                many(preceded(parse_operator_comparison(), parse_additive())),
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
    map(equal(Token::Operator(Operator::Equal)), |_| {
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
            preceded(
                parse_multiplicative(),
                many(preceded(
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
            preceded(
                parse_unary(),
                many(preceded(
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
                preceded(parse_operator_not(), parse_primary()),
                |(op, expr)| ast::Expression::BinaryOp {
                    op,
                    left: Box::new(expr),
                    right: Box::new(ast::Expression::Literal(ast::Literal::String(
                        "OPERATOR_NOT".to_string(),
                    ))),
                },
            )),
            Box::new(map(
                preceded(parse_operator_minus(), parse_primary()),
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
            Box::new(parse_function_call()),
            Box::new(map(parse_literal(), ast::Expression::Literal)),
            Box::new(map(parse_identifier(), ast::Expression::Variable)),
            Box::new(map(parse_state_access(), ast::Expression::StateAccess)),
            Box::new(parse_think()),
            Box::new(parse_request()),
            Box::new(parse_await()),
        ]),
        "primary",
    )
}

fn parse_state_access() -> impl Parser<Token, ast::StateAccessPath> {
    with_context(
        map(
            preceded(
                parse_identifier(),
                many(preceded(as_unit(parse_dot()), parse_identifier())),
            ),
            |(first, rest)| {
                ast::StateAccessPath(
                    std::iter::once(first)
                        .chain(rest.into_iter().map(|s| s.1.to_string()))
                        .collect::<Vec<_>>(),
                )
            },
        ),
        "state access",
    )
}

fn parse_dot() -> impl Parser<Token, Token> {
    with_context(equal(Token::Operator(Operator::Dot)), "dot")
}

fn parse_function_call() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            preceded(
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
    map(
        preceded(
            equal(Token::Identifier("think".to_string())),
            parse_expression(),
        ),
        |expr| ast::Expression::Think {
            args: todo!(),
            with_block: None,
        },
    )
}

fn parse_request() -> impl Parser<Token, ast::Expression> {
    map(
        preceded(
            equal(Token::Identifier("request".to_string())),
            parse_expression(),
        ),
        |expr| ast::Expression::Request {
            agent: todo!(),
            request_type: todo!(),
            parameters: todo!(),
            options: todo!(),
        },
    )
}

fn parse_ok() -> impl Parser<Token, ast::Expression> {
    map(
        preceded(as_unit(parse_ok_ident()), parse_expression()),
        |(_, expression)| ast::Expression::Ok(Box::new(expression)),
    )
}

fn parse_ok_ident() -> impl Parser<Token, Token> {
    equal(Token::Identifier("Ok".to_string()))
}

fn parse_err() -> impl Parser<Token, ast::Expression> {
    map(
        preceded(as_unit(parse_err_ident()), parse_expression()),
        |(_, expression)| ast::Expression::Err(Box::new(expression)),
    )
}

fn parse_err_ident() -> impl Parser<Token, Token> {
    equal(Token::Identifier("Err".to_string()))
}

fn parse_await() -> impl Parser<Token, ast::Expression> {
    choice(vec![
        Box::new(parse_await_single()),
        Box::new(parse_await_multiple()),
    ])
}

fn parse_await_single() -> impl Parser<Token, ast::Expression> {
    map(
        preceded(as_unit(parse_await_ident()), parse_expression()),
        |(_, expression)| ast::Expression::Await(vec![expression]),
    )
}

fn parse_await_multiple() -> impl Parser<Token, ast::Expression> {
    map(
        preceded(
            as_unit(parse_await_ident()),
            delimited(
                as_unit(parse_open_paren()),
                separated_list(parse_expression(), as_unit(parse_comma())),
                as_unit(parse_close_paren()),
            ),
        ),
        |(_, expressions)| ast::Expression::Await(expressions),
    )
}

fn parse_await_ident() -> impl Parser<Token, Token> {
    equal(Token::Identifier("await".to_string()))
}

fn parse_operator_add() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::Plus)), |_| {
        ast::BinaryOperator::Add
    })
}

fn parse_operator_subtract() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::Minus)), |_| {
        ast::BinaryOperator::Subtract
    })
}

fn parse_operator_multiply() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::Multiply)), |_| {
        ast::BinaryOperator::Multiply
    })
}

fn parse_operator_divide() -> impl Parser<Token, ast::BinaryOperator> {
    map(equal(Token::Operator(Operator::Divide)), |_| {
        ast::BinaryOperator::Divide
    })
}

fn parse_literal_expression() -> impl Parser<Token, ast::Expression> {
    map(parse_literal(), ast::Expression::Literal)
}

fn parse_literal() -> impl Parser<Token, ast::Literal> {
    choice(vec![
        Box::new(parse_float()),
        Box::new(parse_integer()),
        Box::new(parse_string()),
        Box::new(parse_boolean()),
        Box::new(parse_list()),
        Box::new(parse_null()),
        // parse_retry
    ])
}

fn parse_string() -> impl Parser<Token, ast::Literal> {
    satisfy(|token| match token {
        Token::Literal(Literal::String(parts)) => {
            // Vec<StringPart> から ast::Literal::String に変換
            // 現時点では単純なLiteralのみをサポート
            if parts.len() == 1 {
                match &parts[0] {
                    StringPart::Literal(s) => Some(ast::Literal::String(s.clone())),
                    // 他のケースは今は未サポート
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    })
}

fn parse_list() -> impl Parser<Token, ast::Literal> {
    map(
        delimited(
            as_unit(parse_open_bracket()),
            separated_list(lazy(parse_literal), as_unit(parse_comma())),
            as_unit(parse_close_bracket()),
        ),
        ast::Literal::List,
    )
}

fn parse_float() -> impl Parser<Token, ast::Literal> {
    map(parse_f64(), ast::Literal::Float)
}

// 数値リテラル（Integer）
fn parse_integer() -> impl Parser<Token, ast::Literal> {
    map(parse_i64(), ast::Literal::Integer)
}

// 真偽値リテラル
fn parse_boolean() -> impl Parser<Token, ast::Literal> {
    choice(vec![Box::new(parse_true()), Box::new(parse_false())])
}

fn parse_true() -> impl Parser<Token, ast::Literal> {
    parse_bool(true)
}

fn parse_false() -> impl Parser<Token, ast::Literal> {
    parse_bool(false)
}

fn parse_bool(b: bool) -> impl Parser<Token, ast::Literal> {
    map(equal(Token::Literal(Literal::Boolean(b))), move |_| {
        ast::Literal::Boolean(b)
    })
}

// nullリテラル
fn parse_null() -> impl Parser<Token, ast::Literal> {
    map(equal(Token::Literal(Literal::Null)), |_| ast::Literal::Null)
}

fn parse_comma() -> impl Parser<Token, Token> {
    with_context(equal(Token::Delimiter(Delimiter::Comma)), "comma")
}

fn parse_colon() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::Colon))
}

fn parse_equal() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::Equal))
}

fn parse_open_paren() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::OpenParen))
}

fn parse_close_paren() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::CloseParen))
}

fn parse_open_bracket() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::OpenBracket))
}

fn parse_close_bracket() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::CloseBracket))
}

fn parse_open_brace() -> impl Parser<Token, Token> {
    with_context(equal(Token::Delimiter(Delimiter::OpenBrace)), "open brace")
}

fn parse_close_brace() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::CloseBrace))
}

fn parse_duration() -> impl Parser<Token, ast::Literal> {
    choice(vec![
        Box::new(parse_duration_millis()),
        Box::new(parse_duration_sec()),
        Box::new(parse_duration_min()),
        Box::new(parse_duration_hour()),
    ])
}

// unitをまとめて取得するヘルパー関数
fn parse_duration_unit() -> impl Parser<Token, String> {
    choice(vec![
        Box::new(parse_ms()),
        Box::new(parse_sec()),
        Box::new(parse_min()),
        Box::new(parse_hour()),
    ])
}

fn parse_duration_millis() -> impl Parser<Token, ast::Literal> {
    map(preceded(parse_u64(), parse_ms()), |(value, _)| {
        ast::Literal::Duration(Duration::from_millis(value))
    })
}

fn parse_duration_sec() -> impl Parser<Token, ast::Literal> {
    map(preceded(parse_u64(), parse_sec()), |(value, _)| {
        ast::Literal::Duration(Duration::from_secs(value))
    })
}

fn parse_duration_min() -> impl Parser<Token, ast::Literal> {
    map(preceded(parse_u64(), parse_min()), |(value, _)| {
        ast::Literal::Duration(Duration::from_secs(value * 60))
    })
}

fn parse_duration_hour() -> impl Parser<Token, ast::Literal> {
    map(preceded(parse_u64(), parse_hour()), |(value, _)| {
        ast::Literal::Duration(Duration::from_secs(value * 60 * 60))
    })
}

fn parse_ms() -> impl Parser<Token, String> {
    expected(parse_identifier(), "ms".to_string())
}

fn parse_sec() -> impl Parser<Token, String> {
    expected(parse_identifier(), "s".to_string())
}

fn parse_min() -> impl Parser<Token, String> {
    expected(parse_identifier(), "min".to_string())
}

fn parse_hour() -> impl Parser<Token, String> {
    expected(parse_identifier(), "h".to_string())
}

fn parse_identifier() -> impl Parser<Token, String> {
    with_context(
        satisfy(|token| match token {
            Token::Identifier(s) => Some(s.clone()),
            _ => None,
        }),
        "identifier",
    )
}

fn parse_f64() -> impl Parser<Token, f64> {
    satisfy(|token| match token {
        Token::Literal(Literal::Float(n)) => Some(*n),
        _ => None,
    })
}

fn parse_i64() -> impl Parser<Token, i64> {
    satisfy(|token| match token {
        Token::Literal(Literal::Integer(n)) => Some(*n),
        _ => None,
    })
}

fn parse_u64() -> impl Parser<Token, u64> {
    satisfy(|token| match token {
        Token::Literal(Literal::Integer(n)) => Some(*n as u64),
        _ => None,
    })
}

fn parse_usize() -> impl Parser<Token, usize> {
    satisfy(|token| match token {
        Token::Literal(Literal::Integer(n)) => Some(*n as usize),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::literal::StringPart;

    use super::*;

    #[test]
    fn test_parse_type_info() {
        // Result型のテスト
        let input = &[
            Token::Identifier("Result".to_string()),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Identifier("String".to_string()),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("Error".to_string()),
            Token::Delimiter(Delimiter::CloseBrace),
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "John".to_string(),
            )])),
            Token::Delimiter(Delimiter::CloseBrace),
        ];

        let (pos, result) = parse_custom_type().parse(input, 0).unwrap();
        assert_eq!(pos, input.len());

        match result {
            TypeInfo::Custom { name, fields } => {
                assert_eq!(name, "Person");
                let field = fields.get("name").unwrap();
                assert_eq!(
                    field.type_info,
                    Some(TypeInfo::Simple("String".to_string()))
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
            TypeInfo::Custom { name, fields } => {
                assert_eq!(name, "Person");
                let field = fields.get("age").unwrap();
                assert_eq!(field.type_info, Some(TypeInfo::Simple("Int".to_string())));
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
            TypeInfo::Custom { name, fields } => {
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "John".to_string(),
            )])),
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
            TypeInfo::Custom { name, fields } => {
                assert_eq!(name, "Person");
                assert_eq!(fields.len(), 2);

                let name_field = fields.get("name").unwrap();
                assert_eq!(
                    name_field.type_info,
                    Some(TypeInfo::Simple("String".to_string()))
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
            TypeInfo::Custom { name, fields } => {
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "test".to_string(),
            )])),
        ];
        let (pos, field_info) = parse_field_typed_with_default().parse(input, 0).unwrap();
        assert_eq!(pos, input.len());
        assert_eq!(
            field_info.type_info,
            Some(TypeInfo::Simple("String".to_string()))
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
            Token::Literal(Literal::Integer(42)),
        ];
        let (pos, expr) = parse_ok().parse(input, 0).unwrap();
        assert_eq!(pos, 2);
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "error message".to_string(),
            )])),
        ];
        let (pos, expr) = parse_err().parse(input, 0).unwrap();
        assert_eq!(pos, 2);
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
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(1)),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (pos, expr) = parse_ok().parse(input, 0).unwrap();
        assert_eq!(pos, 5);
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "test".to_string(),
            )])),
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
            Token::Identifier("await".to_string()),
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
            Token::Identifier("await".to_string()),
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
            Token::Identifier("await".to_string()),
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
            Token::Identifier("await".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("foo".to_string()),
        ];
        assert!(parse_await().parse(input, 0).is_err());
    }

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
            Token::Operator(Operator::Equal),
            Token::Identifier("d".to_string()),
        ];
        let (pos, expr) = parse_binary_expression().parse(input, 0).unwrap();
        assert_eq!(pos, 7);
        println!("{:?}", expr);

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
        let input = &[Token::Operator(Operator::Equal)];
        let (pos, op) = parse_comparison_equal().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, ast::BinaryOperator::Equal);
    }

    #[test]
    fn test_parse_comparison() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Operator(Operator::Equal),
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
            (Token::Operator(Operator::Equal), ast::BinaryOperator::Equal),
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
    fn test_parse_string_literal() {
        // 単純な文字列リテラル
        let input = &[Token::Literal(Literal::String(vec![StringPart::Literal(
            "test string".to_string(),
        )]))];

        let (pos, result) = parse_string().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, ast::Literal::String("test string".to_string()));
    }

    #[test]
    fn test_parse_string_with_interpolation() {
        // 補間を含む文字列
        let input = &[Token::Literal(Literal::String(vec![
            StringPart::Literal("Hello ".to_string()),
            StringPart::Interpolation("name".to_string()),
            StringPart::Literal("!".to_string()),
        ]))];

        // 現時点では未サポート
        assert!(parse_string().parse(input, 0).is_err());
    }

    #[test]
    fn test_parse_string_with_newline() {
        // 改行を含む文字列
        let input = &[Token::Literal(Literal::String(vec![
            StringPart::Literal("line 1".to_string()),
            StringPart::NewLine,
            StringPart::Literal("line 2".to_string()),
        ]))];

        // 現時点では未サポート
        assert!(parse_string().parse(input, 0).is_err());
    }

    #[test]
    fn test_separated_list() {
        // シンプルな数値のカンマ区切りリスト
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Delimiter(Delimiter::Comma),
            Token::Literal(Literal::Integer(2)),
        ];

        let number_parser = choice(vec![
            Box::new(map(equal(Token::Literal(Literal::Integer(1))), |_| 1)),
            Box::new(map(equal(Token::Literal(Literal::Integer(2))), |_| 2)),
        ]);
        let comma_parser = as_unit(equal(Token::Delimiter(Delimiter::Comma)));

        let list_parser = separated_list(number_parser, comma_parser);

        let (pos, result) = list_parser.parse(input, 0).unwrap();
        assert_eq!(pos, 3);
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_separated_list_single() {
        // 単一要素のリスト
        let input = &[Token::Literal(Literal::Integer(1))];

        let number_parser = map(equal(Token::Literal(Literal::Integer(1))), |_| 1);
        let comma_parser = as_unit(equal(Token::Delimiter(Delimiter::Comma)));

        let list_parser = separated_list(number_parser, comma_parser);

        let (pos, result) = list_parser.parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn test_parse_list() {
        let input = &[
            Token::Delimiter(Delimiter::OpenBracket),
            Token::Literal(Literal::Integer(1)),
            Token::Delimiter(Delimiter::Comma),
            Token::Literal(Literal::Integer(2)),
            Token::Delimiter(Delimiter::CloseBracket),
        ];
        let (pos, result) = parse_list().parse(input, 0).unwrap();
        assert_eq!(pos, 5);
        assert_eq!(
            result,
            ast::Literal::List(vec![ast::Literal::Integer(1), ast::Literal::Integer(2)])
        );
    }

    #[test]
    fn test_parse_result_type() {
        let input = &[
            Token::Identifier("Result".to_string()),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Identifier("Success".to_string()),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("Error".to_string()),
            Token::Delimiter(Delimiter::CloseBrace),
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

    #[test]
    fn test_parse_literal() {
        // 整数
        let input = &[Token::Literal(Literal::Integer(42))];
        let (pos, result) = parse_literal().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, ast::Literal::Integer(42));

        // 真偽値
        let input = &[Token::Literal(Literal::Boolean(true))];
        let (pos, result) = parse_literal().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, ast::Literal::Boolean(true));

        // null
        let input = &[Token::Literal(Literal::Null)];
        let (pos, result) = parse_literal().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, ast::Literal::Null);
    }

    #[test]
    fn test_parse_duration() {
        // all pattern for duration
        // 1ms, 1s, 1min, 1h
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("ms".to_string()),
        ];
        let (rest, result) = super::parse_duration().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            super::ast::Literal::Duration(std::time::Duration::from_millis(1))
        );
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("s".to_string()),
        ];
        let (rest, result) = super::parse_duration().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            super::ast::Literal::Duration(std::time::Duration::from_secs(1))
        );
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("min".to_string()),
        ];
        let (rest, result) = super::parse_duration().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            super::ast::Literal::Duration(std::time::Duration::from_secs(60))
        );
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("h".to_string()),
        ];
        let (rest, result) = super::parse_duration().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            super::ast::Literal::Duration(std::time::Duration::from_secs(3600))
        );
    }

    #[test]
    fn test_parse_duration_millis() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("ms".to_string()),
        ];
        let (rest, result) = parse_duration_millis().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            ast::Literal::Duration(std::time::Duration::from_millis(1))
        );
    }

    #[test]
    fn test_parse_duration_sec() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("s".to_string()),
        ];
        let (rest, result) = parse_duration_sec().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            ast::Literal::Duration(std::time::Duration::from_secs(1))
        );
    }

    #[test]
    fn test_parse_duration_min() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("min".to_string()),
        ];
        let (rest, result) = parse_duration_min().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            ast::Literal::Duration(std::time::Duration::from_secs(60))
        );
    }

    #[test]
    fn test_parse_duration_hour() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("h".to_string()),
        ];
        let (rest, result) = parse_duration_hour().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            ast::Literal::Duration(std::time::Duration::from_secs(3600))
        );
    }

    #[test]
    fn test_parse_ms() {
        let input = &[Token::Identifier("ms".to_string())];
        let (rest, result) = parse_ms().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "ms".to_string());
    }
    #[test]
    fn test_parse_sec() {
        let input = &[Token::Identifier("s".to_string())];
        let (rest, result) = parse_sec().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "s".to_string());
    }
    #[test]
    fn test_parse_min() {
        let input = &[Token::Identifier("min".to_string())];
        let (rest, result) = parse_min().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "min".to_string());
    }
    #[test]
    fn test_parse_hour() {
        let input = &[Token::Identifier("h".to_string())];
        let (rest, result) = parse_hour().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "h".to_string());
    }

    #[test]
    fn test_identifier() {
        let input = &[Token::Identifier("foo".to_string())];
        let (rest, result) = parse_identifier().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "foo".to_string());
    }

    #[test]
    fn test_parse_f64() {
        let input = &[Token::Literal(Literal::Float(3.14))];
        let (rest, result) = parse_f64().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 3.14);
    }

    #[test]
    fn test_parse_i64() {
        let input = &[Token::Literal(Literal::Integer(42))];
        let (rest, result) = parse_i64().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_parse_u64() {
        let input = &[Token::Literal(Literal::Integer(42))];
        let (rest, result) = parse_u64().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_parse_usize() {
        let input = &[Token::Literal(Literal::Integer(42))];
        let (rest, result) = parse_usize().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 42);
    }
}

use std::{collections::HashMap, time::Duration};

use tracing::warn;

use crate::{
    tokenizer::{
        keyword::Keyword,
        literal::{Literal, StringPart},
        symbol::{Delimiter, Operator},
        token::Token,
    },
    EventType, FieldInfo, TypeInfo,
};

use super::{ast, prelude::*, Parser};

fn parse_statement() -> impl Parser<Token, ast::Statement> {
    with_context(
        lazy(|| {
            choice(vec![
                Box::new(parse_expression_statement()),
                Box::new(parse_assignment_statement()),
                Box::new(parse_return_statement()),
                Box::new(parse_emit_statement()),
                Box::new(parse_if_statement()),
                Box::new(parse_block_statement()),
            ])
        }),
        "statement",
    )
    // TODO support with_error
}

fn parse_expression_statement() -> impl Parser<Token, ast::Statement> {
    with_context(
        map(parse_expression(), ast::Statement::Expression),
        "expression statement",
    )
}

fn parse_assignment_statement() -> impl Parser<Token, ast::Statement> {
    with_context(
        map(
            tuple3(
                parse_assignment_target(),
                as_unit(parse_equal()),
                parse_literal(),
            ),
            |(target, _, value)| ast::Statement::Assignment {
                target,
                value: ast::Expression::Literal(value),
            },
        ),
        "assignment statement",
    )
}

fn parse_assignment_target() -> impl Parser<Token, Vec<ast::Expression>> {
    with_context(
        choice(vec![
            Box::new(map(parse_expression(), |expr| vec![expr])),
            Box::new(map(
                delimited(
                    as_unit(parse_open_paren()),
                    tuple2(
                        parse_expression(),
                        many(preceded(as_unit(parse_comma()), parse_expression())),
                    ),
                    as_unit(parse_close_paren()),
                ),
                |(target, targets)| {
                    let mut acc = vec![target];
                    acc.extend(targets.iter().map(|t| t.1.clone()).collect::<Vec<_>>());
                    acc
                },
            )),
        ]),
        "assignment target",
    )
}

fn parse_return_statement() -> impl Parser<Token, ast::Statement> {
    with_context(
        map(
            preceded(as_unit(parse_return_keyword()), parse_expression()),
            |(_, value)| ast::Statement::Return(value),
        ),
        "return statement",
    )
}

fn parse_return_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Return)), "return keyword")
}

fn parse_emit_statement() -> impl Parser<Token, ast::Statement> {
    with_context(
        map(
            tuple4(
                as_unit(parse_emit_keyword()),
                parse_identifier(),
                parse_emit_arguments(),
                optional(parse_emit_target()),
            ),
            |(_, event_type, parameters, target)| ast::Statement::Emit {
                event_type: EventType::Custom(event_type),
                parameters,
                target,
            },
        ),
        "emit statement",
    )
}

fn parse_emit_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Emit)), "emit keyword")
}

fn parse_emit_target() -> impl Parser<Token, String> {
    with_context(
        map(
            tuple2(as_unit(parse_to_keyword()), parse_identifier()),
            |(_, target)| target,
        ),
        "emit target",
    )
}

fn parse_emit_arguments() -> impl Parser<Token, Vec<ast::Argument>> {
    with_context(parse_arguments(), "emit arguments")
}

fn parse_to_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("to".to_string())), "to keyword")
}

fn parse_if_statement() -> impl Parser<Token, ast::Statement> {
    with_context(
        map(
            tuple4(
                as_unit(parse_if_keyword()),
                parse_expression(),
                parse_statements(),
                optional(parse_else_statement()),
            ),
            |(_, condition, then_block, else_block)| ast::Statement::If {
                condition,
                then_block,
                else_block,
            },
        ),
        "if statement",
    )
}

fn parse_if_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::If)), "if keyword")
}

fn parse_else_statement() -> impl Parser<Token, ast::Statements> {
    with_context(
        map(
            preceded(as_unit(parse_else_keyword()), parse_statements()),
            |(_, block)| block,
        ),
        "else statement",
    )
}

fn parse_else_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Else)), "else keyword")
}

fn parse_statements() -> impl Parser<Token, ast::Statements> {
    with_context(
        delimited(
            as_unit(parse_open_brace()),
            many(parse_statement()),
            as_unit(parse_close_brace()),
        ),
        "block statement",
    )
}

fn parse_block_statement() -> impl Parser<Token, ast::Statement> {
    with_context(
        map(
            delimited(
                as_unit(parse_open_brace()),
                many(parse_statement()),
                as_unit(parse_close_brace()),
            ),
            ast::Statement::Block,
        ),
        "block statement",
    )
}

fn parse_type_info() -> impl Parser<Token, ast::TypeInfo> {
    with_context(
        lazy(|| {
            choice(vec![
                Box::new(parse_result_type()),
                Box::new(parse_option_type()),
                Box::new(parse_array_type()),
                Box::new(parse_simple_type()),
            ])
        }),
        "type info",
    )
}

fn parse_field() -> impl Parser<Token, (String, FieldInfo)> {
    with_context(
        preceded(
            parse_identifier(),
            choice(vec![
                Box::new(parse_field_typed_with_default()),
                Box::new(parse_field_typed()),
                Box::new(parse_field_inferred()),
            ]),
        ),
        "field",
    )
}

fn parse_field_typed_with_default() -> impl Parser<Token, FieldInfo> {
    with_context(
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
        ),
        "typed field with default value",
    )
}

fn parse_field_typed() -> impl Parser<Token, FieldInfo> {
    with_context(
        map(
            preceded(parse_colon(), parse_type_reference()),
            |(_, type_info)| FieldInfo {
                type_info: Some(type_info),
                default_value: None,
            },
        ),
        "typed field",
    )
}

fn parse_field_inferred() -> impl Parser<Token, FieldInfo> {
    with_context(
        map(preceded(parse_equal(), parse_expression()), |(_, value)| {
            FieldInfo {
                type_info: None,
                default_value: Some(value),
            }
        }),
        "inferred field",
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
            preceded(
                parse_identifier(),
                delimited(
                    as_unit(parse_open_brace()),
                    separated_list(lazy(parse_field), as_unit(parse_comma())),
                    as_unit(parse_close_brace()),
                ),
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
    with_context(
        map(parse_generic_single_arg("Option"), |inner_type| {
            ast::TypeInfo::Option(inner_type)
        }),
        "Option type",
    )
}

fn parse_array_type() -> impl Parser<Token, ast::TypeInfo> {
    with_context(
        map(parse_generic_single_arg("Array"), |element_type| {
            ast::TypeInfo::Array(element_type)
        }),
        "Array type",
    )
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
    with_context(
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
        ),
        "Result type",
    )
}

fn parse_simple_type() -> impl Parser<Token, ast::TypeInfo> {
    with_context(
        map(parse_identifier(), ast::TypeInfo::Simple),
        "simple type",
    )
}

fn parse_expression() -> impl Parser<Token, ast::Expression> {
    with_context(lazy(parse_binary_expression), "expression")
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
            Box::new(parse_think()),
            Box::new(parse_function_call()),
            Box::new(map(parse_literal(), ast::Expression::Literal)),
            Box::new(map(parse_identifier(), ast::Expression::Variable)),
            Box::new(map(parse_state_access(), ast::Expression::StateAccess)),
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
    with_context(
        map(
            tuple3(
                as_unit(parse_think_keyword()),
                parse_think_arguments(),
                optional(parse_think_attributes()),
            ),
            |(_, args, with_block)| ast::Expression::Think { args, with_block },
        ),
        "think",
    )
}

fn parse_think_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Think)), "think keyword")
}

fn parse_think_arguments() -> impl Parser<Token, Vec<ast::Argument>> {
    with_context(parse_arguments(), "think attributes")
}

fn parse_arguments() -> impl Parser<Token, Vec<ast::Argument>> {
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
            |(_, settings)| collect_with_settings(settings),
        ),
        "think attributes",
    )
}

fn parse_with_keyword() -> impl Parser<Token, Token> {
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

fn parse_request() -> impl Parser<Token, ast::Expression> {
    map(
        tuple5(
            as_unit(parse_request_keyword()),
            parse_identifier(), // リクエストタイプ
            as_unit(equal(Token::Identifier("to".to_string()))),
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
            preceded(as_unit(parse_ok_ident()), parse_expression()),
            |(_, expression)| ast::Expression::Ok(Box::new(expression)),
        ),
        "Ok expression",
    )
}

fn parse_ok_ident() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("Ok".to_string())), "Ok")
}

fn parse_err() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            preceded(as_unit(parse_err_ident()), parse_expression()),
            |(_, expression)| ast::Expression::Err(Box::new(expression)),
        ),
        "Err expression",
    )
}

fn parse_err_ident() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("Err".to_string())), "Err")
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

fn parse_await_single() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            preceded(as_unit(parse_await_keyword()), parse_expression()),
            |(_, expression)| ast::Expression::Await(vec![expression]),
        ),
        "await single",
    )
}

fn parse_await_multiple() -> impl Parser<Token, ast::Expression> {
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
            |(_, expressions)| ast::Expression::Await(expressions),
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

fn parse_literal_expression() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(parse_literal(), ast::Expression::Literal),
        "literal expression",
    )
}

fn parse_literal() -> impl Parser<Token, ast::Literal> {
    with_context(
        choice(vec![
            Box::new(parse_float()),
            Box::new(parse_integer()),
            Box::new(parse_string()),
            Box::new(parse_boolean()),
            Box::new(parse_list()),
            Box::new(parse_map()),
            Box::new(parse_retry()),
            Box::new(parse_null()),
        ]),
        "literal",
    )
}

fn parse_string() -> impl Parser<Token, ast::Literal> {
    with_context(
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
        }),
        "string",
    )
}

fn parse_list() -> impl Parser<Token, ast::Literal> {
    with_context(
        map(
            delimited(
                as_unit(parse_open_bracket()),
                separated_list(lazy(parse_literal), as_unit(parse_comma())),
                as_unit(parse_close_bracket()),
            ),
            ast::Literal::List,
        ),
        "list",
    )
}

fn parse_map() -> impl Parser<Token, ast::Literal> {
    with_context(
        map(
            delimited(
                as_unit(parse_open_brace()),
                separated_list(parse_map_entry(), as_unit(parse_comma())),
                as_unit(parse_close_brace()),
            ),
            |entries| {
                let mut map = HashMap::new();
                for (key, value) in entries {
                    map.insert(key, value);
                }
                ast::Literal::Map(map)
            },
        ),
        "map",
    )
}

fn parse_map_entry() -> impl Parser<Token, (String, ast::Literal)> {
    with_context(
        map(
            tuple3(
                parse_identifier(),
                as_unit(parse_colon()),
                lazy(parse_literal),
            ),
            |(key, _, value)| (key, value),
        ),
        "map entry",
    )
}

fn parse_retry() -> impl Parser<Token, ast::Literal> {
    with_context(map(parse_retry_config(), ast::Literal::Retry), "retry")
}

fn parse_retry_ident() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("Retry".to_string())), "Retry")
}

fn parse_retry_config() -> impl Parser<Token, ast::RetryConfig> {
    with_context(
        map(
            tuple3(
                as_unit(parse_retry_ident()),
                parse_u64(),
                parse_retry_delay(),
            ),
            |(_, max_attempts, delay)| ast::RetryConfig {
                max_attempts,
                delay,
            },
        ),
        "retry config",
    )
}

fn parse_retry_delay() -> impl Parser<Token, ast::RetryDelay> {
    with_context(
        choice(vec![
            Box::new(parse_fixed_delay()),
            Box::new(parse_exponential_delay()),
        ]),
        "retry delay",
    )
}

fn parse_fixed_delay() -> impl Parser<Token, ast::RetryDelay> {
    with_context(
        map(
            preceded(as_unit(parse_fixed_ident()), parse_u64()),
            |(_, s)| ast::RetryDelay::Fixed(s),
        ),
        "Fixed",
    )
}

fn parse_fixed_ident() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("Fixed".to_string())), "Fixed")
}

fn parse_exponential_delay() -> impl Parser<Token, ast::RetryDelay> {
    with_context(
        map(
            preceded(
                as_unit(parse_exponential_ident()),
                tuple3(parse_u64(), as_unit(parse_comma()), parse_u64()),
            ),
            |(_, (initial, _, max))| ast::RetryDelay::Exponential { initial, max },
        ),
        "Exponential",
    )
}

fn parse_exponential_ident() -> impl Parser<Token, Token> {
    with_context(
        equal(Token::Identifier("Exponential".to_string())),
        "Exponential",
    )
}

fn parse_float() -> impl Parser<Token, ast::Literal> {
    with_context(map(parse_f64(), ast::Literal::Float), "float")
}

// 数値リテラル（Integer）
fn parse_integer() -> impl Parser<Token, ast::Literal> {
    with_context(map(parse_i64(), ast::Literal::Integer), "integer")
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

fn parse_equal_equal() -> impl Parser<Token, Token> {
    equal(Token::Operator(Operator::EqualEqual))
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
    fn test_parse_expression_statement() {
        let input = vec![Token::Literal(Literal::Integer(42))];
        let expected = ast::Statement::Expression(ast::Expression::Variable("foo".to_string()));
        assert_eq!(
            parse_expression_statement().parse(&input, 0),
            Ok((0, expected))
        );
    }

    #[test]
    fn test_parse_assignment() {
        let input = vec![
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::Equal),
            Token::Literal(Literal::Integer(42)),
        ];
        let expected = ast::Statement::Assignment {
            target: vec![ast::Expression::Variable("foo".to_string())],
            value: ast::Expression::Literal(ast::Literal::Integer(42)),
        };
        assert_eq!(
            parse_assignment_statement().parse(&input, 0),
            Ok((3, expected))
        );

        let input = vec![
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("bar".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Delimiter(Delimiter::Equal),
            Token::Literal(Literal::Integer(42)),
        ];
        let expected = ast::Statement::Assignment {
            target: vec![
                ast::Expression::Variable("foo".to_string()),
                ast::Expression::Variable("bar".to_string()),
            ],
            value: ast::Expression::Literal(ast::Literal::Integer(42)),
        };
        assert_eq!(
            parse_assignment_statement().parse(&input, 0),
            Ok((7, expected))
        );
    }

    #[test]
    fn test_parse_assignment_target() {
        let input = vec![
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::Equal),
            Token::Literal(Literal::Integer(42)),
        ];
        let expected = vec![ast::Expression::Variable("foo".to_string())];
        assert_eq!(
            parse_assignment_target().parse(&input, 0),
            Ok((1, expected))
        );

        let input = vec![
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Delimiter(Delimiter::Equal),
            Token::Literal(Literal::Integer(42)),
        ];
        let expected = vec![ast::Expression::Variable("foo".to_string())];
        assert_eq!(
            parse_assignment_target().parse(&input, 0),
            Ok((3, expected))
        );

        let input = vec![
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("foo".to_string()),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("bar".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Delimiter(Delimiter::Equal),
            Token::Literal(Literal::Integer(42)),
        ];
        let expected = vec![
            ast::Expression::Variable("foo".to_string()),
            ast::Expression::Variable("bar".to_string()),
        ];
        assert_eq!(
            parse_assignment_target().parse(&input, 0),
            Ok((5, expected))
        );
    }

    #[test]
    fn test_parse_return_statement() {
        let input = vec![
            Token::Keyword(Keyword::Return),
            Token::Literal(Literal::Integer(42)),
        ];
        let expected = ast::Statement::Return(ast::Expression::Literal(ast::Literal::Integer(42)));
        assert_eq!(parse_return_statement().parse(&input, 0), Ok((2, expected)));
    }

    #[test]
    fn test_parse_emit_statement() {
        let input = vec![
            Token::Keyword(Keyword::Emit),
            Token::Identifier("test-event".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let expected = ast::Statement::Emit {
            event_type: EventType::Custom("test-event".to_string()),
            parameters: vec![ast::Argument::Positional(ast::Expression::Literal(
                ast::Literal::Integer(42),
            ))],
            target: None,
        };
        assert_eq!(parse_emit_statement().parse(&input, 0), Ok((5, expected)));
    }

    #[test]
    fn test_parse_emit_statement_with_target() {
        let input = vec![
            Token::Keyword(Keyword::Emit),
            Token::Identifier("test-event".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Identifier("to".to_string()),
            Token::Identifier("target".to_string()),
        ];
        let expected = ast::Statement::Emit {
            event_type: EventType::Custom("test-event".to_string()),
            parameters: vec![ast::Argument::Positional(ast::Expression::Literal(
                ast::Literal::Integer(42),
            ))],
            target: Some("target".to_string()),
        };
        assert_eq!(parse_emit_statement().parse(&input, 0), Ok((7, expected)));
    }

    #[test]
    fn test_parse_if_statement() {
        let input = vec![
            Token::Keyword(Keyword::If),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Keyword(Keyword::Return),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseBrace),
        ];
        let expected = ast::Statement::If {
            condition: ast::Expression::Literal(ast::Literal::Integer(42)),
            then_block: vec![ast::Statement::Return(ast::Expression::Literal(
                ast::Literal::Integer(42),
            ))],
            else_block: None,
        };
        assert_eq!(parse_if_statement().parse(&input, 0), Ok((6, expected)));
    }

    #[test]
    fn test_parse_if_else_statement() {
        let input = vec![
            Token::Keyword(Keyword::If),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Keyword(Keyword::Return),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseBrace),
            Token::Keyword(Keyword::Else),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Keyword(Keyword::Return),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseBrace),
        ];
        let expected = ast::Statement::If {
            condition: ast::Expression::Literal(ast::Literal::Integer(42)),
            then_block: vec![ast::Statement::Return(ast::Expression::Literal(
                ast::Literal::Integer(42),
            ))],
            else_block: Some(vec![ast::Statement::Return(ast::Expression::Literal(
                ast::Literal::Integer(42),
            ))]),
        };
        assert_eq!(parse_if_statement().parse(&input, 0), Ok((11, expected)));
    }

    #[test]
    fn test_parse_block_statement() {
        let input = vec![
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Keyword(Keyword::Return),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseBrace),
        ];
        let expected = ast::Statement::Block(vec![ast::Statement::Return(
            ast::Expression::Literal(ast::Literal::Integer(42)),
        )]);
        assert_eq!(parse_block_statement().parse(&input, 0), Ok((4, expected)));
    }

    #[test]
    fn test_parse_think() {
        // 基本的なthink式（with blockなし）
        let input = &[
            Token::Keyword(Keyword::Think),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "Find suitable hotels matching criteria".to_string(),
            )])),
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "Find suitable hotels matching criteria".to_string(),
            )])),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("check_in".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Identifier("start_date".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Keyword(Keyword::With),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Identifier("provider".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "openai".to_string(),
            )])),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("search".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Identifier("filters".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Delimiter(Delimiter::OpenBracket),
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "hotels".to_string(),
            )])),
            Token::Delimiter(Delimiter::CloseBracket),
            // Policy設定
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("policies".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Delimiter(Delimiter::OpenBracket),
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "hotelsPolicy".to_string(),
            )])),
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "openai".to_string(),
            )])),
            Token::Delimiter(Delimiter::Comma),
            Token::Identifier("model".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "gpt-4".to_string(),
            )])),
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "hotels".to_string(),
            )])),
            Token::Delimiter(Delimiter::CloseBracket),
            Token::Delimiter(Delimiter::CloseBrace),
            Token::Delimiter(Delimiter::Comma),
            // key: "value"
            Token::Identifier("key".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "value".to_string(),
            )])),
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "Find suitable hotels".to_string(),
            )])),
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
            Token::Literal(Literal::String(vec![StringPart::Literal(
                "Search query".to_string(),
            )])),
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

    #[test]
    fn test_parse_request() {
        // 基本的なリクエスト
        let input = &[
            Token::Keyword(Keyword::Request),
            Token::Identifier("FindHotels".to_string()),
            Token::Identifier("to".to_string()),
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
            Token::Identifier("to".to_string()),
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

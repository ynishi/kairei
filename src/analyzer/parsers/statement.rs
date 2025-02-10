use super::{
    super::{core::*, prelude::*},
    expression::*,
    *,
};
use crate::ast;
use crate::{
    tokenizer::{keyword::Keyword, token::Token},
    Statement,
};

pub fn parse_statement() -> impl Parser<Token, ast::Statement> {
    with_context(
        map(
            lazy(|| {
                choice(vec![
                    Box::new(tuple2(
                        parse_expression_statement(),
                        optional(parse_error_handler()),
                    )),
                    Box::new(tuple2(
                        parse_assignment_statement(),
                        optional(parse_error_handler()),
                    )),
                    Box::new(tuple2(
                        parse_return_statement(),
                        optional(parse_error_handler()),
                    )),
                    Box::new(tuple2(
                        parse_emit_statement(),
                        optional(parse_error_handler()),
                    )),
                    Box::new(tuple2(
                        parse_if_statement(),
                        optional(parse_error_handler()),
                    )),
                    Box::new(tuple2(
                        parse_block_statement(),
                        optional(parse_error_handler()),
                    )),
                ])
            }),
            |(statement, error_handler)| match error_handler {
                Some(Statement::WithError {
                    statement,
                    error_handler_block,
                }) => Statement::WithError {
                    statement,
                    error_handler_block,
                },
                _ => statement,
            },
        ),
        "statement",
    )
}

fn parse_error_handler() -> impl Parser<Token, ast::Statement> {
    map(
        tuple3(
            as_unit(parse_onfail_keyword()),
            optional(parse_error_binding()),
            delimited(
                as_unit(parse_open_brace()),
                error_handling_statements(),
                as_unit(parse_close_brace()),
            ),
        ),
        |(
            _,
            error_binding,
            ErrorHandlingStatements {
                statements,
                control,
            },
        )| ast::Statement::WithError {
            error_handler_block: ast::ErrorHandlerBlock {
                error_binding,
                error_handler_statements: statements,
                control,
            },
            statement: Box::new(ast::Statement::Expression(ast::Expression::Literal(
                ast::Literal::Null,
            ))),
        },
    )
}

fn on_fail_return_from_expr(expr: ast::Expression) -> Option<ast::OnFailControl> {
    if let ast::Expression::FunctionCall {
        function,
        arguments,
    } = expr
    {
        if let Some(expr) = arguments.first() {
            if function == "Ok" {
                return Some(ast::OnFailControl::Return(ast::OnFailReturn::Ok(
                    expr.clone(),
                )));
            } else if function == "Err" {
                return Some(ast::OnFailControl::Return(ast::OnFailReturn::Err(
                    expr.clone(),
                )));
            }
        }
    }
    None
}

fn error_handling_statements() -> impl Parser<Token, ErrorHandlingStatements> {
    with_context(
        map(
            tuple2(parse_statement_list(), optional(parse_on_fail_control())),
            |(statements, control)| {
                if control.is_none() {
                    if let Some(ast::Statement::Return(expr)) = statements.last().cloned() {
                        if let Some(on_fail_return) = on_fail_return_from_expr(expr) {
                            return ErrorHandlingStatements {
                                statements: statements[..statements.len() - 1].to_vec(),
                                control: Some(on_fail_return),
                            };
                        }
                    }
                }
                ErrorHandlingStatements {
                    statements,
                    control,
                }
            },
        ),
        "error handling statements",
    )
}

struct ErrorHandlingStatements {
    statements: ast::Statements,
    control: Option<ast::OnFailControl>,
}

fn parse_onfail_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::OnFail)), "onFail keyword")
}

fn parse_error_binding() -> impl Parser<Token, String> {
    with_context(
        delimited(
            as_unit(parse_open_paren()),
            parse_identifier(),
            as_unit(parse_close_paren()),
        ),
        "error binding",
    )
}

fn parse_on_fail_control() -> impl Parser<Token, ast::OnFailControl> {
    with_context(
        choice(vec![
            Box::new(parse_return_control()),
            Box::new(parse_rethrow_control()),
        ]),
        "onFail control",
    )
}

fn parse_return_control() -> impl Parser<Token, ast::OnFailControl> {
    with_context(
        map(
            tuple2(
                as_unit(parse_return_keyword()),
                choice(vec![
                    Box::new(parse_ok_return()),
                    Box::new(parse_err_return()),
                ]),
            ),
            |(_, control)| control,
        ),
        "return control",
    )
}

fn parse_ok_return() -> impl Parser<Token, ast::OnFailControl> {
    with_context(
        map(
            tuple2(
                as_unit(parse_ok_ident()),
                delimited(
                    as_unit(parse_open_paren()),
                    parse_expression(),
                    as_unit(parse_close_paren()),
                ),
            ),
            |(_, expr)| ast::OnFailControl::Return(ast::OnFailReturn::Ok(expr)),
        ),
        "ok return",
    )
}

fn parse_err_return() -> impl Parser<Token, ast::OnFailControl> {
    with_context(
        map(
            tuple2(
                as_unit(parse_err_ident()),
                delimited(
                    as_unit(parse_open_paren()),
                    parse_expression(),
                    as_unit(parse_close_paren()),
                ),
            ),
            |(_, expr)| ast::OnFailControl::Return(ast::OnFailReturn::Err(expr)),
        ),
        "err return",
    )
}

fn parse_rethrow_control() -> impl Parser<Token, ast::OnFailControl> {
    with_context(
        map(as_unit(parse_resthrow_keyword()), |_| {
            ast::OnFailControl::Rethrow
        }),
        "rethrow control",
    )
}

fn parse_resthrow_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::ReThrow)), "rethrow keyword")
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
                    acc.extend(targets.to_vec());
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
            ast::Statement::Return,
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
                event_type: ast::EventType::Custom(event_type),
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
            |block| block,
        ),
        "else statement",
    )
}

fn parse_else_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Else)), "else keyword")
}

pub fn parse_statements() -> impl Parser<Token, ast::Statements> {
    with_context(
        delimited(
            as_unit(parse_open_brace()),
            parse_statement_list(),
            as_unit(parse_close_brace()),
        ),
        "block statement",
    )
}

fn parse_statement_list() -> impl Parser<Token, ast::Statements> {
    with_context(many(parse_statement()), "statement list")
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

#[cfg(test)]
mod tests {
    use crate::tokenizer::{literal::*, symbol::*};

    use super::*;

    #[test]
    fn test_parse_error_handler() {
        let input = vec![
            Token::Keyword(Keyword::OnFail),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("err".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Keyword(Keyword::Return),
            Token::Identifier("Ok".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(1)),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Delimiter(Delimiter::CloseBrace),
        ];
        let (rest, handler) = parse_error_handler().parse(&input, 0).unwrap();
        assert_eq!(rest, 11);
        assert_eq!(
            handler,
            ast::Statement::WithError {
                statement: Box::new(ast::Statement::Expression(ast::Expression::Literal(
                    ast::Literal::Null
                ))),
                error_handler_block: ast::ErrorHandlerBlock {
                    error_binding: Some("err".to_string()),
                    error_handler_statements: vec![],
                    control: Some(ast::OnFailControl::Return(ast::OnFailReturn::Ok(
                        ast::Expression::Literal(ast::Literal::Integer(1))
                    ))),
                },
            }
        );
    }

    #[test]
    fn test_parse_error_binding() {
        let input = vec![
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("err".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (rest, binding) = parse_error_binding().parse(&input, 0).unwrap();
        assert_eq!(rest, 3);
        assert_eq!(binding, "err".to_string());
    }

    #[test]
    fn test_parse_on_fail_control() {
        let input = vec![
            Token::Keyword(Keyword::Return),
            Token::Identifier("Ok".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::Integer(1)),
            Token::Delimiter(Delimiter::CloseParen),
        ];
        let (rest, control) = parse_on_fail_control().parse(&input, 0).unwrap();
        assert_eq!(rest, 5);
        assert_eq!(
            control,
            ast::OnFailControl::Return(ast::OnFailReturn::Ok(ast::Expression::Literal(
                ast::Literal::Integer(1)
            )))
        );
    }

    #[test]
    fn test_parse_on_fail_rethrow_control() {
        let input = vec![Token::Keyword(Keyword::ReThrow)];
        let (rest, control) = parse_on_fail_control().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(control, ast::OnFailControl::Rethrow);
    }

    #[test]
    fn test_parse_expression_statement() {
        let input = vec![Token::Literal(Literal::Integer(42))];
        let expected =
            ast::Statement::Expression(ast::Expression::Literal(ast::Literal::Integer(42)));
        assert_eq!(
            parse_expression_statement().parse(&input, 0),
            Ok((1, expected))
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
            event_type: ast::EventType::Custom("test-event".to_string()),
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
            event_type: ast::EventType::Custom("test-event".to_string()),
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
}

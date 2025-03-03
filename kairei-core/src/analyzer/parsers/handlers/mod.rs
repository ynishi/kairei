pub mod answer;
pub mod observe;
pub mod react;

use super::super::{core::*, prelude::*};
use super::{statement::*, *};
use crate::analyzer::parsers::types::parse_type_info;
use crate::{ast, tokenizer::token::Token};

/// Core handler parsing functionality shared between react and world contexts
pub fn parse_handler_def() -> impl Parser<Token, ast::HandlerDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_on_keyword()),
                parse_identifier(),
                parse_parameters(),
                parse_statements(),
            ),
            |(_, event_name, parameters, block)| ast::HandlerDef {
                event_name,
                parameters,
                block: ast::HandlerBlock { statements: block },
            },
        ),
        "handler",
    )
}

pub fn parse_parameters() -> impl Parser<Token, Vec<ast::Parameter>> {
    with_context(
        map(
            delimited(
                as_unit(parse_open_paren()),
                separated_list(parse_parameter(), as_unit(parse_comma())),
                as_unit(parse_close_paren()),
            ),
            |parameters| parameters,
        ),
        "parameters",
    )
}

pub fn parse_parameter() -> impl Parser<Token, ast::Parameter> {
    with_context(
        choice(vec![
            Box::new(map(
                tuple3(
                    parse_identifier(),
                    as_unit(parse_colon()),
                    parse_type_info(),
                ),
                |(name, _, type_info)| ast::Parameter { name, type_info },
            )),
            Box::new(map(parse_identifier(), |name| ast::Parameter {
                name,
                type_info: ast::TypeInfo::any(),
            })),
        ]),
        "parameter",
    )
}

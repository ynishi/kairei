pub mod answer;
pub mod observe;
pub mod react;

use super::super::{core::*, prelude::*};
use crate::{
    ast,
    tokenizer::{keyword::Keyword, token::Token},
};
use super::{statement::*, types::*, *};

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

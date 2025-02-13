use super::{
    super::super::{core::*, prelude::*},
    observe::*,
    parse_handler_def,
};
use crate::ast;
use crate::{
    analyzer::parsers::{statement::*, types::*, *},
    tokenizer::{keyword::Keyword, token::Token},
};

pub fn parse_react() -> impl Parser<Token, ast::ReactDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_react_keyword()),
                as_unit(parse_open_brace()),
                many(parse_event_handler()), // ObserveDefと同じEventHandlerを使用
                as_unit(parse_close_brace()),
            ),
            |(_, _, handlers, _)| ast::ReactDef { handlers },
        ),
        "react",
    )
}

fn parse_react_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::React)), "react keyword")
}

pub fn parse_handler() -> impl Parser<Token, ast::HandlerDef> {
    parse_handler_def()
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

fn parse_parameter() -> impl Parser<Token, ast::Parameter> {
    with_context(
        map(
            tuple3(
                parse_identifier(),
                as_unit(parse_colon()),
                parse_type_info(),
            ),
            |(name, _, type_info)| ast::Parameter { name, type_info },
        ),
        "parameter",
    )
}

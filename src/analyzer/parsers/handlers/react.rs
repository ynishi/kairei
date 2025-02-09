use crate::{analyzer::parsers::{statement::*, types::*, *}, tokenizer::{keyword::Keyword,  token::Token}};
use super::{super::super::{core::*, prelude::*}, observe::*};
use crate::ast;


pub fn parse_react() -> impl Parser<Token, ast::ReactDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_react_keyword()),
                as_unit(parse_open_brace()),
                many(parse_event_handler()),  // ObserveDefと同じEventHandlerを使用
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

pub fn parse_handlers() -> impl Parser<Token, ast::HandlersDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_handlers_keyword()),
                as_unit(parse_open_brace()),
                many(parse_handler()),
                as_unit(parse_close_brace()),
            ),
            |(_, _, handlers, _)| ast::HandlersDef { handlers },
        ),
        "handlers",
    )
}

pub fn parse_handlers_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Handlers)), "handlers keyword")
}

pub fn parse_handler() -> impl Parser<Token, ast::HandlerDef> {
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

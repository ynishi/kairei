use super::{
    super::super::{core::*, prelude::*},
    observe::*,
};
use crate::ast;
use crate::{
    analyzer::parsers::*,
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

use crate::{analyzer::parsers::{expression::*, statement::*, *}, tokenizer::{keyword::Keyword, literal::{Literal, StringPart}, symbol::Delimiter, token::Token}};
use super::{super::super::{core::*, prelude::*}, react::*};
use crate::ast;

pub fn parse_observe() -> impl Parser<Token, ast::ObserveDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_observe_keyword()),
                as_unit(parse_open_brace()),
                many(parse_event_handler()),
                as_unit(parse_close_brace()),
            ),
            |(_, _, handlers, _)| ast::ObserveDef { handlers },
        ),
        "observe",
    )
}

pub fn parse_event_handler() -> impl Parser<Token, ast::EventHandler> {
    with_context(
        map(
            tuple4(
                as_unit(parse_on_keyword()),
                parse_event_type(),
                parse_parameters(),
                parse_statements(),
            ),
            |(_, event_type, parameters, block)| ast::EventHandler {
                event_type,
                parameters,
                block: ast::HandlerBlock { statements: block },
            },
        ),
        "event handler",
    )
}


fn parse_event_type() -> impl Parser<Token, ast::EventType> {
    with_context(
        choice(vec![
            Box::new(map(parse_tick_identify(), |_| ast::EventType::Tick)),
            Box::new(map(
                tuple4(
                    parse_state_updated_keyword(),
                    parse_dot(),
                    parse_identifier(),
                    preceded(as_unit(parse_dot()), parse_identifier()),
                ),
                |(_, _, agent_name, state_name)| ast::EventType::StateUpdated {
                    agent_name,
                    state_name,
                },
            )),
            Box::new(map(parse_identifier(), |name| {
                ast::EventType::Custom(name)
            })),
        ]),
        "event type",
    )
}

fn parse_observe_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Observe)), "observe keyword")
}

fn parse_tick_identify() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("Tick".to_string())), "tick keyword")
}

fn parse_state_updated_keyword() -> impl Parser<Token, Token> {
    with_context(
        equal(Token::Identifier("StateUpdated".to_string())),
        "state updated keyword",
    )
}

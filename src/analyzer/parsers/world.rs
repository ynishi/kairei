use uuid::Uuid;

use super::{
    super::{core::*, prelude::*},
    agent::parse_agent_def,
    handlers::react::*,
    types::parse_type_info,
    *,
};
use crate::ast;
use crate::{
    tokenizer::{keyword::Keyword, token::Token},
    PolicyId,
};
use std::collections::HashMap;

pub fn parse_root() -> impl Parser<Token, ast::Root> {
    with_context(
        map(
            tuple2(optional(parse_world()), many(parse_agent_def())),
            |(world_def, micro_agent_defs)| ast::Root::new(world_def, micro_agent_defs),
        ),
        "root",
    )
}

pub fn parse_world() -> impl Parser<Token, ast::WorldDef> {
    with_context(
        map(
            tuple5(
                as_unit(parse_world_keyword()),
                parse_identifier(),
                parse_open_brace(),
                many(choice(vec![
                    Box::new(map(parse_policy(), WorldDefItem::Policy)),
                    Box::new(map(parse_config(), WorldDefItem::Config)),
                    Box::new(map(parse_events(), WorldDefItem::Events)),
                    Box::new(map(parse_handlers(), WorldDefItem::Handlers)),
                ])),
                parse_close_brace(),
            ),
            |(_, name, _, items, _)| {
                let mut policies = vec![];
                let mut config = None;
                let mut events = None;
                let mut handlers = None;

                for item in items {
                    match item {
                        WorldDefItem::Policy(policy) => policies.push(policy),
                        WorldDefItem::Config(config_def) => config = Some(config_def),
                        WorldDefItem::Events(events_def) => events = Some(events_def),
                        WorldDefItem::Handlers(handlers_def) => handlers = Some(handlers_def),
                    }
                }

                ast::WorldDef {
                    name,
                    policies,
                    config,
                    events: events.unwrap_or_default(),
                    handlers: handlers.unwrap_or_default(),
                }
            },
        ),
        "world",
    )
}

#[derive(Debug, Clone, PartialEq)]
enum WorldDefItem {
    Policy(ast::Policy),
    Config(ast::ConfigDef),
    Events(ast::EventsDef),
    Handlers(ast::HandlersDef),
}

fn parse_world_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::World)), "world keyword")
}
pub fn parse_policy() -> impl Parser<Token, ast::Policy> {
    with_context(
        map(
            preceded(as_unit(parse_policy_keyword()), parse_literal()),
            |text| ast::Policy {
                text: text.to_string(),
                scope: ast::PolicyScope::World(Default::default()),
                internal_id: PolicyId(Uuid::new_v4().to_string()),
            },
        ),
        "policy",
    )
}

fn parse_policy_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Policy)), "policy keyword")
}

fn parse_policy_item() -> impl Parser<Token, (String, ast::Literal)> {
    with_context(
        map(
            tuple3(parse_identifier(), as_unit(parse_colon()), parse_literal()),
            |(name, _, value)| (name, value),
        ),
        "policy item",
    )
}

pub fn parse_config() -> impl Parser<Token, ast::ConfigDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_config_keyword()),
                as_unit(parse_open_brace()),
                many(parse_config_item()),
                as_unit(parse_close_brace()),
            ),
            |(_, _, items, _)| {
                let items_map = items.into_iter().collect::<HashMap<_, _>>();
                ast::ConfigDef::from(items_map)
            },
        ),
        "config",
    )
}

fn parse_config_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Config)), "config keyword")
}

pub fn parse_config_item() -> impl Parser<Token, (String, ast::Literal)> {
    with_context(
        map(
            tuple3(parse_identifier(), as_unit(parse_colon()), parse_literal()),
            |(name, _, value)| (name, value),
        ),
        "config item",
    )
}

pub fn parse_events() -> impl Parser<Token, ast::EventsDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_events_keyword()),
                as_unit(parse_open_brace()),
                many(parse_event()),
                as_unit(parse_close_brace()),
            ),
            |(_, _, events, _)| ast::EventsDef { events },
        ),
        "events",
    )
}

fn parse_events_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Events)), "events keyword")
}

fn parse_event() -> impl Parser<Token, ast::CustomEventDef> {
    with_context(
        map(
            tuple2(parse_identifier(), parse_parameters()),
            |(name, parameters)| ast::CustomEventDef { name, parameters },
        ),
        "event",
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

fn parse_handlers() -> impl Parser<Token, ast::HandlersDef> {
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

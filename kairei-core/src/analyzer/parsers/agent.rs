use super::{
    super::{core::*, prelude::*},
    expression::parse_expression,
    handlers::{answer::*, observe::*, react::*},
    statement::*,
    types::parse_type_info,
    world::parse_policy,
    *,
};
use crate::ast;
use crate::tokenizer::{keyword::Keyword, token::Token};

pub fn parse_agent_def() -> impl Parser<Token, ast::MicroAgentDef> {
    with_context(
        map(
            tuple5(
                as_unit(parse_micro_agent_keyword()),
                parse_identifier(),
                parse_open_brace(),
                many(choice(vec![
                    Box::new(map(parse_policy(), AgentDefItem::Policy)),
                    Box::new(map(parse_lifecycle(), AgentDefItem::Lifecycle)),
                    Box::new(map(parse_state(), AgentDefItem::State)),
                    Box::new(map(parse_observe(), AgentDefItem::Observe)),
                    Box::new(map(parse_answer(), AgentDefItem::Answer)),
                    Box::new(map(parse_react(), AgentDefItem::React)),
                ])),
                parse_close_brace(),
            ),
            |(_, name, _, items, _)| {
                let mut agent = ast::MicroAgentDef {
                    name,
                    ..Default::default()
                };

                for item in items {
                    match item {
                        AgentDefItem::Policy(policy) => agent.policies.push(policy),
                        AgentDefItem::Lifecycle(lifecycle) => agent.lifecycle = Some(lifecycle),
                        AgentDefItem::State(state) => agent.state = Some(state),
                        AgentDefItem::Observe(observe) => agent.observe = Some(observe),
                        AgentDefItem::Answer(answer) => agent.answer = Some(answer),
                        AgentDefItem::React(react) => agent.react = Some(react),
                    }
                }

                agent
            },
        ),
        "agent definition",
    )
}

#[derive(Debug, Clone, PartialEq)]
enum AgentDefItem {
    Policy(ast::Policy),
    Lifecycle(ast::LifecycleDef),
    State(ast::StateDef),
    Observe(ast::ObserveDef),
    Answer(ast::AnswerDef),
    React(ast::ReactDef),
}

pub fn parse_lifecycle() -> impl Parser<Token, ast::LifecycleDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_lifecycle_keyword()),
                parse_open_brace(),
                many(choice(vec![
                    Box::new(map(parse_init_handler(), |handler| ("init", handler))),
                    Box::new(map(parse_destroy_handler(), |handler| ("destroy", handler))),
                ])),
                parse_close_brace(),
            ),
            |(_, _, handlers, _)| {
                let mut on_init = None;
                let mut on_destroy = None;

                for (handler_type, block) in handlers {
                    match handler_type {
                        "init" => on_init = Some(block),
                        "destroy" => on_destroy = Some(block),
                        _ => unreachable!(),
                    }
                }

                ast::LifecycleDef {
                    on_init,
                    on_destroy,
                }
            },
        ),
        "lifecycle",
    )
}

pub fn parse_init_handler() -> impl Parser<Token, ast::HandlerBlock> {
    with_context(
        map(
            preceded(as_unit(parse_init_keyword()), parse_statements()),
            |statements| ast::HandlerBlock { statements },
        ),
        "init handler",
    )
}

pub fn parse_destroy_handler() -> impl Parser<Token, ast::HandlerBlock> {
    with_context(
        map(
            preceded(as_unit(parse_destroy_keyword()), parse_statements()),
            |statements| ast::HandlerBlock { statements },
        ),
        "destroy handler",
    )
}

fn parse_lifecycle_keyword() -> impl Parser<Token, Token> {
    with_context(
        equal(Token::Keyword(Keyword::Lifecycle)),
        "lifecycle keyword",
    )
}

fn parse_init_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::OnInit)), "init keyword")
}

fn parse_destroy_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::OnDestroy)), "destroy keyword")
}

pub fn parse_state() -> impl Parser<Token, ast::StateDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_state_keyword()),
                as_unit(parse_open_brace()),
                many(parse_state_var()),
                as_unit(parse_close_brace()),
            ),
            |(_, _, vars, _)| {
                let variables = vars.into_iter().collect();
                ast::StateDef { variables }
            },
        ),
        "state",
    )
}

fn parse_state_var() -> impl Parser<Token, (String, ast::StateVarDef)> {
    with_context(
        map(
            tuple5(
                parse_identifier(),
                as_unit(parse_colon()),
                parse_type_info(),
                optional(preceded(as_unit(parse_equal()), parse_expression())),
                as_unit(parse_semicolon()),
            ),
            |(name, _, type_info, initial_value, _)| {
                (
                    name.clone(),
                    ast::StateVarDef {
                        name,
                        type_info,
                        initial_value,
                    },
                )
            },
        ),
        "state variable",
    )
}

fn parse_state_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::State)), "state keyword")
}

fn parse_micro_agent_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Micro)), "micro agent keyword")
}

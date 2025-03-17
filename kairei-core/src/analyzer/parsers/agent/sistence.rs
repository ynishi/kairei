use super::{
    super::{super::{core::*, prelude::*}},
    parse_lifecycle, parse_state, parse_policy,
};
use crate::ast;
use crate::tokenizer::{keyword::Keyword, token::Token};
use super::super::handlers::{answer::parse_answer, observe::parse_observe, react::parse_react};

/// Parse a Sistence agent definition
pub fn parse_sistence_agent_def() -> impl Parser<Token, ast::SistenceAgentDef> {
    with_context(
        map(
            tuple5(
                as_unit(parse_sistence_agent_keyword()),
                parse_identifier(),
                parse_open_brace(),
                many(choice(vec![
                    Box::new(map(parse_policy(), SistenceAgentDefItem::Policy)),
                    Box::new(map(parse_lifecycle(), SistenceAgentDefItem::Lifecycle)),
                    Box::new(map(parse_state(), SistenceAgentDefItem::State)),
                    Box::new(map(parse_observe(), SistenceAgentDefItem::Observe)),
                    Box::new(map(parse_answer(), SistenceAgentDefItem::Answer)),
                    Box::new(map(parse_react(), SistenceAgentDefItem::React)),
                    Box::new(map(parse_sistence_config(), SistenceAgentDefItem::SistenceConfig)),
                ])),
                parse_close_brace(),
            ),
            |(_, name, _, items, _)| {
                let mut agent = ast::SistenceAgentDef {
                    name,
                    ..Default::default()
                };

                for item in items {
                    match item {
                        SistenceAgentDefItem::Policy(policy) => agent.policies.push(policy),
                        SistenceAgentDefItem::Lifecycle(lifecycle) => agent.lifecycle = Some(lifecycle),
                        SistenceAgentDefItem::State(state) => agent.state = Some(state),
                        SistenceAgentDefItem::Observe(observe) => agent.observe = Some(observe),
                        SistenceAgentDefItem::Answer(answer) => agent.answer = Some(answer),
                        SistenceAgentDefItem::React(react) => agent.react = Some(react),
                        SistenceAgentDefItem::SistenceConfig(config) => agent.sistence_config = Some(config),
                    }
                }

                agent
            },
        ),
        "sistence agent definition",
    )
}

#[derive(Debug, Clone, PartialEq)]
enum SistenceAgentDefItem {
    Policy(ast::Policy),
    Lifecycle(ast::LifecycleDef),
    State(ast::StateDef),
    Observe(ast::ObserveDef),
    Answer(ast::AnswerDef),
    React(ast::ReactDef),
    SistenceConfig(ast::SistenceConfig),
}

/// Parse the 'sistence' keyword
fn parse_sistence_agent_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Sistence)), "sistence agent keyword")
}

/// Parse Sistence configuration block
fn parse_sistence_config() -> impl Parser<Token, ast::SistenceConfig> {
    with_context(
        map(
            tuple4(
                as_unit(parse_sistence_config_keyword()),
                parse_open_brace(),
                many(parse_sistence_config_item()),
                parse_close_brace(),
            ),
            |(_, _, items, _)| {
                let mut config = ast::SistenceConfig::default();
                
                for (key, value) in items {
                    match (key.as_str(), value) {
                        ("level", ast::Literal::Float(f)) => config.level = f,
                        ("initiative_threshold", ast::Literal::Float(f)) => config.initiative_threshold = f,
                        ("domains", ast::Literal::List(domains)) => {
                            for domain in domains {
                                if let ast::Literal::String(domain_str) = domain {
                                    config.domains.push(domain_str);
                                }
                            }
                        }
                        (key, value) => {
                            config.parameters.insert(key.to_string(), value);
                        }
                    }
                }
                
                config
            },
        ),
        "sistence config",
    )
}

/// Parse a Sistence configuration item
fn parse_sistence_config_item() -> impl Parser<Token, (String, ast::Literal)> {
    with_context(
        map(
            tuple4(
                parse_identifier(),
                as_unit(parse_colon()),
                parse_literal(),
                as_unit(parse_comma()),
            ),
            |(key, _, value, _)| (key, value),
        ),
        "sistence config item",
    )
}

/// Parse the 'sistence_config' keyword
fn parse_sistence_config_keyword() -> impl Parser<Token, Token> {
    with_context(
        map(parse_identifier(), |id| {
            if id == "sistence_config" {
                Token::Identifier(id)
            } else {
                panic!("Expected 'sistence_config' identifier")
            }
        }),
        "sistence_config keyword",
    )
}

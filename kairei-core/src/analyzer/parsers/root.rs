use super::{
    super::{core::*, prelude::*},
    agent::{parse_agent_def, sistence::parse_sistence_agent_def},
    world::parse_world_def,
};
use crate::ast;

pub fn parse_root() -> impl Parser<crate::tokenizer::token::Token, ast::Root> {
    with_context(
        map(
            tuple2(
                optional(parse_world_def()),
                many(choice(vec![
                    Box::new(map(parse_agent_def(), |agent| RootItem::MicroAgent(agent))),
                    Box::new(map(parse_sistence_agent_def(), |agent| RootItem::SistenceAgent(agent))),
                ])),
            ),
            |(world_def, items)| {
                let mut root = ast::Root {
                    world_def,
                    micro_agent_defs: Vec::new(),
                    sistence_agent_defs: Vec::new(),
                };

                for item in items {
                    match item {
                        RootItem::MicroAgent(agent) => root.micro_agent_defs.push(agent),
                        RootItem::SistenceAgent(agent) => root.sistence_agent_defs.push(agent),
                    }
                }

                root
            },
        ),
        "root",
    )
}

#[derive(Debug, Clone, PartialEq)]
enum RootItem {
    MicroAgent(ast::MicroAgentDef),
    SistenceAgent(ast::SistenceAgentDef),
}

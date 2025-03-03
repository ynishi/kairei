use super::super::super::{core::*, prelude::*};
use crate::analyzer::parsers::handlers::parse_parameters;
use crate::ast;
use crate::{
    analyzer::parsers::{expression::*, statement::*, *},
    tokenizer::{keyword::Keyword, token::Token},
};

/// Observe Block Handler Implementation
///
/// The observe block enables agents to monitor and respond to events
/// in their environment, including state changes and system events.
///
/// # Features
/// - Event monitoring
/// - State change detection
/// - Full state access
/// - Automatic state change notifications
///
/// # Example
/// ```text
/// observe {
///     on StateUpdated.otherAgent.status {
///         // Handle state change
///     }
///
///     on CustomEvent(param: String) {
///         // Handle custom event
///     }
/// }
/// ```
///
/// # Built-in Events
/// - `Tick`: System heartbeat event
/// - `StateUpdated`: State change notifications
/// - Custom events defined in World
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

/// Event Handler Parser
///
/// Parses individual event handlers within an observe block.
/// Each handler defines how the agent responds to specific events.
///
/// # Handler Structure
/// - Event type (built-in or custom)
/// - Optional parameters with types
/// - Handler implementation block
///
/// # Example
/// ```text
/// on Tick {
///     // Handle tick event
/// }
///
/// on CustomEvent(data: EventData) {
///     // Handle custom event with data
/// }
/// ```
pub fn parse_event_handler() -> impl Parser<Token, ast::EventHandler> {
    with_context(
        map(
            tuple4(
                as_unit(parse_on_keyword()),
                parse_event_type(),
                optional(parse_parameters()),
                parse_statements(),
            ),
            |(_, event_type, parameters, block)| ast::EventHandler {
                event_type,
                parameters: parameters.unwrap_or_default(),
                block: ast::HandlerBlock { statements: block },
            },
        ),
        "event handler",
    )
}

/// Event Type Parser
///
/// Parses the type of event being handled. Supports:
/// - Tick events (system heartbeat)
/// - State update events (agent state changes)
/// - Custom events (defined in World)
///
/// # Examples
/// ```text
/// Tick
/// StateUpdated.agentName.stateName
/// CustomEvent
/// ```
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
            Box::new(map(parse_identifier(), ast::EventType::Custom)),
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

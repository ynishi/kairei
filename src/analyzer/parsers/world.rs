/// World DSL Parser Implementation
///
/// This module implements the parser for KAIREI's World DSL, which defines the environment
/// where MicroAgents operate. The World DSL provides a structured way to define:
/// - World configuration (tick intervals, agent limits, etc.)
/// - Custom events and their parameters
/// - Event handlers
/// - Global policies
/// - Global type definitions
///
/// # Type System
/// The World DSL supports global type definitions that can be used across all MicroAgents:
/// ```text
/// types {
///     type UserProfile {
///         id: String
///         name: String
///         age: Int
///     }
/// }
/// ```
///
/// # Built-in Types
/// - `String`: Text values
/// - `Int`: Integer numbers
/// - `Float`: Floating point numbers
/// - `Boolean`: True/false values
/// - `List<T>`: Lists of type T
/// - `Map<K, V>`: Key-value maps
/// - `Duration`: Time intervals
/// - `Date`: Calendar dates
/// - `DateTime`: Date and time values
/// - `Json`: JSON data
/// - `Result<T, E>`: Success/failure results
///
/// # Example
/// ```text
/// world TravelPlanningWorld {
///     config {
///         tick_interval: Duration = "1s"
///         max_agents: Int = 100
///         event_buffer_size: Int = 500
///     }
///
///     events {
///         UserRequestedItinerary(user_id: String)
///         TripStarted
///         UpdateTripping
///     }
///
///     handlers {
///         on TripStarted(delta_time: Float) {
///             emit UpdateTripping
///         }
///     }
/// }
/// ```
///
/// # Built-in Events
/// The World DSL provides several built-in events:
/// - `Tick(delta_time: Float)`: Triggered at configured intervals
/// - `AgentJoined(agent_id: String)`: When an agent joins the world
/// - `AgentLeft(agent_id: String)`: When an agent leaves the world
/// - `ErrorOccurred(message: String)`: When an error occurs in agent execution
///
/// # Event-Driven Synchronization
/// The World DSL implements event-driven synchronization for real-world integration:
/// - Tick events serve as external resource synchronization
/// - Enables unified timeline across agents without specialized mechanisms
/// - Focuses on real-world event processing rather than frame-based updates
/// - Allows agents to share a unified timeline for coordinated operations
///
/// Example:
/// ```text
/// world TravelPlanningWorld {
///     handlers {
///         on Tick(delta_time: Float) {
///             // Synchronize with external resources
///             emit UpdateExternalState
///         }
///     }
/// }
/// ```
///
use uuid::Uuid;

use super::{
    super::{core::*, prelude::*},
    agent::parse_agent_def,
    handlers::{parse_handler_def, parse_parameters},
    *,
};
use crate::ast;
use crate::{
    tokenizer::{keyword::Keyword, token::Token},
    PolicyId,
};
use std::collections::HashMap;

/// Parses the root of a KAIREI DSL file, which may contain a World definition
/// and multiple MicroAgent definitions.
///
/// # Returns
/// A parser that produces an `ast::Root` containing the World and MicroAgent definitions
pub fn parse_root() -> impl Parser<Token, ast::Root> {
    with_context(
        map(
            tuple2(optional(parse_world()), many(parse_agent_def())),
            |(world_def, micro_agent_defs)| ast::Root::new(world_def, micro_agent_defs),
        ),
        "root",
    )
}

/// Parses a World definition block, including its configuration, events, and handlers.
///
/// The World block is the top-level container that defines the environment where
/// MicroAgents operate. It can include:
/// - Configuration settings (tick interval, agent limits, etc.)
/// - Custom event definitions
/// - Event handlers
/// - Global policies
///
/// # Example
/// ```text
/// world GameWorld {
///     config {
///         tick_interval: Duration = "1s"
///     }
///     events {
///         GameStarted
///     }
/// }
/// ```
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

/// Represents the different types of items that can appear in a World definition.
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
/// Parses a policy declaration in a World definition.
///
/// Policies define high-level rules and constraints that apply to the entire World.
/// These are used to guide agent behavior and system operations.
///
/// # Example
/// ```text
/// policy "Ensure factual accuracy with multiple sources"
/// policy "Use recent information, prefer within 24 hours"
/// ```
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

/// Parses the configuration block of a World definition.
///
/// The config block allows setting various World parameters:
/// - `tick_interval`: Duration between Tick events (e.g., "1s")
/// - `max_agents`: Maximum number of agents allowed in the World
/// - `event_buffer_size`: Size of the event queue buffer
///
/// # Example
/// ```text
/// config {
///     tick_interval: Duration = "1s"
///     max_agents: Int = 100
///     event_buffer_size: Int = 500
/// }
/// ```
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

/// Parses the events block of a World definition.
///
/// The events block defines custom events that can be emitted and handled within the World.
/// Each event can have typed parameters. In addition to custom events, the system provides
/// built-in events:
/// - `Tick(delta_time: Float)`
/// - `AgentJoined(agent_id: String)`
/// - `AgentLeft(agent_id: String)`
/// - `ErrorOccurred(message: String)`
///
/// # Example
/// ```text
/// events {
///     PlayerJoined(player_id: String)
///     GameStarted
///     PlayerMoved(x: Float, y: Float)
/// }
/// ```
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

/// Parses the handlers block of a World definition.
///
/// The handlers block defines how the World responds to events. Handlers can process
/// both built-in events and custom events defined in the events block.
///
/// # Example
/// ```text
/// handlers {
///     on Tick(delta_time: Float) {
///         emit NextTick(delta_time)
///     }
///     
///     on PlayerJoined(player_id: String) {
///         // Handle player joining
///     }
/// }
/// ```
pub fn parse_handlers() -> impl Parser<Token, ast::HandlersDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_handlers_keyword()),
                as_unit(parse_open_brace()),
                many(parse_handler_def()),
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

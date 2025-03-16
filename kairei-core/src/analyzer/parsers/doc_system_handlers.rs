//! # System Handler Documentation
//!
//! This module provides documented versions of the system handler parsers.
//!
//! ## World vs Agent Handlers
//! System handlers operate at the World level, defining how the entire system responds to events,
//! as opposed to agent-level handlers which define how individual agents respond.
//!
//! World-level handlers can:
//! - Coordinate between multiple agents
//! - Manage global state and resources
//! - Implement system-wide policies
//! - Process events that affect the entire system
//!
//! System handlers are defined in the `handlers` block of a World definition and play a crucial role
//! in orchestrating the behavior of the entire multi-agent system.

use crate::analyzer::core::*;
use crate::analyzer::doc_parser::{DocBuilder, DocParserExt, ParserCategory, document};
use crate::analyzer::documentation_collector::DocumentationProvider;
use crate::analyzer::parsers::handlers::parse_handler_def;
use crate::analyzer::parsers::parse_identifier;
use crate::analyzer::parsers::world::{parse_events, parse_handlers};
use crate::analyzer::prelude::*;
use crate::ast;
use crate::tokenizer::token::Token;
use std::any::Any;

/// Returns a documented version of the parse_handlers function.
///
/// Parses the handlers block of a World definition, which defines how the World
/// responds to events at the system level.
pub fn documented_parse_handlers() -> impl DocParserExt<Token, ast::HandlersDef> {
    let parser = parse_handlers();

    let doc = DocBuilder::new("parse_handlers", ParserCategory::Handler)
        .description("System handlers define how the World responds to events at the system level, affecting all agents in the system. Unlike agent-level handlers (observe, react, answer), system handlers operate globally and can coordinate between multiple agents, manage global state, and implement system-wide policies. They are essential for centralized event processing and global state management.")
        .example(r#"handlers {
    on Tick(delta_time: Float) {
        emit NextTick(delta_time)
    }
    
    on UserJoined(user_id: String) {
        // Handle user joining
    }
}"#)
        .example(r#"handlers {
    on StateUpdated.agentName.stateName {
        // React to state changes from other agents
    }
}"#)
        .related_parser("parse_handler_def")
        .related_parser("parse_events")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the parse_handler_def function.
///
/// Parses individual handler definitions within a handlers block.
pub fn documented_parse_handler_def() -> impl DocParserExt<Token, ast::HandlerDef> {
    let parser = parse_handler_def();

    let doc = DocBuilder::new("parse_handler_def", ParserCategory::Handler)
        .description("Handler definitions specify how to respond to specific events at the World level. Each handler is triggered by an event and contains a block of statements to execute when the event occurs. World-level handlers can emit events that affect the entire system and all agents within it.")
        .example(r#"on Tick(delta_time: Float) {
    emit NextTick(delta_time)
}"#)
        .example(r#"on CustomEvent(param1: String, param2: Int) {
    // Handle custom event
    emit ResponseEvent(param1)
}"#)
        .related_parser("parse_handlers")
        .related_parser("parse_event")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the parse_events function.
///
/// Parses the events block of a World definition, which defines custom events
/// that can be emitted and handled within the World.
pub fn documented_parse_events() -> impl DocParserExt<Token, ast::EventsDef> {
    let parser = parse_events();

    let doc = DocBuilder::new("parse_events", ParserCategory::Handler)
        .description("The events block in a World definition declares custom events that can be emitted and handled within the system. These events form the communication backbone of the multi-agent system, allowing agents to interact with each other and with the World. Events can have typed parameters to carry data between components of the system.")
        .example(r#"events {
    UserJoined(user_id: String)
    MessageSent(from: String, to: String, content: String)
    SystemAlert
}"#)
        .example(r#"events {
    TripStarted
    LocationUpdated(latitude: Float, longitude: Float)
}"#)
        .related_parser("parse_event")
        .related_parser("parse_handlers")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the parse_event function.
///
/// Parses individual event definitions within an events block.
pub fn documented_parse_event() -> impl DocParserExt<Token, ast::CustomEventDef> {
    // Create a simplified version of parse_event since it's private in world.rs
    // We'll use a simpler approach that doesn't require reimplementing the full parser

    // Create a dummy parser that just returns an empty CustomEventDef
    // This is just for documentation purposes
    let parser = map(parse_identifier(), |name| ast::CustomEventDef {
        name,
        parameters: Vec::new(),
    });

    let doc = DocBuilder::new("parse_event", ParserCategory::Handler)
        .description("Event definitions specify custom events that can be emitted and handled within the World. Each event can have typed parameters that carry data. Events are the primary mechanism for communication between agents and the World, enabling a decoupled, event-driven architecture.")
        .example("UserJoined(user_id: String)")
        .example("MessageSent(from: String, to: String, content: String)")
        .example("SystemAlert")
        .related_parser("parse_events")
        .related_parser("parse_handler_def")
        .build();

    document(parser, doc)
}

/// Documentation provider for system handlers.
pub struct SystemHandlerDocProvider;

impl DocumentationProvider for SystemHandlerDocProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        vec![
            as_any_doc_parser(documented_parse_handlers()),
            as_any_doc_parser(documented_parse_handler_def()),
            as_any_doc_parser(documented_parse_events()),
            as_any_doc_parser(documented_parse_event()),
        ]
    }
}

/// Helper function to convert `DocParserExt<Token, T>` to `DocParserExt<Token, Box<dyn Any>>`
fn as_any_doc_parser<O: 'static>(
    parser: impl DocParserExt<Token, O> + 'static,
) -> Box<dyn DocParserExt<Token, Box<dyn Any>>> {
    // Create a wrapper struct that will handle the type conversion
    struct AnyWrapper<P, O: 'static> {
        parser: P,
        _phantom: std::marker::PhantomData<O>,
    }

    // Implement Parser for the wrapper
    impl<P, O: 'static> Parser<Token, Box<dyn Any>> for AnyWrapper<P, O>
    where
        P: Parser<Token, O>,
    {
        fn parse(&self, input: &[Token], pos: usize) -> ParseResult<Box<dyn Any>> {
            // Parse with the original parser
            match self.parser.parse(input, pos) {
                Ok((next_pos, result)) => {
                    // Convert the result to Box<dyn Any>
                    let boxed_result = Box::new(result) as Box<dyn Any>;
                    // Return the result with the correct types
                    Ok((next_pos, boxed_result))
                }
                Err(err) => Err(err),
            }
        }
    }

    // Implement DocParserExt for the wrapper
    impl<P, O: 'static> DocParserExt<Token, Box<dyn Any>> for AnyWrapper<P, O>
    where
        P: DocParserExt<Token, O>,
    {
        fn documentation(&self) -> &crate::analyzer::doc_parser::ParserDocumentation {
            self.parser.documentation()
        }
    }

    // Return the boxed wrapper
    Box::new(AnyWrapper {
        parser,
        _phantom: std::marker::PhantomData,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_attached() {
        // Test handlers parser
        let parser = documented_parse_handlers();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_handlers");
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);

        // Test events parser
        let parser = documented_parse_events();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_events");
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);
    }
}

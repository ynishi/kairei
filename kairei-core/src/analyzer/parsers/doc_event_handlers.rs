//! Documentation for event handler parsers.
//!
//! This module provides documented versions of the event handler parsers
//! from the `handlers` module.

use crate::analyzer::core::*;
use crate::analyzer::doc_parser::{DocBuilder, DocParserExt, ParserCategory, document};
use crate::analyzer::documentation_collector::DocumentationProvider;
use crate::analyzer::parsers::handlers::observe::parse_observe;
use crate::analyzer::parsers::handlers::react::parse_react;
use crate::ast;
use crate::tokenizer::token::Token;
use std::any::Any;

/// Returns a documented version of the observe handler parser
pub fn documented_parse_observe() -> impl DocParserExt<Token, ast::ObserveDef> {
    let parser = parse_observe();

    let doc = DocBuilder::new("parse_observe", ParserCategory::Handler)
        .description("Observe handlers monitor and respond to events in the system. They allow agents to detect and process events, update internal state, and trigger additional actions. Observe handlers are primarily used for passive monitoring and state updates.")
        .example("observe {
    on Tick {
        counter += 1
    }
    
    on UserMessage(text: String) {
        lastMessage = text
    }
    
    on StateUpdated.otherAgent.status {
        if otherAgent.status == \"ready\" {
            emit Ready()
        }
    }
}")
        .example("observe {
    on DataReceived(data) {
        processNewData(data)
    }
}")
        .related_parser("parse_react")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the react handler parser
pub fn documented_parse_react() -> impl DocParserExt<Token, ast::ReactDef> {
    let parser = parse_react();

    let doc = DocBuilder::new("parse_react", ParserCategory::Handler)
        .description("React handlers implement proactive behaviors in response to events. While similar to observe handlers, react handlers are intended for implementing agent behaviors that initiate actions rather than just monitoring. React handlers can modify agent state and emit new events.")
        .example("react {
    on UserJoined(user: User) {
        emit Welcome(recipient: user)
    }
    
    on HighTemperatureAlert {
        adjustThermostat()
        notifyUser()
    }
}")
        .example("react {
    on OrderPlaced(order) {
        processPayment(order)
        updateInventory(order.items)
        emit OrderConfirmation(order_id: order.id)
    }
}")
        .related_parser("parse_observe")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the event handler parser
pub fn documented_parse_event_handler() -> impl DocParserExt<Token, ast::EventHandler> {
    // We'll use the event handler parser from observe.rs
    let parser = crate::analyzer::parsers::handlers::observe::parse_event_handler();

    let doc = DocBuilder::new("parse_event_handler", ParserCategory::Handler)
        .description("Event handlers define how agents respond to specific events. Each handler specifies an event type, optional parameters, and a block of statements to execute when the event occurs.")
        .example("on Tick {
    counter += 1
}")
        .example("on UserMessage(text: String) {
    lastMessage = text
    processMessage(text)
}")
        .example("on StateUpdated.otherAgent.status {
    if otherAgent.status == \"ready\" {
        emit Ready()
    }
}")
        .related_parser("parse_observe")
        .related_parser("parse_react")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the event type parser
pub fn documented_parse_event_type() -> impl DocParserExt<Token, ast::EventType> {
    // We'll use the event type parser from observe.rs
    let parser = crate::analyzer::parsers::handlers::observe::parse_event_type();

    let doc = DocBuilder::new("parse_event_type", ParserCategory::Handler)
        .description("Event types specify what events an agent can respond to. KAIREI supports built-in events like Tick and StateUpdated, as well as custom events defined in the World.")
        .example("Tick")
        .example("StateUpdated.agentName.stateName")
        .example("CustomEvent")
        .related_parser("parse_event_handler")
        .build();

    document(parser, doc)
}

/// Documentation provider for event handler parsers
pub struct EventHandlerDocProvider;

impl DocumentationProvider for EventHandlerDocProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        vec![
            as_any_doc_parser(documented_parse_observe()),
            as_any_doc_parser(documented_parse_react()),
            as_any_doc_parser(documented_parse_event_handler()),
            as_any_doc_parser(documented_parse_event_type()),
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
        // Test observe handler parser
        let parser = documented_parse_observe();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_observe");
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);

        // Test react handler parser
        let parser = documented_parse_react();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_react");
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);
    }
}

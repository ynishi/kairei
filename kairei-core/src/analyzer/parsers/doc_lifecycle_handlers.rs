//! Documentation for lifecycle handler parsers.
//!
//! This module provides documented versions of the lifecycle handler parsers
//! from the `agent.rs` module.

use crate::analyzer::core::*;
use crate::analyzer::doc_parser::{DocBuilder, DocParserExt, ParserCategory, document};
use crate::analyzer::documentation_collector::DocumentationProvider;
use crate::analyzer::parsers::agent::{parse_destroy_handler, parse_init_handler, parse_lifecycle};
use crate::ast;
use crate::tokenizer::token::Token;
use std::any::Any;

/// Returns a documented version of the lifecycle block parser
pub fn documented_parse_lifecycle() -> impl DocParserExt<Token, ast::LifecycleDef> {
    let parser = parse_lifecycle();

    let doc = DocBuilder::new("parse_lifecycle", ParserCategory::Handler)
        .description("Lifecycle blocks define how agents initialize, maintain their state, and clean up resources when destroyed. They provide a structured way to manage the agent's existence from creation to termination.")
        .example("lifecycle {\n  on init {\n    counter = 0\n    memory = MemoryProvider.create()\n  }\n  on destroy {\n    memory.save()\n    resources.release()\n  }\n}")
        .example("lifecycle {\n  on init { loadConfiguration() }\n}")
        .related_parser("parse_init_handler")
        .related_parser("parse_destroy_handler")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the init handler parser
pub fn documented_parse_init_handler() -> impl DocParserExt<Token, ast::HandlerBlock> {
    let parser = parse_init_handler();

    let doc = DocBuilder::new("parse_init_handler", ParserCategory::Handler)
        .description("Init handlers are executed when an agent is first created. They are used to set up initial state, connect to external resources, load configurations, and perform any other setup actions needed before the agent begins responding to events.")
        .example("on init { state.counter = 0 }")
        .example("on init { db = DatabaseConnection.connect(config.dbUrl) }")
        .example("on init { loadUserPreferences(); setupEventListeners() }")
        .related_parser("parse_lifecycle")
        .related_parser("parse_destroy_handler")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the destroy handler parser
pub fn documented_parse_destroy_handler() -> impl DocParserExt<Token, ast::HandlerBlock> {
    let parser = parse_destroy_handler();

    let doc = DocBuilder::new("parse_destroy_handler", ParserCategory::Handler)
        .description("Destroy handlers are executed when an agent is being terminated. They provide an opportunity to clean up resources, save state, close connections, and perform any other actions needed before the agent is destroyed.")
        .example("on destroy { db.close() }")
        .example("on destroy { saveState(); notifyDependentSystems() }")
        .example("on destroy { emit AgentShutdown(reason: \"Terminating\") }")
        .related_parser("parse_lifecycle")
        .related_parser("parse_init_handler")
        .build();

    document(parser, doc)
}

/// Documentation provider for lifecycle handler parsers
pub struct LifecycleHandlerDocProvider;

impl DocumentationProvider for LifecycleHandlerDocProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        vec![
            as_any_doc_parser(documented_parse_lifecycle()),
            as_any_doc_parser(documented_parse_init_handler()),
            as_any_doc_parser(documented_parse_destroy_handler()),
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
        // Test lifecycle parser
        let parser = documented_parse_lifecycle();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_lifecycle");
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);

        // Test init handler parser
        let parser = documented_parse_init_handler();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_init_handler");
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);

        // Test destroy handler parser
        let parser = documented_parse_destroy_handler();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_destroy_handler");
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);
    }
}

//! Documentation for answer handler parsers.
//!
//! This module provides documented versions of the answer handler parsers
//! from the `handlers/answer.rs` module.

use crate::analyzer::core::*;
use crate::analyzer::doc_parser::{DocBuilder, DocParserExt, ParserCategory, document};
use crate::analyzer::documentation_collector::DocumentationProvider;
use crate::analyzer::parsers::handlers::answer::{
    parse_answer, parse_constraints, parse_request_handler, parse_request_type,
};
use crate::ast;
use crate::tokenizer::token::Token;
use std::any::Any;

/// Returns a documented version of the answer handler parser
pub fn documented_parse_answer() -> impl DocParserExt<Token, ast::AnswerDef> {
    let parser = parse_answer();

    let doc = DocBuilder::new("parse_answer", ParserCategory::Handler)
        .description("Answer handlers respond to explicit requests with type-safe responses. They provide a contract-based interaction mechanism allowing agents to respond to questions or perform requested actions with well-defined outputs. Answer handlers have read-only access to agent state and must return Result types.")
        .example("answer {\n  on request GetUserProfile(id: String) -> Result<Profile, Error> {\n    return userDatabase.getProfile(id)\n  }\n}")
        .example("answer {\n  on request CalculateTotal(items: List<Item>) -> Result<Number, Error> {\n    return Ok(items.map(i => i.price).sum())\n  }\n}")
        .example("answer {\n  on request SearchQuery(q: String) -> Result<List<Document>, Error> with { strictness: 0.8 } {\n    return searchIndex.query(q)\n  }\n}")
        .related_parser("parse_request_handler")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the request handler parser
pub fn documented_parse_request_handler() -> impl DocParserExt<Token, ast::RequestHandler> {
    let parser = parse_request_handler();

    let doc = DocBuilder::new("parse_request_handler", ParserCategory::Handler)
        .description("Request handlers define how an agent responds to specific request types. Each handler specifies the request type, parameters, return type (must be a Result type), and optional quality constraints. Request handlers enforce type safety and provide a clear contract for agent interactions.")
        .example("on request GetData(id: String) -> Result<Data, Error> {\n  return dataStore.fetch(id)\n}")
        .example("on query.UserInfo(userId: String) -> Result<UserProfile, Error> {\n  return userDatabase.getProfile(userId)\n}")
        .example("on action.UpdateProfile(profile: Profile) -> Result<Boolean, Error> {\n  return Ok(true)\n}")
        .related_parser("parse_answer")
        .related_parser("parse_request_type")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the request type parser
pub fn documented_parse_request_type() -> impl DocParserExt<Token, ast::RequestType> {
    let parser = parse_request_type();

    let doc = DocBuilder::new("parse_request_type", ParserCategory::Handler)
        .description("Request types define the category and purpose of a request. KAIREI supports three types of requests: query (for data retrieval), action (for operations that may change state), and custom request types. The request type helps determine how the request is processed and what permissions are granted.")
        .example("query.GetUserData")
        .example("action.UpdateProfile")
        .example("request CustomOperation")
        .related_parser("parse_request_handler")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the constraints parser
pub fn documented_parse_constraints() -> impl DocParserExt<Token, ast::Constraints> {
    let parser = parse_constraints();

    let doc = DocBuilder::new("parse_constraints", ParserCategory::Handler)
        .description("Quality constraints allow fine-tuning how requests are processed. Strictness controls the balance between accuracy and creativity (higher values prioritize accuracy). Stability ensures consistent responses across multiple calls (higher values produce more deterministic results). Latency defines response time requirements in milliseconds (lower values prioritize speed over thoroughness).")
        .example("with {\n  strictness: 0.9,  // 90% accuracy requirement\n  stability: 0.8,   // 80% consistency requirement\n  latency: 1000     // 1 second response time limit\n}")
        .example("with { strictness: 0.7 }")
        .example("with { latency: 500, stability: 0.95 }")
        .related_parser("parse_request_handler")
        .build();

    document(parser, doc)
}

/// Documentation provider for answer handler parsers
pub struct AnswerHandlerDocProvider;

impl DocumentationProvider for AnswerHandlerDocProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        vec![
            as_any_doc_parser(documented_parse_answer()),
            as_any_doc_parser(documented_parse_request_handler()),
            as_any_doc_parser(documented_parse_request_type()),
            as_any_doc_parser(documented_parse_constraints()),
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
                    // Return the result with the correct types - ParseResult is (usize, O)
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
        // Test answer parser
        let parser = documented_parse_answer();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_answer");
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);

        // Test request handler parser
        let parser = documented_parse_request_handler();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_request_handler");
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);
    }
}

//! Documentation for expression parsers.
//!
//! This module provides documented versions of the expression parsers
//! from the `expression.rs` module.

use crate::analyzer::core::*;
use crate::analyzer::doc_parser::{DocBuilder, DocParserExt, ParserCategory, document};
use crate::analyzer::documentation_collector::DocumentationProvider;
use crate::analyzer::parsers::expression::{
    parse_await_multiple, parse_await_single, parse_binary_expression, parse_err, parse_expression,
    parse_think_multiple, parse_think_single,
};
use crate::ast;
use crate::tokenizer::token::Token;
use std::any::Any;

/// Returns a documented version of the main expression parser
pub fn documented_parse_expression() -> impl DocParserExt<Token, ast::Expression> {
    let parser = parse_expression();

    let doc = DocBuilder::new("parse_expression", ParserCategory::Expression)
        .description("Expressions in KAIREI represent computations that produce values. They can be simple values like literals or variables, operations between values, function calls, or specialized expressions like 'think'.")
        .example("x + 5")
        .example("foo(bar)")
        .example("\"Hello, \" + name")
        .example("42")
        .example("a && b || c")
        .example("think(\"What should I do?\")")
        .related_parser("parse_binary_expression")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the binary expression parser
pub fn documented_parse_binary_expression() -> impl DocParserExt<Token, ast::Expression> {
    let parser = parse_binary_expression();

    let doc = DocBuilder::new("parse_binary_expression", ParserCategory::Expression)
        .description("Binary expressions combine two values with an operator. KAIREI supports arithmetic (+, -, *, /), logical (&&, ||), and comparison (==, !=, >, <, >=, <=) operators with proper precedence and associativity.")
        .example("a + b")
        .example("x && y || z")
        .example("foo * bar + baz")
        .related_parser("parse_expression")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the think expression parser (virtual)
pub fn documented_parse_think() -> impl DocParserExt<Token, ast::Expression> {
    // For think parser, we'll use think_single as it's public
    let parser = parse_think_single();

    let doc = DocBuilder::new("parse_think", ParserCategory::Expression)
        .description("The 'think' expression invokes an LLM to generate content. You can use think with a single prompt string, with multiple positional arguments, with named arguments, or with additional configuration using the 'with' clause.")
        .example("think(\"What is the capital of France?\")")
        .example("think(\"Summarize this text\", text)")
        .example("think(prompt: \"Generate ideas\", context: data)")
        .example("think(\"Answer this question\") with { model: \"gpt-4\" }")
        .related_parser("parse_think_single")
        .related_parser("parse_think_multiple")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the single-argument think expression parser
pub fn documented_parse_think_single() -> impl DocParserExt<Token, ast::Expression> {
    let parser = parse_think_single();

    let doc = DocBuilder::new("parse_think_single", ParserCategory::Expression)
        .description("A simple form of the 'think' expression that takes a single string prompt or a mix of positional and named arguments. This is the most common way to use think expressions.")
        .example("think(\"What is the capital of France?\")")
        .example("think(\"Summarize this text\", text)")
        .example("think(prompt: \"Generate ideas\", context: data)")
        .related_parser("parse_think")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the multiple-argument think expression parser
pub fn documented_parse_think_multiple() -> impl DocParserExt<Token, ast::Expression> {
    let parser = parse_think_multiple();

    let doc = DocBuilder::new("parse_think_multiple", ParserCategory::Expression)
        .description("An advanced form of the 'think' expression that uses exclusively named arguments. This is useful for complex prompts that need to provide structured information to the LLM.")
        .example("think(prompt: \"Generate ideas\", context: data)")
        .example("think(system: \"You are a helpful assistant\", user: question)")
        .example("think(query: \"Find documents about\", topic: search_topic)")
        .related_parser("parse_think")
        .related_parser("parse_think_single")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the Err expression parser
pub fn documented_parse_err() -> impl DocParserExt<Token, ast::Expression> {
    let parser = parse_err();

    let doc = DocBuilder::new("parse_err", ParserCategory::Expression)
        .description("The 'Err' expression creates an error result for error handling. It can contain a string message, an error code, or a formatted error object.")
        .example("Err(\"Something went wrong\")")
        .example("Err(error_code)")
        .example("Err(format_error(code, message))")
        .related_parser("parse_expression")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the await expression parser (virtual)
pub fn documented_parse_await() -> impl DocParserExt<Token, ast::Expression> {
    // For await parser, we'll use await_single as it's public
    let parser = parse_await_single();

    let doc = DocBuilder::new("parse_await", ParserCategory::Expression)
        .description("The 'await' expression pauses execution until asynchronous operations complete. You can await a single value or multiple values in parallel.")
        .example("await request_result")
        .example("await (request1, request2, request3)")
        .example("await async_function()")
        .related_parser("parse_await_single")
        .related_parser("parse_await_multiple")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the single await expression parser
pub fn documented_parse_await_single() -> impl DocParserExt<Token, ast::Expression> {
    let parser = parse_await_single();

    let doc = DocBuilder::new("parse_await_single", ParserCategory::Expression)
        .description("The simplest form of 'await' that waits for a single asynchronous value to resolve before proceeding.")
        .example("await request_result")
        .example("await get_user_data()")
        .example("await think(\"What should I do?\")")
        .related_parser("parse_await")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the multiple await expression parser
pub fn documented_parse_await_multiple() -> impl DocParserExt<Token, ast::Expression> {
    let parser = parse_await_multiple();

    let doc = DocBuilder::new("parse_await_multiple", ParserCategory::Expression)
        .description("An advanced form of 'await' that waits for multiple asynchronous operations to complete in parallel, making concurrent execution more efficient.")
        .example("await(request1, request2)")
        .example("await(get_weather(), get_news(), get_schedule())")
        .example("await(think(\"Question 1\"), think(\"Question 2\"))")
        .related_parser("parse_await")
        .related_parser("parse_await_single")
        .build();

    document(parser, doc)
}

/// Documentation provider for expression parsers
pub struct ExpressionDocProvider;

impl DocumentationProvider for ExpressionDocProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        vec![
            as_any_doc_parser(documented_parse_expression()),
            as_any_doc_parser(documented_parse_binary_expression()),
            as_any_doc_parser(documented_parse_think()),
            as_any_doc_parser(documented_parse_think_single()),
            as_any_doc_parser(documented_parse_think_multiple()),
            as_any_doc_parser(documented_parse_err()),
            as_any_doc_parser(documented_parse_await()),
            as_any_doc_parser(documented_parse_await_single()),
            as_any_doc_parser(documented_parse_await_multiple()),
        ]
    }
}

/// Helper function to convert DocParserExt<Token, T> to DocParserExt<Token, Box<dyn Any>>
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
        // Test main expression parser
        let parser = documented_parse_expression();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_expression");
        assert_eq!(doc.category, ParserCategory::Expression);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);

        // Test think expression parser
        let parser = documented_parse_think();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_think");
        assert_eq!(doc.category, ParserCategory::Expression);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);
    }
}

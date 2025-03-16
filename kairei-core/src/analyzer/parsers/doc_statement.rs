//! Documentation for statement parsers.
//!
//! This module provides documented versions of the statement parsers
//! from the `statement.rs` module.

use crate::analyzer::core::*;
use crate::analyzer::doc_parser::{DocBuilder, DocParserExt, ParserCategory, document};
use crate::analyzer::documentation_collector::DocumentationProvider;
use crate::analyzer::parsers::statement::parse_statement;
use crate::ast;
use crate::tokenizer::token::Token;
use std::any::Any;
use std::marker::PhantomData;

/// Custom filter combinator that filters the output of a parser based on a predicate
struct FilterParser<P, F> {
    parser: P,
    predicate: F,
}

impl<P, F, I, O> Parser<I, O> for FilterParser<P, F>
where
    P: Parser<I, O>,
    F: Fn(&O) -> bool,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        match self.parser.parse(input, pos) {
            Ok((next_pos, output)) if (self.predicate)(&output) => Ok((next_pos, output)),
            Ok(_) => Err(ParseError::Failure {
                message: "Predicate failed".to_string(),
                position: pos,
                context: None,
            }),
            Err(e) => Err(e),
        }
    }
}

/// Helper function to create a filter parser
fn filter_parser<P, F, I, O>(parser: P, predicate: F) -> FilterParser<P, F>
where
    P: Parser<I, O>,
    F: Fn(&O) -> bool,
{
    FilterParser { parser, predicate }
}

/// Returns a documented version of the main statement parser
pub fn documented_parse_statement() -> impl DocParserExt<Token, ast::Statement> {
    let parser = parse_statement();

    let doc = DocBuilder::new("parse_statement", ParserCategory::Statement)
        .description("Statements in KAIREI represent actions or commands that perform operations but don't necessarily produce values. They control program flow, modify variables, or interact with the environment.")
        .example("x = 42")
        .example("if condition { return result }")
        .example("emit UserCreated(id: user.id)")
        .example("return value")
        .example("{ statement1; statement2 }")
        .related_parser("documented_parse_assignment_statement")
        .related_parser("documented_parse_if_statement")
        .related_parser("documented_parse_return_statement")
        .related_parser("documented_parse_emit_statement")
        .related_parser("documented_parse_block_statement")
        .related_parser("documented_parse_expression_statement")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the assignment statement parser
pub fn documented_parse_assignment_statement() -> impl DocParserExt<Token, ast::Statement> {
    // We'll use the public parse_statement function and filter for assignment statements
    let parser = filter_parser(parse_statement(), |stmt| {
        matches!(stmt, ast::Statement::Assignment { .. })
    });

    let doc = DocBuilder::new("parse_assignment_statement", ParserCategory::Statement)
        .description("Assignment statements allow you to store values in variables for later use. You can assign to a single variable or destructure into multiple variables.")
        .example("x = 42")
        .example("result = await fetch_data()")
        .example("user.name = \"John\"")
        .example("(firstName, lastName) = fullName.split(\" \")")
        .related_parser("documented_parse_statement")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the block statement parser
pub fn documented_parse_block_statement() -> impl DocParserExt<Token, ast::Statement> {
    // We'll use the public parse_statement function and filter for block statements
    let parser = filter_parser(parse_statement(), |stmt| {
        matches!(stmt, ast::Statement::Block(_))
    });

    let doc = DocBuilder::new("parse_block_statement", ParserCategory::Statement)
        .description("Block statements group multiple statements together into a single unit. They are enclosed in curly braces and can be used anywhere a single statement is expected.")
        .example("{ x = 42; y = x * 2; return y }")
        .example("{ think(\"What should I do?\"); emit Decision(choice: \"A\") }")
        .related_parser("documented_parse_statement")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the if statement parser
pub fn documented_parse_if_statement() -> impl DocParserExt<Token, ast::Statement> {
    // We'll use the public parse_statement function and filter for if statements
    let parser = filter_parser(parse_statement(), |stmt| {
        matches!(stmt, ast::Statement::If { .. })
    });

    let doc = DocBuilder::new("parse_if_statement", ParserCategory::Statement)
        .description("If statements allow conditional execution of code based on whether a condition evaluates to true. They can include an optional else clause for alternative execution paths.")
        .example("if x > 10 { return \"High\" }")
        .example("if user.isAdmin { showAdminPanel() } else { showUserPanel() }")
        .example("if condition { action1() } else if otherCondition { action2() } else { action3() }")
        .related_parser("documented_parse_statement")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the expression statement parser
pub fn documented_parse_expression_statement() -> impl DocParserExt<Token, ast::Statement> {
    // We'll use the public parse_statement function and filter for expression statements
    let parser = filter_parser(parse_statement(), |stmt| {
        matches!(stmt, ast::Statement::Expression(_))
    });

    let doc = DocBuilder::new("parse_expression_statement", ParserCategory::Statement)
        .description("Expression statements consist of a single expression whose result is discarded. These are typically function calls or other operations with side effects.")
        .example("print(\"Hello, world!\")")
        .example("calculateTotal()")
        .example("await asyncOperation()")
        .example("think(\"What is the capital of France?\")")
        .related_parser("documented_parse_statement")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the return statement parser
pub fn documented_parse_return_statement() -> impl DocParserExt<Token, ast::Statement> {
    // We'll use the public parse_statement function and filter for return statements
    let parser = filter_parser(parse_statement(), |stmt| {
        matches!(stmt, ast::Statement::Return(_))
    });

    let doc = DocBuilder::new("parse_return_statement", ParserCategory::Statement)
        .description("Return statements exit the current function or block and optionally provide a value as the result. They are used to send data back from functions or to exit early from a block of code.")
        .example("return 42")
        .example("return user.profile")
        .example("return await fetchData()")
        .example("return { success: true, data: result }")
        .related_parser("documented_parse_statement")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the error handler statement parser
pub fn documented_parse_error_handler() -> impl DocParserExt<Token, ast::Statement> {
    // We'll use the public parse_statement function and filter for error handler statements
    let parser = filter_parser(parse_statement(), |stmt| {
        matches!(stmt, ast::Statement::WithError { .. })
    });

    let doc = DocBuilder::new("parse_error_handler", ParserCategory::Statement)
        .description("Error handling statements allow you to catch and process errors that occur during execution. The onFail block executes when the preceding statement throws an error, giving you access to the error object and control over how to respond.")
        .example("await fetchData() onFail(err) { log(err); return Err(\"Failed to fetch data\") }")
        .example("processInput() onFail { return Ok(defaultValue) }")
        .example("validateUser() onFail(error) { rethrow }")
        .related_parser("documented_parse_statement")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the emit statement parser
pub fn documented_parse_emit_statement() -> impl DocParserExt<Token, ast::Statement> {
    // We'll use the public parse_statement function and filter for emit statements
    let parser = filter_parser(parse_statement(), |stmt| {
        matches!(stmt, ast::Statement::Emit { .. })
    });

    let doc = DocBuilder::new("parse_emit_statement", ParserCategory::Statement)
        .description("Emit statements send events to other components in the system. They can include parameters and optionally specify a target recipient for the event.")
        .example("emit UserCreated(id: user.id, name: user.name)")
        .example("emit DataReceived(data)")
        .example("emit RequestApproval(amount: payment.total) to manager")
        .example("emit Notification(message: \"Process complete\")")
        .related_parser("documented_parse_statement")
        .build();

    document(parser, doc)
}

/// Documentation provider for statement parsers
pub struct StatementDocProvider;

impl DocumentationProvider for StatementDocProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        vec![
            as_any_doc_parser(documented_parse_statement()),
            as_any_doc_parser(documented_parse_assignment_statement()),
            as_any_doc_parser(documented_parse_block_statement()),
            as_any_doc_parser(documented_parse_if_statement()),
            as_any_doc_parser(documented_parse_expression_statement()),
            as_any_doc_parser(documented_parse_return_statement()),
            as_any_doc_parser(documented_parse_error_handler()),
            as_any_doc_parser(documented_parse_emit_statement()),
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
        _phantom: PhantomData<O>,
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
        _phantom: PhantomData,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_attached() {
        // Test main statement parser
        let parser = documented_parse_statement();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_statement");
        assert_eq!(doc.category, ParserCategory::Statement);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);

        // Test assignment statement parser
        let parser = documented_parse_assignment_statement();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_assignment_statement");
        assert_eq!(doc.category, ParserCategory::Statement);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);
    }
}

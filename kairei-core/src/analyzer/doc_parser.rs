//! # Documentation Parser Extensions
//!
//! This module provides extension traits and implementations to make parsers
//! self-documenting. This allows the system to generate comprehensive
//! documentation for the KAIREI DSL based on the actual parser implementations.

use crate::analyzer::core::{ParseResult, Parser};
use std::fmt;
use std::marker::PhantomData;

/// Parser category indicating what part of the DSL the parser handles.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParserCategory {
    /// Expression parsers (values, operations, etc.)
    Expression,
    /// Statement parsers (control flow, assignments, etc.)
    Statement,
    /// Handler parsers (answer, observe, react, etc.)
    Handler,
    /// Type parsers (string, number, bool, etc.)
    Type,
    /// Definition parsers for top-level constructs (world, agent, sistence, etc.)
    Definition,
    /// Other categories not fitting the main ones
    Other(String),
}

impl fmt::Display for ParserCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserCategory::Expression => write!(f, "Expression"),
            ParserCategory::Statement => write!(f, "Statement"),
            ParserCategory::Handler => write!(f, "Handler"),
            ParserCategory::Type => write!(f, "Type"),
            ParserCategory::Definition => write!(f, "Definition"),
            ParserCategory::Other(name) => write!(f, "{}", name),
        }
    }
}

/// Documentation metadata for a parser.
#[derive(Debug, Clone)]
pub struct ParserDocumentation {
    /// The name of the parser (e.g., "parse_expression", "parse_logical_or")
    pub name: String,
    /// A description of what the parser does
    pub description: String,
    /// The category this parser belongs to
    pub category: ParserCategory,
    /// Examples of valid syntax this parser can handle
    pub examples: Vec<String>,
    /// Optional deprecation notice if this parser is deprecated
    pub deprecated: Option<String>,
    /// Related parsers (e.g., sub-parsers or alternative parsers)
    pub related_parsers: Vec<String>,
}

impl Default for ParserDocumentation {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            category: ParserCategory::Other("Uncategorized".to_string()),
            examples: Vec::new(),
            deprecated: None,
            related_parsers: Vec::new(),
        }
    }
}

/// Extension trait for parsers that provides documentation metadata.
///
/// This trait allows parsers to describe themselves with human-readable
/// documentation that can be collected and used to generate DSL documentation.
pub trait DocParserExt<I, O>: Parser<I, O> {
    /// Get the documentation for this parser.
    fn documentation(&self) -> &ParserDocumentation;
}

/// A wrapper that adds documentation to an existing parser.
///
/// This wrapper implements the `DocParserExt` trait while delegating the actual
/// parsing to the wrapped parser, allowing documentation to be attached to any
/// parser without changing its behavior.
#[derive(Clone)]
pub struct DocParser<P, I, O> {
    /// The parser being documented
    parser: P,
    /// Documentation metadata for this parser
    documentation: ParserDocumentation,
    _phantom: PhantomData<(I, O)>,
}

impl<P, I, O> DocParser<P, I, O> {
    /// Create a new documented parser wrapping the given parser.
    ///
    /// # Arguments
    ///
    /// * `parser` - The parser to wrap with documentation
    /// * `documentation` - Documentation metadata for the parser
    pub fn new(parser: P, documentation: ParserDocumentation) -> Self {
        Self {
            parser,
            documentation,
            _phantom: PhantomData,
        }
    }
}

impl<P, I, O> Parser<I, O> for DocParser<P, I, O>
where
    P: Parser<I, O>,
{
    /// Delegates parsing to the wrapped parser.
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        self.parser.parse(input, pos)
    }
}

impl<P, I, O> DocParserExt<I, O> for DocParser<P, I, O>
where
    P: Parser<I, O>,
{
    /// Returns the documentation for this parser.
    fn documentation(&self) -> &ParserDocumentation {
        &self.documentation
    }
}

/// Builder for creating parser documentation with a fluent interface.
///
/// This builder simplifies the process of constructing documentation
/// for parsers with method chaining.
pub struct DocBuilder {
    documentation: ParserDocumentation,
}

impl DocBuilder {
    /// Create a new documentation builder with the given name and category.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the parser
    /// * `category` - The category this parser belongs to
    pub fn new(name: impl Into<String>, category: ParserCategory) -> Self {
        Self {
            documentation: ParserDocumentation {
                name: name.into(),
                category,
                ..Default::default()
            },
        }
    }

    /// Add a description for the parser.
    ///
    /// # Arguments
    ///
    /// * `description` - A description of what the parser does
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.documentation.description = description.into();
        self
    }

    /// Add an example of valid syntax for the parser.
    ///
    /// # Arguments
    ///
    /// * `example` - An example string showing valid syntax
    pub fn example(mut self, example: impl Into<String>) -> Self {
        self.documentation.examples.push(example.into());
        self
    }

    /// Add multiple examples of valid syntax for the parser.
    ///
    /// # Arguments
    ///
    /// * `examples` - A collection of example strings
    pub fn examples<S, I>(mut self, examples: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.documentation
            .examples
            .extend(examples.into_iter().map(Into::into));
        self
    }

    /// Mark the parser as deprecated with an optional message.
    ///
    /// # Arguments
    ///
    /// * `message` - Optional message explaining the deprecation
    pub fn deprecated(mut self, message: impl Into<String>) -> Self {
        self.documentation.deprecated = Some(message.into());
        self
    }

    /// Add a related parser reference.
    ///
    /// # Arguments
    ///
    /// * `parser_name` - The name of a related parser
    pub fn related_parser(mut self, parser_name: impl Into<String>) -> Self {
        self.documentation.related_parsers.push(parser_name.into());
        self
    }

    /// Add multiple related parser references.
    ///
    /// # Arguments
    ///
    /// * `parser_names` - A collection of related parser names
    pub fn related_parsers<S, I>(mut self, parser_names: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.documentation
            .related_parsers
            .extend(parser_names.into_iter().map(Into::into));
        self
    }

    /// Build the documentation.
    pub fn build(self) -> ParserDocumentation {
        self.documentation
    }
}

/// Helper functions for creating documented parsers.
///
/// Create a documented parser with the provided parser and documentation.
///
/// # Arguments
///
/// * `parser` - The parser to document
/// * `documentation` - Documentation metadata for the parser
pub fn document<P, I, O>(parser: P, documentation: ParserDocumentation) -> DocParser<P, I, O>
where
    P: Parser<I, O>,
{
    DocParser::new(parser, documentation)
}

/// Create a documented expression parser.
///
/// # Arguments
///
/// * `parser` - The expression parser to document
/// * `name` - The name of the parser
/// * `description` - A description of what the parser does
pub fn document_expression<P, I, O>(
    parser: P,
    name: impl Into<String>,
    description: impl Into<String>,
) -> DocParser<P, I, O>
where
    P: Parser<I, O>,
{
    let doc = DocBuilder::new(name, ParserCategory::Expression)
        .description(description)
        .build();
    DocParser::new(parser, doc)
}

/// Create a documented statement parser.
///
/// # Arguments
///
/// * `parser` - The statement parser to document
/// * `name` - The name of the parser
/// * `description` - A description of what the parser does
pub fn document_statement<P, I, O>(
    parser: P,
    name: impl Into<String>,
    description: impl Into<String>,
) -> DocParser<P, I, O>
where
    P: Parser<I, O>,
{
    let doc = DocBuilder::new(name, ParserCategory::Statement)
        .description(description)
        .build();
    DocParser::new(parser, doc)
}

/// Create a documented handler parser.
///
/// # Arguments
///
/// * `parser` - The handler parser to document
/// * `name` - The name of the parser
/// * `description` - A description of what the parser does
pub fn document_handler<P, I, O>(
    parser: P,
    name: impl Into<String>,
    description: impl Into<String>,
) -> DocParser<P, I, O>
where
    P: Parser<I, O>,
{
    let doc = DocBuilder::new(name, ParserCategory::Handler)
        .description(description)
        .build();
    DocParser::new(parser, doc)
}

/// Create a documented definition parser for top-level constructs.
///
/// # Arguments
///
/// * `parser` - The definition parser to document
/// * `name` - The name of the parser
/// * `description` - A description of what the parser does
pub fn document_definition<P, I, O>(
    parser: P,
    name: impl Into<String>,
    description: impl Into<String>,
) -> DocParser<P, I, O>
where
    P: Parser<I, O>,
{
    let doc = DocBuilder::new(name, ParserCategory::Definition)
        .description(description)
        .build();
    DocParser::new(parser, doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::combinators::Equal;
    use crate::analyzer::core::Parser;

    #[test]
    fn test_doc_parser_delegates_parsing() {
        // Create a simple parser
        let parser = Equal::new(42);

        // Create documentation
        let documentation = DocBuilder::new("equal_42", ParserCategory::Expression)
            .description("Parses the integer 42")
            .build();

        // Create a documented parser
        let doc_parser = DocParser::new(parser, documentation);

        // Test that parsing is correctly delegated
        let input = &[42, 43, 44];
        assert_eq!(doc_parser.parse(input, 0), Ok((1, 42)));
        assert!(doc_parser.parse(input, 1).is_err());
    }

    #[test]
    fn test_doc_parser_provides_documentation() {
        let parser = Equal::new(42);

        let documentation = DocBuilder::new("equal_42", ParserCategory::Expression)
            .description("Parses the integer 42")
            .example("42")
            .build();

        let doc_parser = DocParser::new(parser, documentation);

        // Test that we can access the documentation
        let doc = doc_parser.documentation();
        assert_eq!(doc.name, "equal_42");
        assert_eq!(doc.category, ParserCategory::Expression);
        assert_eq!(doc.description, "Parses the integer 42");
        assert_eq!(doc.examples, vec!["42"]);
    }

    #[test]
    fn test_doc_builder() {
        let doc = DocBuilder::new("test_parser", ParserCategory::Expression)
            .description("A test parser")
            .example("example 1")
            .example("example 2")
            .examples(vec!["example 3", "example 4"])
            .deprecated("Use new_parser instead")
            .related_parser("other_parser")
            .related_parsers(vec!["parser1", "parser2"])
            .build();

        assert_eq!(doc.name, "test_parser");
        assert_eq!(doc.description, "A test parser");
        assert_eq!(
            doc.examples,
            vec!["example 1", "example 2", "example 3", "example 4"]
        );
        assert_eq!(doc.deprecated, Some("Use new_parser instead".to_string()));
        assert_eq!(
            doc.related_parsers,
            vec!["other_parser", "parser1", "parser2"]
        );
    }

    #[test]
    fn test_helper_functions() {
        let parser = Equal::new(42);

        // Test document_expression
        let doc_expr = document_expression(parser.clone(), "equal_42", "Parses the integer 42");
        assert_eq!(
            doc_expr.documentation().category,
            ParserCategory::Expression
        );

        // Test document_statement
        let doc_stmt = document_statement(
            parser.clone(),
            "equal_42_stmt",
            "Parses the integer 42 as a statement",
        );
        assert_eq!(doc_stmt.documentation().category, ParserCategory::Statement);

        // Test document_handler
        let doc_handler = document_handler(
            parser.clone(),
            "equal_42_handler",
            "Parses the integer 42 as a handler",
        );
        assert_eq!(
            doc_handler.documentation().category,
            ParserCategory::Handler
        );

        // Test document_definition
        let doc_def = document_definition(
            parser.clone(),
            "equal_42_definition",
            "Parses the integer 42 as a definition",
        );
        assert_eq!(doc_def.documentation().category, ParserCategory::Definition);
    }
}

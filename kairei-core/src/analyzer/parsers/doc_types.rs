//! Documentation for type parsers.
//!
//! This module provides documented versions of the type parsers
//! from the `types.rs` module.

use crate::analyzer::core::*;
use crate::analyzer::doc_parser::{DocBuilder, DocParserExt, ParserCategory, document};
use crate::analyzer::documentation_collector::DocumentationProvider;
use crate::analyzer::parsers::types::*;
use crate::ast;
use crate::tokenizer::token::Token;
use std::any::Any;

/// Custom filter combinator that filters the output of a parser based on a predicate
struct FilterParser<P, F> {
    parser: P,    // The underlying parser to filter
    predicate: F, // The predicate function that determines which outputs to accept
}

impl<P, F, I, O> Parser<I, O> for FilterParser<P, F>
where
    P: Parser<I, O>,
    F: Fn(&O) -> bool,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        // First, try to parse using the underlying parser
        match self.parser.parse(input, pos) {
            // If parsing succeeds and the predicate is satisfied, return the result
            Ok((next_pos, output)) if (self.predicate)(&output) => Ok((next_pos, output)),

            // If parsing succeeds but the predicate fails, return a failure
            Ok(_) => Err(ParseError::Failure {
                message: "Predicate failed".to_string(),
                position: pos,
                context: None,
            }),

            // If parsing fails, propagate the error
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

/// Returns a documented version of the main type info parser
pub fn documented_parse_type_info() -> impl DocParserExt<Token, ast::TypeInfo> {
    let parser = parse_type_info();

    let doc = DocBuilder::new("parse_type_info", ParserCategory::Type)
        .description("The type system in KAIREI allows you to define and use various types of data. Types can be simple built-in types, custom types with fields, or container types like Option and Result.")
        .example("String")
        .example("Number")
        .example("Boolean")
        .example("Option{String}")
        .example("Result<String, Error>")
        .example("User { name: String, age: Number }")
        .related_parser("parse_custom_type")
        .related_parser("parse_option_type")
        .related_parser("parse_result_type")
        .related_parser("parse_simple_type")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the custom type parser
pub fn documented_parse_custom_type() -> impl DocParserExt<Token, ast::TypeInfo> {
    let parser = parse_custom_type();

    let doc = DocBuilder::new("parse_custom_type", ParserCategory::Type)
        .description("Custom types allow you to define structured data with named fields. They can include type annotations and default values for fields, making it easy to create reusable data structures.")
        .example("type User { name: String, age: Number, isActive: Boolean = true }")
        .example("type Point { x: Number = 0, y: Number = 0 }")
        .example("type Empty {}")
        .related_parser("parse_type_info")
        .related_parser("parse_field")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the option type parser
pub fn documented_parse_option_type() -> impl DocParserExt<Token, ast::TypeInfo> {
    let parser = parse_option_type();

    let doc = DocBuilder::new("parse_option_type", ParserCategory::Type)
        .description("Option types represent values that may or may not exist. They are useful for handling nullable or optional data in a type-safe way.")
        .example("Option{String}")
        .example("Option{User}")
        .example("Option{Result<String, Error>}")
        .related_parser("parse_type_info")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the array type parser
pub fn documented_parse_array_type() -> impl DocParserExt<Token, ast::TypeInfo> {
    // Since parse_array_type is private, we'll use parse_type_info and filter for array types
    let parser = filter_parser(parse_type_info(), |type_info| {
        matches!(type_info, ast::TypeInfo::Array(_))
    });

    let doc = DocBuilder::new("parse_array_type", ParserCategory::Type)
        .description("Array types represent collections of elements of the same type. They allow you to work with lists of data in a structured way.")
        .example("Array{String}")
        .example("Array{Number}")
        .example("Array{User}")
        .related_parser("parse_type_info")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the result type parser
pub fn documented_parse_result_type() -> impl DocParserExt<Token, ast::TypeInfo> {
    let parser = parse_result_type();

    let doc = DocBuilder::new("parse_result_type", ParserCategory::Type)
        .description("Result types represent operations that can either succeed with a value or fail with an error. They are essential for robust error handling in KAIREI.")
        .example("Result<String, Error>")
        .example("Result<User, ValidationError>")
        .example("Result<Array{String}, ParseError>")
        .related_parser("parse_type_info")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the simple type parser
pub fn documented_parse_simple_type() -> impl DocParserExt<Token, ast::TypeInfo> {
    // Since parse_simple_type is private, we'll use parse_type_info and filter for simple types
    let parser = filter_parser(parse_type_info(), |type_info| {
        matches!(type_info, ast::TypeInfo::Simple(_))
    });

    let doc = DocBuilder::new("parse_simple_type", ParserCategory::Type)
        .description("Simple types are the basic building blocks of the KAIREI type system. They include built-in types like String, Number, and Boolean, as well as references to custom types.")
        .example("String")
        .example("Number")
        .example("Boolean")
        .example("User")
        .related_parser("parse_type_info")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the field parser
pub fn documented_parse_field() -> impl DocParserExt<Token, (String, ast::FieldInfo)> {
    let parser = parse_field();

    let doc = DocBuilder::new("parse_field", ParserCategory::Type)
        .description("Fields are the components of custom types. Each field has a name and can include type annotations and default values.")
        .example("name: String")
        .example("age: Number = 30")
        .example("isActive = true")
        .related_parser("parse_field_typed_with_default")
        .related_parser("parse_field_typed")
        .related_parser("parse_field_inferred")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the typed field with default value parser
pub fn documented_parse_field_typed_with_default() -> impl DocParserExt<Token, ast::FieldInfo> {
    let parser = parse_field_typed_with_default();

    let doc = DocBuilder::new("parse_field_typed_with_default", ParserCategory::Type)
        .description("Typed fields with default values specify both the type and an initial value for a field. This provides both type safety and convenient defaults.")
        .example("name: String = \"John\"")
        .example("age: Number = 30")
        .example("isActive: Boolean = true")
        .related_parser("parse_field")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the typed field parser
pub fn documented_parse_field_typed() -> impl DocParserExt<Token, ast::FieldInfo> {
    // Since parse_field_typed is private, we'll filter field info with type but no default
    let parser = filter_parser(parse_field_typed_with_default(), |field_info| {
        field_info.type_info.is_some() && field_info.default_value.is_none()
    });

    let doc = DocBuilder::new("parse_field_typed", ParserCategory::Type)
        .description("Typed fields specify the type of a field without providing a default value. This ensures type safety while requiring explicit initialization.")
        .example("name: String")
        .example("age: Number")
        .example("isActive: Boolean")
        .related_parser("parse_field")
        .build();

    document(parser, doc)
}

/// Returns a documented version of the inferred field parser
pub fn documented_parse_field_inferred() -> impl DocParserExt<Token, ast::FieldInfo> {
    // Since parse_field_inferred is private, we'll filter field info with default but no type
    let parser = filter_parser(parse_field_typed_with_default(), |field_info| {
        field_info.type_info.is_none() && field_info.default_value.is_some()
    });

    let doc = DocBuilder::new("parse_field_inferred", ParserCategory::Type)
        .description("Inferred fields specify a default value without an explicit type annotation. The type is inferred from the default value, providing convenience while maintaining type safety.")
        .example("name = \"John\"")
        .example("age = 30")
        .example("isActive = true")
        .related_parser("parse_field")
        .build();

    document(parser, doc)
}

/// Documentation provider for type parsers
pub struct TypeDocProvider;

impl DocumentationProvider for TypeDocProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        vec![
            as_any_doc_parser(documented_parse_type_info()),
            as_any_doc_parser(documented_parse_custom_type()),
            as_any_doc_parser(documented_parse_option_type()),
            as_any_doc_parser(documented_parse_array_type()),
            as_any_doc_parser(documented_parse_result_type()),
            as_any_doc_parser(documented_parse_simple_type()),
            as_any_doc_parser(documented_parse_field()),
            as_any_doc_parser(documented_parse_field_typed_with_default()),
            as_any_doc_parser(documented_parse_field_typed()),
            as_any_doc_parser(documented_parse_field_inferred()),
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
        // Test main type info parser
        let parser = documented_parse_type_info();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_type_info");
        assert_eq!(doc.category, ParserCategory::Type);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);

        // Test custom type parser
        let parser = documented_parse_custom_type();
        let doc = parser.documentation();

        assert_eq!(doc.name, "parse_custom_type");
        assert_eq!(doc.category, ParserCategory::Type);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert!(doc.examples.len() >= 2);
    }
}

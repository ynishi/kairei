use kairei_core::analyzer::doc_parser::DocBuilder;
use kairei_core::analyzer::prelude::*;
use kairei_core::analyzer::{DocParserExt, Parser, ParserCategory};
use kairei_core::ast;
use kairei_core::tokenizer::{
    literal::{Literal, StringLiteral, StringPart},
    symbol::Delimiter,
    token::Token,
};

/// Creates a documented parser for the think expression
fn create_documented_think_parser() -> impl DocParserExt<Token, ast::Expression> {
    // Create documentation
    let doc = DocBuilder::new("parse_think", ParserCategory::Expression)
        .description("Parses a think expression for LLM invocation")
        .example("think(\"What is the capital of France?\")")
        .example("think(\"Summarize this text\", text)")
        .example("think(prompt: \"Generate ideas\", context: data) with { model: \"gpt-4\" }")
        .build();

    // Get the parser from the normal parse_think function
    // For testing, we'll create a simple parser that matches a specific sequence
    let parser = map(
        delimited(
            as_unit(equal(Token::Keyword(
                kairei_core::tokenizer::keyword::Keyword::Think,
            ))),
            delimited(
                as_unit(equal(Token::Delimiter(Delimiter::OpenParen))),
                equal(Token::Literal(Literal::String(StringLiteral::Single(
                    vec![StringPart::Literal("test".to_string())],
                )))),
                as_unit(equal(Token::Delimiter(Delimiter::CloseParen))),
            ),
            zero(()),
        ),
        |_| ast::Expression::Literal(ast::Literal::String("test".to_string())),
    );

    // Create the documented parser
    document(parser, doc)
}

#[test]
fn test_doc_parser_integration() {
    // Create the documented parser
    let parser = create_documented_think_parser();

    // Test the documentation
    let doc = parser.documentation();
    assert_eq!(doc.name, "parse_think");
    assert_eq!(doc.category, ParserCategory::Expression);
    assert!(doc.description.contains("think expression"));
    assert_eq!(doc.examples.len(), 3);

    // Test the parser functionality
    let input = &[
        Token::Keyword(kairei_core::tokenizer::keyword::Keyword::Think),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Literal(Literal::String(StringLiteral::Single(vec![
            StringPart::Literal("test".to_string()),
        ]))),
        Token::Delimiter(Delimiter::CloseParen),
    ];

    // The parser should still work correctly
    let result = parser.parse(input, 0);
    assert!(result.is_ok());

    let (pos, expr) = result.unwrap();
    assert_eq!(pos, 4);

    match expr {
        ast::Expression::Literal(ast::Literal::String(s)) => {
            assert_eq!(s, "test");
        }
        _ => panic!("Expected string literal"),
    }
}

#[test]
fn test_doc_parser_collection() {
    // Create a collection of documented parsers
    let parsers: Vec<Box<dyn DocParserExt<Token, ast::Expression>>> = vec![
        Box::new(create_documented_think_parser()),
        // We need to map the integer token to an Expression
        Box::new(document_expression(
            map(equal(Token::Literal(Literal::Integer(42))), |_| {
                ast::Expression::Literal(ast::Literal::Integer(42))
            }),
            "parse_integer",
            "Parses an integer literal",
        )),
    ];

    // Check that we can collect documentation from all parsers
    let mut docs = Vec::new();
    for parser in &parsers {
        docs.push(parser.documentation().clone());
    }

    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].name, "parse_think");
    assert_eq!(docs[1].name, "parse_integer");

    // Check that we can filter by category
    let expression_docs: Vec<_> = docs
        .iter()
        .filter(|doc| doc.category == ParserCategory::Expression)
        .collect();

    assert_eq!(expression_docs.len(), 2);
}

#[test]
fn test_expression_doc_provider() {
    use kairei_core::analyzer::documentation_collector::DocumentationProvider;
    use kairei_core::analyzer::parsers::doc_expression::ExpressionDocProvider;

    // Create the provider
    let provider = ExpressionDocProvider;

    // Get the documented parsers
    let parsers = DocumentationProvider::provide_documented_parsers(&provider);

    // Check that we have the expected number of parsers
    assert!(!parsers.is_empty());
    assert!(parsers.len() >= 9); // We should have at least 9 documented expression parsers

    // Check that all parsers have documentation
    for parser in &parsers {
        let doc = parser.documentation();
        assert!(!doc.name.is_empty());
        assert_eq!(doc.category, ParserCategory::Expression);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
    }
}

#[test]
fn test_collecting_expression_docs() {
    use kairei_core::analyzer::documentation_collector::DocumentationCollector;
    use kairei_core::analyzer::parsers::doc_expression::ExpressionDocProvider;

    // Create a documentation collector
    let mut collector = DocumentationCollector::new();

    // Register the expression documentation provider
    collector.register_provider(Box::new(ExpressionDocProvider));

    // Collect the documentation
    collector.collect();

    // Get the collection
    let collection = collector.get_collection();

    // Verify that we collected documentation for expression parsers
    let expression_docs = collection.get_by_category(&ParserCategory::Expression);
    assert!(!expression_docs.is_empty());
    assert!(expression_docs.len() >= 9); // We should have at least 9 documented expression parsers

    // Check specific parsers
    let expression_parser = collection.get_by_name("parse_expression");
    assert!(expression_parser.is_some());

    let think_parser = collection.get_by_name("parse_think");
    assert!(think_parser.is_some());

    // Validate the collection
    let validation_issues = collector.validate();

    // Print any validation issues
    if !validation_issues.is_empty() {
        println!("Validation issues:");
        for issue in &validation_issues {
            println!("  - {}", issue);
        }
    }
    assert!(validation_issues.is_empty());

    // Generate markdown (but don't assert on specifics since other tests might add docs)
    let _markdown = collector.export_markdown();
}

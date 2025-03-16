use kairei_core::analyzer::doc_parser::DocBuilder;
use kairei_core::analyzer::prelude::*;
use kairei_core::analyzer::{
    DocParser, DocParserExt, DocumentationCollector, DocumentationProvider, ParserCategory,
};
use kairei_core::tokenizer::token::Token;
use std::any::Any;

// A simple test parser provider that implements DocumentationProvider
struct TestParserProvider {}

#[allow(clippy::missing_transmute_annotations)]
impl TestParserProvider {
    fn new() -> Self {
        Self {}
    }

    // Helper method to create some documented parsers for testing
    fn create_test_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        // Using transmute for simplicity in testing
        // In a real implementation, you'd want to handle this more safely
        unsafe {
            vec![
                std::mem::transmute(
                    Box::new(self.create_parser_1()) as Box<dyn DocParserExt<Token, Token>>
                ),
                std::mem::transmute(
                    Box::new(self.create_parser_2()) as Box<dyn DocParserExt<Token, Token>>
                ),
                std::mem::transmute(
                    Box::new(self.create_parser_3()) as Box<dyn DocParserExt<Token, Token>>
                ),
            ]
        }
    }

    fn create_parser_1(&self) -> impl DocParserExt<Token, Token> {
        let parser = identity::<Token>();
        let doc = DocBuilder::new("parse_identifier", ParserCategory::Expression)
            .description("Parses an identifier token")
            .example("variable_name")
            .example("camelCaseName")
            .related_parser("parse_qualified_identifier")
            .build();
        DocParser::new(parser, doc)
    }

    fn create_parser_2(&self) -> impl DocParserExt<Token, Token> {
        let parser = identity::<Token>();
        let doc = DocBuilder::new("parse_qualified_identifier", ParserCategory::Expression)
            .description("Parses a qualified identifier (with namespace)")
            .example("namespace.name")
            .example("module.submodule.function")
            .related_parser("parse_identifier")
            .build();
        DocParser::new(parser, doc)
    }

    fn create_parser_3(&self) -> impl DocParserExt<Token, Token> {
        let parser = identity::<Token>();
        let doc = DocBuilder::new("parse_if_statement", ParserCategory::Statement)
            .description("Parses an if statement")
            .example("if (condition) { statements }")
            .example("if (a > b) { return a; } else { return b; }")
            .related_parser("parse_condition")
            .build();
        DocParser::new(parser, doc)
    }
}

impl DocumentationProvider for TestParserProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        self.create_test_parsers()
    }
}

#[test]
fn test_documentation_collector_integration() {
    // Create a collector
    let mut collector = DocumentationCollector::new();

    // Register providers
    collector.register_provider(Box::new(TestParserProvider::new()));

    // Collect documentation
    collector.collect();

    // Get the collection
    let collection = collector.get_collection();

    // Verify collection has the expected number of entries
    assert_eq!(collection.count(), 3);

    // Verify categories
    let categories = collection.get_categories();
    assert_eq!(categories.len(), 2);
    assert!(
        categories
            .iter()
            .any(|c| matches!(c, ParserCategory::Expression))
    );
    assert!(
        categories
            .iter()
            .any(|c| matches!(c, ParserCategory::Statement))
    );

    // Verify expressions
    let expressions = collection.get_by_category(&ParserCategory::Expression);
    assert_eq!(expressions.len(), 2);
    assert!(expressions.iter().any(|d| d.name == "parse_identifier"));
    assert!(
        expressions
            .iter()
            .any(|d| d.name == "parse_qualified_identifier")
    );

    // Verify statements
    let statements = collection.get_by_category(&ParserCategory::Statement);
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].name, "parse_if_statement");

    // Verify specific entry
    let identifier_doc = collection.get_by_name("parse_identifier").unwrap();
    assert_eq!(identifier_doc.description, "Parses an identifier token");
    assert_eq!(identifier_doc.examples.len(), 2);
    assert!(
        identifier_doc
            .examples
            .contains(&"variable_name".to_string())
    );
    assert!(
        identifier_doc
            .examples
            .contains(&"camelCaseName".to_string())
    );
    assert_eq!(identifier_doc.related_parsers.len(), 1);
    assert_eq!(
        identifier_doc.related_parsers[0],
        "parse_qualified_identifier"
    );

    // Verify relation graph
    let graph = collection.build_relation_graph();
    assert!(graph["parse_identifier"].contains(&"parse_qualified_identifier".to_string()));
    assert!(graph["parse_qualified_identifier"].contains(&"parse_identifier".to_string()));

    // Verify validation (should have one issue - reference to non-existent parser)
    let issues = collection.validate();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("non-existent related parser 'parse_condition'"));
}

// Additional test for a more complex provider setup
struct ExpressionParserProvider {}

#[allow(clippy::missing_transmute_annotations)]
impl ExpressionParserProvider {
    fn new() -> Self {
        Self {}
    }

    fn create_expression_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        unsafe {
            vec![
                std::mem::transmute(Box::new(self.create_expression_parser())
                    as Box<dyn DocParserExt<Token, Token>>),
                std::mem::transmute(Box::new(self.create_binary_expression_parser())
                    as Box<dyn DocParserExt<Token, Token>>),
                std::mem::transmute(Box::new(self.create_primary_expression_parser())
                    as Box<dyn DocParserExt<Token, Token>>),
            ]
        }
    }

    fn create_expression_parser(&self) -> impl DocParserExt<Token, Token> {
        let parser = identity::<Token>();
        let doc = DocBuilder::new("parse_expression", ParserCategory::Expression)
            .description("Parses any valid expression")
            .example("a + b")
            .example("foo(bar)")
            .related_parser("parse_binary_expression")
            .related_parser("parse_primary_expression")
            .build();
        DocParser::new(parser, doc)
    }

    fn create_binary_expression_parser(&self) -> impl DocParserExt<Token, Token> {
        let parser = identity::<Token>();
        let doc = DocBuilder::new("parse_binary_expression", ParserCategory::Expression)
            .description("Parses a binary operation expression")
            .example("a + b")
            .example("x * (y + z)")
            .related_parser("parse_expression")
            .related_parser("parse_primary_expression")
            .build();
        DocParser::new(parser, doc)
    }

    fn create_primary_expression_parser(&self) -> impl DocParserExt<Token, Token> {
        let parser = identity::<Token>();
        let doc = DocBuilder::new("parse_primary_expression", ParserCategory::Expression)
            .description("Parses a primary expression (literals, variables, etc.)")
            .example("42")
            .example("variable_name")
            .example("\"string literal\"")
            .related_parser("parse_expression")
            .build();
        DocParser::new(parser, doc)
    }
}

impl DocumentationProvider for ExpressionParserProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        self.create_expression_parsers()
    }
}

struct StatementParserProvider {}

#[allow(clippy::missing_transmute_annotations)]
impl StatementParserProvider {
    fn new() -> Self {
        Self {}
    }

    fn create_statement_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        unsafe {
            vec![
                std::mem::transmute(
                    Box::new(self.create_statement_parser()) as Box<dyn DocParserExt<Token, Token>>
                ),
                std::mem::transmute(Box::new(self.create_if_statement_parser())
                    as Box<dyn DocParserExt<Token, Token>>),
            ]
        }
    }

    fn create_statement_parser(&self) -> impl DocParserExt<Token, Token> {
        let parser = identity::<Token>();
        let doc = DocBuilder::new("parse_statement", ParserCategory::Statement)
            .description("Parses any valid statement")
            .example("x = 1;")
            .example("if (condition) { /* code */ }")
            .related_parser("parse_if_statement")
            .build();
        DocParser::new(parser, doc)
    }

    fn create_if_statement_parser(&self) -> impl DocParserExt<Token, Token> {
        let parser = identity::<Token>();
        let doc = DocBuilder::new("parse_if_statement", ParserCategory::Statement)
            .description("Parses an if statement with optional else clause")
            .example("if (condition) { statements }")
            .example("if (a > b) { return a; } else { return b; }")
            .related_parser("parse_statement")
            .related_parser("parse_expression") // Cross-category relation
            .build();
        DocParser::new(parser, doc)
    }
}

impl DocumentationProvider for StatementParserProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        self.create_statement_parsers()
    }
}

#[test]
fn test_multiple_providers_integration() {
    // Create a collector
    let mut collector = DocumentationCollector::new();

    // Register multiple providers
    collector.register_provider(Box::new(ExpressionParserProvider::new()));
    collector.register_provider(Box::new(StatementParserProvider::new()));

    // Collect documentation
    collector.collect();

    // Get the collection
    let collection = collector.get_collection();

    // Verify collection has the expected number of entries
    assert_eq!(collection.count(), 5);

    // Verify categories
    let categories = collection.get_categories();
    assert_eq!(categories.len(), 2);

    // Verify expressions
    let expressions = collection.get_by_category(&ParserCategory::Expression);
    assert_eq!(expressions.len(), 3);

    // Verify statements
    let statements = collection.get_by_category(&ParserCategory::Statement);
    assert_eq!(statements.len(), 2);

    // Verify cross-category relations in the graph
    let graph = collection.build_relation_graph();

    // Check that parse_if_statement references parse_expression (cross-category)
    assert!(graph["parse_if_statement"].contains(&"parse_expression".to_string()));

    // And that parse_expression has a back-reference to parse_if_statement
    assert!(graph["parse_expression"].contains(&"parse_if_statement".to_string()));

    // Validate for consistency
    let issues = collection.validate();
    assert_eq!(
        issues.len(),
        0,
        "Unexpected validation issues: {:?}",
        issues
    );

    // Test exports
    let markdown = collector.export_markdown();
    assert!(markdown.contains("# KAIREI Language Documentation"));
    assert!(markdown.contains("## Expression"));
    assert!(markdown.contains("### parse_expression"));
    assert!(markdown.contains("### parse_binary_expression"));
    assert!(markdown.contains("### parse_primary_expression"));
    assert!(markdown.contains("## Statement"));
    assert!(markdown.contains("### parse_statement"));
    assert!(markdown.contains("### parse_if_statement"));

    let json_result = collector.export_json();
    assert!(json_result.is_ok());
    let json = json_result.unwrap();
    assert!(json.contains("parse_expression"));
    assert!(json.contains("parse_binary_expression"));
    assert!(json.contains("parse_if_statement"));
    assert!(json.contains("Parses an if statement with optional else clause"));
}

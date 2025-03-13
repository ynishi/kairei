use kairei_core::analyzer::{
    DocParserExt, DocumentationCollector, ParserCategory, ParserDocumentation
};
use kairei_core::analyzer::doc_parser::DocBuilder;
use kairei_core::tokenizer::token::Token;
use std::any::Any;

/// Implements DocumentationProvider for testing
struct TestParserDocumentationProvider {
    /// The categories of parsers to create
    categories: Vec<ParserCategory>,
    /// The number of parsers per category
    parsers_per_category: usize,
}

impl TestParserDocumentationProvider {
    /// Creates a new test provider
    fn new(categories: Vec<ParserCategory>, parsers_per_category: usize) -> Self {
        Self {
            categories,
            parsers_per_category,
        }
    }
}

impl kairei_core::analyzer::DocumentationProvider for TestParserDocumentationProvider {
    fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
        let mut result = Vec::new();

        for category in &self.categories {
            for i in 0..self.parsers_per_category {
                let parser_name = format!("parse_{}_{}",
                    category.to_string().to_lowercase(), i + 1);
                    
                let description = format!("Parses a {} of type {}", 
                    category.to_string().to_lowercase(), i + 1);
                    
                let example = format!("Example of {}: my_code_{}", 
                    category.to_string().to_lowercase(), i + 1);
                    
                // Create documentation
                let doc = DocBuilder::new(&parser_name, category.clone())
                    .description(description)
                    .example(example)
                    .build();
                    
                // Create a dummy parser with that documentation
                struct DummyParser {
                    doc: ParserDocumentation
                }
                
                impl<I, O> kairei_core::analyzer::core::Parser<I, O> for DummyParser {
                    fn parse(&self, _input: &[I], _pos: usize) -> kairei_core::analyzer::core::ParseResult<O> {
                        panic!("This is a dummy parser for testing");
                    }
                }
                
                impl<I, O> DocParserExt<I, O> for DummyParser {
                    fn documentation(&self) -> &ParserDocumentation {
                        &self.doc
                    }
                }
                
                let parser = DummyParser { doc };
                
                // Box it with the right type for the collector
                let boxed: Box<dyn DocParserExt<Token, Box<dyn Any>>> = 
                    unsafe { std::mem::transmute(Box::new(parser) as Box<dyn DocParserExt<(), ()>>) };
                    
                result.push(boxed);
            }
        }
        
        result
    }
}

/// Integration test for the documentation collection system
#[test]
fn test_documentation_system_integration() {
    // Setup categories to test
    let categories = vec![
        ParserCategory::Expression,
        ParserCategory::Statement,
        ParserCategory::Handler,
        ParserCategory::Definition,
    ];
    
    // Create a collector
    let mut collector = DocumentationCollector::new();
    
    // Register a provider that generates documented parsers
    collector.register_provider(Box::new(
        TestParserDocumentationProvider::new(categories.clone(), 3)
    ));
    
    // Collect documentation
    collector.collect();
    
    // Get the collection
    let collection = collector.get_collection();
    
    // Verify the collection has the right number of entries
    assert_eq!(collection.count(), categories.len() * 3);
    
    // Check that each category has the right number of parsers
    for category in &categories {
        let docs = collection.get_by_category(category);
        assert_eq!(docs.len(), 3);
    }
    
    // Test the relation graph - in our test data, there are no related parsers
    let graph = collection.build_relation_graph();
    assert_eq!(graph.len(), categories.len() * 3);
    
    // Test validation - all our test documentation is valid
    let issues = collection.validate();
    assert_eq!(issues.len(), 0);
    
    // Test serialization by verifying we can generate markdown
    let mut markdown = String::new();
    
    // Table of contents
    markdown.push_str("# KAIREI DSL Documentation\n\n");
    
    // Categories
    for category in &categories {
        markdown.push_str(&format!("## {} Syntax\n\n", category));
        
        // Get docs for this category
        let docs = collection.get_by_category(category);
        
        // Print each parser's documentation
        for doc in docs {
            markdown.push_str(&format!("### {}\n\n", doc.name));
            markdown.push_str(&format!("{}\n\n", doc.description));
            
            if !doc.examples.is_empty() {
                markdown.push_str("**Examples:**\n\n");
                for example in &doc.examples {
                    markdown.push_str(&format!("```kairei\n{}\n```\n\n", example));
                }
            }
            
            markdown.push_str("---\n\n");
        }
    }
    
    // Verify the markdown was generated
    assert!(markdown.len() > 0);
    
    // Test with multiple providers
    let mut collector = DocumentationCollector::new();
    
    // Register multiple providers
    collector.register_provider(Box::new(
        TestParserDocumentationProvider::new(vec![ParserCategory::Expression], 2)
    ));
    
    collector.register_provider(Box::new(
        TestParserDocumentationProvider::new(vec![ParserCategory::Statement], 2)
    ));
    
    // Collect documentation
    collector.collect();
    
    // Get the collection
    let collection = collector.get_collection();
    
    // Verify the collection has the right number of entries
    assert_eq!(collection.count(), 4);
    
    // Check that each category has the right number of parsers
    assert_eq!(collection.get_by_category(&ParserCategory::Expression).len(), 2);
    assert_eq!(collection.get_by_category(&ParserCategory::Statement).len(), 2);
}

/// Test with related parsers to verify cross-references
#[test]
fn test_documentation_system_with_related_parsers() {
    // Create a provider that generates related parsers
    struct RelatedParsersProvider {}
    
    impl kairei_core::analyzer::DocumentationProvider for RelatedParsersProvider {
        fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
            let mut result = Vec::new();
            
            // Create expression parser documentation
            let expr_doc = DocBuilder::new("parse_expression", ParserCategory::Expression)
                .description("Parses any expression")
                .example("a + b * c")
                .related_parser("parse_binary_expression")
                .related_parser("parse_primary_expression")
                .build();
                
            let binary_expr_doc = DocBuilder::new("parse_binary_expression", ParserCategory::Expression)
                .description("Parses binary operations")
                .example("a + b")
                .example("x * y")
                .related_parser("parse_expression")
                .related_parser("parse_primary_expression")
                .build();
                
            let primary_expr_doc = DocBuilder::new("parse_primary_expression", ParserCategory::Expression)
                .description("Parses primary expressions like literals and variables")
                .example("42")
                .example("my_var")
                .related_parser("parse_expression")
                .build();
                
            // Create dummy parsers
            struct DummyParser {
                doc: ParserDocumentation
            }
            
            impl<I, O> kairei_core::analyzer::core::Parser<I, O> for DummyParser {
                fn parse(&self, _input: &[I], _pos: usize) -> kairei_core::analyzer::core::ParseResult<O> {
                    panic!("This is a dummy parser for testing");
                }
            }
            
            impl<I, O> DocParserExt<I, O> for DummyParser {
                fn documentation(&self) -> &ParserDocumentation {
                    &self.doc
                }
            }
            
            // Create parsers with the documentation
            let expr_parser = DummyParser { doc: expr_doc };
            let binary_expr_parser = DummyParser { doc: binary_expr_doc };
            let primary_expr_parser = DummyParser { doc: primary_expr_doc };
            
            // Box them with the right type
            unsafe {
                result.push(std::mem::transmute(Box::new(expr_parser) as Box<dyn DocParserExt<(), ()>>));
                result.push(std::mem::transmute(Box::new(binary_expr_parser) as Box<dyn DocParserExt<(), ()>>));
                result.push(std::mem::transmute(Box::new(primary_expr_parser) as Box<dyn DocParserExt<(), ()>>));
            }
            
            result
        }
    }
    
    // Create a collector
    let mut collector = DocumentationCollector::new();
    
    // Register the provider
    collector.register_provider(Box::new(RelatedParsersProvider {}));
    
    // Collect documentation
    collector.collect();
    
    // Get the collection
    let collection = collector.get_collection();
    
    // Verify the collection has the right number of entries
    assert_eq!(collection.count(), 3);
    
    // Check that all parsers are in the Expression category
    assert_eq!(collection.get_by_category(&ParserCategory::Expression).len(), 3);
    
    // Test the relation graph
    let graph = collection.build_relation_graph();
    
    // Verify that each parser has the expected number of related parsers
    assert_eq!(graph["parse_expression"].len(), 2);
    assert_eq!(graph["parse_binary_expression"].len(), 2);
    assert_eq!(graph["parse_primary_expression"].len(), 2);
    
    // Verify that the relationships are bidirectional
    assert!(graph["parse_expression"].contains(&"parse_binary_expression".to_string()));
    assert!(graph["parse_expression"].contains(&"parse_primary_expression".to_string()));
    
    assert!(graph["parse_binary_expression"].contains(&"parse_expression".to_string()));
    assert!(graph["parse_binary_expression"].contains(&"parse_primary_expression".to_string()));
    
    assert!(graph["parse_primary_expression"].contains(&"parse_expression".to_string()));
    assert!(graph["parse_primary_expression"].contains(&"parse_binary_expression".to_string()));
    
    // Test validation - all our test documentation is valid
    let issues = collection.validate();
    assert_eq!(issues.len(), 0);
}

/// Test validation of documentation
#[test]
fn test_documentation_validation() {
    // Create a provider that generates invalid documentation
    struct InvalidDocumentationProvider {}
    
    impl kairei_core::analyzer::DocumentationProvider for InvalidDocumentationProvider {
        fn provide_documented_parsers(&self) -> Vec<Box<dyn DocParserExt<Token, Box<dyn Any>>>> {
            let mut result = Vec::new();
            
            // Create invalid documentation
            
            // 1. Missing description
            let mut missing_desc_doc = DocBuilder::new("missing_description", ParserCategory::Expression)
                .example("example")
                .build();
            missing_desc_doc.description = "".to_string();
            
            // 2. Missing examples
            let missing_examples_doc = DocBuilder::new("missing_examples", ParserCategory::Expression)
                .description("A parser with no examples")
                .build();
                
            // 3. Non-existent related parser
            let bad_reference_doc = DocBuilder::new("bad_reference", ParserCategory::Expression)
                .description("A parser with a reference to a non-existent parser")
                .example("example")
                .related_parser("non_existent_parser")
                .build();
                
            // Create dummy parser struct
            struct DummyParser {
                doc: ParserDocumentation
            }
            
            impl<I, O> kairei_core::analyzer::core::Parser<I, O> for DummyParser {
                fn parse(&self, _input: &[I], _pos: usize) -> kairei_core::analyzer::core::ParseResult<O> {
                    panic!("This is a dummy parser for testing");
                }
            }
            
            impl<I, O> DocParserExt<I, O> for DummyParser {
                fn documentation(&self) -> &ParserDocumentation {
                    &self.doc
                }
            }
            
            // Create parsers with the invalid documentation
            let missing_desc_parser = DummyParser { doc: missing_desc_doc };
            let missing_examples_parser = DummyParser { doc: missing_examples_doc };
            let bad_reference_parser = DummyParser { doc: bad_reference_doc };
            
            // Box them with the right type
            unsafe {
                result.push(std::mem::transmute(Box::new(missing_desc_parser) as Box<dyn DocParserExt<(), ()>>));
                result.push(std::mem::transmute(Box::new(missing_examples_parser) as Box<dyn DocParserExt<(), ()>>));
                result.push(std::mem::transmute(Box::new(bad_reference_parser) as Box<dyn DocParserExt<(), ()>>));
            }
            
            result
        }
    }
    
    // Create a collector
    let mut collector = DocumentationCollector::new();
    
    // Register the provider
    collector.register_provider(Box::new(InvalidDocumentationProvider {}));
    
    // Collect documentation
    collector.collect();
    
    // Validate
    let issues = collector.validate();
    
    // Verify the issues
    assert_eq!(issues.len(), 3);
    
    // Check for specific issues
    assert!(issues.iter().any(|issue| issue.contains("missing_description") && issue.contains("empty description")));
    assert!(issues.iter().any(|issue| issue.contains("missing_examples") && issue.contains("no examples")));
    assert!(issues.iter().any(|issue| issue.contains("bad_reference") && issue.contains("non-existent related parser")));
}
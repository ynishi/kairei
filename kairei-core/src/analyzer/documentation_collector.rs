//! # Documentation Collector
//!
//! This module provides a system for collecting and organizing documentation
//! from self-documenting parsers. It allows the KAIREI system to generate
//! comprehensive documentation for the DSL based on the actual parser implementations.

use crate::analyzer::doc_parser::{DocParserExt, ParserCategory, ParserDocumentation};
use crate::tokenizer::token::Token;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a collection of parser documentation organized by category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentationCollection {
    /// Documentation entries organized by category
    pub by_category: HashMap<ParserCategory, Vec<ParserDocumentation>>,
    /// Documentation entries organized by name
    pub by_name: HashMap<String, ParserDocumentation>,
}

impl DocumentationCollection {
    /// Creates a new empty documentation collection.
    pub fn new() -> Self {
        Self {
            by_category: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    /// Adds a parser's documentation to the collection.
    ///
    /// # Arguments
    ///
    /// * `doc` - The documentation to add
    pub fn add(&mut self, doc: ParserDocumentation) {
        // Add to by_category map
        self.by_category
            .entry(doc.category.clone())
            .or_default()
            .push(doc.clone());

        // Add to by_name map
        self.by_name.insert(doc.name.clone(), doc);
    }

    /// Returns all documentation entries for a specific category.
    ///
    /// # Arguments
    ///
    /// * `category` - The parser category to filter by
    pub fn get_by_category(&self, category: &ParserCategory) -> Vec<&ParserDocumentation> {
        match self.by_category.get(category) {
            Some(docs) => docs.iter().collect(),
            None => Vec::new(),
        }
    }

    /// Returns a documentation entry by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the parser
    pub fn get_by_name(&self, name: &str) -> Option<&ParserDocumentation> {
        self.by_name.get(name)
    }

    /// Returns all documentation entries.
    pub fn get_all(&self) -> Vec<&ParserDocumentation> {
        self.by_name.values().collect()
    }

    /// Returns all categories that have documentation entries.
    pub fn get_categories(&self) -> Vec<&ParserCategory> {
        self.by_category.keys().collect()
    }

    /// Returns the number of documentation entries in the collection.
    pub fn count(&self) -> usize {
        self.by_name.len()
    }

    /// Builds a graph of related parsers.
    ///
    /// Returns a HashMap where the key is a parser name and the value is a
    /// list of parser names that are related to it.
    pub fn build_relation_graph(&self) -> HashMap<String, Vec<String>> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Build the graph
        for (name, doc) in &self.by_name {
            let related = doc.related_parsers.clone();
            graph.insert(name.clone(), related);
        }

        // Add bidirectional relationships
        let names: HashSet<String> = self.by_name.keys().cloned().collect();
        let mut additions: HashMap<String, Vec<String>> = HashMap::new();

        for (name, related) in &graph {
            for related_name in related {
                if names.contains(related_name) {
                    additions
                        .entry(related_name.clone())
                        .or_default()
                        .push(name.clone());
                }
            }
        }

        // Apply the bidirectional additions
        for (name, related) in additions {
            if let Some(existing) = graph.get_mut(&name) {
                for r in related {
                    if !existing.contains(&r) {
                        existing.push(r);
                    }
                }
            }
        }

        graph
    }

    /// Validates the collection, checking for completeness and consistency.
    ///
    /// Returns a list of validation issues found.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for parsers with empty descriptions
        for (name, doc) in &self.by_name {
            if doc.description.is_empty() {
                issues.push(format!("Parser '{}' has an empty description", name));
            }

            if doc.examples.is_empty() {
                issues.push(format!("Parser '{}' has no examples", name));
            }
        }

        // Check for related parsers that don't exist
        for (name, doc) in &self.by_name {
            for related in &doc.related_parsers {
                if !self.by_name.contains_key(related) {
                    issues.push(format!(
                        "Parser '{}' references non-existent related parser '{}'",
                        name, related
                    ));
                }
            }
        }

        issues
    }

    /// Export the collection to Markdown format.
    ///
    /// # Returns
    ///
    /// A string containing the Markdown representation of the documentation.
    pub fn export_markdown(&self) -> String {
        let mut md = String::new();

        // Title
        md.push_str("# KAIREI Language Documentation\n\n");

        // Table of Contents
        md.push_str("## Table of Contents\n\n");

        for category in self.get_categories() {
            md.push_str(&format!(
                "- [{}](#{})\n",
                category,
                category.to_string().to_lowercase().replace(" ", "-")
            ));
        }
        md.push('\n');

        // Categories
        for category in self.get_categories() {
            md.push_str(&format!("## {}\n\n", category));

            // Sort parsers alphabetically by name within category
            let mut parsers = self.get_by_category(category);
            parsers.sort_by(|a, b| a.name.cmp(&b.name));

            for doc in parsers {
                md.push_str(&format!("### {}\n\n", doc.name));
                md.push_str(&format!("{}\n\n", doc.description));

                if !doc.examples.is_empty() {
                    md.push_str("**Examples**:\n\n");
                    for example in &doc.examples {
                        md.push_str(&format!("```\n{}\n```\n\n", example));
                    }
                }

                if !doc.related_parsers.is_empty() {
                    md.push_str("**Related**:\n\n");
                    for related in &doc.related_parsers {
                        if self.by_name.contains_key(related) {
                            md.push_str(&format!("- [{}](#{})\n", related, related));
                        } else {
                            md.push_str(&format!("- {} (undefined)\n", related));
                        }
                    }
                    md.push('\n');
                }

                if let Some(deprecated) = &doc.deprecated {
                    md.push_str(&format!("**Deprecated**: {}\n\n", deprecated));
                }
            }
        }

        md
    }

    /// Export the collection to JSON format.
    ///
    /// # Returns
    ///
    /// A Result containing the JSON string representation of the documentation.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// A trait for systems that can provide documented parsers.
pub trait DocumentationProvider {
    /// Returns a list of documented parsers.
    fn provide_documented_parsers(
        &self,
    ) -> Vec<Box<dyn DocParserExt<Token, Box<dyn std::any::Any>>>>;
}

/// The central documentation collector that aggregates documentation from multiple providers.
#[derive(Default)]
pub struct DocumentationCollector {
    /// The collection of documentation
    collection: DocumentationCollection,
    /// Registered documentation providers
    providers: Vec<Box<dyn DocumentationProvider>>,
}

impl DocumentationCollector {
    /// Creates a new documentation collector.
    pub fn new() -> Self {
        Self {
            collection: DocumentationCollection::new(),
            providers: Vec::new(),
        }
    }

    /// Registers a documentation provider.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider to register
    pub fn register_provider(&mut self, provider: Box<dyn DocumentationProvider>) {
        self.providers.push(provider);
    }

    /// Collects documentation from all registered providers.
    pub fn collect(&mut self) {
        for provider in &self.providers {
            let parsers = provider.provide_documented_parsers();
            for parser in parsers {
                let doc = parser.documentation().clone();
                self.collection.add(doc);
            }
        }
    }

    /// Returns the collection of documentation.
    pub fn get_collection(&self) -> &DocumentationCollection {
        &self.collection
    }

    /// Returns a mutable reference to the collection of documentation.
    pub fn get_collection_mut(&mut self) -> &mut DocumentationCollection {
        &mut self.collection
    }

    /// Validates the collection, checking for completeness and consistency.
    ///
    /// Returns a list of validation issues found.
    pub fn validate(&self) -> Vec<String> {
        self.collection.validate()
    }

    /// Exports the collected documentation to Markdown format.
    ///
    /// # Returns
    ///
    /// A string containing the Markdown representation of the documentation.
    pub fn export_markdown(&self) -> String {
        self.collection.export_markdown()
    }

    /// Exports the collected documentation to JSON format.
    ///
    /// # Returns
    ///
    /// A Result containing the JSON string representation of the documentation.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        self.collection.export_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::doc_parser::DocBuilder;

    struct TestDocProvider {
        docs: Vec<ParserDocumentation>,
    }

    impl TestDocProvider {
        fn new() -> Self {
            // Create documentation
            let doc1 = DocBuilder::new("equal_42", ParserCategory::Expression)
                .description("Parses the integer 42")
                .example("42")
                .related_parser("equal_100")
                .build();

            let doc2 = DocBuilder::new("equal_100", ParserCategory::Expression)
                .description("Parses the integer 100")
                .example("100")
                .related_parser("equal_42")
                .build();

            let doc3 = DocBuilder::new("if_keyword", ParserCategory::Statement)
                .description("Parses the 'if' keyword")
                .example("if condition { /* code */ }")
                .build();

            Self {
                docs: vec![doc1, doc2, doc3],
            }
        }
    }

    impl DocumentationProvider for TestDocProvider {
        fn provide_documented_parsers(
            &self,
        ) -> Vec<Box<dyn DocParserExt<Token, Box<dyn std::any::Any>>>> {
            // In tests, we don't need actual parsers - we just need the documentation
            // Create dummy parsers with documentation for testing
            let mut result = Vec::new();

            for doc in &self.docs {
                // Create a simple struct that implements DocParserExt for testing
                struct DummyParser {
                    doc: ParserDocumentation,
                }

                impl<I, O> crate::analyzer::core::Parser<I, O> for DummyParser {
                    fn parse(
                        &self,
                        _input: &[I],
                        _pos: usize,
                    ) -> crate::analyzer::core::ParseResult<O> {
                        panic!("This is a dummy parser for testing documentation only")
                    }
                }

                impl<I, O> DocParserExt<I, O> for DummyParser {
                    fn documentation(&self) -> &ParserDocumentation {
                        &self.doc
                    }
                }

                let dummy = DummyParser { doc: doc.clone() };
                // This is safe for testing since we never call parse()
                let boxed: Box<dyn DocParserExt<Token, Box<dyn std::any::Any>>> = unsafe {
                    std::mem::transmute(Box::new(dummy) as Box<dyn DocParserExt<(), ()>>)
                };
                result.push(boxed);
            }

            result
        }
    }

    #[test]
    fn test_documentation_collection() {
        let mut collection = DocumentationCollection::new();

        // Create some test documentation
        let doc1 = DocBuilder::new("parser1", ParserCategory::Expression)
            .description("Parser 1 description")
            .example("example1")
            .related_parser("parser2")
            .build();

        let doc2 = DocBuilder::new("parser2", ParserCategory::Expression)
            .description("Parser 2 description")
            .example("example2")
            .related_parser("parser1")
            .build();

        let doc3 = DocBuilder::new("parser3", ParserCategory::Statement)
            .description("Parser 3 description")
            .example("example3")
            .build();

        // Add to collection
        collection.add(doc1);
        collection.add(doc2);
        collection.add(doc3);

        // Test by_category access
        let expressions = collection.get_by_category(&ParserCategory::Expression);
        assert_eq!(expressions.len(), 2);
        assert!(expressions.iter().any(|d| d.name == "parser1"));
        assert!(expressions.iter().any(|d| d.name == "parser2"));

        let statements = collection.get_by_category(&ParserCategory::Statement);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].name, "parser3");

        // Test by_name access
        let p1 = collection.get_by_name("parser1").unwrap();
        assert_eq!(p1.description, "Parser 1 description");

        // Test get_all
        assert_eq!(collection.get_all().len(), 3);

        // Test get_categories
        let categories = collection.get_categories();
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&&ParserCategory::Expression));
        assert!(categories.contains(&&ParserCategory::Statement));

        // Test build_relation_graph
        let graph = collection.build_relation_graph();
        assert_eq!(graph.len(), 3);
        assert_eq!(graph["parser1"], vec!["parser2"]);
        assert_eq!(graph["parser2"], vec!["parser1"]);
        assert!(graph["parser3"].is_empty());
    }

    #[test]
    fn test_documentation_collector() {
        let mut collector = DocumentationCollector::new();
        let provider = Box::new(TestDocProvider::new());

        collector.register_provider(provider);
        collector.collect();

        let collection = collector.get_collection();
        assert_eq!(collection.count(), 3);

        // Check expressions
        let expressions = collection.get_by_category(&ParserCategory::Expression);
        assert_eq!(expressions.len(), 2);
        assert!(expressions.iter().any(|d| d.name == "equal_42"));
        assert!(expressions.iter().any(|d| d.name == "equal_100"));

        // Check statements
        let statements = collection.get_by_category(&ParserCategory::Statement);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].name, "if_keyword");

        // Check relation graph
        let graph = collection.build_relation_graph();
        assert!(graph["equal_42"].contains(&"equal_100".to_string()));
        assert!(graph["equal_100"].contains(&"equal_42".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut collection = DocumentationCollection::new();

        // Add valid documentation
        let valid_doc = DocBuilder::new("valid_parser", ParserCategory::Expression)
            .description("Valid parser description")
            .example("example")
            .related_parser("related_parser")
            .build();
        collection.add(valid_doc);

        // Add invalid documentation (empty description)
        let mut invalid_doc1 = DocBuilder::new("invalid_parser1", ParserCategory::Expression)
            .example("example")
            .build();
        invalid_doc1.description = "".to_string();
        collection.add(invalid_doc1);

        // Add invalid documentation (no examples)
        let invalid_doc2 = DocBuilder::new("invalid_parser2", ParserCategory::Expression)
            .description("Description without examples")
            .build();
        collection.add(invalid_doc2);

        // Add documentation with non-existent related parser
        let invalid_doc3 = DocBuilder::new("invalid_parser3", ParserCategory::Expression)
            .description("Description with bad reference")
            .example("example")
            .related_parser("non_existent_parser")
            .build();
        collection.add(invalid_doc3);

        // Add the referenced parser
        let related_doc = DocBuilder::new("related_parser", ParserCategory::Expression)
            .description("Related parser")
            .example("example")
            .build();
        collection.add(related_doc);

        // Validate
        let issues = collection.validate();

        // Should have 3 issues: empty description, no examples, and non-existent parser
        assert_eq!(issues.len(), 3);
        assert!(
            issues
                .iter()
                .any(|i| i.contains("invalid_parser1") && i.contains("empty description"))
        );
        assert!(
            issues
                .iter()
                .any(|i| i.contains("invalid_parser2") && i.contains("no examples"))
        );
        assert!(
            issues
                .iter()
                .any(|i| i.contains("invalid_parser3") && i.contains("non-existent"))
        );
    }

    #[test]
    fn test_markdown_export() {
        let mut collection = DocumentationCollection::new();

        // Add some test documentation
        let doc1 = DocBuilder::new("parser1", ParserCategory::Expression)
            .description("Parser 1 description")
            .example("example1")
            .related_parser("parser2")
            .build();

        let doc2 = DocBuilder::new("parser2", ParserCategory::Expression)
            .description("Parser 2 description")
            .example("example2")
            .related_parser("parser1")
            .build();

        collection.add(doc1);
        collection.add(doc2);

        // Generate Markdown
        let markdown = collection.export_markdown();

        // Basic assertions to verify markdown structure
        assert!(markdown.contains("# KAIREI Language Documentation"));
        assert!(markdown.contains("## Table of Contents"));
        assert!(markdown.contains("## Expression"));
        assert!(markdown.contains("### parser1"));
        assert!(markdown.contains("### parser2"));
        assert!(markdown.contains("Parser 1 description"));
        assert!(markdown.contains("```\nexample1\n```"));
    }

    #[test]
    fn test_json_export() {
        let mut collection = DocumentationCollection::new();

        // Add some test documentation
        let doc = DocBuilder::new("test_parser", ParserCategory::Expression)
            .description("Test parser description")
            .example("example code")
            .build();

        collection.add(doc);

        // Generate JSON
        let json_result = collection.export_json();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.contains("test_parser"));
        assert!(json.contains("Test parser description"));
        assert!(json.contains("example code"));
    }
}

use kairei_core::analyzer::doc_parser::ParserCategory;
use kairei_core::analyzer::documentation_collector::DocumentationCollector;
use kairei_core::analyzer::documentation_collector::DocumentationProvider;
use kairei_core::analyzer::parsers::doc_answer_handlers::AnswerHandlerDocProvider;

#[test]
fn test_answer_handler_doc_provider() {
    // Create the provider
    let provider = AnswerHandlerDocProvider;

    // Get the documented parsers
    let parsers = provider.provide_documented_parsers();

    // Check that we have the expected number of parsers
    assert!(!parsers.is_empty());
    assert_eq!(parsers.len(), 4); // We should have 4 documented answer handler parsers

    // Check that all parsers have documentation
    for parser in &parsers {
        let doc = parser.documentation();
        assert!(!doc.name.is_empty());
        assert_eq!(doc.category, ParserCategory::Handler);
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
    }
}

#[test]
fn test_collecting_answer_handler_docs() {
    // Create a documentation collector
    let mut collector = DocumentationCollector::new();

    // Register the answer handler documentation provider
    collector.register_provider(Box::new(AnswerHandlerDocProvider));

    // Collect the documentation
    collector.collect();

    // Get the collection
    let collection = collector.get_collection();

    // Verify that we collected documentation for handler parsers
    let handler_docs = collection.get_by_category(&ParserCategory::Handler);
    assert!(!handler_docs.is_empty());
    assert!(handler_docs.len() >= 4); // We should have at least 4 documented handler parsers

    // Check specific parsers
    let answer_parser = collection.get_by_name("parse_answer");
    assert!(answer_parser.is_some());

    let constraints_parser = collection.get_by_name("parse_constraints");
    assert!(constraints_parser.is_some());

    // Validate the collection
    let validation_issues = collector.validate();
    assert!(validation_issues.is_empty());
}

use kairei_core::analyzer::doc_parser::ParserCategory;
use kairei_core::analyzer::documentation_collector::DocumentationCollector;
use kairei_core::analyzer::documentation_collector::DocumentationProvider;
use kairei_core::analyzer::parsers::doc_event_handlers::EventHandlerDocProvider;

#[test]
fn test_event_handler_doc_provider() {
    // Create the provider
    let provider = EventHandlerDocProvider;

    // Get the documented parsers
    let parsers = DocumentationProvider::provide_documented_parsers(&provider);

    // Check that we have the expected number of parsers
    assert!(!parsers.is_empty());
    assert!(parsers.len() >= 4); // We should have at least 4 documented event handler parsers

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
fn test_collecting_event_handler_docs() {
    // Create a documentation collector
    let mut collector = DocumentationCollector::new();

    // Register the event handler documentation provider
    collector.register_provider(Box::new(EventHandlerDocProvider));

    // Collect the documentation
    collector.collect();

    // Get the collection
    let collection = collector.get_collection();

    // Verify that we collected documentation for event handler parsers
    let handler_docs = collection.get_by_category(&ParserCategory::Handler);
    assert!(!handler_docs.is_empty());
    assert!(handler_docs.len() >= 4); // We should have at least 4 documented event handler parsers

    // Check specific parsers
    let observe_parser = collection.get_by_name("parse_observe");
    assert!(observe_parser.is_some());

    let react_parser = collection.get_by_name("parse_react");
    assert!(react_parser.is_some());

    // Validate the collection
    let validation_issues = collector.validate();

    // Print any validation issues for debugging
    if !validation_issues.is_empty() {
        println!("Validation issues:");
        for issue in &validation_issues {
            println!("  - {}", issue);
        }
    }

    // Now validate there are no issues
    assert!(validation_issues.is_empty());
}

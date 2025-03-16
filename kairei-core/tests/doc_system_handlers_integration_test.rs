use kairei_core::analyzer::doc_parser::ParserCategory;
use kairei_core::analyzer::documentation_collector::DocumentationCollector;
use kairei_core::analyzer::documentation_collector::DocumentationProvider;
use kairei_core::analyzer::parsers::doc_system_handlers::SystemHandlerDocProvider;

#[test]
fn test_system_handler_doc_provider() {
    // Create the provider
    let provider = SystemHandlerDocProvider;

    // Get the documented parsers
    let parsers = DocumentationProvider::provide_documented_parsers(&provider);

    // Check that we have the expected number of parsers
    assert!(!parsers.is_empty());
    assert_eq!(parsers.len(), 4); // We should have 4 documented system handler parsers

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
fn test_collecting_system_handler_docs() {
    // Create a documentation collector
    let mut collector = DocumentationCollector::new();

    // Register the system handler documentation provider
    collector.register_provider(Box::new(SystemHandlerDocProvider));

    // Collect the documentation
    collector.collect();

    // Get the collection
    let collection = collector.get_collection();

    // Verify that we collected documentation for system handler parsers
    let handler_docs = collection.get_by_category(&ParserCategory::Handler);
    assert!(!handler_docs.is_empty());
    assert!(handler_docs.len() >= 4); // We should have at least 4 documented system handler parsers

    // Check specific parsers
    let handlers_parser = collection.get_by_name("parse_handlers");
    assert!(handlers_parser.is_some());

    let handler_def_parser = collection.get_by_name("parse_handler_def");
    assert!(handler_def_parser.is_some());

    let events_parser = collection.get_by_name("parse_events");
    assert!(events_parser.is_some());

    let event_parser = collection.get_by_name("parse_event");
    assert!(event_parser.is_some());

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

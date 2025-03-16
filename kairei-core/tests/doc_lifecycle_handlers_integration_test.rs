use kairei_core::analyzer::doc_parser::ParserCategory;
use kairei_core::analyzer::documentation_collector::DocumentationCollector;
use kairei_core::analyzer::documentation_collector::DocumentationProvider;
use kairei_core::analyzer::parsers::doc_lifecycle_handlers::LifecycleHandlerDocProvider;

#[test]
fn test_lifecycle_handler_doc_provider() {
    // Create the provider
    let provider = LifecycleHandlerDocProvider;

    // Get the documented parsers
    let parsers = DocumentationProvider::provide_documented_parsers(&provider);

    // Check that we have the expected number of parsers
    assert!(!parsers.is_empty());
    assert_eq!(parsers.len(), 3); // We should have 3 documented lifecycle handler parsers

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
fn test_collecting_lifecycle_handler_docs() {
    // Create a documentation collector
    let mut collector = DocumentationCollector::new();

    // Register the lifecycle handler documentation provider
    collector.register_provider(Box::new(LifecycleHandlerDocProvider));

    // Collect the documentation
    collector.collect();

    // Get the collection
    let collection = collector.get_collection();

    // Verify that we collected documentation for lifecycle handler parsers
    let handler_docs = collection.get_by_category(&ParserCategory::Handler);
    assert!(!handler_docs.is_empty());
    assert!(handler_docs.len() >= 3); // We should have at least 3 documented lifecycle handler parsers

    // Check specific parsers
    let lifecycle_parser = collection.get_by_name("parse_lifecycle");
    assert!(lifecycle_parser.is_some());

    let init_parser = collection.get_by_name("parse_init_handler");
    assert!(init_parser.is_some());

    let destroy_parser = collection.get_by_name("parse_destroy_handler");
    assert!(destroy_parser.is_some());

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

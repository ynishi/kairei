use kairei_core::analyzer::ParserCategory;
use kairei_core::analyzer::documentation_collector::{
    DocumentationCollector, DocumentationProvider,
};
use kairei_core::analyzer::parsers::doc_types::TypeDocProvider;

#[test]
fn test_type_doc_provider() {
    // Create the provider
    let provider = TypeDocProvider;

    // Get the documented parsers
    let parsers = provider.provide_documented_parsers();

    // Verify that we have parsers
    assert!(!parsers.is_empty());

    // Verify that all parsers have documentation
    for parser in &parsers {
        let doc = parser.documentation();
        assert!(!doc.name.is_empty());
        assert!(!doc.description.is_empty());
        assert!(!doc.examples.is_empty());
        assert_eq!(doc.category, ParserCategory::Type);
    }
}

#[test]
fn test_collecting_type_docs() {
    // Create a documentation collector
    let mut collector = DocumentationCollector::new();

    // Register the type documentation provider
    collector.register_provider(Box::new(TypeDocProvider));

    // Collect the documentation
    collector.collect();

    // Get the collection
    let collection = collector.get_collection();

    // Verify that we have type documentation
    let type_docs = collection.get_by_category(&ParserCategory::Type);
    assert!(!type_docs.is_empty());

    // Verify that we have specific parsers
    assert!(collection.get_by_name("parse_type_info").is_some());
    assert!(collection.get_by_name("parse_custom_type").is_some());
    assert!(collection.get_by_name("parse_option_type").is_some());
    assert!(collection.get_by_name("parse_result_type").is_some());
}

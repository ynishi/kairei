//! Tests for the documentation reference system.

use crate::provider::config::{
    doc_references, ProviderConfigError, ProviderError, SchemaError, ValidationError,
};

#[test]
fn test_schema_doc_references() {
    // Test that schema documentation references are correctly defined
    assert!(doc_references::schema::missing_field().contains(doc_references::anchors::SCHEMA));
    assert!(doc_references::schema::invalid_type().contains(doc_references::anchors::SCHEMA));
    assert!(doc_references::schema::invalid_structure().contains(doc_references::anchors::SCHEMA));
}

#[test]
fn test_validation_doc_references() {
    // Test that validation documentation references are correctly defined
    assert!(doc_references::validation::invalid_value()
        .contains(doc_references::anchors::ERROR_HANDLING));
    assert!(doc_references::validation::constraint_violation()
        .contains(doc_references::anchors::ERROR_HANDLING));
    assert!(doc_references::validation::dependency_error()
        .contains(doc_references::anchors::ERROR_HANDLING));
}

#[test]
fn test_provider_doc_references() {
    // Test that provider documentation references are correctly defined
    assert!(doc_references::provider::initialization()
        .contains(doc_references::anchors::TROUBLESHOOTING));
    assert!(
        doc_references::provider::capability().contains(doc_references::anchors::TROUBLESHOOTING)
    );
    assert!(doc_references::provider::configuration()
        .contains(doc_references::anchors::TROUBLESHOOTING));
}

#[test]
fn test_schema_error_with_documentation() {
    // Test that schema errors include documentation references
    let error = SchemaError::missing_field("test_field");
    if let SchemaError::MissingField { context } = error {
        assert!(context.documentation.is_some());
        assert!(context
            .documentation
            .unwrap()
            .contains(doc_references::anchors::SCHEMA));
    } else {
        panic!("Expected MissingField error");
    }

    let error = SchemaError::invalid_type("test_field", "string", "number");
    if let SchemaError::InvalidType { context, .. } = error {
        assert!(context.documentation.is_some());
        assert!(context
            .documentation
            .unwrap()
            .contains(doc_references::anchors::SCHEMA));
    } else {
        panic!("Expected InvalidType error");
    }

    let error = SchemaError::invalid_structure("test_field", "Invalid structure");
    if let SchemaError::InvalidStructure { context, .. } = error {
        assert!(context.documentation.is_some());
        assert!(context
            .documentation
            .unwrap()
            .contains(doc_references::anchors::SCHEMA));
    } else {
        panic!("Expected InvalidStructure error");
    }
}

#[test]
fn test_validation_error_with_documentation() {
    // Test that validation errors include documentation references
    let error = ValidationError::invalid_value("test_field", "Invalid value");
    if let ValidationError::InvalidValue { context, .. } = error {
        assert!(context.documentation.is_some());
        assert!(context
            .documentation
            .unwrap()
            .contains(doc_references::anchors::ERROR_HANDLING));
    } else {
        panic!("Expected InvalidValue error");
    }

    let error = ValidationError::constraint_violation("test_field", "Constraint violation");
    if let ValidationError::ConstraintViolation { context, .. } = error {
        assert!(context.documentation.is_some());
        assert!(context
            .documentation
            .unwrap()
            .contains(doc_references::anchors::ERROR_HANDLING));
    } else {
        panic!("Expected ConstraintViolation error");
    }

    let error = ValidationError::dependency_error("test_field", "Dependency error");
    if let ValidationError::DependencyError { context, .. } = error {
        assert!(context.documentation.is_some());
        assert!(context
            .documentation
            .unwrap()
            .contains(doc_references::anchors::ERROR_HANDLING));
    } else {
        panic!("Expected DependencyError error");
    }
}

#[test]
fn test_provider_error_with_documentation() {
    // Test that provider errors include documentation references
    let error = ProviderError::initialization("test_field", "Initialization error");
    if let ProviderError::Initialization { context, .. } = error {
        assert!(context.documentation.is_some());
        assert!(context
            .documentation
            .unwrap()
            .contains(doc_references::anchors::TROUBLESHOOTING));
    } else {
        panic!("Expected Initialization error");
    }

    let error = ProviderError::capability("test_field", "Capability error");
    if let ProviderError::Capability { context, .. } = error {
        assert!(context.documentation.is_some());
        assert!(context
            .documentation
            .unwrap()
            .contains(doc_references::anchors::TROUBLESHOOTING));
    } else {
        panic!("Expected Capability error");
    }

    let error = ProviderError::configuration("test_field", "Configuration error");
    if let ProviderError::Configuration { context, .. } = error {
        assert!(context.documentation.is_some());
        assert!(context
            .documentation
            .unwrap()
            .contains(doc_references::anchors::TROUBLESHOOTING));
    } else {
        panic!("Expected Configuration error");
    }
}

#[test]
fn test_error_display_with_documentation() {
    // Test that error display includes documentation references
    let error = SchemaError::missing_field("test_field");
    let error_string = format!("{}", error);
    assert!(error_string.contains("Missing required field"));
    assert!(error_string.contains("(see: "));
    assert!(error_string.contains(doc_references::anchors::SCHEMA));

    let error = ValidationError::invalid_value("test_field", "Invalid value");
    let error_string = format!("{}", error);
    assert!(error_string.contains("Invalid value"));
    assert!(error_string.contains("(see: "));
    assert!(error_string.contains(doc_references::anchors::ERROR_HANDLING));

    let error = ProviderError::initialization("test_field", "Initialization error");
    let error_string = format!("{}", error);
    assert!(error_string.contains("Provider initialization error"));
    assert!(error_string.contains("(see: "));
    assert!(error_string.contains(doc_references::anchors::TROUBLESHOOTING));
}

#[test]
fn test_provider_config_error_with_documentation() {
    // Test that provider config errors include documentation references
    let schema_error = SchemaError::missing_field("test_field");
    let provider_config_error = ProviderConfigError::Schema(schema_error);
    let error_string = format!("{}", provider_config_error);
    assert!(error_string.contains("Schema error"));
    assert!(error_string.contains("(see: "));
    assert!(error_string.contains(doc_references::anchors::SCHEMA));

    let validation_error = ValidationError::invalid_value("test_field", "Invalid value");
    let provider_config_error = ProviderConfigError::Validation(validation_error);
    let error_string = format!("{}", provider_config_error);
    assert!(error_string.contains("Validation error"));
    assert!(error_string.contains("(see: "));
    assert!(error_string.contains(doc_references::anchors::ERROR_HANDLING));

    let provider_error = ProviderError::initialization("test_field", "Initialization error");
    let provider_config_error = ProviderConfigError::Provider(provider_error);
    let error_string = format!("{}", provider_config_error);
    assert!(error_string.contains("Provider error"));
    assert!(error_string.contains("(see: "));
    assert!(error_string.contains(doc_references::anchors::TROUBLESHOOTING));
}

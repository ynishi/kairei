//! Expanded unit tests for provider configuration error types.

use crate::provider::config::errors::{
    ErrorContext, ErrorSeverity, ProviderConfigError, ProviderError, SchemaError, SourceLocation,
    ValidationError,
};

#[test]
fn test_schema_error_variants() {
    // Test SchemaError variants that exist in the implementation
    let missing_field = SchemaError::missing_field("test_field");
    let invalid_type = SchemaError::invalid_type("test_field", "string", "number");
    let invalid_structure = SchemaError::invalid_structure("test_field", "Field must be an object");

    // Verify each variant
    match missing_field {
        SchemaError::MissingField { context } => {
            assert_eq!(context.location.field, Some("test_field".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Error);
        }
        _ => panic!("Expected MissingField variant"),
    }

    match invalid_type {
        SchemaError::InvalidType {
            expected,
            actual,
            context,
        } => {
            assert_eq!(expected, "string");
            assert_eq!(actual, "number");
            assert_eq!(context.location.field, Some("test_field".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Error);
        }
        _ => panic!("Expected InvalidType variant"),
    }

    match invalid_structure {
        SchemaError::InvalidStructure { message, context } => {
            assert_eq!(message, "Field must be an object");
            assert_eq!(context.location.field, Some("test_field".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Error);
        }
        _ => panic!("Expected InvalidStructure variant"),
    }
}

#[test]
fn test_validation_error_variants() {
    // Test ValidationError variants that exist in the implementation
    let invalid_value = ValidationError::invalid_value("test_field", "Value must be positive");
    let constraint_violation =
        ValidationError::constraint_violation("test_field", "Value must be less than 100");
    let dependency_error =
        ValidationError::dependency_error("test_field", "Depends on missing field 'other_field'");

    // Verify each variant
    match invalid_value {
        ValidationError::InvalidValue { message, context } => {
            assert_eq!(message, "Value must be positive");
            assert_eq!(context.location.field, Some("test_field".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Error);
        }
        _ => panic!("Expected InvalidValue variant"),
    }

    match constraint_violation {
        ValidationError::ConstraintViolation { message, context } => {
            assert_eq!(message, "Value must be less than 100");
            assert_eq!(context.location.field, Some("test_field".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Error);
        }
        _ => panic!("Expected ConstraintViolation variant"),
    }

    match dependency_error {
        ValidationError::DependencyError { message, context } => {
            assert_eq!(message, "Depends on missing field 'other_field'");
            assert_eq!(context.location.field, Some("test_field".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Error);
        }
        _ => panic!("Expected DependencyError variant"),
    }
}

#[test]
fn test_provider_error_variants() {
    // Test ProviderError variants that exist in the implementation
    let initialization =
        ProviderError::initialization("test_field", "Failed to initialize provider");
    let capability = ProviderError::capability("test_field", "Required capability not supported");
    let configuration =
        ProviderError::configuration("test_field", "Invalid provider configuration");

    // Verify each variant
    match initialization {
        ProviderError::Initialization { message, context } => {
            assert_eq!(message, "Failed to initialize provider");
            assert_eq!(context.location.field, Some("test_field".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Critical); // Note: This is Critical, not Error
        }
        _ => panic!("Expected Initialization variant"),
    }

    match capability {
        ProviderError::Capability { message, context } => {
            assert_eq!(message, "Required capability not supported");
            assert_eq!(context.location.field, Some("test_field".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Error);
        }
        _ => panic!("Expected Capability variant"),
    }

    match configuration {
        ProviderError::Configuration { message, context } => {
            assert_eq!(message, "Invalid provider configuration");
            assert_eq!(context.location.field, Some("test_field".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Error);
        }
        _ => panic!("Expected Configuration variant"),
    }
}

#[test]
fn test_error_context_with_all_fields() {
    // Test creating an ErrorContext with all fields
    let context = ErrorContext::new_with_field("test_field")
        .with_severity(ErrorSeverity::Warning)
        .with_documentation("https://docs.example.com")
        .with_suggestion("Try using a different value")
        .with_error_code("TEST_001")
        .with_additional_context("Additional context information");

    // Verify all fields
    assert_eq!(context.location.field, Some("test_field".to_string()));
    assert_eq!(context.severity, ErrorSeverity::Warning);
    assert_eq!(
        context.documentation,
        Some("https://docs.example.com".to_string())
    );
    assert_eq!(
        context.suggestion,
        Some("Try using a different value".to_string())
    );
    assert_eq!(context.error_code, Some("TEST_001".to_string()));
    assert_eq!(
        context.additional_context,
        Some("Additional context information".to_string())
    );
}

#[test]
fn test_provider_config_error_conversion() {
    // Test conversion from SchemaError to ProviderConfigError
    let schema_error = SchemaError::missing_field("test_field");
    let provider_config_error: ProviderConfigError = schema_error.into();
    match provider_config_error {
        ProviderConfigError::Schema(_) => {}
        _ => panic!("Expected Schema variant"),
    }

    // Test conversion from ValidationError to ProviderConfigError
    let validation_error = ValidationError::invalid_value("test_field", "Value must be positive");
    let provider_config_error: ProviderConfigError = validation_error.into();
    match provider_config_error {
        ProviderConfigError::Validation(_) => {}
        _ => panic!("Expected Validation variant"),
    }

    // Test conversion from ProviderError to ProviderConfigError
    let provider_error =
        ProviderError::initialization("test_field", "Failed to initialize provider");
    let provider_config_error: ProviderConfigError = provider_error.into();
    match provider_config_error {
        ProviderConfigError::Provider(_) => {}
        _ => panic!("Expected Provider variant"),
    }
}

#[test]
fn test_error_severity_comparison() {
    // Test severity equality
    assert_eq!(ErrorSeverity::Critical, ErrorSeverity::Critical);
    assert_eq!(ErrorSeverity::Error, ErrorSeverity::Error);
    assert_eq!(ErrorSeverity::Warning, ErrorSeverity::Warning);
    assert_eq!(ErrorSeverity::Info, ErrorSeverity::Info);

    // Note: ErrorSeverity doesn't implement PartialOrd in the current implementation
    // so we can't directly compare severities with > or <
}

#[test]
fn test_source_location_builder() {
    // Test building a SourceLocation
    let location = SourceLocation::new_with_field("test_field");

    assert_eq!(location.field, Some("test_field".to_string()));
    assert_eq!(location.file, None);
    assert_eq!(location.line, None);
    assert_eq!(location.column, None);

    // Test display formatting
    assert_eq!(location.to_string(), "in field 'test_field'");

    // Test with empty location
    let empty_location = SourceLocation::new();

    assert_eq!(empty_location.field, None);
    assert_eq!(empty_location.file, None);
    assert_eq!(empty_location.line, None);
    assert_eq!(empty_location.column, None);
}

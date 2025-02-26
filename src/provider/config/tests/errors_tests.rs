use crate::provider::config::{
    ErrorContext, ErrorSeverity, ProviderConfigError, ProviderError, SchemaError, SourceLocation,
    ValidationError,
};

#[test]
fn test_source_location_display() {
    let location = SourceLocation {
        file: Some("test.rs".to_string()),
        line: Some(42),
        column: Some(10),
        field: Some("test_field".to_string()),
    };

    assert_eq!(
        location.to_string(),
        "in field 'test_field' at test.rs:42:10"
    );

    let location_no_column = SourceLocation {
        file: Some("test.rs".to_string()),
        line: Some(42),
        column: None,
        field: Some("test_field".to_string()),
    };

    assert_eq!(
        location_no_column.to_string(),
        "in field 'test_field' at test.rs:42"
    );

    let location_field_only = SourceLocation {
        file: None,
        line: None,
        column: None,
        field: Some("test_field".to_string()),
    };

    assert_eq!(location_field_only.to_string(), "in field 'test_field'");
}

#[test]
fn test_error_severity_display() {
    assert_eq!(ErrorSeverity::Critical.to_string(), "CRITICAL");
    assert_eq!(ErrorSeverity::Error.to_string(), "ERROR");
    assert_eq!(ErrorSeverity::Warning.to_string(), "WARNING");
    assert_eq!(ErrorSeverity::Info.to_string(), "INFO");
}

#[test]
fn test_error_context_builder() {
    let context = ErrorContext::new_with_field("test_field")
        .with_severity(ErrorSeverity::Warning)
        .with_documentation("https://docs.example.com")
        .with_suggestion("Try using a different value")
        .with_error_code("TEST_001");

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
}

#[test]
fn test_schema_error_creation() {
    let missing_field = SchemaError::missing_field("test_field");
    match missing_field {
        SchemaError::MissingField { context } => {
            assert_eq!(context.location.field, Some("test_field".to_string()));
        }
        _ => panic!("Expected MissingField variant"),
    }

    let invalid_type = SchemaError::invalid_type("test_field", "string", "number");
    match invalid_type {
        SchemaError::InvalidType {
            expected,
            actual,
            context,
        } => {
            assert_eq!(expected, "string");
            assert_eq!(actual, "number");
            assert_eq!(context.location.field, Some("test_field".to_string()));
        }
        _ => panic!("Expected InvalidType variant"),
    }

    let invalid_structure =
        SchemaError::invalid_structure("test_field", "Field must be an object");
    match invalid_structure {
        SchemaError::InvalidStructure { message, context } => {
            assert_eq!(message, "Field must be an object");
            assert_eq!(context.location.field, Some("test_field".to_string()));
        }
        _ => panic!("Expected InvalidStructure variant"),
    }
}

#[test]
fn test_validation_error_creation() {
    let invalid_value = ValidationError::invalid_value("test_field", "Value must be positive");
    match invalid_value {
        ValidationError::InvalidValue { message, context } => {
            assert_eq!(message, "Value must be positive");
            assert_eq!(context.location.field, Some("test_field".to_string()));
        }
        _ => panic!("Expected InvalidValue variant"),
    }

    let constraint_violation =
        ValidationError::constraint_violation("test_field", "Value must be less than 100");
    match constraint_violation {
        ValidationError::ConstraintViolation { message, context } => {
            assert_eq!(message, "Value must be less than 100");
            assert_eq!(context.location.field, Some("test_field".to_string()));
        }
        _ => panic!("Expected ConstraintViolation variant"),
    }

    let dependency_error =
        ValidationError::dependency_error("test_field", "Depends on missing field 'other_field'");
    match dependency_error {
        ValidationError::DependencyError { message, context } => {
            assert_eq!(message, "Depends on missing field 'other_field'");
            assert_eq!(context.location.field, Some("test_field".to_string()));
        }
        _ => panic!("Expected DependencyError variant"),
    }
}

#[test]
fn test_provider_error_creation() {
    let initialization =
        ProviderError::initialization("test_field", "Failed to initialize provider");
    match initialization {
        ProviderError::Initialization { message, context } => {
            assert_eq!(message, "Failed to initialize provider");
            assert_eq!(context.location.field, Some("test_field".to_string()));
        }
        _ => panic!("Expected Initialization variant"),
    }

    let capability = ProviderError::capability("test_field", "Required capability not supported");
    match capability {
        ProviderError::Capability { message, context } => {
            assert_eq!(message, "Required capability not supported");
            assert_eq!(context.location.field, Some("test_field".to_string()));
        }
        _ => panic!("Expected Capability variant"),
    }

    let configuration =
        ProviderError::configuration("test_field", "Invalid provider configuration");
    match configuration {
        ProviderError::Configuration { message, context } => {
            assert_eq!(message, "Invalid provider configuration");
            assert_eq!(context.location.field, Some("test_field".to_string()));
        }
        _ => panic!("Expected Configuration variant"),
    }
}

#[test]
fn test_provider_config_error_conversion() {
    let schema_error = SchemaError::missing_field("test_field");
    let provider_config_error: ProviderConfigError = schema_error.into();
    match provider_config_error {
        ProviderConfigError::Schema(_) => {}
        _ => panic!("Expected Schema variant"),
    }

    let validation_error = ValidationError::invalid_value("test_field", "Value must be positive");
    let provider_config_error: ProviderConfigError = validation_error.into();
    match provider_config_error {
        ProviderConfigError::Validation(_) => {}
        _ => panic!("Expected Validation variant"),
    }

    let provider_error =
        ProviderError::initialization("test_field", "Failed to initialize provider");
    let provider_config_error: ProviderConfigError = provider_error.into();
    match provider_config_error {
        ProviderConfigError::Provider(_) => {}
        _ => panic!("Expected Provider variant"),
    }

    let generic_error = ProviderConfigError::generic("Generic error message");
    match generic_error {
        ProviderConfigError::Generic(message) => {
            assert_eq!(message, "Generic error message");
        }
        _ => panic!("Expected Generic variant"),
    }

    let legacy_error = ConfigError::MissingField("test_field".to_string());
    let provider_config_error: ProviderConfigError = legacy_error.into();
    match provider_config_error {
        ProviderConfigError::Legacy(_) => {}
        _ => panic!("Expected Legacy variant"),
    }
}

#[test]
fn test_error_code_generation() {
    let schema_error = SchemaError::missing_field("test_field");
    let provider_config_error: ProviderConfigError = schema_error.into();
    assert_eq!(provider_config_error.error_code(), "SCHEMA_0001");

    let validation_error = ValidationError::invalid_value("test_field", "Value must be positive");
    let provider_config_error: ProviderConfigError = validation_error.into();
    assert_eq!(provider_config_error.error_code(), "VALIDATION_0001");

    let provider_error =
        ProviderError::initialization("test_field", "Failed to initialize provider");
    let provider_config_error: ProviderConfigError = provider_error.into();
    assert_eq!(provider_config_error.error_code(), "PROVIDER_0001");

    let generic_error = ProviderConfigError::generic("Generic error message");
    assert_eq!(generic_error.error_code(), "GENERIC_0001");
    
    let legacy_error = ConfigError::MissingField("test_field".to_string());
    let provider_config_error: ProviderConfigError = legacy_error.into();
    assert_eq!(provider_config_error.error_code(), "LEGACY_0001");
}

#[test]
fn test_error_display() {
    let schema_error = SchemaError::missing_field("test_field");
    assert_eq!(
        schema_error.to_string(),
        "Missing required field in field 'test_field'"
    );

    let validation_error = ValidationError::invalid_value("test_field", "Value must be positive");
    assert_eq!(
        validation_error.to_string(),
        "Invalid value in field 'test_field': Value must be positive"
    );

    let provider_error =
        ProviderError::initialization("test_field", "Failed to initialize provider");
    assert_eq!(
        provider_error.to_string(),
        "Provider initialization error in field 'test_field': Failed to initialize provider"
    );

    let provider_config_error: ProviderConfigError = schema_error.into();
    assert_eq!(
        provider_config_error.to_string(),
        "Schema error: Missing required field in field 'test_field'"
    );
}

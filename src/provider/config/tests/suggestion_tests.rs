//! Tests for the suggestion generator.

use crate::provider::config::{
    base::ConfigError,
    errors::{
        ErrorContext, ProviderConfigError, ProviderError, SchemaError, SourceLocation,
        ValidationError,
    },
    suggestions::{DefaultSuggestionGenerator, SuggestionGenerator},
};

#[test]
fn test_default_generator_schema_missing_field() {
    let generator = DefaultSuggestionGenerator;
    let error = SchemaError::MissingField {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    };

    let suggestion = generator.generate_schema_suggestion(&error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Add the required 'test_field' field to your configuration"
    );
}

#[test]
fn test_default_generator_schema_invalid_type() {
    let generator = DefaultSuggestionGenerator;
    let error = SchemaError::InvalidType {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        expected: "string".to_string(),
        actual: "integer".to_string(),
    };

    let suggestion = generator.generate_schema_suggestion(&error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Change the type of 'test_field' from integer to string"
    );
}

#[test]
fn test_default_generator_schema_invalid_structure() {
    let generator = DefaultSuggestionGenerator;
    let error = SchemaError::InvalidStructure {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        message: "Invalid structure".to_string(),
    };

    let suggestion = generator.generate_schema_suggestion(&error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Fix the structure of 'test_field': Invalid structure"
    );
}

#[test]
fn test_default_generator_validation_invalid_value() {
    let generator = DefaultSuggestionGenerator;
    let error = ValidationError::InvalidValue {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        message: "Value must be positive".to_string(),
    };

    let suggestion = generator.generate_validation_suggestion(&error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Provide a valid value for 'test_field': Value must be positive"
    );
}

#[test]
fn test_default_generator_validation_constraint_violation() {
    let generator = DefaultSuggestionGenerator;
    let error = ValidationError::ConstraintViolation {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        message: "Value must be between 1 and 10".to_string(),
    };

    let suggestion = generator.generate_validation_suggestion(&error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Ensure 'test_field' meets the required constraints: Value must be between 1 and 10"
    );
}

#[test]
fn test_default_generator_validation_dependency_error() {
    let generator = DefaultSuggestionGenerator;
    let error = ValidationError::DependencyError {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        message: "Depends on 'other_field'".to_string(),
    };

    let suggestion = generator.generate_validation_suggestion(&error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Resolve dependency issues for 'test_field': Depends on 'other_field'"
    );
}

#[test]
fn test_default_generator_provider_initialization() {
    let generator = DefaultSuggestionGenerator;
    let error = ProviderError::Initialization {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        message: "Failed to initialize".to_string(),
    };

    let suggestion = generator.generate_provider_suggestion(&error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Fix initialization issues for 'test_field': Failed to initialize"
    );
}

#[test]
fn test_default_generator_provider_capability() {
    let generator = DefaultSuggestionGenerator;
    let error = ProviderError::Capability {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        message: "Missing required capability".to_string(),
    };

    let suggestion = generator.generate_provider_suggestion(&error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Ensure 'test_field' has the required capabilities: Missing required capability"
    );
}

#[test]
fn test_default_generator_provider_configuration() {
    let generator = DefaultSuggestionGenerator;
    let error = ProviderError::Configuration {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        message: "Invalid configuration".to_string(),
    };

    let suggestion = generator.generate_provider_suggestion(&error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Fix configuration issues for 'test_field': Invalid configuration"
    );
}

#[test]
fn test_generate_suggestion_for_provider_config_error() {
    let generator = DefaultSuggestionGenerator;

    // Test Schema error
    let schema_error = SchemaError::MissingField {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let provider_config_error = ProviderConfigError::Schema(schema_error);
    let suggestion = generator.generate_suggestion(&provider_config_error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Add the required 'test_field' field to your configuration"
    );

    // Test Validation error
    let validation_error = ValidationError::InvalidValue {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        message: "Value must be positive".to_string(),
    };
    let provider_config_error = ProviderConfigError::Validation(validation_error);
    let suggestion = generator.generate_suggestion(&provider_config_error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Provide a valid value for 'test_field': Value must be positive"
    );

    // Test Provider error
    let provider_error = ProviderError::Configuration {
        context: ErrorContext {
            location: SourceLocation {
                field: Some("test_field".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        message: "Invalid configuration".to_string(),
    };
    let provider_config_error = ProviderConfigError::Provider(provider_error);
    let suggestion = generator.generate_suggestion(&provider_config_error);
    assert!(suggestion.is_some());
    assert_eq!(
        suggestion.unwrap(),
        "Fix configuration issues for 'test_field': Invalid configuration"
    );

    // Test Generic error (should return None)
    let provider_config_error = ProviderConfigError::Generic("Generic error".to_string());
    let suggestion = generator.generate_suggestion(&provider_config_error);
    assert!(suggestion.is_none());

    // Test Legacy error (should return None)
    let provider_config_error =
        ProviderConfigError::Legacy(ConfigError::ValidationError("Legacy error".to_string()));
    let suggestion = generator.generate_suggestion(&provider_config_error);
    assert!(suggestion.is_none());
}

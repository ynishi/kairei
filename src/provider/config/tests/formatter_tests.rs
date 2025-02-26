//! Tests for the error formatter.

use crate::provider::config::{
    errors::{
        ErrorContext, ErrorSeverity, ProviderConfigError, ProviderError, SchemaError, SourceLocation,
        ValidationError,
    },
    formatters::{DefaultErrorFormatter, ErrorFormatter, FormatOptions},
};

#[test]
fn test_default_formatter_schema_error() {
    let formatter = DefaultErrorFormatter::default();
    let error = SchemaError::missing_field("test_field");
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: false,
    };

    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("Schema Error"));
    assert!(formatted.contains("Missing required field"));
    assert!(formatted.contains("test_field"));
    assert!(formatted.contains("[SCHEMA_0001]"));
    assert!(formatted.contains("Suggestion:"));
}

#[test]
fn test_default_formatter_validation_error() {
    let formatter = DefaultErrorFormatter::default();
    let error = ValidationError::invalid_value("test_field", "Value must be positive");
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: false,
    };

    let formatted = formatter.format_validation_error(&error, &options);
    assert!(formatted.contains("Validation Error"));
    assert!(formatted.contains("Invalid value"));
    assert!(formatted.contains("test_field"));
    assert!(formatted.contains("Value must be positive"));
    assert!(formatted.contains("[VALIDATION_0001]"));
}

#[test]
fn test_default_formatter_provider_error() {
    let formatter = DefaultErrorFormatter::default();
    let error = ProviderError::capability("test_field", "Missing required capability");
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: false,
    };

    let formatted = formatter.format_provider_error(&error, &options);
    assert!(formatted.contains("Provider Error"));
    assert!(formatted.contains("Provider capability error"));
    assert!(formatted.contains("test_field"));
    assert!(formatted.contains("Missing required capability"));
    assert!(formatted.contains("[PROVIDER_0002]"));
}

#[test]
fn test_default_formatter_with_color() {
    let formatter = DefaultErrorFormatter::default();
    let error = SchemaError::missing_field("test_field");
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: true,
    };

    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("\x1b[31mSchema Error:\x1b[0m"));
    assert!(formatted.contains("\x1b[90m[SCHEMA_0001]\x1b[0m"));
    assert!(formatted.contains("\x1b[32mSuggestion:\x1b[0m"));
}

#[test]
fn test_default_formatter_without_options() {
    let formatter = DefaultErrorFormatter::default();
    let error = SchemaError::missing_field("test_field");
    let options = FormatOptions::default();

    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("Schema Error"));
    assert!(formatted.contains("Missing required field"));
    assert!(formatted.contains("test_field"));
    assert!(!formatted.contains("[SCHEMA_0001]"));
    assert!(!formatted.contains("Suggestion:"));
}

#[test]
fn test_default_formatter_with_documentation() {
    let formatter = DefaultErrorFormatter::default();
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_documentation("See documentation at: https://example.com/docs");
    
    let error = SchemaError::InvalidStructure {
        message: "Invalid structure".to_string(),
        context,
    };
    
    let options = FormatOptions {
        include_error_codes: false,
        include_suggestions: false,
        include_documentation: true,
        use_color: false,
    };

    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("Schema Error"));
    assert!(formatted.contains("Invalid structure"));
    assert!(formatted.contains("Documentation: See documentation at: https://example.com/docs"));
}

#[test]
fn test_default_formatter_with_suggestion() {
    let formatter = DefaultErrorFormatter::default();
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_suggestion("Try adding the required field");
    
    let error = SchemaError::MissingField {
        context,
    };
    
    let options = FormatOptions {
        include_error_codes: false,
        include_suggestions: true,
        include_documentation: false,
        use_color: false,
    };

    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("Schema Error"));
    assert!(formatted.contains("Missing required field"));
    assert!(formatted.contains("Suggestion: Try adding the required field"));
}

#[test]
fn test_format_error() {
    let formatter = DefaultErrorFormatter::default();
    let options = FormatOptions::default();
    
    // Test Schema error
    let schema_error = SchemaError::missing_field("test_field");
    let provider_config_error = ProviderConfigError::Schema(schema_error);
    let formatted = formatter.format_error(&provider_config_error, &options);
    assert!(formatted.contains("Schema Error"));
    
    // Test Validation error
    let validation_error = ValidationError::invalid_value("test_field", "Value must be positive");
    let provider_config_error = ProviderConfigError::Validation(validation_error);
    let formatted = formatter.format_error(&provider_config_error, &options);
    assert!(formatted.contains("Validation Error"));
    
    // Test Provider error
    let provider_error = ProviderError::capability("test_field", "Missing required capability");
    let provider_config_error = ProviderConfigError::Provider(provider_error);
    let formatted = formatter.format_error(&provider_config_error, &options);
    assert!(formatted.contains("Provider Error"));
    
    // Test Generic error
    let provider_config_error = ProviderConfigError::Generic("Generic error message".to_string());
    let formatted = formatter.format_error(&provider_config_error, &options);
    assert!(formatted.contains("Configuration error: Generic error message"));
}

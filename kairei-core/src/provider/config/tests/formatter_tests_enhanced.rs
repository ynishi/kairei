//! Tests for the enhanced error formatter features.

use crate::provider::config::{
    errors::{ErrorContext, ErrorSeverity, ProviderError, SchemaError, ValidationError},
    formatters::{DefaultErrorFormatter, ErrorFormatter, FormatOptions},
};

#[test]
fn test_formatter_with_severity() {
    let formatter = DefaultErrorFormatter::default();
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_severity(ErrorSeverity::Critical);
    
    let error = SchemaError::MissingField { context };
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: false,
    };

    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("[CRITICAL]"));
    assert!(formatted.contains("Schema Error"));
    assert!(formatted.contains("Missing required field"));
    assert!(formatted.contains("test_field"));
}

#[test]
fn test_formatter_with_additional_context() {
    let formatter = DefaultErrorFormatter::default();
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_additional_context("This field is required for memory providers");
    
    let error = SchemaError::MissingField { context };
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: false,
    };

    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("Context: This field is required for memory providers"));
}

#[test]
fn test_formatter_with_different_severity_levels() {
    let formatter = DefaultErrorFormatter::default();
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: false,
    };

    // Test Critical severity
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_severity(ErrorSeverity::Critical);
    let error = SchemaError::MissingField { context };
    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("[CRITICAL]"));

    // Test Warning severity
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_severity(ErrorSeverity::Warning);
    let error = SchemaError::MissingField { context };
    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("[WARNING]"));

    // Test Info severity
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_severity(ErrorSeverity::Info);
    let error = SchemaError::MissingField { context };
    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("[INFO]"));
}

#[test]
fn test_error_codes() {
    let formatter = DefaultErrorFormatter::default();
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: false,
        include_documentation: false,
        use_color: false,
    };

    // Test SchemaError codes
    let error = SchemaError::missing_field("test_field");
    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("[SCHEMA_0001]"));

    let error = SchemaError::invalid_type("test_field", "string", "integer");
    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("[SCHEMA_0002]"));

    // Test ValidationError codes
    let error = ValidationError::invalid_value("test_field", "Value must be positive");
    let formatted = formatter.format_validation_error(&error, &options);
    assert!(formatted.contains("[VALIDATION_0001]"));

    // Test ProviderError codes
    let error = ProviderError::capability("test_field", "Missing required capability");
    let formatted = formatter.format_provider_error(&error, &options);
    assert!(formatted.contains("[PROVIDER_0002]"));
}

#[test]
fn test_colored_severity_levels() {
    let formatter = DefaultErrorFormatter::default();
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: true,
    };

    // Test Critical severity with color
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_severity(ErrorSeverity::Critical);
    let error = SchemaError::MissingField { context };
    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("\x1b[31;1m[CRITICAL]\x1b[0m"));

    // Test Error severity with color
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_severity(ErrorSeverity::Error);
    let error = SchemaError::MissingField { context };
    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("\x1b[31m[ERROR]\x1b[0m"));

    // Test Warning severity with color
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_severity(ErrorSeverity::Warning);
    let error = SchemaError::MissingField { context };
    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("\x1b[33m[WARNING]\x1b[0m"));

    // Test Info severity with color
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_severity(ErrorSeverity::Info);
    let error = SchemaError::MissingField { context };
    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("\x1b[36m[INFO]\x1b[0m"));
}

#[test]
fn test_additional_context_with_color() {
    let formatter = DefaultErrorFormatter::default();
    let mut context = ErrorContext::new_with_field("test_field");
    context = context.with_additional_context("This field is required for memory providers");
    
    let error = SchemaError::MissingField { context };
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: true,
    };

    let formatted = formatter.format_schema_error(&error, &options);
    assert!(formatted.contains("\x1b[34mContext:\x1b[0m This field is required for memory providers"));
}

#[test]
fn test_validation_error_with_severity_and_context() {
    let formatter = DefaultErrorFormatter::default();
    let mut context = ErrorContext::new_with_field("max_connections");
    context = context.with_severity(ErrorSeverity::Warning);
    context = context.with_additional_context("Current value may lead to performance issues");
    
    let error = ValidationError::InvalidValue {
        message: "Value exceeds recommended maximum".to_string(),
        context,
    };
    
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: false,
    };

    let formatted = formatter.format_validation_error(&error, &options);
    assert!(formatted.contains("[WARNING]"));
    assert!(formatted.contains("Validation Error"));
    assert!(formatted.contains("Value exceeds recommended maximum"));
    assert!(formatted.contains("max_connections"));
    assert!(formatted.contains("Context: Current value may lead to performance issues"));
}

#[test]
fn test_provider_error_with_severity_and_context() {
    let formatter = DefaultErrorFormatter::default();
    let mut context = ErrorContext::new_with_field("database_url");
    context = context.with_severity(ErrorSeverity::Critical);
    context = context.with_additional_context("Connection to the database failed");
    
    let error = ProviderError::Initialization {
        message: "Failed to initialize database connection".to_string(),
        context,
    };
    
    let options = FormatOptions {
        include_error_codes: true,
        include_suggestions: true,
        include_documentation: true,
        use_color: false,
    };

    let formatted = formatter.format_provider_error(&error, &options);
    assert!(formatted.contains("[CRITICAL]"));
    assert!(formatted.contains("Provider Error"));
    assert!(formatted.contains("Failed to initialize database connection"));
    assert!(formatted.contains("database_url"));
    assert!(formatted.contains("Context: Connection to the database failed"));
}

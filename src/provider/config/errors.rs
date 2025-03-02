//! Error types for provider configuration validation.
//!
//! This module defines a comprehensive error hierarchy for provider configuration
//! validation, including schema errors, validation errors, and provider-specific errors.
//! It also provides utilities for error formatting, source location tracking, and
//! error metadata.
//!
//! # Error Hierarchy
//!
//! The error system is organized into a hierarchical structure:
//!
//! - [`ProviderConfigError`]: Top-level error type that encompasses all validation errors
//!   - [`SchemaError`]: Errors related to schema validation (missing fields, invalid types)
//!   - [`ValidationError`]: Errors related to value validation (invalid values, constraint violations)
//!   - [`ProviderError`]: Provider-specific errors (initialization, capabilities)
//!
//! # Error Context
//!
//! Each error includes rich contextual information through the [`ErrorContext`] struct:
//!
//! - Source location tracking (file, line, column, field)
//! - Error severity levels (Critical, Error, Warning, Info)
//! - Documentation references for troubleshooting
//! - Suggestions for fixing the error
//! - Error codes for reference
//!
//! # Error Collection
//!
//! Errors can be collected during validation using the `validate_collecting` method
//! from the `ProviderConfigValidator` trait, which returns a vector of all validation
//! errors instead of stopping at the first error.
//!
//! # Examples
//!
//! Creating and handling schema errors:
//!
//! ```rust,ignore
//! use kairei::provider::config::errors::{SchemaError, ProviderConfigError};
//!
//! // Create a missing field error
//! let error = SchemaError::missing_field("type");
//! let provider_error: ProviderConfigError = error.into();
//!
//! // Get the error code
//! let code = provider_error.error_code(); // "SCHEMA_0001"
//!
//! // Handle the error
//! match provider_error {
//!     ProviderConfigError::Schema(schema_error) => {
//!         println!("Schema validation failed: {}", schema_error);
//!     },
//!     _ => println!("Other error type"),
//! }
//! ```

use crate::provider::config::base::ConfigError;
use crate::provider::config::doc_references;
use thiserror::Error;

/// Represents the location in source code where an error occurred.
///
/// The `SourceLocation` struct tracks where an error occurred in the source code,
/// including the file path, line number, column number, and field name. This
/// information helps developers pinpoint the exact location of validation errors
/// in their provider configurations.
///
/// # Examples
///
/// ```rust,ignore
/// use kairei::provider::config::errors::SourceLocation;
///
/// // Create a source location with just a field name
/// let location = SourceLocation::new_with_field("type");
///
/// // Create a source location with file and line information
/// let mut location = SourceLocation::new();
/// location.file = Some("config.json".to_string());
/// location.line = Some(42);
/// location.column = Some(10);
/// location.field = Some("ttl".to_string());
///
/// // Display the location
/// println!("Error {}", location); // Output: "Error in field 'ttl' at config.json:42:10"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceLocation {
    /// File path where the error occurred
    pub file: Option<String>,
    /// Line number where the error occurred
    pub line: Option<u32>,
    /// Column number where the error occurred
    pub column: Option<u32>,
    /// Field name associated with the error
    pub field: Option<String>,
}

impl SourceLocation {
    /// Creates a new source location with the given field name
    pub fn new_with_field(field: impl Into<String>) -> Self {
        Self {
            file: None,
            line: None,
            column: None,
            field: Some(field.into()),
        }
    }

    /// Creates a new empty source location
    pub fn new() -> Self {
        Self {
            file: None,
            line: None,
            column: None,
            field: None,
        }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(field) = &self.field {
            write!(f, "in field '{}'", field)?;
        }

        if let (Some(file), Some(line)) = (&self.file, &self.line) {
            write!(f, " at {}:{}", file, line)?;
            if let Some(column) = &self.column {
                write!(f, ":{}", column)?;
            }
        }

        Ok(())
    }
}

/// Represents the severity level of an error.
///
/// The `ErrorSeverity` enum defines different levels of severity for validation errors,
/// allowing developers to prioritize and filter errors based on their impact.
///
/// # Severity Levels
///
/// - `Critical`: Errors that prevent the system from functioning at all
/// - `Error`: Standard errors that affect functionality but don't completely break the system
/// - `Warning`: Issues that should be addressed but don't affect core functionality
/// - `Info`: Informational messages about potential issues or optimization opportunities
///
/// # Examples
///
/// ```rust,ignore
/// use kairei::provider::config::errors::{ErrorContext, ErrorSeverity};
///
/// // Create an error context with warning severity
/// let context = ErrorContext::new_with_field("ttl")
///     .with_severity(ErrorSeverity::Warning);
///
/// // Check severity level
/// if let ErrorSeverity::Warning = context.severity {
///     println!("This is a warning, not a critical error");
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Critical errors that prevent the system from functioning
    Critical,
    /// Errors that affect functionality but don't prevent the system from running
    Error,
    /// Issues that should be addressed but don't affect core functionality
    Warning,
    /// Informational messages about potential issues
    Info,
}

impl Default for ErrorSeverity {
    fn default() -> Self {
        Self::Error
    }
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARNING"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

/// Provides additional context for errors.
///
/// The `ErrorContext` struct enriches error information with contextual details
/// such as location, severity, documentation references, suggestions, error codes,
/// and additional context. This rich error context helps developers understand,
/// diagnose, and fix validation issues more effectively.
///
/// # Builder Pattern
///
/// The struct implements a builder pattern for easy construction:
///
/// ```rust,ignore
/// use kairei::provider::config::errors::{ErrorContext, ErrorSeverity};
///
/// let context = ErrorContext::new_with_field("ttl")
///     .with_severity(ErrorSeverity::Warning)
///     .with_documentation("https://docs.example.com/validation#ttl")
///     .with_suggestion("Consider using a TTL value between 60 and 3600 seconds")
///     .with_error_code("W001")
///     .with_additional_context("TTL values below 60 seconds may impact performance");
/// ```
///
/// # Error Context Components
///
/// - `location`: Source location information (file, line, column, field)
/// - `severity`: Error severity level (Critical, Error, Warning, Info)
/// - `documentation`: Link to relevant documentation
/// - `suggestion`: Suggested fix for the error
/// - `error_code`: Unique error code for reference
/// - `additional_context`: Additional information about the error
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// Location in source code where the error occurred
    pub location: SourceLocation,
    /// Severity level of the error
    pub severity: ErrorSeverity,
    /// Documentation reference for the error
    pub documentation: Option<String>,
    /// Suggested fix for the error
    pub suggestion: Option<String>,
    /// Error code for reference
    pub error_code: Option<String>,
    /// Additional context information
    pub additional_context: Option<String>,
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error context: {}", self.location)
    }
}

impl std::error::Error for ErrorContext {}

impl ErrorContext {
    /// Creates a new error context with the given field name
    pub fn new_with_field(field: impl Into<String>) -> Self {
        Self {
            location: SourceLocation::new_with_field(field),
            ..Default::default()
        }
    }

    /// Sets the severity level
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Sets the documentation reference
    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }

    /// Sets the suggested fix
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Sets the error code
    pub fn with_error_code(mut self, error_code: impl Into<String>) -> Self {
        self.error_code = Some(error_code.into());
        self
    }

    /// Sets additional context information
    pub fn with_additional_context(mut self, context: impl Into<String>) -> Self {
        self.additional_context = Some(context.into());
        self
    }
}

/// Errors related to schema validation.
///
/// The `SchemaError` enum represents errors that occur during schema validation
/// of provider configurations. These errors typically indicate structural issues
/// with the configuration, such as missing required fields, invalid types, or
/// invalid structure.
///
/// # Error Types
///
/// - `MissingField`: A required field is missing from the configuration
/// - `InvalidType`: A field has an incorrect type (e.g., string instead of number)
/// - `InvalidStructure`: The overall structure of the configuration is invalid
///
/// # Factory Methods
///
/// The enum provides factory methods for creating specific error types:
///
/// - `missing_field(field)`: Creates a MissingField error for the specified field
/// - `invalid_type(field, expected, actual)`: Creates an InvalidType error
/// - `invalid_structure(field, message)`: Creates an InvalidStructure error
///
/// # Examples
///
/// ```rust,ignore
/// use kairei::provider::config::errors::SchemaError;
///
/// // Create a missing field error
/// let error = SchemaError::missing_field("type");
///
/// // Create an invalid type error
/// let error = SchemaError::invalid_type("ttl", "number", "string");
///
/// // Create an invalid structure error
/// let error = SchemaError::invalid_structure("capabilities", "Invalid capabilities structure");
/// ```
#[derive(Debug, Error, Clone)]
pub enum SchemaError {
    /// A required field is missing
    #[error("Missing required field{}{}", format_location(&context.location), format_documentation(&context.documentation))]
    MissingField {
        /// Context for the error
        #[source]
        context: ErrorContext,
    },

    /// A field has an invalid type
    #[error("Invalid type{}: expected {expected}, found {actual}{}", format_location(&context.location), format_documentation(&context.documentation))]
    InvalidType {
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
        /// Context for the error
        #[source]
        context: ErrorContext,
    },

    /// The structure of the configuration is invalid
    #[error("Invalid structure{}: {message}{}", format_location(&context.location), format_documentation(&context.documentation))]
    InvalidStructure {
        /// Error message
        message: String,
        /// Context for the error
        #[source]
        context: ErrorContext,
    },
}

impl SchemaError {
    /// Creates a new MissingField error
    pub fn missing_field(field: impl Into<String>) -> Self {
        let mut context = ErrorContext::new_with_field(field);
        if let Some(doc_ref) = doc_references::schema::get_doc_reference("missing_field") {
            context = context.with_documentation(doc_ref);
        }
        // Set severity level to Error
        context = context.with_severity(ErrorSeverity::Error);
        Self::MissingField { context }
    }

    /// Creates a new InvalidType error
    pub fn invalid_type(
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        let mut context = ErrorContext::new_with_field(field);
        if let Some(doc_ref) = doc_references::schema::get_doc_reference("invalid_type") {
            context = context.with_documentation(doc_ref);
        }
        // Set severity level to Error
        context = context.with_severity(ErrorSeverity::Error);
        Self::InvalidType {
            expected: expected.into(),
            actual: actual.into(),
            context,
        }
    }

    /// Creates a new InvalidStructure error
    pub fn invalid_structure(field: impl Into<String>, message: impl Into<String>) -> Self {
        let mut context = ErrorContext::new_with_field(field);
        if let Some(doc_ref) = doc_references::schema::get_doc_reference("invalid_structure") {
            context = context.with_documentation(doc_ref);
        }
        // Set severity level to Error
        context = context.with_severity(ErrorSeverity::Error);
        Self::InvalidStructure {
            message: message.into(),
            context,
        }
    }
}

/// Errors related to value validation.
///
/// The `ValidationError` enum represents errors that occur during value validation
/// of provider configurations. These errors typically indicate issues with the
/// values of configuration fields, such as invalid values, constraint violations,
/// or dependency errors.
///
/// # Error Types
///
/// - `InvalidValue`: A field has a value that is invalid (e.g., negative number for a positive-only field)
/// - `ConstraintViolation`: A constraint on the configuration was violated (e.g., mutually exclusive fields)
/// - `DependencyError`: A dependency requirement was not satisfied
///
/// # Factory Methods
///
/// The enum provides factory methods for creating specific error types:
///
/// - `invalid_value(field, message)`: Creates an InvalidValue error
/// - `constraint_violation(field, message)`: Creates a ConstraintViolation error
/// - `dependency_error(field, message)`: Creates a DependencyError error
///
/// # Examples
///
/// ```rust,ignore
/// use kairei::provider::config::errors::ValidationError;
///
/// // Create an invalid value error
/// let error = ValidationError::invalid_value("ttl", "TTL must be greater than 0");
///
/// // Create a constraint violation error
/// let error = ValidationError::constraint_violation(
///     "max_tokens",
///     "max_tokens must be less than model_context_size"
/// );
///
/// // Create a dependency error
/// let error = ValidationError::dependency_error(
///     "dependencies[0].version",
///     "Dependency version must be in format x.y.z"
/// );
/// ```
#[derive(Debug, Error, Clone)]
pub enum ValidationError {
    /// A field has an invalid value
    #[error("Invalid value{}: {message}{}", format_location(&context.location), format_documentation(&context.documentation))]
    InvalidValue {
        /// Error message
        message: String,
        /// Context for the error
        #[source]
        context: ErrorContext,
    },

    /// A constraint was violated
    #[error("Constraint violation{}: {message}{}", format_location(&context.location), format_documentation(&context.documentation))]
    ConstraintViolation {
        /// Error message
        message: String,
        /// Context for the error
        #[source]
        context: ErrorContext,
    },

    /// A dependency was not satisfied
    #[error("Dependency error{}: {message}{}", format_location(&context.location), format_documentation(&context.documentation))]
    DependencyError {
        /// Error message
        message: String,
        /// Context for the error
        #[source]
        context: ErrorContext,
    },
}

impl ValidationError {
    /// Creates a new InvalidValue error
    pub fn invalid_value(field: impl Into<String>, message: impl Into<String>) -> Self {
        let mut context = ErrorContext::new_with_field(field);
        if let Some(doc_ref) = doc_references::validation::get_doc_reference("invalid_value") {
            context = context.with_documentation(doc_ref);
        }
        // Set severity level to Error
        context = context.with_severity(ErrorSeverity::Error);
        Self::InvalidValue {
            message: message.into(),
            context,
        }
    }

    /// Creates a new ConstraintViolation error
    pub fn constraint_violation(field: impl Into<String>, message: impl Into<String>) -> Self {
        let mut context = ErrorContext::new_with_field(field);
        if let Some(doc_ref) = doc_references::validation::get_doc_reference("constraint_violation")
        {
            context = context.with_documentation(doc_ref);
        }
        // Set severity level to Error
        context = context.with_severity(ErrorSeverity::Error);
        Self::ConstraintViolation {
            message: message.into(),
            context,
        }
    }

    /// Creates a new DependencyError error
    pub fn dependency_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        let mut context = ErrorContext::new_with_field(field);
        if let Some(doc_ref) = doc_references::validation::get_doc_reference("dependency_error") {
            context = context.with_documentation(doc_ref);
        }
        // Set severity level to Error
        context = context.with_severity(ErrorSeverity::Error);
        Self::DependencyError {
            message: message.into(),
            context,
        }
    }
}

/// Errors related to provider configuration.
///
/// The `ProviderError` enum represents errors that are specific to providers,
/// such as initialization errors, capability errors, and configuration errors.
/// These errors typically indicate issues with the provider itself rather than
/// with the schema or values of the configuration.
///
/// # Error Types
///
/// - `Initialization`: Errors that occur during provider initialization
/// - `Capability`: Errors related to provider capabilities
/// - `Configuration`: Errors in provider configuration
///
/// # Factory Methods
///
/// The enum provides factory methods for creating specific error types:
///
/// - `initialization(field, message)`: Creates an Initialization error
/// - `capability(field, message)`: Creates a Capability error
/// - `configuration(field, message)`: Creates a Configuration error
///
/// # Examples
///
/// ```rust,ignore
/// use kairei::provider::config::errors::ProviderError;
///
/// // Create an initialization error
/// let error = ProviderError::initialization(
///     "provider",
///     "Failed to initialize memory provider"
/// );
///
/// // Create a capability error
/// let error = ProviderError::capability(
///     "capabilities.memory",
///     "Memory provider requires memory capability"
/// );
///
/// // Create a configuration error
/// let error = ProviderError::configuration(
///     "config",
///     "Invalid memory provider configuration"
/// );
/// ```
#[derive(Debug, Error, Clone)]
pub enum ProviderError {
    /// Error during provider initialization
    #[error("Provider initialization error{}: {message}{}", format_location(&context.location), format_documentation(&context.documentation))]
    Initialization {
        /// Error message
        message: String,
        /// Context for the error
        #[source]
        context: ErrorContext,
    },

    /// Error related to provider capabilities
    #[error("Provider capability error{}: {message}{}", format_location(&context.location), format_documentation(&context.documentation))]
    Capability {
        /// Error message
        message: String,
        /// Context for the error
        #[source]
        context: ErrorContext,
    },

    /// Error in provider configuration
    #[error("Provider configuration error{}: {message}{}", format_location(&context.location), format_documentation(&context.documentation))]
    Configuration {
        /// Error message
        message: String,
        /// Context for the error
        #[source]
        context: ErrorContext,
    },
}

impl ProviderError {
    /// Creates a new Initialization error
    pub fn initialization(field: impl Into<String>, message: impl Into<String>) -> Self {
        let mut context = ErrorContext::new_with_field(field);
        if let Some(doc_ref) = doc_references::provider::get_doc_reference("initialization") {
            context = context.with_documentation(doc_ref);
        }
        // Set severity level to Critical
        context = context.with_severity(ErrorSeverity::Critical);
        Self::Initialization {
            message: message.into(),
            context,
        }
    }

    /// Creates a new Capability error
    pub fn capability(field: impl Into<String>, message: impl Into<String>) -> Self {
        let mut context = ErrorContext::new_with_field(field);
        if let Some(doc_ref) = doc_references::provider::get_doc_reference("capability") {
            context = context.with_documentation(doc_ref);
        }
        // Set severity level to Error
        context = context.with_severity(ErrorSeverity::Error);
        Self::Capability {
            message: message.into(),
            context,
        }
    }

    /// Creates a new Configuration error
    pub fn configuration(field: impl Into<String>, message: impl Into<String>) -> Self {
        let mut context = ErrorContext::new_with_field(field);
        if let Some(doc_ref) = doc_references::provider::get_doc_reference("configuration") {
            context = context.with_documentation(doc_ref);
        }
        // Set severity level to Error
        context = context.with_severity(ErrorSeverity::Error);
        Self::Configuration {
            message: message.into(),
            context,
        }
    }
}

/// Top-level error type for provider configuration validation.
///
/// This is the main error type for provider configuration validation. It encompasses
/// all types of validation errors, including schema errors, validation errors,
/// provider-specific errors, and a Legacy variant for backward compatibility with
/// the existing ConfigError type.
///
/// # Error Hierarchy
///
/// The `ProviderConfigError` enum serves as the top-level error type in the validation
/// error hierarchy:
///
/// - `Schema`: Errors related to schema validation (missing fields, invalid types)
/// - `Validation`: Errors related to value validation (invalid values, constraint violations)
/// - `Provider`: Provider-specific errors (initialization, capabilities)
/// - `Generic`: Generic errors with a simple message
/// - `Legacy`: Legacy ConfigError for backward compatibility
///
/// # Error Collection
///
/// Errors can be collected during validation using the `validate_collecting` method
/// from the `ProviderConfigValidator` trait, which returns a vector of `ProviderConfigError`
/// objects instead of stopping at the first error.
///
/// # Examples
///
/// ```rust,ignore
/// use kairei::provider::config::errors::{ProviderConfigError, SchemaError};
/// use kairei::provider::config::validator::ProviderConfigValidator;
/// use kairei::provider::config::validators::type_checker::TypeCheckerValidator;
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// // Create a validator
/// let validator = TypeCheckerValidator;
///
/// // Create an invalid configuration
/// let config = serde_json::from_value(json!({
///     // Missing required "type" field
/// })).unwrap();
///
/// // Validate the configuration
/// match validator.validate(&config) {
///     Ok(()) => println!("Configuration is valid"),
///     Err(error) => {
///         println!("Validation error: {}", error);
///         println!("Error code: {}", error.error_code());
///     }
/// }
///
/// // Collect all validation errors
/// let errors = validator.validate_collecting(&config);
/// for error in errors {
///     println!("Validation error: {}", error);
/// }
/// ```
#[derive(Debug, Error, Clone)]
pub enum ProviderConfigError {
    /// Schema validation errors
    #[error("Schema error: {0}")]
    Schema(#[from] SchemaError),

    /// Value validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Provider-specific errors
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    /// Generic errors
    #[error("Configuration error: {0}")]
    Generic(String),

    /// Legacy ConfigError for backward compatibility
    #[error("{0}")]
    Legacy(#[from] ConfigError),
}

impl ProviderConfigError {
    /// Creates a new Generic error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic(message.into())
    }

    /// Gets the error code for this error
    pub fn error_code(&self) -> String {
        match self {
            Self::Schema(e) => format!("SCHEMA_{:04}", error_code_for_schema(e)),
            Self::Validation(e) => format!("VALIDATION_{:04}", error_code_for_validation(e)),
            Self::Provider(e) => format!("PROVIDER_{:04}", error_code_for_provider(e)),
            Self::Generic(_) => "GENERIC_0001".to_string(),
            Self::Legacy(e) => match e {
                ConfigError::MissingField(_) => "LEGACY_0001".to_string(),
                ConfigError::InvalidValue { .. } => "LEGACY_0002".to_string(),
                ConfigError::ValidationError(_) => "LEGACY_0003".to_string(),
            },
        }
    }
}

/// Helper function to format documentation references for error messages
fn format_documentation(doc_ref: &Option<String>) -> String {
    if let Some(doc) = doc_ref {
        format!(" (see: {})", doc)
    } else {
        String::new()
    }
}

/// Helper function to format location for error messages
fn format_location(location: &SourceLocation) -> String {
    if location.field.is_none() && location.file.is_none() {
        return String::new();
    }
    format!(" {}", location)
}

/// Generate error code for schema errors
fn error_code_for_schema(error: &SchemaError) -> u16 {
    match error {
        SchemaError::MissingField { .. } => 1,
        SchemaError::InvalidType { .. } => 2,
        SchemaError::InvalidStructure { .. } => 3,
    }
}

/// Generate error code for validation errors
fn error_code_for_validation(error: &ValidationError) -> u16 {
    match error {
        ValidationError::InvalidValue { .. } => 1,
        ValidationError::ConstraintViolation { .. } => 2,
        ValidationError::DependencyError { .. } => 3,
    }
}

/// Generate error code for provider errors
fn error_code_for_provider(error: &ProviderError) -> u16 {
    match error {
        ProviderError::Initialization { .. } => 1,
        ProviderError::Capability { .. } => 2,
        ProviderError::Configuration { .. } => 3,
    }
}

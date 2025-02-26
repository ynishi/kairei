//! Error formatters for provider configuration validation.
//!
//! This module defines formatters for provider configuration validation errors,
//! allowing for rich error messages with context, suggestions, and documentation references.

use crate::provider::config::errors::{
    ErrorContext, ProviderConfigError, ProviderError, SchemaError, ValidationError,
};
use std::fmt::Write;

mod default;

pub use default::DefaultErrorFormatter;

/// Format options for error messages
#[derive(Debug, Clone, Default)]
pub struct FormatOptions {
    /// Whether to include error codes in the output
    pub include_error_codes: bool,
    /// Whether to include suggestions in the output
    pub include_suggestions: bool,
    /// Whether to include documentation references in the output
    pub include_documentation: bool,
    /// Whether to use color in the output
    pub use_color: bool,
}

/// Trait for formatting provider configuration errors
pub trait ErrorFormatter {
    /// Format a provider configuration error
    fn format_error(&self, error: &ProviderConfigError, options: &FormatOptions) -> String;
    
    /// Format a schema error
    fn format_schema_error(&self, error: &SchemaError, options: &FormatOptions) -> String;
    
    /// Format a validation error
    fn format_validation_error(&self, error: &ValidationError, options: &FormatOptions) -> String;
    
    /// Format a provider error
    fn format_provider_error(&self, error: &ProviderError, options: &FormatOptions) -> String;
}

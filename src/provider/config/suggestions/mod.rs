//! Suggestion generators for provider configuration validation errors.
//!
//! This module defines generators for creating helpful suggestions for
//! provider configuration validation errors.

use crate::provider::config::errors::{
    ProviderConfigError, ProviderError, SchemaError, ValidationError,
};

mod default;

pub use default::DefaultSuggestionGenerator;

/// Trait for generating suggestions for provider configuration errors
pub trait SuggestionGenerator: std::fmt::Debug {
    /// Generate a suggestion for a schema error
    fn generate_schema_suggestion(&self, error: &SchemaError) -> Option<String>;

    /// Generate a suggestion for a validation error
    fn generate_validation_suggestion(&self, error: &ValidationError) -> Option<String>;

    /// Generate a suggestion for a provider error
    fn generate_provider_suggestion(&self, error: &ProviderError) -> Option<String>;

    /// Generate a suggestion for a provider configuration error
    fn generate_suggestion(&self, error: &ProviderConfigError) -> Option<String> {
        match error {
            ProviderConfigError::Schema(e) => self.generate_schema_suggestion(e),
            ProviderConfigError::Validation(e) => self.generate_validation_suggestion(e),
            ProviderConfigError::Provider(e) => self.generate_provider_suggestion(e),
            ProviderConfigError::Generic(_) => None,
            ProviderConfigError::Legacy(_) => None,
        }
    }
}

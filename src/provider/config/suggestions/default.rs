//! Default suggestion generator implementation.
//!
//! This module provides a default implementation of the SuggestionGenerator trait
//! that generates context-aware suggestions for different error types.

use super::SuggestionGenerator;
use crate::provider::config::errors::{ProviderError, SchemaError, ValidationError};

/// Default suggestion generator
#[derive(Debug, Default)]
pub struct DefaultSuggestionGenerator;

impl SuggestionGenerator for DefaultSuggestionGenerator {
    fn generate_schema_suggestion(&self, error: &SchemaError) -> Option<String> {
        match error {
            SchemaError::MissingField { context } => {
                // Return existing suggestion if available
                if let Some(suggestion) = &context.suggestion {
                    return Some(suggestion.clone());
                }

                // Generate context-aware suggestion
                let field = context.location.field.as_ref()?;
                Some(format!(
                    "Add the required '{}' field to your configuration",
                    field
                ))
            }
            SchemaError::InvalidType {
                context,
                expected,
                actual,
            } => {
                // Return existing suggestion if available
                if let Some(suggestion) = &context.suggestion {
                    return Some(suggestion.clone());
                }

                // Generate context-aware suggestion
                let field = context.location.field.as_ref()?;
                Some(format!(
                    "Change the type of '{}' from {} to {}",
                    field, actual, expected
                ))
            }
            SchemaError::InvalidStructure { context, message } => {
                // Return existing suggestion if available
                if let Some(suggestion) = &context.suggestion {
                    return Some(suggestion.clone());
                }

                // Generate context-aware suggestion
                let field = context.location.field.as_ref()?;
                Some(format!("Fix the structure of '{}': {}", field, message))
            }
        }
    }

    fn generate_validation_suggestion(&self, error: &ValidationError) -> Option<String> {
        match error {
            ValidationError::InvalidValue { context, message } => {
                // Return existing suggestion if available
                if let Some(suggestion) = &context.suggestion {
                    return Some(suggestion.clone());
                }

                // Generate context-aware suggestion
                let field = context.location.field.as_ref()?;
                Some(format!(
                    "Provide a valid value for '{}': {}",
                    field, message
                ))
            }
            ValidationError::ConstraintViolation { context, message } => {
                // Return existing suggestion if available
                if let Some(suggestion) = &context.suggestion {
                    return Some(suggestion.clone());
                }

                // Generate context-aware suggestion
                let field = context.location.field.as_ref()?;
                Some(format!(
                    "Ensure '{}' meets the required constraints: {}",
                    field, message
                ))
            }
            ValidationError::DependencyError { context, message } => {
                // Return existing suggestion if available
                if let Some(suggestion) = &context.suggestion {
                    return Some(suggestion.clone());
                }

                // Generate context-aware suggestion
                let field = context.location.field.as_ref()?;
                Some(format!(
                    "Resolve dependency issues for '{}': {}",
                    field, message
                ))
            }
        }
    }

    fn generate_provider_suggestion(&self, error: &ProviderError) -> Option<String> {
        match error {
            ProviderError::Initialization { context, message } => {
                // Return existing suggestion if available
                if let Some(suggestion) = &context.suggestion {
                    return Some(suggestion.clone());
                }

                // Generate context-aware suggestion
                let field = context.location.field.as_ref()?;
                Some(format!(
                    "Fix initialization issues for '{}': {}",
                    field, message
                ))
            }
            ProviderError::Capability { context, message } => {
                // Return existing suggestion if available
                if let Some(suggestion) = &context.suggestion {
                    return Some(suggestion.clone());
                }

                // Generate context-aware suggestion
                let field = context.location.field.as_ref()?;
                Some(format!(
                    "Ensure '{}' has the required capabilities: {}",
                    field, message
                ))
            }
            ProviderError::Configuration { context, message } => {
                // Return existing suggestion if available
                if let Some(suggestion) = &context.suggestion {
                    return Some(suggestion.clone());
                }

                // Generate context-aware suggestion
                let field = context.location.field.as_ref()?;
                Some(format!(
                    "Fix configuration issues for '{}': {}",
                    field, message
                ))
            }
        }
    }
}

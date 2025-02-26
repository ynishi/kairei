//! Default error formatter implementation.
//!
//! This module provides a default implementation of the ErrorFormatter trait
//! that formats errors with rich context, suggestions, and documentation references.

use super::{ErrorFormatter, FormatOptions};
use crate::provider::config::errors::{
    ErrorContext, ProviderConfigError, ProviderError, SchemaError, ValidationError,
};
use std::fmt::Write;

/// Default error formatter
#[derive(Debug, Default)]
pub struct DefaultErrorFormatter;

impl ErrorFormatter for DefaultErrorFormatter {
    fn format_error(&self, error: &ProviderConfigError, options: &FormatOptions) -> String {
        match error {
            ProviderConfigError::Schema(e) => self.format_schema_error(e, options),
            ProviderConfigError::Validation(e) => self.format_validation_error(e, options),
            ProviderConfigError::Provider(e) => self.format_provider_error(e, options),
            ProviderConfigError::Generic(msg) => format!("Configuration error: {}", msg),
            ProviderConfigError::Legacy(e) => format!("{}", e),
        }
    }
    
    fn format_schema_error(&self, error: &SchemaError, options: &FormatOptions) -> String {
        let mut output = String::new();
        
        // Add error code if enabled
        if options.include_error_codes {
            let error_code = match error {
                SchemaError::MissingField { .. } => "SCHEMA_0001",
                SchemaError::InvalidType { .. } => "SCHEMA_0002",
                SchemaError::InvalidStructure { .. } => "SCHEMA_0003",
            };
            
            if options.use_color {
                write!(output, "\x1b[90m[{}]\x1b[0m ", error_code).unwrap();
            } else {
                write!(output, "[{}] ", error_code).unwrap();
            }
        }
        
        // Add error message
        if options.use_color {
            write!(output, "\x1b[31mSchema Error:\x1b[0m {}", error).unwrap();
        } else {
            write!(output, "Schema Error: {}", error).unwrap();
        }
        
        // Add documentation reference if available and enabled
        if options.include_documentation {
            if let Some(doc) = self.get_documentation_for_schema_error(error) {
                if options.use_color {
                    write!(output, "\n\x1b[36mDocumentation:\x1b[0m {}", doc).unwrap();
                } else {
                    write!(output, "\nDocumentation: {}", doc).unwrap();
                }
            }
        }
        
        // Add suggestion if available and enabled
        if options.include_suggestions {
            if let Some(suggestion) = self.get_suggestion_for_schema_error(error) {
                if options.use_color {
                    write!(output, "\n\x1b[32mSuggestion:\x1b[0m {}", suggestion).unwrap();
                } else {
                    write!(output, "\nSuggestion: {}", suggestion).unwrap();
                }
            }
        }
        
        output
    }
    
    fn format_validation_error(&self, error: &ValidationError, options: &FormatOptions) -> String {
        let mut output = String::new();
        
        // Add error code if enabled
        if options.include_error_codes {
            let error_code = match error {
                ValidationError::InvalidValue { .. } => "VALIDATION_0001",
                ValidationError::ConstraintViolation { .. } => "VALIDATION_0002",
                ValidationError::DependencyError { .. } => "VALIDATION_0003",
            };
            
            if options.use_color {
                write!(output, "\x1b[90m[{}]\x1b[0m ", error_code).unwrap();
            } else {
                write!(output, "[{}] ", error_code).unwrap();
            }
        }
        
        // Add error message
        if options.use_color {
            write!(output, "\x1b[31mValidation Error:\x1b[0m {}", error).unwrap();
        } else {
            write!(output, "Validation Error: {}", error).unwrap();
        }
        
        // Add documentation reference if available and enabled
        if options.include_documentation {
            if let Some(doc) = self.get_documentation_for_validation_error(error) {
                if options.use_color {
                    write!(output, "\n\x1b[36mDocumentation:\x1b[0m {}", doc).unwrap();
                } else {
                    write!(output, "\nDocumentation: {}", doc).unwrap();
                }
            }
        }
        
        // Add suggestion if available and enabled
        if options.include_suggestions {
            if let Some(suggestion) = self.get_suggestion_for_validation_error(error) {
                if options.use_color {
                    write!(output, "\n\x1b[32mSuggestion:\x1b[0m {}", suggestion).unwrap();
                } else {
                    write!(output, "\nSuggestion: {}", suggestion).unwrap();
                }
            }
        }
        
        output
    }
    
    fn format_provider_error(&self, error: &ProviderError, options: &FormatOptions) -> String {
        let mut output = String::new();
        
        // Add error code if enabled
        if options.include_error_codes {
            let error_code = match error {
                ProviderError::Initialization { .. } => "PROVIDER_0001",
                ProviderError::Capability { .. } => "PROVIDER_0002",
                ProviderError::Configuration { .. } => "PROVIDER_0003",
            };
            
            if options.use_color {
                write!(output, "\x1b[90m[{}]\x1b[0m ", error_code).unwrap();
            } else {
                write!(output, "[{}] ", error_code).unwrap();
            }
        }
        
        // Add error message
        if options.use_color {
            write!(output, "\x1b[31mProvider Error:\x1b[0m {}", error).unwrap();
        } else {
            write!(output, "Provider Error: {}", error).unwrap();
        }
        
        // Add documentation reference if available and enabled
        if options.include_documentation {
            if let Some(doc) = self.get_documentation_for_provider_error(error) {
                if options.use_color {
                    write!(output, "\n\x1b[36mDocumentation:\x1b[0m {}", doc).unwrap();
                } else {
                    write!(output, "\nDocumentation: {}", doc).unwrap();
                }
            }
        }
        
        // Add suggestion if available and enabled
        if options.include_suggestions {
            if let Some(suggestion) = self.get_suggestion_for_provider_error(error) {
                if options.use_color {
                    write!(output, "\n\x1b[32mSuggestion:\x1b[0m {}", suggestion).unwrap();
                } else {
                    write!(output, "\nSuggestion: {}", suggestion).unwrap();
                }
            }
        }
        
        output
    }
}

impl DefaultErrorFormatter {
    /// Get documentation reference for a schema error
    fn get_documentation_for_schema_error(&self, error: &SchemaError) -> Option<String> {
        match error {
            SchemaError::MissingField { context } => context.documentation.clone(),
            SchemaError::InvalidType { context, .. } => context.documentation.clone(),
            SchemaError::InvalidStructure { context, .. } => context.documentation.clone(),
        }
    }
    
    /// Get documentation reference for a validation error
    fn get_documentation_for_validation_error(&self, error: &ValidationError) -> Option<String> {
        match error {
            ValidationError::InvalidValue { context, .. } => context.documentation.clone(),
            ValidationError::ConstraintViolation { context, .. } => context.documentation.clone(),
            ValidationError::DependencyError { context, .. } => context.documentation.clone(),
        }
    }
    
    /// Get documentation reference for a provider error
    fn get_documentation_for_provider_error(&self, error: &ProviderError) -> Option<String> {
        match error {
            ProviderError::Initialization { context, .. } => context.documentation.clone(),
            ProviderError::Capability { context, .. } => context.documentation.clone(),
            ProviderError::Configuration { context, .. } => context.documentation.clone(),
        }
    }
    
    /// Get suggestion for a schema error
    fn get_suggestion_for_schema_error(&self, error: &SchemaError) -> Option<String> {
        match error {
            SchemaError::MissingField { context } => {
                context.suggestion.clone().or_else(|| {
                    let field = context.location.field.as_ref()?;
                    Some(format!("Add the required '{}' field to your configuration", field))
                })
            },
            SchemaError::InvalidType { context, expected, actual } => {
                context.suggestion.clone().or_else(|| {
                    let field = context.location.field.as_ref()?;
                    Some(format!("Change the type of '{}' from {} to {}", field, actual, expected))
                })
            },
            SchemaError::InvalidStructure { context, .. } => context.suggestion.clone(),
        }
    }
    
    /// Get suggestion for a validation error
    fn get_suggestion_for_validation_error(&self, error: &ValidationError) -> Option<String> {
        match error {
            ValidationError::InvalidValue { context, .. } => context.suggestion.clone(),
            ValidationError::ConstraintViolation { context, .. } => context.suggestion.clone(),
            ValidationError::DependencyError { context, .. } => context.suggestion.clone(),
        }
    }
    
    /// Get suggestion for a provider error
    fn get_suggestion_for_provider_error(&self, error: &ProviderError) -> Option<String> {
        match error {
            ProviderError::Initialization { context, .. } => context.suggestion.clone(),
            ProviderError::Capability { context, .. } => context.suggestion.clone(),
            ProviderError::Configuration { context, .. } => context.suggestion.clone(),
        }
    }
}

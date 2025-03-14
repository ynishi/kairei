//! Default error formatter implementation.
//!
//! This module provides a default implementation of the ErrorFormatter trait
//! that formats errors with rich context, suggestions, and documentation references.

use super::{ErrorFormatter, FormatOptions};
use crate::provider::config::errors::{
    ErrorSeverity, ProviderConfigError, ProviderError, SchemaError, ValidationError,
};
use crate::provider::config::suggestions::{DefaultSuggestionGenerator, SuggestionGenerator};
use std::fmt::Write;

/// Default error formatter
#[derive(Debug)]
pub struct DefaultErrorFormatter {
    suggestion_generator: Box<dyn SuggestionGenerator>,
}

impl Default for DefaultErrorFormatter {
    fn default() -> Self {
        Self {
            suggestion_generator: Box::new(DefaultSuggestionGenerator),
        }
    }
}

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

        // Add severity level
        let severity = match error {
            SchemaError::MissingField { context } => &context.severity,
            SchemaError::InvalidType { context, .. } => &context.severity,
            SchemaError::InvalidStructure { context, .. } => &context.severity,
        };

        if options.use_color {
            let color = match severity {
                ErrorSeverity::Critical => "\x1b[31;1m", // Bold red
                ErrorSeverity::Error => "\x1b[31m",      // Red
                ErrorSeverity::Warning => "\x1b[33m",    // Yellow
                ErrorSeverity::Info => "\x1b[36m",       // Cyan
            };
            write!(output, "{}[{}]\x1b[0m ", color, severity).unwrap();
        } else {
            write!(output, "[{}] ", severity).unwrap();
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

        // Add additional context if available
        let additional_context = match error {
            SchemaError::MissingField { context } => &context.additional_context,
            SchemaError::InvalidType { context, .. } => &context.additional_context,
            SchemaError::InvalidStructure { context, .. } => &context.additional_context,
        };

        if let Some(context) = additional_context {
            if options.use_color {
                write!(output, "\n\x1b[34mContext:\x1b[0m {}", context).unwrap();
            } else {
                write!(output, "\nContext: {}", context).unwrap();
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

        // Add severity level
        let severity = match error {
            ValidationError::InvalidValue { context, .. } => &context.severity,
            ValidationError::ConstraintViolation { context, .. } => &context.severity,
            ValidationError::DependencyError { context, .. } => &context.severity,
        };

        if options.use_color {
            let color = match severity {
                ErrorSeverity::Critical => "\x1b[31;1m", // Bold red
                ErrorSeverity::Error => "\x1b[31m",      // Red
                ErrorSeverity::Warning => "\x1b[33m",    // Yellow
                ErrorSeverity::Info => "\x1b[36m",       // Cyan
            };
            write!(output, "{}[{}]\x1b[0m ", color, severity).unwrap();
        } else {
            write!(output, "[{}] ", severity).unwrap();
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

        // Add additional context if available
        let additional_context = match error {
            ValidationError::InvalidValue { context, .. } => &context.additional_context,
            ValidationError::ConstraintViolation { context, .. } => &context.additional_context,
            ValidationError::DependencyError { context, .. } => &context.additional_context,
        };

        if let Some(context) = additional_context {
            if options.use_color {
                write!(output, "\n\x1b[34mContext:\x1b[0m {}", context).unwrap();
            } else {
                write!(output, "\nContext: {}", context).unwrap();
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

        // Add severity level
        let severity = match error {
            ProviderError::Initialization { context, .. } => &context.severity,
            ProviderError::Capability { context, .. } => &context.severity,
            ProviderError::Configuration { context, .. } => &context.severity,
        };

        if options.use_color {
            let color = match severity {
                ErrorSeverity::Critical => "\x1b[31;1m", // Bold red
                ErrorSeverity::Error => "\x1b[31m",      // Red
                ErrorSeverity::Warning => "\x1b[33m",    // Yellow
                ErrorSeverity::Info => "\x1b[36m",       // Cyan
            };
            write!(output, "{}[{}]\x1b[0m ", color, severity).unwrap();
        } else {
            write!(output, "[{}] ", severity).unwrap();
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

        // Add additional context if available
        let additional_context = match error {
            ProviderError::Initialization { context, .. } => &context.additional_context,
            ProviderError::Capability { context, .. } => &context.additional_context,
            ProviderError::Configuration { context, .. } => &context.additional_context,
        };

        if let Some(context) = additional_context {
            if options.use_color {
                write!(output, "\n\x1b[34mContext:\x1b[0m {}", context).unwrap();
            } else {
                write!(output, "\nContext: {}", context).unwrap();
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
        // First try to get a suggestion from the error context
        match error {
            SchemaError::MissingField { context } => context.suggestion.clone(),
            SchemaError::InvalidType { context, .. } => context.suggestion.clone(),
            SchemaError::InvalidStructure { context, .. } => context.suggestion.clone(),
        }
        // If no suggestion is available in the context, generate one
        .or_else(|| self.suggestion_generator.generate_schema_suggestion(error))
    }

    /// Get suggestion for a validation error
    fn get_suggestion_for_validation_error(&self, error: &ValidationError) -> Option<String> {
        // First try to get a suggestion from the error context
        match error {
            ValidationError::InvalidValue { context, .. } => context.suggestion.clone(),
            ValidationError::ConstraintViolation { context, .. } => context.suggestion.clone(),
            ValidationError::DependencyError { context, .. } => context.suggestion.clone(),
        }
        // If no suggestion is available in the context, generate one
        .or_else(|| {
            self.suggestion_generator
                .generate_validation_suggestion(error)
        })
    }

    /// Get suggestion for a provider error
    fn get_suggestion_for_provider_error(&self, error: &ProviderError) -> Option<String> {
        // First try to get a suggestion from the error context
        match error {
            ProviderError::Initialization { context, .. } => context.suggestion.clone(),
            ProviderError::Capability { context, .. } => context.suggestion.clone(),
            ProviderError::Configuration { context, .. } => context.suggestion.clone(),
        }
        // If no suggestion is available in the context, generate one
        .or_else(|| {
            self.suggestion_generator
                .generate_provider_suggestion(error)
        })
    }
}

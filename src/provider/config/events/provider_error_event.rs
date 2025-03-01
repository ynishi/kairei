//! Provider error event handling.
//!
//! This module defines the ProviderErrorEvent struct and related functionality
//! for handling provider configuration validation errors as events.

use crate::event::event_bus::{ErrorEvent, ErrorSeverity, Value};
use crate::provider::config::errors::{ProviderConfigError, SourceLocation};
use std::collections::HashMap;

/// Represents an error event from provider configuration validation.
///
/// This struct encapsulates provider configuration validation errors
/// and provides conversion to the system-wide ErrorEvent type.
#[derive(Debug, Clone)]
pub struct ProviderErrorEvent {
    /// The provider configuration error
    pub error: ProviderConfigError,
    /// The provider ID associated with the error
    pub provider_id: String,
    /// Additional context information
    pub context: Option<String>,
}

impl ProviderErrorEvent {
    /// Creates a new provider error event.
    ///
    /// # Parameters
    ///
    /// * `error` - The provider configuration error
    /// * `provider_id` - The provider ID associated with the error
    pub fn new(error: ProviderConfigError, provider_id: impl Into<String>) -> Self {
        Self {
            error,
            provider_id: provider_id.into(),
            context: None,
        }
    }

    /// Adds context information to the error event.
    ///
    /// # Parameters
    ///
    /// * `context` - Additional context information
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Converts the provider error event to a system-wide error event.
    ///
    /// # Returns
    ///
    /// * `ErrorEvent` - The system-wide error event
    pub fn to_error_event(&self) -> ErrorEvent {
        let error_type = match &self.error {
            ProviderConfigError::Schema(_) => "provider_config_schema_error",
            ProviderConfigError::Validation(_) => "provider_config_validation_error",
            ProviderConfigError::Provider(_) => "provider_error",
            ProviderConfigError::Generic(_) => "provider_config_generic_error",
            ProviderConfigError::Legacy(_) => "provider_config_legacy_error",
        };

        let severity = self.get_severity();
        let message = self.error.to_string();
        let mut parameters = HashMap::new();

        parameters.insert(
            "provider_id".to_string(),
            Value::String(self.provider_id.clone()),
        );
        parameters.insert(
            "error_code".to_string(),
            Value::String(self.error.error_code()),
        );

        if let Some(context) = &self.context {
            parameters.insert("context".to_string(), Value::String(context.clone()));
        }

        // Add source location if available
        if let Some(location) = self.get_source_location() {
            if let Some(field) = &location.field {
                parameters.insert("field".to_string(), Value::String(field.clone()));
            }
            if let Some(file) = &location.file {
                parameters.insert("file".to_string(), Value::String(file.clone()));
            }
            if let Some(line) = &location.line {
                parameters.insert("line".to_string(), Value::Integer(*line as i64));
            }
            if let Some(column) = &location.column {
                parameters.insert("column".to_string(), Value::Integer(*column as i64));
            }
        }

        ErrorEvent {
            error_type: error_type.to_string(),
            message,
            severity,
            parameters,
        }
    }

    /// Gets the severity of the error.
    ///
    /// # Returns
    ///
    /// * `ErrorSeverity` - The severity of the error
    fn get_severity(&self) -> ErrorSeverity {
        if let ProviderConfigError::Provider(
            crate::provider::config::errors::ProviderError::Initialization { context, .. },
        ) = &self.error
        {
            if context.severity == crate::provider::config::errors::ErrorSeverity::Critical {
                return ErrorSeverity::Critical;
            }
        }
        ErrorSeverity::Error
    }

    /// Gets the source location of the error, if available.
    ///
    /// # Returns
    ///
    /// * `Option<SourceLocation>` - The source location of the error, if available
    fn get_source_location(&self) -> Option<&SourceLocation> {
        match &self.error {
            ProviderConfigError::Schema(schema_error) => match schema_error {
                crate::provider::config::errors::SchemaError::MissingField { context, .. } => {
                    Some(&context.location)
                }
                crate::provider::config::errors::SchemaError::InvalidType { context, .. } => {
                    Some(&context.location)
                }
                crate::provider::config::errors::SchemaError::InvalidStructure {
                    context, ..
                } => Some(&context.location),
            },
            ProviderConfigError::Validation(validation_error) => match validation_error {
                crate::provider::config::errors::ValidationError::InvalidValue {
                    context, ..
                } => Some(&context.location),
                crate::provider::config::errors::ValidationError::ConstraintViolation {
                    context,
                    ..
                } => Some(&context.location),
                crate::provider::config::errors::ValidationError::DependencyError {
                    context,
                    ..
                } => Some(&context.location),
            },
            ProviderConfigError::Provider(provider_error) => match provider_error {
                crate::provider::config::errors::ProviderError::Initialization {
                    context, ..
                } => Some(&context.location),
                crate::provider::config::errors::ProviderError::Capability { context, .. } => {
                    Some(&context.location)
                }
                crate::provider::config::errors::ProviderError::Configuration {
                    context, ..
                } => Some(&context.location),
            },
            _ => None,
        }
    }
}

impl From<ProviderErrorEvent> for ErrorEvent {
    fn from(event: ProviderErrorEvent) -> Self {
        event.to_error_event()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::config::errors::SchemaError;

    #[test]
    fn test_provider_error_event_conversion() {
        // Create a provider config error
        let error = ProviderConfigError::Schema(SchemaError::missing_field("test_field"));

        // Create a provider error event
        let provider_error_event = ProviderErrorEvent::new(error, "test_provider");

        // Convert to error event
        let error_event = provider_error_event.to_error_event();

        // Verify error event properties
        assert_eq!(error_event.error_type, "provider_config_schema_error");
        assert_eq!(error_event.severity, ErrorSeverity::Error);

        // Verify parameters
        let provider_id = error_event.parameters.get("provider_id").unwrap();
        if let Value::String(id) = provider_id {
            assert_eq!(id, "test_provider");
        } else {
            panic!("provider_id is not a string");
        }

        let field = error_event.parameters.get("field").unwrap();
        if let Value::String(field_name) = field {
            assert_eq!(field_name, "test_field");
        } else {
            panic!("field is not a string");
        }
    }
}

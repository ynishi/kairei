use super::{
    base::ConfigError,
    error::{ValidationError, ValidationPhase},
    types::{ProviderCapabilities, ProviderConfigValidator, ProviderDependency, ProviderSpecificConfig, Schema},
};
use serde_json::Value;

// Legacy validation functions maintained for backward compatibility
pub fn validate_required_field<T>(field: &Option<T>, field_name: &str) -> Result<(), ConfigError> {
    field
        .as_ref()
        .ok_or_else(|| ConfigError::MissingField(field_name.to_string()))?;
    Ok(())
}

pub fn validate_range<T>(value: T, min: T, max: T, field_name: &str) -> Result<(), ConfigError>
where
    T: PartialOrd + std::fmt::Display,
{
    if value < min || value > max {
        return Err(ConfigError::InvalidValue {
            field: field_name.to_string(),
            message: format!("Value must be between {} and {}", min, max),
        });
    }
    Ok(())
}

/// Validates that all required properties exist in the config
pub fn check_required_properties(config: &Value, props: &[&str]) -> Result<(), ConfigError> {
    for prop in props {
        if config.get(prop).is_none() {
            return Err(ConfigError::MissingField(prop.to_string()));
        }
    }
    Ok(())
}

/// Validates that a property has the expected type
pub fn check_property_type(
    config: &Value,
    prop: &str,
    expected_type: &str,
) -> Result<(), ConfigError> {
    let value = config
        .get(prop)
        .ok_or_else(|| ConfigError::MissingField(prop.to_string()))?;

    let type_matches = match expected_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        _ => false,
    };

    if !type_matches {
        return Err(ConfigError::InvalidValue {
            field: prop.to_string(),
            message: format!(
                "Expected type '{}' but found '{}'",
                expected_type,
                get_value_type(value)
            ),
        });
    }
    Ok(())
}

/// Helper function to get the type of a JSON value as a string
fn get_value_type(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Object(_) => "object",
        Value::Array(_) => "array",
        Value::Null => "null",
    }
}

// New validation implementation
pub struct BaseValidator {
    provider_type: String,
    validation_phase: ValidationPhase,
}

impl BaseValidator {
    pub fn new(provider_type: String, validation_phase: ValidationPhase) -> Self {
        Self {
            provider_type,
            validation_phase,
        }
    }
}

impl ProviderConfigValidator for BaseValidator {
    fn validate_schema(&self, schema: &Schema) -> Result<(), ValidationError> {
        // Basic schema validation
        if schema.required_fields.is_empty() {
            return Err(ValidationError::Schema {
                message: "Schema must have at least one required field".to_string(),
                location: Default::default(),
                context: Default::default(),
            });
        }
        Ok(())
    }

    fn validate_provider_specific(&self, config: &ProviderSpecificConfig) -> Result<(), ValidationError> {
        // Basic provider-specific validation
        if config.config.is_empty() {
            return Err(ValidationError::ProviderSpecific {
                message: "Provider specific config cannot be empty".to_string(),
                location: Default::default(),
                context: Default::default(),
            });
        }
        Ok(())
    }

    fn validate_capabilities(&self, capabilities: &ProviderCapabilities) -> Result<(), ValidationError> {
        // Basic capability validation
        if capabilities.features.is_empty() {
            return Err(ValidationError::Capability {
                message: "Provider must specify at least one feature".to_string(),
                location: Default::default(),
                context: Default::default(),
            });
        }
        Ok(())
    }

    fn validate_dependencies(&self, dependencies: &[ProviderDependency]) -> Result<(), ValidationError> {
        // Basic dependency validation
        for dep in dependencies {
            if dep.version.is_empty() {
                return Err(ValidationError::Dependency {
                    message: format!("Dependency {} must specify a version", dep.name),
                    location: Default::default(),
                    context: Default::default(),
                });
            }
        }
        Ok(())
    }
}

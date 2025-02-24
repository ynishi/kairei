use serde_json::Value;
use super::base::ConfigError;

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
        if !config.get(prop).is_some() {
            return Err(ConfigError::MissingField(prop.to_string()));
        }
    }
    Ok(())
}

/// Validates that a property has the expected type
pub fn check_property_type(config: &Value, prop: &str, expected_type: &str) -> Result<(), ConfigError> {
    let value = config.get(prop).ok_or_else(|| ConfigError::MissingField(prop.to_string()))?;
    
    let type_matches = match expected_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        _ => false
    };

    if !type_matches {
        return Err(ConfigError::InvalidValue {
            field: prop.to_string(),
            message: format!("Expected type '{}' but found '{}'", expected_type, get_value_type(value)),
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

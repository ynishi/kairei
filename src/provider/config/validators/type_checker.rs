//! Type checker validator for provider configurations.
//!
//! This module defines a validator that performs compile-time type checking
//! for provider configurations.

use crate::provider::config::{
    errors::{ErrorContext, ProviderConfigError, SchemaError},
    validator::ProviderConfigValidator,
    validation::{check_required_properties, check_property_type},
};
use std::collections::HashMap;

/// Validator that performs compile-time type checking for provider configurations.
#[derive(Debug, Default)]
pub struct TypeCheckerValidator;

impl ProviderConfigValidator for TypeCheckerValidator {
    fn validate_schema(&self, config: &HashMap<String, serde_json::Value>) -> Result<(), ProviderConfigError> {
        // Check required properties
        let required_props = match config.get("type") {
            Some(serde_json::Value::String(plugin_type)) => {
                match plugin_type.as_str() {
                    "memory" => vec!["type"],
                    "rag" => vec!["type", "chunk_size", "max_tokens"],
                    "search" => vec!["type", "max_results"],
                    _ => vec!["type"],
                }
            }
            _ => {
                return Err(SchemaError::missing_field("type").into());
            }
        };

        check_required_properties(&serde_json::Value::Object(config.clone()), &required_props)
            .map_err(ProviderConfigError::from)?;

        // Check property types
        if let Some(serde_json::Value::String(plugin_type)) = config.get("type") {
            match plugin_type.as_str() {
                "memory" => {
                    if let Some(ttl) = config.get("ttl") {
                        check_property_type(&serde_json::Value::Object(config.clone()), "ttl", "number")
                            .map_err(ProviderConfigError::from)?;
                    }
                }
                "rag" => {
                    check_property_type(&serde_json::Value::Object(config.clone()), "chunk_size", "number")
                        .map_err(ProviderConfigError::from)?;
                    check_property_type(&serde_json::Value::Object(config.clone()), "max_tokens", "number")
                        .map_err(ProviderConfigError::from)?;
                    if let Some(_similarity) = config.get("similarity_threshold") {
                        check_property_type(&serde_json::Value::Object(config.clone()), "similarity_threshold", "number")
                            .map_err(ProviderConfigError::from)?;
                    }
                }
                "search" => {
                    check_property_type(&serde_json::Value::Object(config.clone()), "max_results", "number")
                        .map_err(ProviderConfigError::from)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn validate_provider_specific(&self, _config: &HashMap<String, serde_json::Value>) -> Result<(), ProviderConfigError> {
        // Type checker doesn't perform provider-specific validation
        Ok(())
    }

    fn validate_capabilities(&self, _config: &HashMap<String, serde_json::Value>) -> Result<(), ProviderConfigError> {
        // Type checker doesn't perform capability validation
        Ok(())
    }

    fn validate_dependencies(&self, _config: &HashMap<String, serde_json::Value>) -> Result<(), ProviderConfigError> {
        // Type checker doesn't perform dependency validation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_schema_valid_memory() {
        let validator = TypeCheckerValidator::default();
        let config = serde_json::from_value(json!({
            "type": "memory",
            "ttl": 3600
        })).unwrap();
        
        assert!(validator.validate_schema(&config).is_ok());
    }

    #[test]
    fn test_validate_schema_valid_rag() {
        let validator = TypeCheckerValidator::default();
        let config = serde_json::from_value(json!({
            "type": "rag",
            "chunk_size": 512,
            "max_tokens": 1000,
            "similarity_threshold": 0.7
        })).unwrap();
        
        assert!(validator.validate_schema(&config).is_ok());
    }

    #[test]
    fn test_validate_schema_missing_required() {
        let validator = TypeCheckerValidator::default();
        let config = serde_json::from_value(json!({
            "type": "rag",
            "chunk_size": 512
            // missing max_tokens
        })).unwrap();
        
        assert!(validator.validate_schema(&config).is_err());
    }

    #[test]
    fn test_validate_schema_invalid_type() {
        let validator = TypeCheckerValidator::default();
        let config = serde_json::from_value(json!({
            "type": "rag",
            "chunk_size": "not a number", // should be a number
            "max_tokens": 1000
        })).unwrap();
        
        assert!(validator.validate_schema(&config).is_err());
    }
}

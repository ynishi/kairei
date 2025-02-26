//! Type checker validator for provider configurations.
//!
//! This module defines a validator that performs compile-time type checking
//! for provider configurations.

use crate::provider::config::{
    errors::{ProviderConfigError, SchemaError},
    validation::{check_property_type, check_required_properties},
    validator::ProviderConfigValidator,
};
use serde_json::Map;
use std::collections::HashMap;

/// Validator that performs compile-time type checking for provider configurations.
#[derive(Debug, Default)]
pub struct TypeCheckerValidator;

impl ProviderConfigValidator for TypeCheckerValidator {
    fn validate_schema(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Check required properties
        let required_props = match config.get("type") {
            Some(serde_json::Value::String(plugin_type)) => match plugin_type.as_str() {
                "memory" => vec!["type"],
                "rag" => vec!["type", "chunk_size", "max_tokens"],
                "search" => vec!["type", "max_results"],
                _ => vec!["type"],
            },
            _ => {
                return Err(ProviderConfigError::Schema(Box::new(SchemaError::missing_field("type"))));
            }
        };

        let config_map: Map<String, serde_json::Value> = config.clone().into_iter().collect();
        check_required_properties(&serde_json::Value::Object(config_map), &required_props)
            .map_err(ProviderConfigError::Legacy)?;

        // Check property types
        if let Some(serde_json::Value::String(plugin_type)) = config.get("type") {
            match plugin_type.as_str() {
                "memory" => {
                    if let Some(_ttl) = config.get("ttl") {
                        let config_map: Map<String, serde_json::Value> = config.clone().into_iter().collect();
                        check_property_type(
                            &serde_json::Value::Object(config_map),
                            "ttl",
                            "number",
                        )
                        .map_err(ProviderConfigError::Legacy)?;
                    }
                }
                "rag" => {
                    let config_map: Map<String, serde_json::Value> = config.clone().into_iter().collect();
                    check_property_type(
                        &serde_json::Value::Object(config_map),
                        "chunk_size",
                        "number",
                    )
                    .map_err(ProviderConfigError::Legacy)?;
                    let config_map2: Map<String, serde_json::Value> = config.clone().into_iter().collect();
                    check_property_type(
                        &serde_json::Value::Object(config_map2),
                        "max_tokens",
                        "number",
                    )
                    .map_err(ProviderConfigError::Legacy)?;
                    if let Some(_similarity) = config.get("similarity_threshold") {
                        let config_map3: Map<String, serde_json::Value> = config.clone().into_iter().collect();
                        check_property_type(
                            &serde_json::Value::Object(config_map3),
                            "similarity_threshold",
                            "number",
                        )
                        .map_err(ProviderConfigError::Legacy)?;
                    }
                }
                "search" => {
                    let config_map4: Map<String, serde_json::Value> = config.clone().into_iter().collect();
                    check_property_type(
                        &serde_json::Value::Object(config_map4),
                        "max_results",
                        "number",
                    )
                    .map_err(ProviderConfigError::Legacy)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn validate_provider_specific(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Type checker doesn't perform provider-specific validation
        Ok(())
    }

    fn validate_capabilities(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Type checker doesn't perform capability validation
        Ok(())
    }

    fn validate_dependencies(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
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
        let validator = TypeCheckerValidator;
        let config = serde_json::from_value(json!({
            "type": "memory",
            "ttl": 3600
        }))
        .unwrap();

        assert!(validator.validate_schema(&config).is_ok());
    }

    #[test]
    fn test_validate_schema_valid_rag() {
        let validator = TypeCheckerValidator;
        let config = serde_json::from_value(json!({
            "type": "rag",
            "chunk_size": 512,
            "max_tokens": 1000,
            "similarity_threshold": 0.7
        }))
        .unwrap();

        assert!(validator.validate_schema(&config).is_ok());
    }

    #[test]
    fn test_validate_schema_missing_required() {
        let validator = TypeCheckerValidator;
        let config = serde_json::from_value(json!({
            "type": "rag",
            "chunk_size": 512
            // missing max_tokens
        }))
        .unwrap();

        assert!(validator.validate_schema(&config).is_err());
    }

    #[test]
    fn test_validate_schema_invalid_type() {
        let validator = TypeCheckerValidator;
        let config = serde_json::from_value(json!({
            "type": "rag",
            "chunk_size": "not a number", // should be a number
            "max_tokens": 1000
        }))
        .unwrap();

        assert!(validator.validate_schema(&config).is_err());
    }
}

//! Type checker validator for provider configurations.
//!
//! This module defines a validator that performs compile-time type checking
//! for provider configurations.

use crate::provider::config::{
    errors::{ErrorContext, ErrorSeverity, ProviderConfigError, SchemaError, ValidationError},
    validation::{check_property_type, check_required_properties},
    validator::ProviderConfigValidator,
};
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
                return Err(SchemaError::missing_field("type").into());
            }
        };

        // Convert HashMap to serde_json::Map for validation functions
        let mut json_map = serde_json::Map::new();
        for (k, v) in config {
            json_map.insert(k.clone(), v.clone());
        }
        let json_obj = serde_json::Value::Object(json_map);

        check_required_properties(&json_obj, &required_props).map_err(ProviderConfigError::from)?;

        // Check property types
        if let Some(serde_json::Value::String(plugin_type)) = config.get("type") {
            match plugin_type.as_str() {
                "memory" => {
                    if let Some(_ttl) = config.get("ttl") {
                        check_property_type(&json_obj, "ttl", "number")
                            .map_err(ProviderConfigError::from)?;
                    }
                }
                "rag" => {
                    check_property_type(&json_obj, "chunk_size", "number")
                        .map_err(ProviderConfigError::from)?;
                    check_property_type(&json_obj, "max_tokens", "number")
                        .map_err(ProviderConfigError::from)?;
                    if let Some(_similarity) = config.get("similarity_threshold") {
                        check_property_type(&json_obj, "similarity_threshold", "number")
                            .map_err(ProviderConfigError::from)?;
                    }
                }
                "search" => {
                    check_property_type(&json_obj, "max_results", "number")
                        .map_err(ProviderConfigError::from)?;
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

    fn validate_schema_warnings(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        let mut warnings = Vec::new();

        // Check for deprecated fields based on provider type
        if let Some(serde_json::Value::String(plugin_type)) = config.get("type") {
            match plugin_type.as_str() {
                "memory" => {
                    // Check for deprecated fields in memory configuration
                    if config.contains_key("legacy_mode") {
                        let mut context = ErrorContext::new_with_field("legacy_mode");
                        context = context.with_severity(ErrorSeverity::Warning);
                        context = context.with_suggestion("The 'legacy_mode' field is deprecated and will be removed in a future version.");
                        warnings.push(
                            ValidationError::InvalidValue {
                                message: "Deprecated field 'legacy_mode' is used".to_string(),
                                context,
                            }
                            .into(),
                        );
                    }
                }
                "rag" => {
                    // Check for deprecated fields in RAG configuration
                    if config.contains_key("use_legacy_chunking") {
                        let mut context = ErrorContext::new_with_field("use_legacy_chunking");
                        context = context.with_severity(ErrorSeverity::Warning);
                        context = context.with_suggestion("The 'use_legacy_chunking' field is deprecated. Use 'chunking_strategy' instead.");
                        warnings.push(
                            ValidationError::InvalidValue {
                                message: "Deprecated field 'use_legacy_chunking' is used"
                                    .to_string(),
                                context,
                            }
                            .into(),
                        );
                    }

                    // Check for deprecated similarity configuration
                    if config.contains_key("similarity_method")
                        && !config.contains_key("similarity_strategy")
                    {
                        let mut context = ErrorContext::new_with_field("similarity_method");
                        context = context.with_severity(ErrorSeverity::Warning);
                        context = context.with_suggestion("The 'similarity_method' field is deprecated. Use 'similarity_strategy' instead.");
                        warnings.push(
                            ValidationError::InvalidValue {
                                message: "Deprecated field 'similarity_method' is used".to_string(),
                                context,
                            }
                            .into(),
                        );
                    }
                }
                "search" => {
                    // Check for deprecated fields in search configuration
                    if config.contains_key("use_fuzzy") {
                        let mut context = ErrorContext::new_with_field("use_fuzzy");
                        context = context.with_severity(ErrorSeverity::Warning);
                        context = context.with_suggestion("The 'use_fuzzy' field is deprecated. Use 'search_strategy' with value 'fuzzy' instead.");
                        warnings.push(
                            ValidationError::InvalidValue {
                                message: "Deprecated field 'use_fuzzy' is used".to_string(),
                                context,
                            }
                            .into(),
                        );
                    }
                }
                _ => {}
            }
        }

        warnings
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
        }))
        .unwrap();

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
        }))
        .unwrap();

        assert!(validator.validate_schema(&config).is_ok());
    }

    #[test]
    fn test_validate_schema_missing_required() {
        let validator = TypeCheckerValidator::default();
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
        let validator = TypeCheckerValidator::default();
        let config = serde_json::from_value(json!({
            "type": "rag",
            "chunk_size": "not a number", // should be a number
            "max_tokens": 1000
        }))
        .unwrap();

        assert!(validator.validate_schema(&config).is_err());
    }
}

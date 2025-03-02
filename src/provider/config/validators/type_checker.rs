//! Type checker validator for provider configurations.
//!
//! This module defines a validator that performs compile-time type checking
//! for provider configurations. The type checker validator focuses on validating
//! the structure and types of provider configurations before they are used at runtime.
//!
//! # Validation Process
//!
//! The type checker validator performs the following validations:
//!
//! 1. **Required Properties**: Ensures that all required properties for a specific
//!    provider type are present in the configuration.
//! 2. **Property Types**: Validates that property values have the correct types
//!    (e.g., numbers, strings, booleans).
//! 3. **Deprecated Fields**: Generates warnings for the use of deprecated fields.
//!
//! # Provider Types
//!
//! The validator supports different provider types, each with its own validation rules:
//!
//! - **Memory Provider**: Validates memory-specific configuration options.
//! - **RAG Provider**: Validates Retrieval-Augmented Generation specific configuration.
//! - **Search Provider**: Validates search-specific configuration options.
//!
//! # Usage
//!
//! The type checker validator is typically used during the initialization phase
//! of providers to ensure that configurations are valid before they are used.

use crate::provider::config::{
    errors::{ErrorContext, ErrorSeverity, ProviderConfigError, SchemaError, ValidationError},
    validation::{check_property_type, check_required_properties},
    validator::ProviderConfigValidator,
};
use std::collections::HashMap;

/// Validator that performs compile-time type checking for provider configurations.
///
/// The `TypeCheckerValidator` focuses on validating the structure and types of
/// provider configurations during the compile-time phase. It ensures that:
///
/// 1. All required properties for a specific provider type are present
/// 2. Property values have the correct types
/// 3. Deprecated fields are identified and warnings are generated
///
/// # Provider Type Validation
///
/// The validator applies different validation rules based on the provider type:
///
/// ## Memory Provider
///
/// - Required fields: `type`
/// - Optional fields: `ttl` (number)
/// - Deprecated fields: `legacy_mode`
///
/// ## RAG Provider
///
/// - Required fields: `type`, `chunk_size` (number), `max_tokens` (number)
/// - Optional fields: `similarity_threshold` (number)
/// - Deprecated fields: `use_legacy_chunking`, `similarity_method`
///
/// ## Search Provider
///
/// - Required fields: `type`, `max_results` (number)
/// - Deprecated fields: `use_fuzzy`
///
/// # Examples
///
/// ```rust,ignore
/// use std::collections::HashMap;
/// use serde_json::json;
/// use kairei::provider::config::validator::ProviderConfigValidator;
/// use kairei::provider::config::validators::type_checker::TypeCheckerValidator;
///
/// // Create a validator
/// let validator = TypeCheckerValidator;
///
/// // Create a valid memory provider configuration
/// let memory_config = serde_json::from_value(json!({
///     "type": "memory",
///     "ttl": 3600
/// })).unwrap();
///
/// // Validate the configuration
/// match validator.validate_schema(&memory_config) {
///     Ok(()) => println!("Memory configuration is valid"),
///     Err(error) => println!("Validation error: {}", error),
/// }
///
/// // Create a valid RAG provider configuration
/// let rag_config = serde_json::from_value(json!({
///     "type": "rag",
///     "chunk_size": 512,
///     "max_tokens": 1000,
///     "similarity_threshold": 0.7
/// })).unwrap();
///
/// // Validate the configuration
/// match validator.validate_schema(&rag_config) {
///     Ok(()) => println!("RAG configuration is valid"),
///     Err(error) => println!("Validation error: {}", error),
/// }
/// ```
#[derive(Debug, Default)]
pub struct TypeCheckerValidator;

impl ProviderConfigValidator for TypeCheckerValidator {
    /// Validates the schema of a provider configuration.
    ///
    /// This method performs compile-time type checking for provider configurations,
    /// ensuring that:
    ///
    /// 1. All required properties for the specific provider type are present
    /// 2. Property values have the correct types
    ///
    /// # Required Properties by Provider Type
    ///
    /// - **Memory Provider**: `type`
    /// - **RAG Provider**: `type`, `chunk_size`, `max_tokens`
    /// - **Search Provider**: `type`, `max_results`
    ///
    /// # Property Type Validation
    ///
    /// - **Memory Provider**:
    ///   - `ttl`: number (optional)
    ///
    /// - **RAG Provider**:
    ///   - `chunk_size`: number (required)
    ///   - `max_tokens`: number (required)
    ///   - `similarity_threshold`: number (optional)
    ///
    /// - **Search Provider**:
    ///   - `max_results`: number (required)
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the schema is valid
    /// * `Err(ProviderConfigError)` if the schema is invalid, with details about the error
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

    /// Validates provider-specific aspects of a configuration.
    ///
    /// The type checker validator does not perform provider-specific validation
    /// as this is handled by the evaluator validator at runtime. This method
    /// always returns `Ok(())`.
    ///
    /// # Parameters
    ///
    /// * `_config` - A HashMap containing the provider configuration (unused)
    ///
    /// # Returns
    ///
    /// * `Ok(())` always
    fn validate_provider_specific(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Type checker doesn't perform provider-specific validation
        Ok(())
    }

    /// Validates that the configuration is compatible with the required capabilities.
    ///
    /// The type checker validator does not perform capability validation
    /// as this is handled by the evaluator validator at runtime. This method
    /// always returns `Ok(())`.
    ///
    /// # Parameters
    ///
    /// * `_config` - A HashMap containing the provider configuration (unused)
    ///
    /// # Returns
    ///
    /// * `Ok(())` always
    fn validate_capabilities(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Type checker doesn't perform capability validation
        Ok(())
    }

    /// Validates that the configuration's dependencies are satisfied.
    ///
    /// The type checker validator does not perform dependency validation
    /// as this is handled by the evaluator validator at runtime. This method
    /// always returns `Ok(())`.
    ///
    /// # Parameters
    ///
    /// * `_config` - A HashMap containing the provider configuration (unused)
    ///
    /// # Returns
    ///
    /// * `Ok(())` always
    fn validate_dependencies(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Type checker doesn't perform dependency validation
        Ok(())
    }

    /// Validates the schema of the configuration and returns warnings.
    ///
    /// This method checks for non-critical issues in the schema structure
    /// and returns warnings instead of errors. It specifically identifies
    /// deprecated fields in provider configurations and generates appropriate
    /// warnings with suggestions for alternatives.
    ///
    /// # Deprecated Fields by Provider Type
    ///
    /// - **Memory Provider**:
    ///   - `legacy_mode`: Use standard configuration instead
    ///
    /// - **RAG Provider**:
    ///   - `use_legacy_chunking`: Use `chunking_strategy` instead
    ///   - `similarity_method`: Use `similarity_strategy` instead
    ///
    /// - **Search Provider**:
    ///   - `use_fuzzy`: Use `search_strategy` with value `fuzzy` instead
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// A Vec of `ProviderConfigError` objects representing warnings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = TypeCheckerValidator;
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("memory"));
    /// config.insert("legacy_mode".to_string(), json!(true)); // Deprecated field
    ///
    /// // Get schema warnings
    /// let warnings = validator.validate_schema_warnings(&config);
    /// for warning in warnings {
    ///     println!("Warning: {}", warning);
    /// }
    /// ```
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

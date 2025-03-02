//! Evaluator validator for provider configurations.
//!
//! This module defines a validator that performs runtime validation
//! for provider configurations. The evaluator validator focuses on validating
//! provider-specific aspects, capabilities, and dependencies at runtime.
//!
//! # Validation Process
//!
//! The evaluator validator performs the following validations:
//!
//! 1. **Provider-Specific Validation**: Validates provider-specific configuration values
//!    and constraints, such as numeric ranges and thresholds.
//! 2. **Capability Validation**: Ensures that the provider has the required capabilities
//!    for its type (e.g., memory capability for memory providers).
//! 3. **Dependency Validation**: Verifies that dependencies are properly configured
//!    with required fields and valid version formats.
//! 4. **Configuration Warnings**: Generates warnings for suboptimal configurations
//!    that may impact performance or quality.
//!
//! # Provider Types
//!
//! The validator supports different provider types, each with its own validation rules:
//!
//! - **Memory Provider**: Validates TTL values and memory capabilities.
//! - **RAG Provider**: Validates chunk sizes, token limits, similarity thresholds, and RAG capabilities.
//! - **Search Provider**: Validates search result limits and search capabilities.
//!
//! # Usage
//!
//! The evaluator validator is typically used during the runtime phase of providers
//! to ensure that configurations are valid before they are used for actual operations.

use crate::provider::config::{
    errors::{ErrorContext, ErrorSeverity, ProviderConfigError, ProviderError, ValidationError},
    validator::ProviderConfigValidator,
};
use std::collections::HashMap;

/// Validator that performs runtime validation for provider configurations.
///
/// The `EvaluatorValidator` focuses on validating provider configurations during runtime,
/// ensuring that:
///
/// 1. Provider-specific values are within valid ranges
/// 2. Required capabilities are present for each provider type
/// 3. Dependencies are properly configured with valid version formats
/// 4. Configurations are optimized for performance and quality
///
/// # Provider Type Validation
///
/// The validator applies different validation rules based on the provider type:
///
/// ## Memory Provider
///
/// - TTL must be greater than 0
/// - Requires memory capability
/// - Warns about very low TTL values (< 60 seconds) that may impact performance
/// - Warns about very high TTL values (> 30 days) that may impact resource usage
///
/// ## RAG Provider
///
/// - Chunk size must be greater than 0
/// - Max tokens must be greater than 0
/// - Similarity threshold must be between 0.0 and 1.0
/// - Requires RAG capability
/// - Warns about very small chunk sizes (< 100) that may impact quality
/// - Warns about very large chunk sizes (> 1000) that may impact performance
/// - Warns about very low similarity thresholds (< 0.3) that may impact quality
/// - Warns about very high similarity thresholds (> 0.9) that may exclude relevant results
///
/// ## Search Provider
///
/// - Max results must be greater than 0
/// - Requires search capability
/// - Warns about very high max results values (> 100) that may impact performance
///
/// # Examples
///
/// ```rust,ignore
/// use std::collections::HashMap;
/// use serde_json::json;
/// use kairei::provider::config::validator::ProviderConfigValidator;
/// use kairei::provider::config::validators::evaluator::EvaluatorValidator;
///
/// // Create a validator
/// let validator = EvaluatorValidator;
///
/// // Create a valid memory provider configuration
/// let memory_config = serde_json::from_value(json!({
///     "type": "memory",
///     "ttl": 3600,
///     "capabilities": {
///         "memory": true
///     }
/// })).unwrap();
///
/// // Validate provider-specific aspects
/// match validator.validate_provider_specific(&memory_config) {
///     Ok(()) => println!("Provider-specific validation passed"),
///     Err(error) => println!("Validation error: {}", error),
/// }
///
/// // Validate capabilities
/// match validator.validate_capabilities(&memory_config) {
///     Ok(()) => println!("Capability validation passed"),
///     Err(error) => println!("Capability error: {}", error),
/// }
///
/// // Check for warnings
/// let warnings = validator.validate_provider_specific_warnings(&memory_config);
/// for warning in warnings {
///     println!("Warning: {}", warning);
/// }
/// ```
#[derive(Debug, Default)]
pub struct EvaluatorValidator;

impl ProviderConfigValidator for EvaluatorValidator {
    /// Validates the schema of a provider configuration.
    ///
    /// The evaluator validator does not perform schema validation
    /// as this is handled by the type checker validator at compile time.
    /// This method always returns `Ok(())`.
    ///
    /// # Parameters
    ///
    /// * `_config` - A HashMap containing the provider configuration (unused)
    ///
    /// # Returns
    ///
    /// * `Ok(())` always
    fn validate_schema(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Evaluator doesn't perform schema validation
        Ok(())
    }

    /// Validates provider-specific aspects of a configuration.
    ///
    /// This method performs runtime validation of provider-specific configuration values,
    /// ensuring that:
    ///
    /// - For Memory Providers:
    ///   - TTL values are greater than 0
    ///
    /// - For RAG Providers:
    ///   - Chunk size is greater than 0
    ///   - Max tokens is greater than 0
    ///   - Similarity threshold is between 0.0 and 1.0
    ///
    /// - For Search Providers:
    ///   - Max results is greater than 0
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the provider-specific validation passes
    /// * `Err(ProviderConfigError)` if validation fails, with details about the error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Create a valid memory provider configuration
    /// let memory_config = serde_json::from_value(json!({
    ///     "type": "memory",
    ///     "ttl": 3600
    /// })).unwrap();
    ///
    /// // Validate provider-specific aspects
    /// match validator.validate_provider_specific(&memory_config) {
    ///     Ok(()) => println!("Provider-specific validation passed"),
    ///     Err(error) => println!("Validation error: {}", error),
    /// }
    /// ```
    fn validate_provider_specific(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Validate provider-specific aspects
        if let Some(serde_json::Value::String(plugin_type)) = config.get("type") {
            match plugin_type.as_str() {
                "memory" => {
                    if let Some(serde_json::Value::Number(ttl)) = config.get("ttl") {
                        if let Some(ttl) = ttl.as_u64() {
                            if ttl == 0 {
                                return Err(ValidationError::invalid_value(
                                    "ttl",
                                    "TTL must be greater than 0",
                                )
                                .into());
                            }
                        }
                    }
                }
                "rag" => {
                    if let Some(serde_json::Value::Number(chunk_size)) = config.get("chunk_size") {
                        if let Some(chunk_size) = chunk_size.as_u64() {
                            if chunk_size == 0 {
                                return Err(ValidationError::invalid_value(
                                    "chunk_size",
                                    "Chunk size must be greater than 0",
                                )
                                .into());
                            }
                        }
                    }

                    if let Some(serde_json::Value::Number(max_tokens)) = config.get("max_tokens") {
                        if let Some(max_tokens) = max_tokens.as_u64() {
                            if max_tokens == 0 {
                                return Err(ValidationError::invalid_value(
                                    "max_tokens",
                                    "Max tokens must be greater than 0",
                                )
                                .into());
                            }
                        }
                    }

                    if let Some(serde_json::Value::Number(similarity_threshold)) =
                        config.get("similarity_threshold")
                    {
                        if let Some(similarity_threshold) = similarity_threshold.as_f64() {
                            if !(0.0..=1.0).contains(&similarity_threshold) {
                                return Err(ValidationError::invalid_value(
                                    "similarity_threshold",
                                    "Similarity threshold must be between 0.0 and 1.0",
                                )
                                .into());
                            }
                        }
                    }
                }
                "search" => {
                    if let Some(serde_json::Value::Number(max_results)) = config.get("max_results")
                    {
                        if let Some(max_results) = max_results.as_u64() {
                            if max_results == 0 {
                                return Err(ValidationError::invalid_value(
                                    "max_results",
                                    "Max results must be greater than 0",
                                )
                                .into());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Validates that the configuration is compatible with the required capabilities.
    ///
    /// This method ensures that providers have the necessary capabilities for their type:
    ///
    /// - Memory Providers require the `memory` capability
    /// - RAG Providers require the `rag` capability
    /// - Search Providers require the `search` capability
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the capability validation passes
    /// * `Err(ProviderConfigError)` if validation fails, with details about the error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Create a valid RAG provider configuration with capabilities
    /// let rag_config = serde_json::from_value(json!({
    ///     "type": "rag",
    ///     "capabilities": {
    ///         "rag": true
    ///     }
    /// })).unwrap();
    ///
    /// // Validate capabilities
    /// match validator.validate_capabilities(&rag_config) {
    ///     Ok(()) => println!("Capability validation passed"),
    ///     Err(error) => println!("Capability error: {}", error),
    /// }
    /// ```
    fn validate_capabilities(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Validate capabilities
        if let Some(serde_json::Value::String(plugin_type)) = config.get("type") {
            match plugin_type.as_str() {
                "memory" => {
                    // Memory plugin requires memory capability
                    if let Some(serde_json::Value::Object(capabilities)) =
                        config.get("capabilities")
                    {
                        if let Some(serde_json::Value::Bool(memory)) = capabilities.get("memory") {
                            if !memory {
                                return Err(ProviderError::capability(
                                    "capabilities.memory",
                                    "Memory plugin requires memory capability",
                                )
                                .into());
                            }
                        } else {
                            return Err(ProviderError::capability(
                                "capabilities.memory",
                                "Memory plugin requires memory capability",
                            )
                            .into());
                        }
                    }
                }
                "rag" => {
                    // RAG plugin requires rag capability
                    if let Some(serde_json::Value::Object(capabilities)) =
                        config.get("capabilities")
                    {
                        if let Some(serde_json::Value::Bool(rag)) = capabilities.get("rag") {
                            if !rag {
                                return Err(ProviderError::capability(
                                    "capabilities.rag",
                                    "RAG plugin requires rag capability",
                                )
                                .into());
                            }
                        } else {
                            return Err(ProviderError::capability(
                                "capabilities.rag",
                                "RAG plugin requires rag capability",
                            )
                            .into());
                        }
                    }
                }
                "search" => {
                    // Search plugin requires search capability
                    if let Some(serde_json::Value::Object(capabilities)) =
                        config.get("capabilities")
                    {
                        if let Some(serde_json::Value::Bool(search)) = capabilities.get("search") {
                            if !search {
                                return Err(ProviderError::capability(
                                    "capabilities.search",
                                    "Search plugin requires search capability",
                                )
                                .into());
                            }
                        } else {
                            return Err(ProviderError::capability(
                                "capabilities.search",
                                "Search plugin requires search capability",
                            )
                            .into());
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Validates that the configuration's dependencies are satisfied.
    ///
    /// This method verifies that dependencies are properly configured with:
    ///
    /// 1. Required fields: `name` and `version`
    /// 2. Valid version format (must contain periods, e.g., "1.0.0")
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the dependency validation passes
    /// * `Err(ProviderConfigError)` if validation fails, with details about the error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Create a configuration with dependencies
    /// let config = serde_json::from_value(json!({
    ///     "dependencies": [
    ///         {
    ///             "name": "some-lib",
    ///             "version": "1.0.0"
    ///         }
    ///     ]
    /// })).unwrap();
    ///
    /// // Validate dependencies
    /// match validator.validate_dependencies(&config) {
    ///     Ok(()) => println!("Dependency validation passed"),
    ///     Err(error) => println!("Dependency error: {}", error),
    /// }
    /// ```
    fn validate_dependencies(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Validate dependencies
        if let Some(serde_json::Value::Array(dependencies)) = config.get("dependencies") {
            for (i, dependency) in dependencies.iter().enumerate() {
                if let serde_json::Value::Object(dep) = dependency {
                    // Check required fields
                    if !dep.contains_key("name") {
                        return Err(ValidationError::dependency_error(
                            format!("dependencies[{}].name", i),
                            "Dependency name is required",
                        )
                        .into());
                    }

                    if !dep.contains_key("version") {
                        return Err(ValidationError::dependency_error(
                            format!("dependencies[{}].version", i),
                            "Dependency version is required",
                        )
                        .into());
                    }

                    // Check version format
                    if let Some(serde_json::Value::String(version)) = dep.get("version") {
                        if !version.contains('.') {
                            return Err(ValidationError::dependency_error(
                                format!("dependencies[{}].version", i),
                                "Dependency version must be in format x.y.z",
                            )
                            .into());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validates the provider-specific aspects of the configuration and returns warnings.
    ///
    /// This method checks for non-critical issues in provider configurations
    /// and returns warnings instead of errors. It identifies suboptimal configurations
    /// that may impact performance or quality and generates appropriate warnings
    /// with suggestions for improvements.
    ///
    /// # Warning Checks by Provider Type
    ///
    /// - **Memory Provider**:
    ///   - TTL < 60 seconds: May impact performance
    ///   - TTL > 30 days: May impact resource usage
    ///
    /// - **RAG Provider**:
    ///   - Chunk size < 100: May impact quality
    ///   - Chunk size > 1000: May impact performance
    ///   - Similarity threshold < 0.3: May impact result quality
    ///   - Similarity threshold > 0.9: May exclude relevant results
    ///
    /// - **Search Provider**:
    ///   - Max results > 100: May impact performance
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
    /// // Create a memory provider configuration with a low TTL
    /// let config = serde_json::from_value(json!({
    ///     "type": "memory",
    ///     "ttl": 30 // Low TTL that will generate a warning
    /// })).unwrap();
    ///
    /// // Get warnings
    /// let warnings = validator.validate_provider_specific_warnings(&config);
    /// for warning in warnings {
    ///     println!("Warning: {}", warning);
    ///     if let Some(suggestion) = warning.suggestion() {
    ///         println!("Suggestion: {}", suggestion);
    ///     }
    /// }
    /// ```
    fn validate_provider_specific_warnings(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        let mut warnings = Vec::new();

        // Check for suboptimal configurations based on provider type
        if let Some(serde_json::Value::String(plugin_type)) = config.get("type") {
            match plugin_type.as_str() {
                "memory" => {
                    // Warn about low TTL values which may impact performance
                    if let Some(serde_json::Value::Number(ttl)) = config.get("ttl") {
                        if let Some(ttl) = ttl.as_u64() {
                            if ttl > 0 && ttl < 60 {
                                let mut context = ErrorContext::new_with_field("ttl");
                                context = context.with_severity(ErrorSeverity::Warning);
                                context = context.with_suggestion("Consider using a TTL of at least 60 seconds for better performance.");
                                warnings.push(
                                    ValidationError::InvalidValue {
                                        message: "TTL is very low, which may impact performance"
                                            .to_string(),
                                        context,
                                    }
                                    .into(),
                                );
                            } else if ttl > 86400 * 30 {
                                // 30 days
                                let mut context = ErrorContext::new_with_field("ttl");
                                context = context.with_severity(ErrorSeverity::Warning);
                                context = context.with_suggestion(
                                    "Consider using a lower TTL to conserve memory resources.",
                                );
                                warnings.push(
                                    ValidationError::InvalidValue {
                                        message:
                                            "TTL is very high, which may impact resource usage"
                                                .to_string(),
                                        context,
                                    }
                                    .into(),
                                );
                            }
                        }
                    }
                }
                "rag" => {
                    // Warn about suboptimal chunk sizes
                    if let Some(serde_json::Value::Number(chunk_size)) = config.get("chunk_size") {
                        if let Some(chunk_size) = chunk_size.as_u64() {
                            if chunk_size > 0 && chunk_size < 100 {
                                let mut context = ErrorContext::new_with_field("chunk_size");
                                context = context.with_severity(ErrorSeverity::Warning);
                                context = context.with_suggestion("Consider using a chunk size of at least 100 for better results.");
                                warnings.push(
                                    ValidationError::InvalidValue {
                                        message:
                                            "Chunk size is very small, which may impact quality"
                                                .to_string(),
                                        context,
                                    }
                                    .into(),
                                );
                            } else if chunk_size > 1000 {
                                let mut context = ErrorContext::new_with_field("chunk_size");
                                context = context.with_severity(ErrorSeverity::Warning);
                                context = context.with_suggestion(
                                    "Consider using a smaller chunk size for better performance.",
                                );
                                warnings.push(
                                    ValidationError::InvalidValue {
                                        message:
                                            "Chunk size is very large, which may impact performance"
                                                .to_string(),
                                        context,
                                    }
                                    .into(),
                                );
                            }
                        }
                    }

                    // Warn about suboptimal similarity threshold
                    if let Some(serde_json::Value::Number(similarity_threshold)) =
                        config.get("similarity_threshold")
                    {
                        if let Some(similarity_threshold) = similarity_threshold.as_f64() {
                            if similarity_threshold < 0.3 {
                                let mut context =
                                    ErrorContext::new_with_field("similarity_threshold");
                                context = context.with_severity(ErrorSeverity::Warning);
                                context = context.with_suggestion("Consider using a higher similarity threshold for better quality results.");
                                warnings.push(ValidationError::InvalidValue {
                                    message: "Similarity threshold is very low, which may impact result quality".to_string(),
                                    context,
                                }.into());
                            } else if similarity_threshold > 0.9 {
                                let mut context =
                                    ErrorContext::new_with_field("similarity_threshold");
                                context = context.with_severity(ErrorSeverity::Warning);
                                context = context.with_suggestion("Consider using a lower similarity threshold to avoid missing relevant results.");
                                warnings.push(ValidationError::InvalidValue {
                                    message: "Similarity threshold is very high, which may exclude relevant results".to_string(),
                                    context,
                                }.into());
                            }
                        }
                    }
                }
                "search" => {
                    // Warn about high max_results values
                    if let Some(serde_json::Value::Number(max_results)) = config.get("max_results")
                    {
                        if let Some(max_results) = max_results.as_u64() {
                            if max_results > 100 {
                                let mut context = ErrorContext::new_with_field("max_results");
                                context = context.with_severity(ErrorSeverity::Warning);
                                context = context.with_suggestion("Consider using a lower max_results value for better performance.");
                                warnings.push(
                                    ValidationError::InvalidValue {
                                        message:
                                            "Max results is very high, which may impact performance"
                                                .to_string(),
                                        context,
                                    }
                                    .into(),
                                );
                            }
                        }
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
    fn test_validate_provider_specific_valid_memory() {
        let validator = EvaluatorValidator;
        let config = serde_json::from_value(json!({
            "type": "memory",
            "ttl": 3600
        }))
        .unwrap();

        assert!(validator.validate_provider_specific(&config).is_ok());
    }

    #[test]
    fn test_validate_provider_specific_invalid_memory() {
        let validator = EvaluatorValidator;
        let config = serde_json::from_value(json!({
            "type": "memory",
            "ttl": 0 // Invalid: must be > 0
        }))
        .unwrap();

        assert!(validator.validate_provider_specific(&config).is_err());
    }

    #[test]
    fn test_validate_capabilities_valid_rag() {
        let validator = EvaluatorValidator;
        let config = serde_json::from_value(json!({
            "type": "rag",
            "capabilities": {
                "rag": true
            }
        }))
        .unwrap();

        assert!(validator.validate_capabilities(&config).is_ok());
    }

    #[test]
    fn test_validate_capabilities_invalid_rag() {
        let validator = EvaluatorValidator;
        let config = serde_json::from_value(json!({
            "type": "rag",
            "capabilities": {
                "rag": false // Invalid: rag capability required
            }
        }))
        .unwrap();

        assert!(validator.validate_capabilities(&config).is_err());
    }

    #[test]
    fn test_validate_dependencies_valid() {
        let validator = EvaluatorValidator;
        let config = serde_json::from_value(json!({
            "dependencies": [
                {
                    "name": "some-lib",
                    "version": "1.0.0"
                }
            ]
        }))
        .unwrap();

        assert!(validator.validate_dependencies(&config).is_ok());
    }

    #[test]
    fn test_validate_dependencies_invalid() {
        let validator = EvaluatorValidator;
        let config = serde_json::from_value(json!({
            "dependencies": [
                {
                    "name": "some-lib",
                    "version": "1" // Invalid: must be in format x.y.z
                }
            ]
        }))
        .unwrap();

        assert!(validator.validate_dependencies(&config).is_err());
    }
}

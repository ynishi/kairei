//! Evaluator validator for provider configurations.
//!
//! This module defines a validator that performs runtime validation
//! for provider configurations.

use crate::provider::config::{
    errors::{ErrorContext, ErrorSeverity, ProviderConfigError, ProviderError, ValidationError},
    validator::ProviderConfigValidator,
};
use std::collections::HashMap;

/// Validator that performs runtime validation for provider configurations.
#[derive(Debug, Default)]
pub struct EvaluatorValidator;

impl ProviderConfigValidator for EvaluatorValidator {
    fn validate_schema(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        // Evaluator doesn't perform schema validation
        Ok(())
    }

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

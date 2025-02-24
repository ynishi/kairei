use crate::{
    eval::expression::Value,
    type_checker::{TypeCheckError, TypeCheckResult},
};
use std::collections::HashMap;

/// Trait for plugin configuration validation
pub trait PluginValidator {
    /// Validates the basic structure of a plugin configuration
    fn validate_basic_structure(&self, config: &HashMap<String, Value>) -> TypeCheckResult<()>;

    /// Validates plugin-specific configuration requirements
    #[allow(dead_code)]
    fn validate_plugin_specific(&self, config: &HashMap<String, Value>) -> TypeCheckResult<()>;
}

/// Common validator implementation for plugin configurations
pub struct CommonPluginValidator;

impl PluginValidator for CommonPluginValidator {
    fn validate_basic_structure(&self, config: &HashMap<String, Value>) -> TypeCheckResult<()> {
        // Validate required provider_type field
        let _provider_type = config.get("provider_type").ok_or_else(|| {
            TypeCheckError::invalid_type_arguments(
                "Missing required field 'provider_type'".to_string(),
                Default::default(),
            )
        })?;

        // Validate required name field
        let _name = config.get("name").ok_or_else(|| {
            TypeCheckError::invalid_type_arguments(
                "Missing required field 'name'".to_string(),
                Default::default(),
            )
        })?;

        Ok(())
    }

    fn validate_plugin_specific(&self, _config: &HashMap<String, Value>) -> TypeCheckResult<()> {
        // Base implementation does no plugin-specific validation
        // This will be implemented by specific plugin validators
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_basic_structure_success() {
        let validator = CommonPluginValidator;
        let mut config = HashMap::new();
        config.insert(
            "provider_type".to_string(),
            Value::String("test".to_string()),
        );
        config.insert("name".to_string(), Value::String("test".to_string()));

        assert!(validator.validate_basic_structure(&config).is_ok());
    }

    #[test]
    fn test_validate_basic_structure_missing_provider_type() {
        let validator = CommonPluginValidator;
        let mut config = HashMap::new();
        config.insert("name".to_string(), Value::String("test".to_string()));

        assert!(validator.validate_basic_structure(&config).is_err());
    }

    #[test]
    fn test_validate_basic_structure_missing_name() {
        let validator = CommonPluginValidator;
        let mut config = HashMap::new();
        config.insert(
            "provider_type".to_string(),
            Value::String("test".to_string()),
        );

        assert!(validator.validate_basic_structure(&config).is_err());
    }
}

use crate::type_checker::TypeCheckResult;
use super::types::Config;
use crate::eval::expression::Value;
use std::collections::HashMap;

pub trait ProviderConfigValidator {
    fn validate_schema(&self, schema: &Config) -> TypeCheckResult<()>;
    fn validate_basic_types(&self, config: &HashMap<String, Value>) -> TypeCheckResult<()>;
}

pub struct CommonValidator;

impl ProviderConfigValidator for CommonValidator {
    fn validate_schema(&self, schema: &Config) -> TypeCheckResult<()> {
        // Validate required fields
        if schema.name.is_empty() {
            return Err(crate::type_checker::TypeCheckError::invalid_type_arguments(
                "Provider name cannot be empty".to_string(),
                Default::default(),
            ));
        }

        Ok(())
    }

    fn validate_basic_types(&self, config: &HashMap<String, Value>) -> TypeCheckResult<()> {
        // Validate required fields
        let provider_type = config.get("provider_type").ok_or_else(|| {
            crate::type_checker::TypeCheckError::invalid_type_arguments(
                "Missing required field 'provider_type'".to_string(),
                Default::default(),
            )
        })?;

        match provider_type {
            Value::String(_) => (),
            _ => return Err(crate::type_checker::TypeCheckError::invalid_type_arguments(
                "provider_type must be a string".to_string(),
                Default::default(),
            )),
        }

        let name = config.get("name").ok_or_else(|| {
            crate::type_checker::TypeCheckError::invalid_type_arguments(
                "Missing required field 'name'".to_string(),
                Default::default(),
            )
        })?;

        match name {
            Value::String(_) => (),
            _ => return Err(crate::type_checker::TypeCheckError::invalid_type_arguments(
                "name must be a string".to_string(),
                Default::default(),
            )),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_schema_success() {
        let validator = CommonValidator;
        let config = Config {
            provider_type: Default::default(),
            name: "test".to_string(),
            common_config: Default::default(),
            provider_specific: Default::default(),
        };

        assert!(validator.validate_schema(&config).is_ok());
    }

    #[test]
    fn test_validate_schema_empty_name() {
        let validator = CommonValidator;
        let config = Config {
            provider_type: Default::default(),
            name: "".to_string(),
            common_config: Default::default(),
            provider_specific: Default::default(),
        };

        assert!(validator.validate_schema(&config).is_err());
    }

    #[test]
    fn test_validate_basic_types_success() {
        let validator = CommonValidator;
        let mut config = HashMap::new();
        config.insert(
            "provider_type".to_string(),
            Value::String("test".to_string()),
        );
        config.insert("name".to_string(), Value::String("test".to_string()));

        assert!(validator.validate_basic_types(&config).is_ok());
    }

    #[test]
    fn test_validate_basic_types_missing_provider_type() {
        let validator = CommonValidator;
        let mut config = HashMap::new();
        config.insert("name".to_string(), Value::String("test".to_string()));

        assert!(validator.validate_basic_types(&config).is_err());
    }

    #[test]
    fn test_validate_basic_types_missing_name() {
        let validator = CommonValidator;
        let mut config = HashMap::new();
        config.insert(
            "provider_type".to_string(),
            Value::String("test".to_string()),
        );

        assert!(validator.validate_basic_types(&config).is_err());
    }

    #[test]
    fn test_validate_basic_types_invalid_provider_type() {
        let validator = CommonValidator;
        let mut config = HashMap::new();
        config.insert("provider_type".to_string(), Value::Number(42.into()));
        config.insert("name".to_string(), Value::String("test".to_string()));

        assert!(validator.validate_basic_types(&config).is_err());
    }

    #[test]
    fn test_validate_basic_types_invalid_name() {
        let validator = CommonValidator;
        let mut config = HashMap::new();
        config.insert(
            "provider_type".to_string(),
            Value::String("test".to_string()),
        );
        config.insert("name".to_string(), Value::Number(42.into()));

        assert!(validator.validate_basic_types(&config).is_err());
    }
}

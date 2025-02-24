use super::{validation::ProviderConfigValidator, types::Config};
use crate::type_checker::TypeCheckResult;
use crate::eval::expression::Value;
use std::collections::HashMap;

pub struct EvalProviderValidator;

impl ProviderConfigValidator for EvalProviderValidator {
    fn validate_schema(&self, schema: &Config) -> TypeCheckResult<()> {
        // Runtime schema validation
        if schema.name.is_empty() {
            return Err(crate::type_checker::TypeCheckError::invalid_type_arguments(
                "Provider name cannot be empty".to_string(),
                Default::default(),
            ));
        }

        // Validate provider-specific configuration
        if !schema.provider_specific.is_empty() {
            // Additional runtime validation for provider-specific config
            for (key, value) in &schema.provider_specific {
                match value {
                    Value::String(_) => (),
                    _ => return Err(crate::type_checker::TypeCheckError::invalid_type_arguments(
                        format!("Invalid type for provider-specific config key '{}', expected string", key),
                        Default::default(),
                    )),
                }
            }
        }

        Ok(())
    }

    fn validate_basic_types(&self, config: &HashMap<String, Value>) -> TypeCheckResult<()> {
        // Runtime type validation
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

        // Validate optional fields if present
        if let Some(common_config) = config.get("common_config") {
            match common_config {
                Value::Map(_) => (),
                _ => return Err(crate::type_checker::TypeCheckError::invalid_type_arguments(
                    "common_config must be an object".to_string(),
                    Default::default(),
                )),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_schema_success() {
        let validator = EvalProviderValidator;
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
        let validator = EvalProviderValidator;
        let config = Config {
            provider_type: Default::default(),
            name: "".to_string(),
            common_config: Default::default(),
            provider_specific: Default::default(),
        };

        assert!(validator.validate_schema(&config).is_err());
    }

    #[test]
    fn test_validate_schema_invalid_provider_specific() {
        let validator = EvalProviderValidator;
        let mut provider_specific = HashMap::new();
        provider_specific.insert("test".to_string(), Value::List(vec![]));

        let config = Config {
            provider_type: Default::default(),
            name: "test".to_string(),
            common_config: Default::default(),
            provider_specific,
        };

        assert!(validator.validate_schema(&config).is_err());
    }

    #[test]
    fn test_validate_basic_types_success() {
        let validator = EvalProviderValidator;
        let mut config = HashMap::new();
        config.insert(
            "provider_type".to_string(),
            Value::String("test".to_string()),
        );
        config.insert("name".to_string(), Value::String("test".to_string()));

        assert!(validator.validate_basic_types(&config).is_ok());
    }

    #[test]
    fn test_validate_basic_types_with_common_config() {
        let validator = EvalProviderValidator;
        let mut config = HashMap::new();
        config.insert(
            "provider_type".to_string(),
            Value::String("test".to_string()),
        );
        config.insert("name".to_string(), Value::String("test".to_string()));
        config.insert(
            "common_config".to_string(),
            Value::Map(HashMap::new()),
        );

        assert!(validator.validate_basic_types(&config).is_ok());
    }

    #[test]
    fn test_validate_basic_types_invalid_common_config() {
        let validator = EvalProviderValidator;
        let mut config = HashMap::new();
        config.insert(
            "provider_type".to_string(),
            Value::String("test".to_string()),
        );
        config.insert("name".to_string(), Value::String("test".to_string()));
        config.insert("common_config".to_string(), Value::String("invalid".to_string()));

        assert!(validator.validate_basic_types(&config).is_err());
    }
}

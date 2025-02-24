use std::collections::HashMap;
use crate::{
    eval::expression::Value,
    provider::config::{
        validation::CommonValidator,
        types::Config,
        validation::ProviderConfigValidator,
    },
};

#[test]
fn test_basic_validation() {
    let validator = CommonValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("test".to_string()));
    config.insert("name".to_string(), Value::String("test".to_string()));
    
    assert!(validator.validate_basic_types(&config).is_ok());
}

#[test]
fn test_missing_required_fields() {
    let validator = CommonValidator;
    let mut config = HashMap::new();
    config.insert("name".to_string(), Value::String("test".to_string()));
    
    assert!(validator.validate_basic_types(&config).is_err());

    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("test".to_string()));
    
    assert!(validator.validate_basic_types(&config).is_err());
}

#[test]
fn test_invalid_field_types() {
    let validator = CommonValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::Integer(42));
    config.insert("name".to_string(), Value::String("test".to_string()));
    
    assert!(validator.validate_basic_types(&config).is_err());

    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("test".to_string()));
    config.insert("name".to_string(), Value::Integer(42));
    
    assert!(validator.validate_basic_types(&config).is_err());
}

#[test]
fn test_schema_validation() {
    let validator = CommonValidator;
    let config = Config {
        provider_type: Default::default(),
        name: "test".to_string(),
        common_config: Default::default(),
        provider_specific: Default::default(),
    };
    
    assert!(validator.validate_schema(&config).is_ok());

    let config = Config {
        provider_type: Default::default(),
        name: "".to_string(),
        common_config: Default::default(),
        provider_specific: Default::default(),
    };
    
    assert!(validator.validate_schema(&config).is_err());
}

use std::collections::HashMap;
use crate::{
    eval::expression::Value,
    provider::config::{
        type_check::TypeProviderValidator,
        types::Config,
        validation::ProviderConfigValidator,
    },
};

#[test]
fn test_compile_time_validation() {
    let validator = TypeProviderValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("test".to_string()));
    config.insert("name".to_string(), Value::String("test".to_string()));
    
    assert!(validator.validate_basic_types(&config).is_ok());
}

#[test]
fn test_compile_time_schema_validation() {
    let validator = TypeProviderValidator;
    let config = Config {
        provider_type: Default::default(),
        name: "test".to_string(),
        common_config: Default::default(),
        provider_specific: Default::default(),
    };
    
    assert!(validator.validate_schema(&config).is_ok());
}

#[test]
fn test_compile_time_error_handling() {
    let validator = TypeProviderValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::Integer(42));
    config.insert("name".to_string(), Value::String("test".to_string()));
    
    let result = validator.validate_basic_types(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("provider_type must be a string"));
}

#[test]
fn test_plugin_specific_validation() {
    let validator = TypeProviderValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("memory".to_string()));
    config.insert("name".to_string(), Value::String("test_memory".to_string()));
    
    assert!(validator.validate_basic_types(&config).is_ok());
}

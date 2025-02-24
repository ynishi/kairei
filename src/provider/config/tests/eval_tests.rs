use std::collections::HashMap;
use crate::{
    eval::expression::Value,
    provider::config::{
        eval::EvalProviderValidator,
        types::Config,
        validation::ProviderConfigValidator,
    },
};

#[test]
fn test_runtime_validation() {
    let validator = EvalProviderValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("test".to_string()));
    config.insert("name".to_string(), Value::String("test".to_string()));
    
    assert!(validator.validate_basic_types(&config).is_ok());
}

#[test]
fn test_runtime_schema_validation() {
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
fn test_runtime_error_handling() {
    let validator = EvalProviderValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("test".to_string()));
    config.insert("name".to_string(), Value::String("test".to_string()));
    config.insert("common_config".to_string(), Value::String("invalid".to_string()));
    
    let result = validator.validate_basic_types(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("common_config must be an object"));
}

#[test]
fn test_plugin_specific_runtime_validation() {
    let validator = EvalProviderValidator;
    let mut provider_specific = HashMap::new();
    provider_specific.insert("test".to_string(), Value::String("valid".to_string()));

    let config = Config {
        provider_type: Default::default(),
        name: "test".to_string(),
        common_config: Default::default(),
        provider_specific,
    };
    
    assert!(validator.validate_schema(&config).is_ok());

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
fn test_memory_plugin_validation() {
    let validator = EvalProviderValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("memory".to_string()));
    config.insert("name".to_string(), Value::String("test_memory".to_string()));
    config.insert("common_config".to_string(), Value::Map(HashMap::new()));
    
    assert!(validator.validate_basic_types(&config).is_ok());
}

#[test]
fn test_rag_plugin_validation() {
    let validator = EvalProviderValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("rag".to_string()));
    config.insert("name".to_string(), Value::String("test_rag".to_string()));
    config.insert("common_config".to_string(), Value::Map(HashMap::new()));
    
    assert!(validator.validate_basic_types(&config).is_ok());
}

#[test]
fn test_search_plugin_validation() {
    let validator = EvalProviderValidator;
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Value::String("search".to_string()));
    config.insert("name".to_string(), Value::String("test_search".to_string()));
    config.insert("common_config".to_string(), Value::Map(HashMap::new()));
    
    assert!(validator.validate_basic_types(&config).is_ok());
}

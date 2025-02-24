use std::collections::HashMap;

use crate::{
    eval::expression::Value,
    type_checker::{CommonPluginValidator, PluginValidator},
};

#[test]
fn test_basic_validation_success() {
    let validator = CommonPluginValidator;
    let mut config = HashMap::new();
    config.insert(
        "provider_type".to_string(),
        Value::String("memory".to_string()),
    );
    config.insert("name".to_string(), Value::String("test_memory".to_string()));

    assert!(validator.validate_basic_structure(&config).is_ok());
}

#[test]
fn test_missing_provider_type() {
    let validator = CommonPluginValidator;
    let mut config = HashMap::new();
    config.insert("name".to_string(), Value::String("test_memory".to_string()));

    let result = validator.validate_basic_structure(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing required field 'provider_type'"));
}

#[test]
fn test_missing_name() {
    let validator = CommonPluginValidator;
    let mut config = HashMap::new();
    config.insert(
        "provider_type".to_string(),
        Value::String("memory".to_string()),
    );

    let result = validator.validate_basic_structure(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing required field 'name'"));
}

#[test]
fn test_plugin_specific_validation() {
    let validator = CommonPluginValidator;
    let mut config = HashMap::new();
    config.insert(
        "provider_type".to_string(),
        Value::String("memory".to_string()),
    );
    config.insert("name".to_string(), Value::String("test_memory".to_string()));

    // Plugin-specific validation should succeed by default
    assert!(validator.validate_plugin_specific(&config).is_ok());
}

#[test]
fn test_validation_with_different_plugin_types() {
    let validator = CommonPluginValidator;
    let plugin_types = vec!["memory", "rag", "search"];

    for plugin_type in plugin_types {
        let mut config = HashMap::new();
        config.insert(
            "provider_type".to_string(),
            Value::String(plugin_type.to_string()),
        );
        config.insert("name".to_string(), Value::String(format!("test_{}", plugin_type)));

        assert!(validator.validate_basic_structure(&config).is_ok());
    }
}

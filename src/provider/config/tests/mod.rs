use super::*;
use crate::provider::config::{
    base::PluginType,
    plugins::MemoryConfig,
    validation::{validate_range, validate_required_field},
};
use crate::provider::provider::ProviderType;
use std::collections::HashMap;

// Include validator tests
mod validator_tests;
mod plugin_config_test;
mod errors_tests;

#[test]
fn test_plugin_config_validation() {
    // Test strict mode with known types
    let config = PluginConfig {
        provider_type: ProviderType::SimpleExpert,
        plugin_type: PluginType::Memory(MemoryConfig::default()),
        strict: true,
    };
    assert!(config.validate().is_ok());

    // Test strict mode with Unknown provider type
    let invalid_provider_config = PluginConfig {
        provider_type: ProviderType::Unknown,
        plugin_type: PluginType::Memory(MemoryConfig::default()),
        strict: true,
    };
    assert!(invalid_provider_config.validate().is_err());

    // Test strict mode with Unknown plugin type
    let invalid_plugin_config = PluginConfig {
        provider_type: ProviderType::SimpleExpert,
        plugin_type: PluginType::Unknown(HashMap::new()),
        strict: true,
    };
    assert!(invalid_plugin_config.validate().is_err());

    // Test non-strict mode with Unknown types
    let non_strict_config = PluginConfig {
        provider_type: ProviderType::Unknown,
        plugin_type: PluginType::Unknown(HashMap::new()),
        strict: false,
    };
    assert!(non_strict_config.validate().is_ok());
}

#[test]
fn test_validation_utilities() {
    let field: Option<i32> = None;
    assert!(validate_required_field(&field, "test").is_err());

    assert!(validate_range(5, 0, 10, "test").is_ok());
    assert!(validate_range(15, 0, 10, "test").is_err());
}

#[test]
fn test_check_required_properties() {
    use serde_json::json;

    let config = json!({
        "name": "test",
        "value": 42
    });

    // Test valid case
    assert!(check_required_properties(&config, &["name", "value"]).is_ok());

    // Test missing property
    assert!(check_required_properties(&config, &["name", "missing"]).is_err());
}

#[test]
fn test_check_property_type() {
    use serde_json::json;

    let config = json!({
        "string_field": "test",
        "number_field": 42,
        "boolean_field": true,
        "object_field": {"key": "value"},
        "array_field": [1, 2, 3]
    });

    // Test valid cases
    assert!(check_property_type(&config, "string_field", "string").is_ok());
    assert!(check_property_type(&config, "number_field", "number").is_ok());
    assert!(check_property_type(&config, "boolean_field", "boolean").is_ok());
    assert!(check_property_type(&config, "object_field", "object").is_ok());
    assert!(check_property_type(&config, "array_field", "array").is_ok());

    // Test invalid cases
    assert!(check_property_type(&config, "string_field", "number").is_err());
    assert!(check_property_type(&config, "number_field", "string").is_err());
    assert!(check_property_type(&config, "missing_field", "string").is_err());
}

#[test]
fn test_plugin_type_serialization() {
    let config = PluginConfig {
        provider_type: ProviderType::SimpleExpert,
        plugin_type: PluginType::Memory(MemoryConfig::default()),
        strict: true,
    };
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: PluginConfig = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized.plugin_type, PluginType::Memory(_)));
    assert_eq!(deserialized.provider_type, ProviderType::SimpleExpert);
}

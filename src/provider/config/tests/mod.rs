use super::*;
use crate::provider::config::base::PluginType;
use crate::provider::config::validation::{validate_range, validate_required_field};

#[test]
fn test_plugin_config_validation() {
    // Test strict mode with known type
    let config = PluginConfig {
        plugin_type: PluginType::Memory,
        strict: true,
    };
    assert!(config.validate().is_ok());

    // Test strict mode with Unknown type
    let invalid_config = PluginConfig {
        plugin_type: PluginType::Unknown,
        strict: true,
    };
    assert!(invalid_config.validate().is_err());

    // Test non-strict mode with Unknown type
    let non_strict_config = PluginConfig {
        plugin_type: PluginType::Unknown,
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
fn test_plugin_type_serialization() {
    let config = PluginConfig {
        plugin_type: PluginType::Memory,
        strict: true,
    };
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: PluginConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.plugin_type, PluginType::Memory);
}

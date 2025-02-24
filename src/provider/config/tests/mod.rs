use super::*;
use crate::config::{MemoryConfig, RagConfig, SearchConfig};
use crate::provider::config::base::PluginType;
use crate::provider::config::validation::{validate_range, validate_required_field};
use crate::provider::provider::ProviderType;
use std::collections::HashMap;

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

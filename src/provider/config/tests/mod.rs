use super::*;
use crate::provider::config::validation::{validate_range, validate_required_field};
use crate::provider::provider::ProviderType;

#[test]
fn test_plugin_config_validation() {
    let config = PluginConfig {
        provider_type: ProviderType::SimpleExpert,
        strict: true,
    };
    assert!(config.validate().is_ok());

    let invalid_config = PluginConfig {
        provider_type: ProviderType::Unknown,
        strict: true,
    };
    assert!(invalid_config.validate().is_err());

    let non_strict_config = PluginConfig {
        provider_type: ProviderType::Unknown,
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

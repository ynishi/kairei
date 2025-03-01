//! Utility functions for provider configuration.

use crate::config::ProviderConfig;
use serde_json::Value;
use std::collections::HashMap;

/// Converts a ProviderConfig to a HashMap<String, Value>.
///
/// This function is used to convert a ProviderConfig to a format
/// that can be used with the ProviderConfigValidator trait.
///
/// # Parameters
///
/// * `config` - The provider configuration to convert
///
/// # Returns
///
/// A HashMap<String, Value> representation of the configuration
pub fn config_to_map(config: &ProviderConfig) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    // Add common config
    map.insert(
        "temperature".to_string(),
        Value::from(config.common_config.temperature),
    );
    map.insert(
        "max_tokens".to_string(),
        Value::from(config.common_config.max_tokens),
    );
    map.insert(
        "model".to_string(),
        Value::from(config.common_config.model.clone()),
    );

    // Add provider-specific config
    for (key, value) in &config.provider_specific {
        map.insert(key.clone(), value.clone());
    }

    // Add endpoint config
    map.insert(
        "endpoint".to_string(),
        serde_json::to_value(&config.endpoint).unwrap_or_default(),
    );

    // Add plugin configs
    let plugin_configs = serde_json::to_value(&config.plugin_configs).unwrap_or_default();
    map.insert("plugin_configs".to_string(), plugin_configs);

    map
}

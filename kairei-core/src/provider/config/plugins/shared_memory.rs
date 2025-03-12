//! Shared Memory plugin configuration.

use super::{BasePluginConfig, ProviderSpecificConfig};
use crate::provider::config::base::ConfigError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Shared Memory plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedMemoryConfig {
    #[serde(default)]
    pub base: BasePluginConfig,
    #[serde(default = "default_max_keys")]
    pub max_keys: usize,
    #[serde(default = "default_ttl", with = "crate::config::duration_ms")]
    pub ttl: Duration,
}

impl Default for SharedMemoryConfig {
    fn default() -> Self {
        Self {
            base: BasePluginConfig::default(),
            max_keys: default_max_keys(),
            ttl: default_ttl(),
        }
    }
}

impl ProviderSpecificConfig for SharedMemoryConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate max keys
        if self.max_keys == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_keys".to_string(),
                message: "Max keys must be greater than 0".to_string(),
            });
        }

        // Validate TTL
        if self.ttl.as_secs() == 0 {
            return Err(ConfigError::InvalidValue {
                field: "ttl".to_string(),
                message: "TTL must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    fn merge_defaults(&mut self) {
        if self.max_keys == 0 {
            self.max_keys = default_max_keys();
        }
        if self.ttl.as_secs() == 0 {
            self.ttl = default_ttl();
        }
    }
}

fn default_max_keys() -> usize {
    1000
}

fn default_ttl() -> Duration {
    Duration::from_secs(3600) // 1 hour
}

//! Memory plugin configuration.

use super::{BasePluginConfig, ProviderSpecificConfig};
use crate::provider::config::base::ConfigError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Memory plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryConfig {
    #[serde(default)]
    pub base: BasePluginConfig,
    #[serde(default = "default_max_items")]
    pub max_items: usize,
    #[serde(default = "default_ttl", with = "crate::config::duration_ms")]
    pub ttl: Duration,
    #[serde(default = "default_importance_threshold")]
    pub importance_threshold: f32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            base: BasePluginConfig::default(),
            max_items: default_max_items(),
            ttl: default_ttl(),
            importance_threshold: default_importance_threshold(),
        }
    }
}

impl ProviderSpecificConfig for MemoryConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate max items
        if self.max_items == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_items".to_string(),
                message: "Max items must be greater than 0".to_string(),
            });
        }

        // Validate TTL
        if self.ttl.as_secs() == 0 {
            return Err(ConfigError::InvalidValue {
                field: "ttl".to_string(),
                message: "TTL must be greater than 0".to_string(),
            });
        }

        // Validate importance threshold
        if !(0.0..=1.0).contains(&self.importance_threshold) {
            return Err(ConfigError::InvalidValue {
                field: "importance_threshold".to_string(),
                message: "Importance threshold must be between 0.0 and 1.0".to_string(),
            });
        }

        Ok(())
    }

    fn merge_defaults(&mut self) {
        if self.max_items == 0 {
            self.max_items = default_max_items();
        }
        if self.ttl.as_secs() == 0 {
            self.ttl = default_ttl();
        }
        if self.importance_threshold == 0.0 {
            self.importance_threshold = default_importance_threshold();
        }
    }
}

fn default_max_items() -> usize {
    1000
}

fn default_ttl() -> Duration {
    Duration::from_secs(3600) // 1 hour
}

fn default_importance_threshold() -> f32 {
    0.5
}

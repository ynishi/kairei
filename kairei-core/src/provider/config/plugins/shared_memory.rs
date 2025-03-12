//! Shared Memory plugin configuration.

use super::{BasePluginConfig, ProviderSpecificConfig};
use crate::provider::config::base::ConfigError;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;

/// Shared Memory plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct SharedMemoryConfig {
    /// Base plugin configuration
    #[serde(default)]
    pub base: BasePluginConfig,

    /// Maximum number of keys allowed in the shared memory store
    /// Setting to 0 means unlimited (only limited by available memory)
    #[serde(default = "default_max_keys")]
    pub max_keys: usize,

    /// Time-to-live for entries, after which they are automatically removed
    /// Setting to 0 means entries don't expire
    #[serde(default = "default_ttl", with = "crate::config::duration_ms")]
    #[schema(value_type = u64, pattern = "uint64 as milliseconds")]
    pub ttl: Duration,

    /// Default namespace prefix for keys
    /// Used to isolate keys between different components
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

impl Default for SharedMemoryConfig {
    fn default() -> Self {
        Self {
            base: BasePluginConfig::default(),
            max_keys: default_max_keys(),
            ttl: default_ttl(),
            namespace: default_namespace(),
        }
    }
}

/// Validates a namespace string
fn is_valid_namespace(namespace: &str) -> bool {
    namespace
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

impl ProviderSpecificConfig for SharedMemoryConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate maximum keys (if specified)
        if self.max_keys > 0 && self.max_keys < 10 {
            return Err(ConfigError::InvalidValue {
                field: "max_keys".to_string(),
                message: "If specified, max_keys must be at least 10".to_string(),
            });
        }

        // Validate TTL (if not unlimited)
        if self.ttl.as_millis() > 0 && self.ttl.as_millis() < 1000 {
            return Err(ConfigError::InvalidValue {
                field: "ttl".to_string(),
                message: "If specified, TTL must be at least 1000ms (1 second)".to_string(),
            });
        }

        // Validate namespace format
        if !self.namespace.is_empty() && !is_valid_namespace(&self.namespace) {
            return Err(ConfigError::InvalidValue {
                field: "namespace".to_string(),
                message:
                    "Namespace must contain only alphanumeric characters, underscores, and dashes"
                        .to_string(),
            });
        }

        Ok(())
    }

    fn merge_defaults(&mut self) {
        if self.max_keys == 0 {
            self.max_keys = default_max_keys();
        }
        if self.ttl.as_millis() == 0 {
            self.ttl = default_ttl();
        }
        if self.namespace.is_empty() {
            self.namespace = default_namespace();
        }
    }
}

/// Default maximum number of keys
fn default_max_keys() -> usize {
    10000 // 10k keys by default
}

/// Default time-to-live duration
fn default_ttl() -> Duration {
    Duration::from_secs(3600) // 1 hour
}

/// Default namespace
fn default_namespace() -> String {
    "shared".to_string()
}

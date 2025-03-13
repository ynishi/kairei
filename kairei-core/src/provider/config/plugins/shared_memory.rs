//! Shared Memory plugin configuration.
//!
//! This module defines the configuration options for the SharedMemoryCapability
//! plugin, allowing customization of behavior such as capacity limits, TTL,
//! and namespace isolation.
//!
//! # Example
//!
//! ```no_run
//! use kairei_core::provider::config::plugins::SharedMemoryConfig;
//! use kairei_core::provider::config::BasePluginConfig;
//! use std::time::Duration;
//!
//! let config = SharedMemoryConfig {
//!     base: BasePluginConfig::default(),
//!     max_keys: 5000,                      // Limit to 5000 keys
//!     ttl: Duration::from_secs(7200),      // 2 hour expiration
//!     namespace: "my_application".to_string(),
//! };
//! ```

use super::{BasePluginConfig, ProviderSpecificConfig};
use crate::provider::config::base::ConfigError;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;

/// Shared Memory plugin configuration
///
/// This structure defines the configuration options for the SharedMemoryCapability
/// plugin, allowing customization of behavior such as capacity limits, TTL,
/// and namespace isolation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct SharedMemoryConfig {
    /// Base plugin configuration
    #[serde(default)]
    pub base: BasePluginConfig,

    /// Maximum number of keys allowed in the shared memory store
    ///
    /// Setting to 0 means unlimited (only limited by available memory).
    /// The default is 10,000 keys.
    ///
    /// # Performance Considerations
    ///
    /// - Setting this too high may lead to excessive memory usage
    /// - Setting this too low may cause operations to fail when capacity is reached
    #[serde(default = "default_max_keys")]
    pub max_keys: usize,

    /// Time-to-live for entries, after which they are automatically removed
    ///
    /// Setting to 0 means entries don't expire. The default is 1 hour.
    ///
    /// # Usage Patterns
    ///
    /// - Short TTL (seconds to minutes): For temporary cache data
    /// - Medium TTL (hours): For session data
    /// - Long TTL (days) or 0: For persistent configuration
    #[serde(default = "default_ttl", with = "crate::config::duration_ms")]
    #[schema(value_type = u64, pattern = "uint64 as milliseconds")]
    pub ttl: Duration,

    /// Default namespace prefix for keys
    ///
    /// Used to isolate keys between different components. The default is "shared".
    ///
    /// # Namespace Isolation
    ///
    /// Different namespaces provide complete isolation - keys in one namespace
    /// are not visible to plugins using a different namespace.
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
///
/// A valid namespace contains only alphanumeric characters, underscores, and dashes.
fn is_valid_namespace(namespace: &str) -> bool {
    namespace
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

impl ProviderSpecificConfig for SharedMemoryConfig {
    /// Validates the configuration values
    ///
    /// # Validation Rules
    ///
    /// - If specified, max_keys must be at least 10
    /// - If specified, TTL must be at least 1000ms (1 second)
    /// - Namespace must contain only alphanumeric characters, underscores, and dashes
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

    /// Merges default values for unspecified fields
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
///
/// The default is 10,000 keys, which provides a reasonable balance between
/// memory usage and capacity for most applications.
fn default_max_keys() -> usize {
    10000 // 10k keys by default
}

/// Default time-to-live duration
///
/// The default is 1 hour (3600 seconds), which is suitable for most
/// session-like data.
fn default_ttl() -> Duration {
    Duration::from_secs(3600) // 1 hour
}

/// Default namespace
///
/// The default namespace is "shared", which is used when no specific
/// namespace is provided.
fn default_namespace() -> String {
    "shared".to_string()
}

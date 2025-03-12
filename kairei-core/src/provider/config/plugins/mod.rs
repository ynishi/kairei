//! Plugin-specific configuration types and validation.

mod memory;
mod rag;
mod search;
mod shared_memory;

pub use memory::*;
pub use rag::*;
pub use search::*;
pub use shared_memory::*;

use crate::provider::config::base::ConfigError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Base configuration shared by all plugins
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BasePluginConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_strict_mode")]
    pub strict_mode: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    #[serde(default = "default_timeout", with = "crate::config::duration_ms")]
    pub timeout: Duration,
}

impl Default for BasePluginConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            strict_mode: default_strict_mode(),
            max_retries: default_max_retries(),
            timeout: default_timeout(),
        }
    }
}

/// Provider-specific configuration trait
pub trait ProviderSpecificConfig: Send + Sync + Clone {
    fn validate(&self) -> Result<(), ConfigError>;
    fn merge_defaults(&mut self);
}

fn default_enabled() -> bool {
    true
}

fn default_strict_mode() -> bool {
    false
}

fn default_max_retries() -> usize {
    3
}

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

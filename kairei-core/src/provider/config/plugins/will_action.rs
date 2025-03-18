//! Configuration for Will Action plugins

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::provider::config::plugins::BasePluginConfig;

/// Configuration for Will Action plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillActionConfig {
    /// Base configuration
    #[serde(default)]
    pub base: BasePluginConfig,

    /// Whether to use built-in actions
    #[serde(default = "default_use_built_in_actions")]
    pub use_built_in_actions: bool,

    /// Maximum number of actions that can be registered
    #[serde(default = "default_max_actions")]
    pub max_actions: usize,

    /// Timeout for action execution
    #[serde(default = "default_action_timeout")]
    pub action_timeout: Duration,
}

fn default_use_built_in_actions() -> bool {
    true
}

fn default_max_actions() -> usize {
    100
}

fn default_action_timeout() -> Duration {
    Duration::from_secs(30)
}

impl Default for WillActionConfig {
    fn default() -> Self {
        Self {
            base: Default::default(),
            use_built_in_actions: default_use_built_in_actions(),
            max_actions: default_max_actions(),
            action_timeout: default_action_timeout(),
        }
    }
}

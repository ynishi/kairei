//! Search plugin configuration.

use super::{BasePluginConfig, ProviderSpecificConfig};
use crate::provider::config::base::ConfigError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Search plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchConfig {
    #[serde(default)]
    pub base: BasePluginConfig,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_search_window", with = "crate::config::duration_ms")]
    pub search_window: Duration,
    #[serde(default = "default_filters")]
    pub filters: Vec<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            base: BasePluginConfig::default(),
            max_results: default_max_results(),
            search_window: default_search_window(),
            filters: default_filters(),
        }
    }
}

impl ProviderSpecificConfig for SearchConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate max results
        if self.max_results == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_results".to_string(),
                message: "Max results must be greater than 0".to_string(),
            });
        }

        // Validate search window
        if self.search_window.as_secs() == 0 {
            return Err(ConfigError::InvalidValue {
                field: "search_window".to_string(),
                message: "Search window must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    fn merge_defaults(&mut self) {
        if self.max_results == 0 {
            self.max_results = default_max_results();
        }
        if self.search_window.as_secs() == 0 {
            self.search_window = default_search_window();
        }
        if self.filters.is_empty() {
            self.filters = default_filters();
        }
    }
}

fn default_max_results() -> usize {
    10
}

fn default_search_window() -> Duration {
    Duration::from_secs(3600) // 1 hour
}

fn default_filters() -> Vec<String> {
    vec![]
}

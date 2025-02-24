//! Provider-specific configuration types.

mod openai;

pub use openai::*;

use super::plugins::ProviderSpecificConfig;
use crate::provider::config::base::ConfigError;
use serde::{Deserialize, Serialize};

/// Generic provider-specific plugin configuration wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPluginConfig<T: ProviderSpecificConfig> {
    pub base: T,
    #[serde(flatten)]
    pub provider_specific: Option<serde_json::Value>,
}

impl<T: ProviderSpecificConfig> ProviderPluginConfig<T> {
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.base.validate()?;
        Ok(())
    }

    pub fn merge_defaults(&mut self) {
        self.base.merge_defaults();
    }
}

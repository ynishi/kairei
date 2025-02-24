//! OpenAI-specific provider configurations.

use super::ProviderPluginConfig;
use crate::provider::config::{
    base::ConfigError,
    plugins::{MemoryConfig, ProviderSpecificConfig, RagConfig, SearchConfig},
};
use serde::{Deserialize, Serialize};

/// OpenAI API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApiConfig {
    pub model: String,
    pub api_version: Option<String>,
    pub organization_id: Option<String>,
}

/// OpenAI-specific RAG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIRagConfig {
    #[serde(flatten)]
    pub base: RagConfig,
    pub api_config: OpenAIApiConfig,
}

impl ProviderSpecificConfig for OpenAIRagConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        self.base.validate()?;
        if self.api_config.model.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "model".to_string(),
                message: "Model name cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    fn merge_defaults(&mut self) {
        self.base.merge_defaults();
    }
}

/// OpenAI-specific Memory configuration
pub type OpenAIMemoryConfig = ProviderPluginConfig<MemoryConfig>;

/// OpenAI-specific Search configuration
pub type OpenAISearchConfig = ProviderPluginConfig<SearchConfig>;

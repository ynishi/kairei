//! RAG (Retrieval Augmented Generation) plugin configuration.

use super::{BasePluginConfig, ProviderSpecificConfig};
use crate::provider::config::base::ConfigError;
use serde::{Deserialize, Serialize};

/// Base RAG plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RagConfig {
    #[serde(default)]
    pub base: BasePluginConfig,
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            base: BasePluginConfig::default(),
            chunk_size: default_chunk_size(),
            max_tokens: default_max_tokens(),
            similarity_threshold: default_similarity_threshold(),
        }
    }
}

impl ProviderSpecificConfig for RagConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate chunk size
        if self.chunk_size == 0 {
            return Err(ConfigError::InvalidValue {
                field: "chunk_size".to_string(),
                message: "Chunk size must be greater than 0".to_string(),
            });
        }

        // Validate max tokens
        if self.max_tokens == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_tokens".to_string(),
                message: "Max tokens must be greater than 0".to_string(),
            });
        }

        // Validate similarity threshold
        if !(0.0..=1.0).contains(&self.similarity_threshold) {
            return Err(ConfigError::InvalidValue {
                field: "similarity_threshold".to_string(),
                message: "Similarity threshold must be between 0.0 and 1.0".to_string(),
            });
        }

        Ok(())
    }

    fn merge_defaults(&mut self) {
        if self.chunk_size == 0 {
            self.chunk_size = default_chunk_size();
        }
        if self.max_tokens == 0 {
            self.max_tokens = default_max_tokens();
        }
        if self.similarity_threshold == 0.0 {
            self.similarity_threshold = default_similarity_threshold();
        }
    }
}

fn default_chunk_size() -> usize {
    512
}

fn default_max_tokens() -> usize {
    1000
}

fn default_similarity_threshold() -> f32 {
    0.7
}

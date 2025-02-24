use serde::{Deserialize, Serialize};
use strum;
use thiserror::Error;

use crate::provider::config::plugins::ProviderSpecificConfig;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },

    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub trait ConfigValidation {
    fn validate(&self) -> Result<(), ConfigError>;

    fn validate_with_context(&self, context: &str) -> Result<(), ConfigError> {
        self.validate()
            .map_err(|e| ConfigError::ValidationError(format!("{} in {}", e, context)))
    }
}

use crate::provider::{
    config::plugins::{MemoryConfig, RagConfig, SearchConfig},
    provider::ProviderType,
};
use std::collections::HashMap;

/// Represents the type of a plugin in the system.
/// Used to validate plugin configurations and ensure type safety.
///
/// This enum defines the supported plugin types in the KAIREI system,
/// allowing for proper validation and configuration of different plugin
/// implementations. Each variant includes its associated configuration type.
#[derive(Debug, Clone, Serialize, Deserialize, strum::Display, strum::EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum PluginType {
    /// Memory plugin for storing and retrieving context
    Memory(MemoryConfig),
    /// RAG (Retrieval Augmented Generation) plugin
    Rag(RagConfig),
    /// Search plugin for external information retrieval
    Search(SearchConfig),
    /// Unknown plugin type (used for backward compatibility)
    Unknown(HashMap<String, serde_json::Value>),
}

impl Default for PluginType {
    fn default() -> Self {
        Self::Unknown(HashMap::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(flatten)]
    pub provider_type: ProviderType,
    #[serde(flatten)]
    pub plugin_type: PluginType,
    #[serde(default)]
    pub strict: bool,
}

impl ConfigValidation for PluginConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.strict {
            // Validate provider type
            if self.provider_type == ProviderType::Unknown {
                return Err(ConfigError::ValidationError(
                    "Provider type must be specified when strict mode is enabled".to_string(),
                ));
            }
            // Validate plugin type
            if matches!(self.plugin_type, PluginType::Unknown(_)) {
                return Err(ConfigError::ValidationError(
                    "Plugin type must be specified when strict mode is enabled".to_string(),
                ));
            }
        }

        // Validate plugin-specific configuration
        match &self.plugin_type {
            PluginType::Memory(config) => config.validate(),
            PluginType::Rag(config) => config.validate(),
            PluginType::Search(config) => config.validate(),
            PluginType::Unknown(_) => Ok(()),
        }
    }
}

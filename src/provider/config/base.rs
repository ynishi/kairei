use serde::{Deserialize, Serialize};
use strum;
use thiserror::Error;

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

/// Represents the type of a plugin in the system.
/// Used to validate plugin configurations and ensure type safety.
///
/// This enum defines the supported plugin types in the KAIREI system,
/// allowing for proper validation and configuration of different plugin
/// implementations.
#[derive(Debug, Clone, Serialize, Deserialize, strum::Display, strum::EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum PluginType {
    /// Memory plugin for storing and retrieving context
    Memory,
    /// RAG (Retrieval Augmented Generation) plugin
    Rag,
    /// Search plugin for external information retrieval
    Search,
    /// Unknown plugin type (used for backward compatibility)
    Unknown,
}

impl Default for PluginType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(flatten)]
    pub plugin_type: PluginType,
    #[serde(default)]
    pub strict: bool,
}

impl ConfigValidation for PluginConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.strict && self.plugin_type == PluginType::Unknown {
            return Err(ConfigError::ValidationError(
                "Plugin type must be specified when strict mode is enabled".to_string(),
            ));
        }
        Ok(())
    }
}

use crate::provider::provider::ProviderType;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(flatten)]
    pub provider_type: ProviderType,
    #[serde(default)]
    pub strict: bool,
}

impl ConfigValidation for PluginConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.strict && self.provider_type == ProviderType::Unknown {
            return Err(ConfigError::ValidationError(
                "Plugin type must be specified".to_string(),
            ));
        }
        Ok(())
    }
}

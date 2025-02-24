use super::error::ValidationError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub field_types: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSpecificConfig {
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub features: Vec<String>,
    pub limits: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDependency {
    pub name: String,
    pub version: String,
    pub required: bool,
}

pub trait ProviderConfigValidator {
    fn validate_schema(&self, schema: &Schema) -> Result<(), ValidationError>;
    fn validate_provider_specific(&self, config: &ProviderSpecificConfig) -> Result<(), ValidationError>;
    fn validate_capabilities(&self, capabilities: &ProviderCapabilities) -> Result<(), ValidationError>;
    fn validate_dependencies(&self, dependencies: &[ProviderDependency]) -> Result<(), ValidationError>;
}

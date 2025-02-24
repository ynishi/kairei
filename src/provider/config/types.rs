use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::provider::provider::ProviderType;
use crate::eval::expression::Value;
use crate::type_checker::TypeCheckResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub provider_type: ProviderType,
    pub name: String,
    pub common_config: CommonConfig,
    pub provider_specific: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommonConfig {
    pub endpoint: Option<String>,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

impl TryFrom<HashMap<String, Value>> for Config {
    type Error = crate::type_checker::TypeCheckError;

    fn try_from(value: HashMap<String, Value>) -> TypeCheckResult<Self> {
        let provider_type = value.get("provider_type")
            .and_then(|v| match v {
                Value::String(s) => Some(s.parse().unwrap_or_default()),
                _ => None
            })
            .unwrap_or_default();

        let name = value.get("name")
            .and_then(|v| match v {
                Value::String(s) => Some(s.to_string()),
                _ => None
            })
            .unwrap_or_default();

        Ok(Self {
            provider_type,
            name,
            common_config: CommonConfig::default(),
            provider_specific: HashMap::new(),
        })
    }
}

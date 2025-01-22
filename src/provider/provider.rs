use core::fmt;
use std::collections::HashMap;

use crate::{
    config::{ProviderConfig, ProviderSecretConfig},
    timestamp::Timestamp,
};

use super::{
    capability::Capabilities,
    request::{ProviderContext, ProviderRequest, ProviderResponse},
    types::ProviderResult,
};
use async_trait::async_trait;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Provider: Send + Sync {
    async fn execute(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse>;
    async fn capabilities(&self) -> Capabilities;

    fn name(&self) -> &str;

    // validate the provider configuration
    async fn initialize(
        &mut self,
        config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<()>;

    async fn shutdown(&self) -> ProviderResult<()>;

    async fn health_check(&self) -> ProviderResult<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Section {
    pub content: String,
    pub priority: i32,
    pub metadata: SectionMetadata,
}

impl Section {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            ..Default::default()
        }
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SectionMetadata {
    pub source: String,
    pub timestamp: Timestamp,
}

impl SectionMetadata {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            timestamp: Timestamp::now(),
        }
    }
}

impl fmt::Display for SectionMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source: {}, timestamp: {}", self.source, self.timestamp)
    }
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    Default,
    PartialEq,
    PartialOrd,
)]
pub enum ProviderType {
    #[default]
    OpenAIAssistant,
    SimpleExpert,
    OpenAIChat,
    Unknown,
}

impl From<ProviderType> for String {
    fn from(provider_type: ProviderType) -> Self {
        provider_type.to_string()
    }
}

#[derive(Clone, Default)]
pub struct ProviderSecret {
    pub api_key: SecretString,
    pub additional_auth: HashMap<String, SecretString>,
}

impl From<ProviderSecretConfig> for ProviderSecret {
    fn from(secret: ProviderSecretConfig) -> Self {
        let additional_auth = secret
            .additional_auth
            .into_iter()
            .map(|(k, v)| (k, SecretString::from(v)))
            .collect();
        Self {
            api_key: SecretString::from(secret.api_key),
            additional_auth,
        }
    }
}

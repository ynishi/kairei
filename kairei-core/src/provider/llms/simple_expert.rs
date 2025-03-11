use async_trait::async_trait;
use dashmap::DashMap;
use tracing::debug;

use crate::{
    config::ProviderConfig,
    provider::{
        capability::{Capabilities, CapabilityType},
        llm::{LLMResponse, ProviderLLM, ResponseMetadata},
        provider::ProviderSecret,
        types::{ProviderError, ProviderResult},
    },
    timestamp::Timestamp,
};

type Pattern = String;

type Answer = String;

pub struct KnowledgeBase {
    values: DashMap<Pattern, Answer>,
}

pub struct SimpleExpertProviderLLM {
    name: String,
}

impl KnowledgeBase {
    pub fn new(values: DashMap<Pattern, Answer>) -> Self {
        Self { values }
    }
}

impl SimpleExpertProviderLLM {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn get_answer(&self, prompt: &str, knowledge_base: KnowledgeBase) -> Vec<String> {
        knowledge_base
            .values
            .iter()
            .filter(|entry| prompt.contains(entry.key()))
            .map(|entry| entry.value().clone())
            .collect()
    }
}

impl From<&ProviderConfig> for KnowledgeBase {
    fn from(config: &ProviderConfig) -> Self {
        let values = DashMap::new();
        for (key, value) in config.provider_specific.clone() {
            let value = if let Some(value) = value.as_str() {
                value.to_string()
            } else {
                value.to_string()
            };
            values.insert(key, value);
        }
        KnowledgeBase::new(values)
    }
}

#[async_trait]
impl ProviderLLM for SimpleExpertProviderLLM {
    #[tracing::instrument(skip(self, prompt, config), level = "debug")]
    async fn send_message(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> ProviderResult<LLMResponse> {
        let knowledge_base = KnowledgeBase::from(config);
        let responses: Vec<String> = self.get_answer(prompt, knowledge_base);
        if responses.is_empty() {
            return Err(ProviderError::ApiError("No response found".to_string()));
        }
        debug!("response: {:?}", responses);
        Ok(LLMResponse {
            content: responses[0].clone(),
            metadata: ResponseMetadata {
                model: self.name.clone(),
                created_at: Timestamp::now(),
                token_usage: None,
                finish_reason: None,
            },
        })
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::from(vec![CapabilityType::Generate])
    }

    fn name(&self) -> &str {
        &self.name
    }

    #[tracing::instrument(skip(self, _config, _secret), level = "debug")]
    async fn initialize(
        &mut self,
        _config: &ProviderConfig,
        _secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        Ok(())
    }
}

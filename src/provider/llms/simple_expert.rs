use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use tracing::debug;

use crate::{
    config::ProviderConfig,
    provider::{
        capability::Capabilities,
        llm::{LLMResponse, ProviderLLM, ResponseMetadata},
        types::{ProviderError, ProviderResult},
    },
    timestamp::Timestamp,
};

type Pattern = String;

type Answer = String;

pub type KnowledgeBase = DashMap<Pattern, Answer>;

pub struct SimpleExpertProviderLLM {
    name: String,
    knowledge_base: Arc<KnowledgeBase>,
}

impl SimpleExpertProviderLLM {
    pub fn new(name: String, knowledge_base: Arc<KnowledgeBase>) -> Self {
        Self {
            name,
            knowledge_base,
        }
    }
}

#[async_trait]
impl ProviderLLM for SimpleExpertProviderLLM {
    async fn send_message(
        &self,
        prompt: String,
        _config: &ProviderConfig,
    ) -> ProviderResult<LLMResponse> {
        // find content include pattern
        let responses: Vec<String> = self
            .knowledge_base
            .iter()
            .filter(|entry| prompt.contains(entry.key()))
            .map(|entry| entry.value().clone())
            .collect::<Vec<String>>();
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
        Capabilities::default()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

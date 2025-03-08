use crate::{
    provider::{capability::HasCapabilities, llm::ProviderLLM, provider::ProviderSecret},
    timestamp::Timestamp,
};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
    },
};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use std::collections::HashSet;
use tracing::debug;

use crate::{
    config::ProviderConfig,
    provider::{
        capability::{Capabilities, CapabilityType},
        llm::{LLMResponse, ResponseMetadata},
        types::{ProviderError, ProviderResult},
    },
};

pub struct OpenAIChatProviderLLM {
    client: Option<Client<OpenAIConfig>>,
    name: String,
    capabilities: Capabilities,
}

impl OpenAIChatProviderLLM {
    pub fn new(name: impl Into<String>) -> Self {
        let mut capabilities = HashSet::new();
        capabilities.insert(CapabilityType::Generate);
        capabilities.insert(CapabilityType::SystemPrompt);

        Self {
            client: None,
            name: name.into(),
            capabilities: Capabilities::new(capabilities),
        }
    }

    #[tracing::instrument(skip(self, config))]
    async fn chat_completion(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> ProviderResult<LLMResponse> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Authentication("Client not initialized".into()))?;

        debug!("prompt: {}", prompt);

        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(prompt.to_string()),
                name: None,
            },
        )];

        let request = CreateChatCompletionRequest {
            model: config.common_config.model.clone(),
            messages,
            temperature: Some(config.common_config.temperature),
            max_completion_tokens: Some(config.common_config.max_tokens as u32),
            ..Default::default()
        };

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| ProviderError::ApiError("No response content".into()))?;

        Ok(LLMResponse {
            content,
            metadata: ResponseMetadata {
                model: config.common_config.model.clone(),
                created_at: Timestamp::now(),
                token_usage: response
                    .usage
                    .map(|u| (u.prompt_tokens as usize, u.completion_tokens as usize)),
                finish_reason: response
                    .choices
                    .first()
                    .map(|c| format!("{:?}", c.finish_reason)),
            },
        })
    }
}

#[async_trait]
impl ProviderLLM for OpenAIChatProviderLLM {
    async fn send_message(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> ProviderResult<LLMResponse> {
        self.chat_completion(prompt, config).await
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn initialize(
        &mut self,
        _config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        let api_key = secret.api_key.clone();
        let mut openai_config = OpenAIConfig::new().with_api_key(api_key.expose_secret());

        if let Some(org_id) = secret.additional_auth.get("organization_id") {
            openai_config = openai_config.with_org_id(org_id.expose_secret());
        }

        self.client = Some(Client::with_config(openai_config));
        Ok(())
    }
}

impl HasCapabilities for OpenAIChatProviderLLM {
    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capabilities() {
        let provider = OpenAIChatProviderLLM::new("test");
        assert!(provider.supports(&CapabilityType::Generate));
        assert!(provider.supports(&CapabilityType::SystemPrompt));
        assert!(!provider.supports(&CapabilityType::Thread));
    }
}

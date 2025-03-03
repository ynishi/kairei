use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        CreateAssistantRequest, CreateMessageRequest, CreateMessageRequestContent,
        CreateRunRequest, CreateThreadRequest, MessageContent, MessageRole, RunStatus,
    },
};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use serde_json::json;
use std::{collections::HashSet, time::Duration};

use crate::{
    config::ProviderConfig,
    provider::{
        capability::{Capabilities, CapabilityType, HasCapabilities},
        llm::{LLMResponse, ProviderLLM, ResponseMetadata},
        provider::ProviderSecret,
        types::{ProviderError, ProviderResult},
    },
    timestamp::Timestamp,
};

pub struct OpenAIAssistantProviderLLM {
    client: Option<Client<OpenAIConfig>>,
    name: String,
    capabilities: Capabilities,
    assistant_id: Option<String>,
}

impl OpenAIAssistantProviderLLM {
    pub fn new(name: impl Into<String>) -> Self {
        let mut capabilities = HashSet::new();
        capabilities.insert(CapabilityType::Generate);
        capabilities.insert(CapabilityType::SystemPrompt);
        capabilities.insert(CapabilityType::Thread);

        Self {
            client: None,
            name: name.into(),
            capabilities: Capabilities::new(capabilities),
            assistant_id: None,
        }
    }

    async fn create_and_run_thread(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> ProviderResult<String> {
        if prompt.is_empty() {
            return Err(ProviderError::InvalidRequest("Prompt is empty".into()));
        }
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Authentication("Client not initialized".into()))?;

        // Create thread
        let thread = client
            .threads()
            .create(CreateThreadRequest::default())
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        // Add message
        let message_request = CreateMessageRequest {
            role: MessageRole::User,
            content: CreateMessageRequestContent::Content(prompt.to_string()),
            ..Default::default()
        };

        client
            .threads()
            .messages(&thread.id)
            .create(message_request)
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        // Create and run
        let assistant_id = self
            .assistant_id
            .as_ref()
            .ok_or_else(|| ProviderError::Configuration("Assistant not initialized".into()))?;

        let run = client
            .threads()
            .runs(&thread.id)
            .create(CreateRunRequest {
                assistant_id: assistant_id.clone(),
                instructions: config
                    .provider_specific
                    .get("instruction")
                    .and_then(|v| v.as_str())
                    .map_or(Some("You are An AI Assistant Agent".to_string()), |v| {
                        Some(v.to_string())
                    }),
                ..Default::default()
            })
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        // Wait for completion
        self.wait_for_run(&thread.id, &run.id).await
    }

    async fn wait_for_run(&self, thread_id: &str, run_id: &str) -> ProviderResult<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Authentication("Client not initialized".into()))?;

        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 60;

        while attempts < MAX_ATTEMPTS {
            let run = client
                .threads()
                .runs(thread_id)
                .retrieve(run_id)
                .await
                .map_err(|e| ProviderError::ApiError(e.to_string()))?;

            match run.status {
                RunStatus::Completed => {
                    let response = self.get_latest_message(thread_id).await?;
                    // Clean up thread
                    let _ = client.threads().delete(thread_id).await;
                    return Ok(response);
                }
                RunStatus::Failed | RunStatus::Cancelled | RunStatus::Expired => {
                    let _ = client.threads().delete(thread_id).await;
                    return Err(ProviderError::ApiError(format!(
                        "Run failed with status: {:?}",
                        run.status
                    )));
                }
                _ => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    attempts += 1;
                }
            }
        }

        let _ = client.threads().delete(thread_id).await;
        Err(ProviderError::ApiError("Run timed out".to_string()))
    }

    async fn get_latest_message(&self, thread_id: &str) -> ProviderResult<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Authentication("Client not initialized".into()))?;

        let messages = client
            .threads()
            .messages(thread_id)
            .list(&json!({"limit": 1, "order": "desc"}))
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        messages
            .data
            .first()
            .and_then(|msg| msg.content.first())
            .and_then(|content| match content {
                MessageContent::Text(text) => Some(text.text.value.clone()),
                _ => None,
            })
            .ok_or_else(|| ProviderError::ApiError("No response content".into()))
    }
}

#[async_trait]
impl ProviderLLM for OpenAIAssistantProviderLLM {
    async fn send_message(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> ProviderResult<LLMResponse> {
        let content = self.create_and_run_thread(prompt, config).await?;

        Ok(LLMResponse {
            content,
            metadata: ResponseMetadata {
                model: config.common_config.model.clone(),
                created_at: Timestamp::now(),
                token_usage: None, // Assistant APIは現状usage情報を提供していない
                finish_reason: Some("completed".to_string()),
            },
        })
    }

    async fn initialize(
        &mut self,
        config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        let api_key = secret.api_key.clone();
        let mut openai_config = OpenAIConfig::new().with_api_key(api_key.expose_secret());

        if let Some(org_id) = secret.additional_auth.get("organization_id") {
            openai_config = openai_config.with_org_id(org_id.expose_secret());
        }

        let client = Client::with_config(openai_config);

        // Create assistant
        let assistant = client
            .assistants()
            .create(CreateAssistantRequest {
                model: config.common_config.model.clone(),
                name: config
                    .provider_specific
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                instructions: config
                    .provider_specific
                    .get("instructions")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                ..Default::default()
            })
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        self.assistant_id = Some(assistant.id);
        self.client = Some(client);
        Ok(())
    }

    async fn stop(&self) -> ProviderResult<()> {
        if let Some(client) = &self.client {
            if let Some(assistant_id) = &self.assistant_id {
                let _ = client.assistants().delete(assistant_id).await;
            }
        }
        Ok(())
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl HasCapabilities for OpenAIAssistantProviderLLM {
    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capabilities() {
        let provider = OpenAIAssistantProviderLLM::new("test");
        assert!(provider.supports(&CapabilityType::Generate));
        assert!(provider.supports(&CapabilityType::SystemPrompt));
        assert!(provider.supports(&CapabilityType::Thread));
    }
}

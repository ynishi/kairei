use async_openai::{
    config::OpenAIConfig,
    types::{
        CreateAssistantRequest, CreateMessageRequest, CreateMessageRequestContent,
        CreateRunRequest, CreateThreadRequest, MessageContent, MessageRole, RunStatus,
    },
    Client,
};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use serde_json::json;
use std::{sync::Arc, time::Duration};

use crate::{config::ProviderConfig, provider::types::ProviderError};

use super::types::{LLMProvider, ProviderResult, ProviderSecret};

#[derive(Debug, Clone)]
pub struct OpenAIAssistantProvider {
    pub client: Option<Client<OpenAIConfig>>,
    pub config: Option<ProviderConfig>,
}

impl Default for OpenAIAssistantProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAIAssistantProvider {
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
        }
    }

    async fn wait_for_run(&self, thread_id: &str, run_id: &str) -> ProviderResult<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Configuration("Client not initialized".to_string()))?;

        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 60; // 5分間のタイムアウト

        while attempts < MAX_ATTEMPTS {
            let run = client
                .threads()
                .runs(thread_id)
                .retrieve(run_id)
                .await
                .map_err(|e| ProviderError::ApiError(e.to_string()))?;

            match run.status {
                RunStatus::Completed => {
                    return self.get_latest_message(thread_id).await;
                }
                RunStatus::Failed | RunStatus::Cancelled | RunStatus::Expired => {
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

        Err(ProviderError::ApiError("Run timed out".to_string()))
    }

    async fn get_latest_message(&self, thread_id: &str) -> ProviderResult<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Configuration("Client not initialized".to_string()))?;

        let messages = client
            .threads()
            .messages(thread_id)
            .list(&json!({
                "limit": 10,
                "sort": "desc",
            }))
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        // 最新のメッセージの内容を取得
        messages
            .data
            .first()
            .and_then(|msg| msg.content.first())
            .and_then(|content| match content {
                MessageContent::Text(text) => Some(text.text.value.clone()),
                _ => None,
            })
            .ok_or_else(|| ProviderError::ApiError("Failed to extract message content".to_string()))
    }
}

#[async_trait]
impl LLMProvider for OpenAIAssistantProvider {
    fn name(&self) -> &str {
        "openai-assistant"
    }

    async fn initialize(
        &self,
        config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<Arc<dyn LLMProvider>> {
        let api_key = secret.api_key.clone();
        let mut openai_config = OpenAIConfig::new().with_api_key(api_key.expose_secret());

        if let Some(org_id) = secret.additional_auth.get("organization_id") {
            openai_config = openai_config.with_org_id(org_id.expose_secret());
        }

        let mut provider = self.clone();
        provider.client = Some(Client::with_config(openai_config));
        provider.config = Some(config.clone());

        Ok(Arc::new(provider))
    }

    async fn shutdown(&self) -> ProviderResult<()> {
        Ok(())
    }

    async fn validate_config(&self, config: &ProviderConfig) -> ProviderResult<()> {
        // モデル名の検証
        let model = config.common_config.model.clone();
        if model.is_empty() {
            return Err(ProviderError::Configuration(
                "Model not specified".to_string(),
            ));
        }

        if !model.starts_with("gpt-4") && !model.starts_with("gpt-3.5") {
            return Err(ProviderError::Configuration(
                "Invalid model specified".to_string(),
            ));
        }

        Ok(())
    }

    async fn create_assistant(&self, config: &ProviderConfig) -> ProviderResult<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Configuration("Client not initialized".to_string()))?;

        let model = config.common_config.model.clone();

        let request = CreateAssistantRequest {
            model: model.to_string(),
            name: config
                .provider_specific
                .get("name")
                .and_then(|v| v.as_str())
                .map(String::from),
            description: config
                .provider_specific
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            instructions: config
                .provider_specific
                .get("instructions")
                .and_then(|v| v.as_str())
                .map(String::from),
            tools: None,
            metadata: None,
            ..Default::default()
        };

        let assistant = client
            .assistants()
            .create(request)
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(assistant.id)
    }

    async fn create_thread(&self) -> ProviderResult<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Configuration("Client not initialized".to_string()))?;

        let request = CreateThreadRequest::default();

        let thread = client
            .threads()
            .create(request)
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(thread.id)
    }

    async fn send_message(
        &self,
        thread_id: &str,
        assistant_id: &str,
        content: &str,
    ) -> ProviderResult<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Configuration("Client not initialized".to_string()))?;

        // メッセージの作成
        let message_request = CreateMessageRequest {
            role: MessageRole::User,
            content: CreateMessageRequestContent::Content(content.to_string()),
            ..Default::default()
        };

        client
            .threads()
            .messages(thread_id)
            .create(message_request)
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        // Runの作成と実行
        let run_request = CreateRunRequest {
            assistant_id: assistant_id.to_string(),
            ..Default::default()
        };

        let run = client
            .threads()
            .runs(thread_id)
            .create(run_request)
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        // Runの完了を待ち、結果を取得
        self.wait_for_run(thread_id, &run.id).await
    }

    async fn delete_thread(&self, thread_id: &str) -> ProviderResult<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ProviderError::Configuration("Client not initialized".to_string()))?;

        client
            .threads()
            .delete(thread_id)
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(())
    }

    async fn health_check(&self) -> ProviderResult<()> {
        if self.client.is_none() {
            return Err(ProviderError::Authentication(
                "Client not initialized".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::config::CommonConfig;

    use super::*;

    #[tokio::test]
    async fn test_validate_config() {
        let provider = OpenAIAssistantProvider::new();
        let mut provider_specific = HashMap::new();
        provider_specific.insert(
            "model".to_string(),
            serde_json::Value::String("gpt-4".to_string()),
        );

        let config = ProviderConfig {
            name: "test".to_string(),
            common_config: CommonConfig {
                temperature: 0.7,
                max_tokens: 1000,
                model: "gpt-4".to_string(),
            },
            provider_specific,
            ..Default::default()
        };

        assert!(provider.validate_config(&config).await.is_ok());
    }

    #[tokio::test]
    async fn test_validate_config_invalid_model() {
        let provider = OpenAIAssistantProvider::new();
        let mut provider_specific = HashMap::new();
        provider_specific.insert(
            "model".to_string(),
            serde_json::Value::String("invalid-model".to_string()),
        );

        let config = ProviderConfig {
            name: "test".to_string(),
            common_config: CommonConfig {
                temperature: 0.7,
                max_tokens: 1000,
                model: "abc-1".to_string(),
            },
            provider_specific,
            ..Default::default()
        };

        assert!(provider.validate_config(&config).await.is_err());
    }
}

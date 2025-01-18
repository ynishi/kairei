use crate::{config::ProviderConfig, timestamp::Timestamp};

use super::{capability::Capabilities, types::*};
use async_trait::async_trait;

#[async_trait]
#[mockall::automock]
pub trait ProviderLLM: Send + Sync {
    async fn send_message(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> ProviderResult<LLMResponse>;
    fn capabilities(&self) -> Capabilities;

    fn name(&self) -> &str;
}

#[derive(Debug, Default, Clone)]
pub struct LLMResponse {
    pub content: String,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Default, Clone)]
pub struct ResponseMetadata {
    pub model: String,
    pub created_at: Timestamp,
    pub token_usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
}

type TokenUsage = (usize, usize);

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        config::{CommonConfig, EndpointConfig, ProviderConfig},
        provider::{llms::simple_expert::SimpleExpertProviderLLM, provider::ProviderType},
    };
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_send_message() {
        // use simple_expert::SimpleExpertProvider;
        let config = ProviderConfig {
            name: "test".to_string(),
            common_config: CommonConfig {
                temperature: 0.7,
                max_tokens: 1000,
                model: "gpt-4".to_string(),
            },
            provider_specific: {
                let mut provider_specific = HashMap::new();
                provider_specific.insert("Hello".to_string(), json!("World"));
                provider_specific
            },
            provider_type: ProviderType::SimpleExpert,
            endpoint: EndpointConfig::default(),
            plugin_configs: HashMap::new(),
        };
        let provider = SimpleExpertProviderLLM::new("test");
        let response = provider.send_message("Hello", &config).await.unwrap();
        assert_eq!(response.content, "World");
    }
}

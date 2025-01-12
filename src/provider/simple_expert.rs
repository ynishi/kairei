use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use tracing::{debug, warn};

use crate::config::ProviderConfig;

use super::types::{LLMProvider, ProviderError, ProviderResult, ProviderSecret};

type Pattern = String;

type Answer = String;

type KnowledgeBase = DashMap<Pattern, Answer>;

#[derive(Clone, Default)]
pub struct SimpleExpertProvider {
    name: String,
    knowledge_base: Arc<KnowledgeBase>,
}

impl From<ProviderConfig> for KnowledgeBase {
    fn from(config: ProviderConfig) -> Self {
        let knowledge_base = DashMap::new();

        for (key, value) in config.provider_specific {
            if let Some(value) = value.as_str() {
                knowledge_base.insert(key, value.to_string());
            } else {
                warn!("Invalid value for key: {}", key);
            }
        }

        knowledge_base
    }
}

#[async_trait]
impl LLMProvider for SimpleExpertProvider {
    /// プロバイダー名を取得
    fn name(&self) -> &str {
        &self.name
    }

    /// 初期化処理
    async fn initialize(
        &self,
        config: &ProviderConfig,
        _secret: &ProviderSecret,
    ) -> ProviderResult<Arc<dyn LLMProvider>> {
        let knowledge_base = KnowledgeBase::from(config.clone());
        let provider = SimpleExpertProvider {
            name: self.name.clone(),
            knowledge_base: Arc::new(knowledge_base),
        };
        Ok(Arc::new(provider))
    }

    /// シャットダウン処理
    async fn shutdown(&self) -> ProviderResult<()> {
        Ok(())
    }

    /// 設定のバリデーション
    async fn validate_config(&self, config: &ProviderConfig) -> ProviderResult<()> {
        if KnowledgeBase::from(config.clone()).is_empty() {
            return Err(ProviderError::Configuration(
                "Knowledge base is empty".to_string(),
            ));
        }
        Ok(())
    }

    /// アシスタントの作成
    async fn create_assistant(&self, _config: &ProviderConfig) -> ProviderResult<String> {
        Ok("default".to_string())
    }

    /// スレッドの作成
    async fn create_thread(&self) -> ProviderResult<String> {
        Ok("default".to_string())
    }

    /// メッセージの送信と応答の取得
    async fn send_message(
        &self,
        _thread_id: &str,
        _assistant_id: &str,
        content: &str,
    ) -> ProviderResult<String> {
        // find content include pattern
        let responses: Vec<String> = self
            .knowledge_base
            .iter()
            .filter(|entry| content.contains(entry.key()))
            .map(|entry| entry.value().clone())
            .collect::<Vec<String>>();
        if responses.is_empty() {
            return Err(ProviderError::ApiError("No response found".to_string()));
        }
        debug!("response: {:?}", responses);
        Ok(responses[0].clone())
    }

    /// スレッドの削除
    async fn delete_thread(&self, _thread_id: &str) -> ProviderResult<()> {
        Ok(())
    }

    /// ヘルスチェック
    async fn health_check(&self) -> ProviderResult<()> {
        Ok(())
    }
}

use async_trait::async_trait;
use mockall::automock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

use crate::{config::ProviderConfig, timestamp::Timestamp};

use super::{
    capability::CapabilityType,
    provider::{Provider, ProviderSecret},
};

/// LLMプロバイダーの基本トレイト
#[async_trait]
#[automock]
pub trait LLMProvider: Send + Sync {
    /// プロバイダー名を取得
    fn name(&self) -> &str;

    /// 初期化処理
    async fn initialize(
        &self,
        config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<Arc<dyn LLMProvider>>;

    /// シャットダウン処理
    async fn shutdown(&self) -> ProviderResult<()>;

    /// 設定のバリデーション
    async fn validate_config(&self, config: &ProviderConfig) -> ProviderResult<()>;

    /// アシスタントの作成
    async fn create_assistant(&self, config: &ProviderConfig) -> ProviderResult<String>;

    /// スレッドの作成
    async fn create_thread(&self) -> ProviderResult<String>;

    /// メッセージの送信と応答の取得
    async fn send_message(
        &self,
        thread_id: &str,
        assistant_id: &str,
        content: &str,
    ) -> ProviderResult<String>;

    /// スレッドの削除
    async fn delete_thread(&self, thread_id: &str) -> ProviderResult<()>;

    /// ヘルスチェック
    async fn health_check(&self) -> ProviderResult<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrix {
    pub is_healthy: bool,
    pub last_health_check: Timestamp,
    pub error_count: u32,
    pub last_error: Option<String>,
}

/// LLMプロバイダーのエラー
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Provider initialization error: {0}")]
    Initialization(String),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Primary Provider name not set")]
    PrimaryNameNotSet,

    #[error("Provider secret not found: {0}")]
    SecretNotFound(String),

    #[error("Unknown provider: {0}")]
    UnknownProvider(String),

    #[error("Unsupported capability: {0}")]
    UnsupportedCapability(String),

    //lack of capability
    #[error("Missing Capabilities: {0:?}")]
    MissingCapabilities(Vec<CapabilityType>),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use dashmap::DashMap;
    use serde_json::json;

    use crate::{
        config::ContextConfig,
        context::{AgentInfo, ExecutionContext, StateAccessMode},
        event_bus::EventBus,
        provider::{provider::MockProvider, provider_registry::ProviderInstance},
    };

    use super::*;

    async fn setup_test_context(mock: MockProvider) -> Arc<ExecutionContext> {
        let mut provider_specific = HashMap::new();
        provider_specific.insert("assistant_id".to_string(), json!("mock_assistant_id"));
        let config = ProviderConfig {
            name: "MockProvider".to_string(),
            provider_specific,
            ..Default::default()
        };
        let provider_instance = Arc::new(ProviderInstance {
            config,
            provider: Arc::new(mock),
            secret: ProviderSecret::default(),
        });
        let providers = Arc::new(DashMap::new());
        providers.insert("MockProvider".to_string(), provider_instance.clone());
        let event_bus = Arc::new(EventBus::new(10));
        let context = ExecutionContext::new(
            event_bus,
            AgentInfo::default(),
            StateAccessMode::ReadWrite,
            ContextConfig::default(),
            provider_instance,
            providers,
        );
        Arc::new(context)
    }
    /*
    #[tokio::test]
    async fn test_basic_think_evaluation() {
        let mut mock = MockProvider::new();

        mock.expect_create_thread()
            .returning(|| Box::pin(async move { Ok("test_thread".to_string()) }));

        mock.expect_send_message().returning(|_, _, _| {
            Box::pin(async move { Ok("Mock response about Rust".to_string()) })
        });

        mock.expect_delete_thread()
            .returning(|_| Box::pin(async move { Ok(()) }));

        let context = setup_test_context(mock).await;
        let evaluator = Evaluator::new();
        let args = vec![Argument::Positional(Expression::Literal(Literal::String(
            "Tell me about Rust".to_string(),
        )))];

        let think = Expression::Think {
            args,
            with_block: None,
        };
        let result = evaluator.eval_expression(&think, context).await.unwrap();

        assert!(matches!(result, expression::Value::String(_)));
        assert!(if let expression::Value::String(response) = result {
            response.contains("Mock response about Rust")
        } else {
            false
        });
    }

    #[tokio::test]
    async fn test_error_handling() {
        let mut mock = MockProvider::new();

        mock.expect_create_thread()
            .returning(|| Box::pin(async move { Ok("test_thread".to_string()) }));

        mock.expect_send_message().returning(|_, _, _| {
            Box::pin(async move { Err(ProviderError::ApiError("Simulated error".to_string())) })
        });

        // エラーが発生しても delete_thread が呼ばれることを確認
        mock.expect_delete_thread()
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));

        let context = setup_test_context(mock).await;
        let evaluator = Evaluator::new();
        let args = vec![Argument::Positional(Expression::Literal(Literal::String(
            "Test query".to_string(),
        )))];

        let think = Expression::Think {
            args,
            with_block: None,
        };

        let result = evaluator.eval_expression(&think, context).await;

        assert!(matches!(result, Err(EvalError::Provider(_))));
    }

    #[tokio::test]
    async fn test_think_with_policies() {
        let mut mock = MockProvider::new();

        mock.expect_send_message()
            .withf(|_, _, content| {
                content.contains("Be concise") && content.contains("Use technical terms")
            })
            .returning(|_, _, _| {
                Box::pin(async move { Ok("Response with applied policies".to_string()) })
            });

        mock.expect_create_thread()
            .returning(|| Box::pin(async move { Ok("test_thread".to_string()) }));

        mock.expect_delete_thread()
            .returning(|_| Box::pin(async move { Ok(()) }));

        let context = setup_test_context(mock).await;
        let evaluator = Evaluator::new();

        let policies = vec![
            Policy {
                text: "Be concise".to_string(),
                scope: PolicyScope::Think,
                internal_id: PolicyId::new(),
            },
            Policy {
                text: "Use technical terms".to_string(),
                scope: PolicyScope::Agent("TestAgent".to_string()),
                internal_id: PolicyId::new(),
            },
        ];

        let think_attrs = Some(ThinkAttributes {
            provider: None,
            policies,
            prompt_generator_type: Some(PromptGeneratorType::Standard),
            ..Default::default()
        });

        let args = vec![Argument::Positional(Expression::Literal(Literal::String(
            "Explain ownership".to_string(),
        )))];

        let think = Expression::Think {
            args,
            with_block: think_attrs,
        };

        let result = evaluator.eval_expression(&think, context).await.unwrap();

        assert!(matches!(result, expression::Value::String(_)));
        assert!(if let expression::Value::String(response) = result {
            response.contains("Response with applied policies")
        } else {
            false
        });
    }

    #[tokio::test]
    async fn test_cleanup_on_error() {
        let mut mock = MockProvider::new();

        mock.expect_create_thread()
            .returning(|| Box::pin(async move { Ok("test_thread".to_string()) }));

        mock.expect_send_message().returning(|_, _, _| {
            Box::pin(async move { Err(ProviderError::ApiError("API Error".to_string())) })
        });

        mock.expect_delete_thread()
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));

        let context = setup_test_context(mock).await;
        let evaluator = Evaluator::new();
        let args = vec![Argument::Positional(Expression::Literal(Literal::String(
            "Test query".to_string(),
        )))];

        let think = Expression::Think {
            args,
            with_block: None,
        };

        let _ = evaluator.eval_expression(&think, context).await;
    }
    */
}

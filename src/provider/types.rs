use async_trait::async_trait;
use mockall::automock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use thiserror::Error;

/// LLMプロバイダーの基本トレイト
#[async_trait]
#[automock]
pub trait LLMProvider: Send + Sync {
    /// プロバイダー名を取得
    fn name(&self) -> &str;

    /// 初期化処理
    async fn initialize(&self, params: ProviderParams) -> ProviderResult<Arc<dyn LLMProvider>>;

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

/// プロバイダーの状態
#[derive(Clone)]
pub struct ProviderInstance {
    pub config: ProviderConfig,
    pub provider: Arc<dyn LLMProvider>,
}

impl Default for ProviderInstance {
    fn default() -> Self {
        Self {
            config: ProviderConfig::default(),
            // TODO: モックを削除
            provider: Arc::new(MockLLMProvider::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderState {
    pub is_healthy: bool,
    pub last_health_check: SystemTime,
}

/// LLMプロバイダーの設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    /// プロバイダー名
    pub name: String,
    /// 共通設定
    pub common_config: CommonConfig,
    /// プロバイダー固有の設定
    pub provider_specific: HashMap<String, serde_json::Value>,
}

/// 共通設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonConfig {
    /// 温度パラメータ (0.0-1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// 最大トークン数
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

fn default_temperature() -> f32 {
    0.7
}
fn default_max_tokens() -> usize {
    1000
}

/// プロバイダーの初期化パラメータ
#[derive(Debug, Clone)]
pub struct ProviderParams {
    pub config: ProviderConfig,
    pub auth: HashMap<String, String>,
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
    NotFound(String),

    #[error("Provider name not specified")]
    NameNotSpecified,
}

pub type ProviderResult<T> = Result<T, ProviderError>;

#[cfg(test)]
mod tests {

    use serde_json::json;

    use crate::{
        config::ContextConfig,
        context::{AgentInfo, ExecutionContext, StateAccessMode},
        evaluator::{EvalError, Evaluator},
        event_bus::EventBus,
        expression, Argument, Expression, Literal, Policy, PolicyId, PolicyScope,
        PromptGeneratorType, ThinkAttributes,
    };

    use super::*;

    async fn setup_test_context(mock: MockLLMProvider) -> Arc<ExecutionContext> {
        let mut provider_specific = HashMap::new();
        provider_specific.insert("assistant_id".to_string(), json!("mock_assistant_id"));
        let config = ProviderConfig {
            name: "MockProvider".to_string(),
            common_config: CommonConfig::default(),
            provider_specific,
        };
        let provider_instance = Arc::new(ProviderInstance {
            config,
            provider: Arc::new(mock),
        });
        let event_bus = Arc::new(EventBus::new(10));
        let mut context = ExecutionContext::new(
            event_bus,
            AgentInfo::default(),
            StateAccessMode::ReadWrite,
            ContextConfig::default(),
        );
        Arc::new(context.with_provider(provider_instance).clone())
    }

    #[tokio::test]
    async fn test_basic_think_evaluation() {
        let mut mock = MockLLMProvider::new();

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
        let mut mock = MockLLMProvider::new();

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
        let mut mock = MockLLMProvider::new();

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
        let mut mock = MockLLMProvider::new();

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
}

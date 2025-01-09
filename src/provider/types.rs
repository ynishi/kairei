use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use thiserror::Error;

/// LLMプロバイダーの基本トレイト
#[async_trait]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderState {
    pub is_healthy: bool,
    pub last_health_check: SystemTime,
}

/// LLMプロバイダーの設定
#[derive(Debug, Clone, Serialize, Deserialize)]
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

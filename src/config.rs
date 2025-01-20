use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader, path::Path, time::Duration};

use crate::{provider::provider::ProviderType, Error, InternalResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    #[serde(default = "default_event_buffer_size")]
    pub event_buffer_size: usize,

    #[serde(default = "default_max_agents")]
    pub max_agents: usize,

    #[serde(default = "default_init_timeout", with = "duration_ms")]
    pub init_timeout: Duration,

    #[serde(default = "default_shutdown_timeout", with = "duration_ms")]
    pub shutdown_timeout: Duration,

    #[serde(default = "default_request_timeout", with = "duration_ms")]
    pub request_timeout: Duration,

    #[serde(default)]
    pub agent_config: AgentConfig,

    #[serde(default)]
    pub native_feature_config: NativeFeatureConfig,

    #[serde(default)]
    pub provider_configs: ProviderConfigs,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    #[serde(default)]
    pub context: ContextConfig,

    #[serde(default)]
    pub scale_manager: Option<ScaleManagerConfig>,

    #[serde(default)]
    pub monitor: Option<MonitorConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    #[serde(default = "default_access_timeout")]
    pub access_timeout: Duration,
    #[serde(default = "default_request_timeout", with = "duration_ms")]
    pub request_timeout: Duration,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            access_timeout: default_access_timeout(),
            request_timeout: default_request_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleManagerConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_max_instances")]
    pub max_instances_per_agent: usize,

    #[serde(default = "default_scale_interval", with = "duration_ms")]
    pub scale_check_interval: Duration,
}

impl Default for ScaleManagerConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            max_instances_per_agent: default_max_instances(),
            scale_check_interval: default_scale_interval(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_metrics_interval", with = "duration_ms")]
    pub metrics_interval: Duration,

    #[serde(default = "default_retention_period", with = "duration_ms")]
    pub retention_period: Duration,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            metrics_interval: default_metrics_interval(),
            retention_period: default_retention_period(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeFeatureConfig {
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout: Duration,

    #[serde(default = "default_ticker_config")]
    pub ticker: Option<TickerConfig>,
}

impl Default for NativeFeatureConfig {
    fn default() -> Self {
        Self {
            shutdown_timeout: default_shutdown_timeout(),
            ticker: default_ticker_config(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_tick_interval", with = "duration_ms")]
    pub tick_interval: Duration,
}

impl Default for TickerConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            tick_interval: default_tick_interval(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigs {
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default = "some_default_provider_name")]
    pub primary_provider: Option<String>,
}

impl Default for ProviderConfigs {
    fn default() -> Self {
        Self {
            providers: {
                let mut map = HashMap::new();
                map.insert(default_provider_name(), ProviderConfig::default());
                map
            },
            primary_provider: some_default_provider_name(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    // 基本情報
    #[serde(default = "ProviderType::default")]
    pub provider_type: ProviderType,

    #[serde(default = "default_provider_name")]
    pub name: String,

    // 共通設定
    #[serde(default)]
    pub common_config: CommonConfig,

    // エンドポイント設定
    #[serde(default)]
    pub endpoint: EndpointConfig,

    // プロバイダー固有設定
    #[serde(default)]
    pub provider_specific: HashMap<String, serde_json::Value>,

    pub plugin_configs: HashMap<String, PluginConfig>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::default(),
            name: default_provider_name(),
            common_config: CommonConfig::default(),
            endpoint: EndpointConfig::default(),
            provider_specific: HashMap::new(),
            plugin_configs: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum PluginConfig {
    Memory(MemoryConfig),
    Rag(RagConfig),
    Search(SearchConfig),
    Unknown(HashMap<String, serde_json::Value>),
}

/// メモリプラグインの設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryConfig {
    #[serde(default = "default_max_short_term")]
    pub max_short_term: usize,
    #[serde(default = "default_max_long_term")]
    pub max_long_term: usize,
    #[serde(default = "default_importance_threshold")]
    pub importance_threshold: f64,
    #[serde(default = "default_max_items")]
    pub max_items: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_short_term: default_max_short_term(),
            max_long_term: default_max_long_term(),
            importance_threshold: default_importance_threshold(),
            max_items: default_max_items(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RagConfig {
    #[serde(default = "default_collection_name")]
    pub collection_name: String,
    #[serde(default = "default_rag_max_results")]
    pub max_results: usize,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            collection_name: default_collection_name(),
            max_results: default_max_results(),
            similarity_threshold: default_similarity_threshold(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchConfig {
    #[serde(default = "default_search_window")]
    pub search_window: Duration,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_search_filters")]
    pub filters: Vec<String>,
    #[serde(default = "default_max_fetch_per_result")]
    pub max_fetch_per_result: usize,
    // fetch_timeout
    #[serde(default = "default_fetch_timeout", with = "duration_ms")]
    pub fetch_timeout: Duration,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            search_window: default_search_window(),
            max_results: default_max_results(),
            filters: default_search_filters(),
            max_fetch_per_result: default_max_fetch_per_result(),
            fetch_timeout: default_fetch_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonConfig {
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    #[serde(default = "default_model")]
    pub model: String,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            model: default_model(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    #[serde(default = "default_endpoint")]
    pub url: Option<String>,
    #[serde(default = "default_api_version")]
    pub api_version: Option<String>,
    pub deployment_id: Option<String>,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            url: default_endpoint(),
            api_version: default_api_version(),
            deployment_id: None,
        }
    }
}

/// シークレット設定(secret.json)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretConfig {
    pub providers: HashMap<String, ProviderSecretConfig>,
}

impl Default for SecretConfig {
    fn default() -> Self {
        Self {
            providers: {
                let mut map = HashMap::new();
                map.insert(default_provider_name(), ProviderSecretConfig::default());
                map
            },
        }
    }
}

pub fn from_file<T: for<'de> Deserialize<'de>, P: AsRef<Path>>(path: P) -> InternalResult<T> {
    let file = File::open(path)
        .map_err(|e| Error::Internal(format!("Failed to open secret file: {}", e)))?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)
        .map_err(|e| Error::Internal(format!("Failed to parse secret file: {}", e)))?;
    Ok(config)
}

pub fn from_str<T: for<'de> Deserialize<'de>>(s: &str) -> InternalResult<T> {
    let config = serde_json::from_str(s)
        .map_err(|e| Error::Internal(format!("Failed to parse secret file: {}", e)))?;
    Ok(config)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProviderSecretConfig {
    pub api_key: String,
    #[serde(default = "HashMap::new")]
    pub additional_auth: HashMap<String, String>, // 追加の認証情報
}

// デフォルト値の定義
fn default_event_buffer_size() -> usize {
    1000
}
fn default_tick_interval() -> Duration {
    Duration::from_millis(100)
}
fn default_max_agents() -> usize {
    100
}
fn default_init_timeout() -> Duration {
    Duration::from_secs(30)
}
fn default_shutdown_timeout() -> Duration {
    Duration::from_secs(30)
}
fn default_request_timeout() -> Duration {
    Duration::from_secs(60)
}
fn default_true() -> bool {
    true
}
fn default_max_instances() -> usize {
    10
}
fn default_scale_interval() -> Duration {
    Duration::from_secs(60)
}
fn default_metrics_interval() -> Duration {
    Duration::from_secs(10)
}
fn default_retention_period() -> Duration {
    Duration::from_secs(3600)
}

fn default_access_timeout() -> Duration {
    Duration::from_secs(5)
}

fn default_ticker_config() -> Option<TickerConfig> {
    Some(TickerConfig::default())
}

fn default_provider_name() -> String {
    "default_provider".to_string()
}

fn some_default_provider_name() -> Option<String> {
    Some(default_provider_name())
}

fn default_temperature() -> f32 {
    0.7
}
fn default_max_tokens() -> usize {
    1000
}

fn default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

fn default_endpoint() -> Option<String> {
    Some("https://api.openai.com".to_string())
}

fn default_api_version() -> Option<String> {
    Some("v1".to_string())
}

fn default_max_short_term() -> usize {
    5
}

fn default_max_long_term() -> usize {
    10
}

fn default_importance_threshold() -> f64 {
    0.5
}

fn default_max_items() -> usize {
    100
}

fn default_search_window() -> Duration {
    Duration::from_secs(60)
}

fn default_max_results() -> usize {
    10
}

fn default_search_filters() -> Vec<String> {
    vec![]
}

fn default_max_fetch_per_result() -> usize {
    3
}

fn default_collection_name() -> String {
    "default_collection".to_string()
}

fn default_similarity_threshold() -> f64 {
    0.5
}

fn default_rag_max_results() -> usize {
    10
}

fn default_fetch_timeout() -> Duration {
    Duration::from_secs(5)
}

// Duration型のシリアライズ/デシリアライズヘルパー
mod duration_ms {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            event_buffer_size: default_event_buffer_size(),
            max_agents: default_max_agents(),
            init_timeout: default_init_timeout(),
            shutdown_timeout: default_shutdown_timeout(),
            request_timeout: default_request_timeout(),
            agent_config: AgentConfig::default(),
            native_feature_config: NativeFeatureConfig::default(),
            provider_configs: ProviderConfigs::default(),
        }
    }
}

impl SystemConfig {
    // JSONファイルから設定を読み込む
    pub fn from_file(path: &str) -> InternalResult<Self> {
        from_file(path)
    }
}

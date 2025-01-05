use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    #[serde(default = "default_event_buffer_size")]
    pub event_buffer_size: usize,

    #[serde(default = "default_tick_interval", with = "duration_ms")]
    pub tick_interval: Duration,

    #[serde(default = "default_max_agents")]
    pub max_agents: usize,

    #[serde(default = "default_init_timeout", with = "duration_ms")]
    pub init_timeout: Duration,

    #[serde(default = "default_shutdown_timeout", with = "duration_ms")]
    pub shutdown_timeout: Duration,

    #[serde(default)]
    pub agent_config: AgentConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextConfig {
    #[serde(default = "default_access_timeout")]
    pub access_timeout: Duration,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_metrics_interval", with = "duration_ms")]
    pub metrics_interval: Duration,

    #[serde(default = "default_retention_period", with = "duration_ms")]
    pub retention_period: Duration,
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
            tick_interval: default_tick_interval(),
            max_agents: default_max_agents(),
            init_timeout: default_init_timeout(),
            shutdown_timeout: default_shutdown_timeout(),
            agent_config: AgentConfig::default(),
        }
    }
}

impl SystemConfig {
    // JSONファイルから設定を読み込む
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let config = serde_json::from_reader(file)?;
        Ok(config)
    }
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    config::ProviderConfig, context::AgentInfo, expression::Value, timestamp::Timestamp, Policy,
};

use super::{llm::LLMResponse, provider::ProviderSecret};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ProviderRequest {
    // 1. 基本的な入力データ
    pub input: RequestInput,
    // 2. 実行コンテキスト
    pub state: ExecutionState,
    // 3. プロバイダー設定(settings from with block)
    pub config: RuntimeConfig,
}

type RuntimeConfig = ProviderConfig;

// divide from Instance.
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ProviderContext {
    pub config: ProviderConfig,
    #[serde(skip)]
    pub secret: ProviderSecret,
}

// 1. 基本的な入力データ
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct RequestInput {
    // メインのプロンプト/クエリ
    pub query: Value,
    // Think式から渡されるパラメータ
    pub parameters: HashMap<String, Value>,
}

// 2. 実行時の状態
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ExecutionState {
    // セッション管理
    pub session_id: String,
    pub timestamp: Timestamp,

    // エージェントのコンテキスト
    pub agent_name: String,
    pub agent_info: AgentInfo,

    // Policy
    pub policies: Vec<Policy>,

    // 実行トレース（デバッグ用）
    pub trace_id: String,
}

#[derive(Debug, Default)]
pub struct ProviderResponse {
    pub output: String,
    pub metadata: ResponseMetadata,
}

impl From<LLMResponse> for ProviderResponse {
    fn from(response: LLMResponse) -> Self {
        Self {
            output: response.content,
            metadata: ResponseMetadata {
                timestamp: response.metadata.created_at,
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResponseMetadata {
    pub timestamp: Timestamp,
}

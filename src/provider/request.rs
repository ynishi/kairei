use std::collections::HashMap;

use crate::{config::ProviderConfig, expression::Value};

use super::{
    llm::LLMResponse,
    types::{ProviderSecret, Timestamp},
};

#[derive(Debug, Default)]
pub struct ProviderRequest {
    // 1. 基本的な入力データ
    pub input: RequestInput,
    // 2. 実行コンテキスト
    pub state: ExecutionState,
    // 3. プロバイダー設定(settings from with block)
    pub config: RuntimeConfig,
}

type RuntimeConfig = ProviderConfig;

// devide from Instance.
#[derive(Default)]
pub struct ProviderContext {
    pub config: ProviderConfig,
    pub secret: ProviderSecret,
}

// 1. 基本的な入力データ
#[derive(Debug, Default)]
pub struct RequestInput {
    // メインのプロンプト/クエリ
    pub query: String,
    // Think式から渡されるパラメータ
    pub parameters: HashMap<String, Value>,
}

// 2. 実行時の状態
#[derive(Debug, Default)]
pub struct ExecutionState {
    // セッション管理
    pub session_id: String,
    pub timestamp: Timestamp,

    // エージェントのコンテキスト
    pub agent_name: String,
    pub agent_state: HashMap<String, Value>,

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

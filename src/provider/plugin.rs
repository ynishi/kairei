use std::collections::HashMap;

use crate::config::PluginConfig;

use super::{
    capability::CapabilityType,
    llm::LLMResponse,
    provider::Section,
    request::{ProviderContext, ProviderRequest},
    types::*,
};
use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
#[mockall::automock]
pub trait ProviderPlugin: Send + Sync {
    /// プラグインの優先度
    fn priority(&self) -> i32;

    /// capabilityの提供(SingleCapabilityのみ)
    fn capability(&self) -> CapabilityType;

    /// プロンプト生成前の処理
    async fn generate_section<'a>(&self, contxt: &PluginContext<'a>) -> ProviderResult<Section>;

    /// LLMレスポンス後の処理
    async fn process_response<'a>(
        &self,
        context: &PluginContext<'a>,
        response: &LLMResponse,
    ) -> ProviderResult<()>;
}

#[derive(Clone, Serialize)]
pub struct PluginContext<'a> {
    pub request: &'a ProviderRequest,
    pub configs: &'a HashMap<String, PluginConfig>,
    pub context: &'a ProviderContext,
}

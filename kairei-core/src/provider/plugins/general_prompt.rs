use async_trait::async_trait;
use tracing::debug;

use crate::{
    expression,
    provider::{
        capabilities::common::CapabilityType,
        llm::LLMResponse,
        plugin::{PluginContext, ProviderPlugin},
        provider::Section,
        types::{ProviderError, ProviderResult},
    },
};

#[derive(Debug, Default, Clone)]
pub struct GeneralPromptPlugin;

#[async_trait]
impl ProviderPlugin for GeneralPromptPlugin {
    fn priority(&self) -> i32 {
        0 // 最も低い優先度
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::GeneralPrompt
    }

    #[tracing::instrument(skip(self, context), level = "debug")]
    async fn generate_section<'a>(&self, context: &PluginContext<'a>) -> ProviderResult<Section> {
        // コンテキストからクエリを取得して基本的なセクションを生成
        let query = match context.request.input.query.clone() {
            expression::Value::String(s) => s,
            _ => return Err(ProviderError::InvalidRequest("Invalid query format".into())),
        };
        debug!("query: {}", query);

        let params = context.request.input.parameters.clone();
        let params_str = params
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join(", ");

        let content = if params_str.is_empty() {
            query.to_string()
        } else {
            format!("{}\n\nparameters:({})", query, params_str)
        };

        Ok(Section {
            content,
            priority: self.priority(),
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, _context, _response), level = "debug")]
    async fn process_response<'a>(
        &self,
        _context: &PluginContext<'a>,
        _response: &LLMResponse,
    ) -> ProviderResult<()> {
        // 基本プラグインでは後処理は不要
        Ok(())
    }
}

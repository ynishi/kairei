pub mod general_prompt;
pub mod memory;
pub mod policy;
pub mod shared_memory;
pub mod web_search_serper;

#[cfg(test)]
mod provider_tests {
    use std::collections::HashMap;

    use crate::{
        config::PluginConfig,
        provider::{
            plugin::PluginContext,
            request::{ProviderContext, ProviderRequest, RequestInput},
        },
    };

    // テストコンテキストのホルダー構造体
    #[derive(Clone)]
    pub struct TestContextHolder {
        pub request: ProviderRequest,
        context: ProviderContext,
        configs: HashMap<String, PluginConfig>,
    }

    impl TestContextHolder {
        pub fn new(request_content: &str) -> Self {
            let input = RequestInput {
                query: crate::expression::Value::String(request_content.to_string()),
                ..Default::default()
            };
            let request = ProviderRequest {
                input,
                ..Default::default()
            };

            Self {
                request,
                context: ProviderContext::default(),
                configs: HashMap::new(),
            }
        }

        pub fn get_plugin_context(&self) -> PluginContext<'_> {
            PluginContext {
                request: &self.request,
                context: &self.context,
                configs: &self.configs,
            }
        }
    }
}

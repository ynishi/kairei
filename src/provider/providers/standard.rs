use std::sync::Arc;
use tokio::sync::RwLock;

use async_trait::async_trait;
use tracing::debug;

use crate::{
    config::ProviderConfig,
    provider::{
        capability::{Capabilities, CapabilityType, RequiredCapabilities, RequiresCapabilities},
        generator::generator::{Generator, PromptGenerator},
        llm::{LLMResponse, ProviderLLM},
        llms::simple_expert::SimpleExpertProviderLLM,
        plugin::{PluginContext, ProviderPlugin},
        plugins::{general_prompt::GeneralPromptPlugin, policy::PolicyPlugin},
        provider::{Provider, ProviderSecret, Section},
        request::{ProviderContext, ProviderRequest, ProviderResponse},
        types::ProviderResult,
    },
};

pub struct StandardProvider {
    name: String,
    llm: Arc<RwLock<dyn ProviderLLM>>,
    plugins: Vec<Arc<dyn ProviderPlugin>>,
    generator: Arc<dyn Generator>,
}

impl Default for StandardProvider {
    fn default() -> Self {
        Self {
            name: "StandardProvider".to_string(),
            llm: Arc::new(RwLock::new(SimpleExpertProviderLLM::new(
                "SimpleExpertProviderLLM",
            ))),

            plugins: vec![Arc::new(GeneralPromptPlugin), Arc::new(PolicyPlugin)],
            generator: Arc::new(PromptGenerator::new(None)),
        }
    }
}

#[async_trait]
impl Provider for StandardProvider {
    async fn initialize(
        &mut self,
        config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        self.llm.write().await.initialize(config, secret).await?;

        let required = self.required_capabilities();
        let current = self.capabilities().await;

        required.unsupported(&current)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, context, request))]
    async fn execute(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse> {
        // 1. プラグインによるセクション生成
        let context = Arc::new(PluginContext {
            context,
            request,
            configs: &request.config.plugin_configs,
        });
        debug!("context: {:?}", request);
        let sections = self.generate_plugin_sections(&context).await?;
        debug!("sections: {:?}", sections);
        // 2. プロンプトの生成
        let prompt = self.generator.generate(sections).await?;
        debug!("prompt: {}", prompt);
        // 3. LLMの実行
        let llm_response = self
            .llm
            .read()
            .await
            .send_message(&prompt, &request.config)
            .await?;
        debug!("llm_response: {:?}", llm_response);

        // 4. プラグインの後処理
        self.process_plugins_response(&context, &llm_response)
            .await?;

        // 5. レスポンスの構築
        Ok(ProviderResponse::from(llm_response))
    }
    async fn capabilities(&self) -> Capabilities {
        self.llm.read().await.capabilities().or(self
            .plugins
            .iter()
            .fold(Capabilities::default(), |acc, p| {
                acc.or(Capabilities::from(p.capability()))
            }))
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    async fn shutdown(&self) -> ProviderResult<()> {
        self.llm.write().await.stop().await?;
        Ok(())
    }
}

impl RequiresCapabilities for StandardProvider {
    fn required_capabilities(&self) -> RequiredCapabilities {
        RequiredCapabilities::new(vec![
            CapabilityType::Generate,
            CapabilityType::PolicyPrompt,
            CapabilityType::SystemPrompt,
        ])
    }
}

impl StandardProvider {
    pub fn new<T: ProviderLLM + 'static>(llm: T, plugins: Vec<Arc<dyn ProviderPlugin>>) -> Self {
        let default = Self::default();
        let name = llm.name().to_string();
        let llm = Arc::new(RwLock::new(llm));
        if plugins.is_empty() {
            return Self {
                name,
                llm,
                plugins: default.plugins,
                generator: default.generator,
            };
        }
        Self {
            name,
            llm,
            plugins,
            generator: default.generator,
        }
    }

    pub fn register_plugin(&mut self, plugin: Arc<dyn ProviderPlugin>) -> ProviderResult<()> {
        self.plugins.push(plugin);
        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    async fn generate_plugin_sections<'a>(
        &self,
        context: &PluginContext<'a>,
    ) -> ProviderResult<Vec<Section>> {
        debug!("generate_plugin_sections");
        let mut sections = Vec::new();

        let mut plugins = self.plugins.clone();
        plugins.sort_by_key(|p| p.priority());

        let llm_capabilities = self.llm.read().await.capabilities();

        for plugin in &plugins {
            if llm_capabilities.supports(&plugin.capability()) {
                continue;
            }
            let section = plugin.generate_section(context).await?;
            sections.push(section);
        }

        Ok(sections)
    }

    #[allow(clippy::needless_lifetimes)]
    async fn process_plugins_response<'a>(
        &self,
        context: &PluginContext<'a>,
        response: &LLMResponse,
    ) -> ProviderResult<()> {
        let mut plugins = self.plugins.clone();
        plugins.sort_by_key(|p| p.priority());

        for plugin in &plugins {
            plugin.process_response(context, response).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        config::ProviderConfig,
        expression,
        provider::request::{ProviderContext, ProviderRequest, RequestInput},
    };

    use super::*;
    use async_trait::async_trait;

    struct MockLLM {
        name: String,
        capabilities: Capabilities,
    }

    #[async_trait]
    impl ProviderLLM for MockLLM {
        async fn send_message(
            &self,
            _prompt: &str,
            _config: &ProviderConfig,
        ) -> ProviderResult<LLMResponse> {
            Ok(LLMResponse::default())
        }

        fn capabilities(&self) -> Capabilities {
            self.capabilities.clone()
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn initialize(
            &mut self,
            _config: &ProviderConfig,
            _secret: &ProviderSecret,
        ) -> ProviderResult<()> {
            Ok(())
        }
    }

    #[allow(dead_code)]
    struct MockPlugin {
        name: String,
        capability: CapabilityType,
        priority: i32,
    }
    #[async_trait]
    impl ProviderPlugin for MockPlugin {
        fn priority(&self) -> i32 {
            self.priority
        }

        fn capability(&self) -> CapabilityType {
            self.capability.clone()
        }

        async fn generate_section<'a>(
            &self,
            _context: &PluginContext<'a>,
        ) -> ProviderResult<Section> {
            Ok(Section::default())
        }

        async fn process_response<'a>(
            &self,
            _context: &PluginContext<'a>,
            _response: &LLMResponse,
        ) -> ProviderResult<()> {
            Ok(())
        }
    }

    fn create_valid_request() -> ProviderRequest {
        let input = RequestInput {
            query: expression::Value::String("test".to_string()),
            parameters: HashMap::new(),
        };
        let mut request = ProviderRequest::default();
        request.input = input;
        request
    }

    // LLM -> Generate
    // Plugin -> Search
    // response has output
    #[tokio::test]
    async fn test_execute() {
        let llm = MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        };

        let plugin = Arc::new(MockPlugin {
            name: "mock_plugin".to_string(),
            capability: CapabilityType::Search,
            priority: 0,
        });

        let mut provider = StandardProvider::new(llm, vec![]);
        provider.register_plugin(plugin).unwrap();

        let context = ProviderContext::default();
        let request = create_valid_request();
        let response = provider.execute(&context, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }

    // LLM -> Generate
    // Plugin -> Search
    // response has output
    #[tokio::test]
    async fn test_execute_with_plugin() {
        let llm = MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        };

        let plugin = Arc::new(MockPlugin {
            name: "mock_plugin".to_string(),
            capability: CapabilityType::Search,
            priority: 0,
        });

        let mut provider = StandardProvider::new(llm, vec![]);
        provider.register_plugin(plugin).unwrap();

        let context = ProviderContext::default();
        let request = create_valid_request();
        let response = provider.execute(&context, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_with_multiple_plugins() {
        let llm = MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        };

        let plugin1 = Arc::new(MockPlugin {
            name: "mock_plugin1".to_string(),
            capability: CapabilityType::Search,
            priority: 0,
        });

        let plugin2 = Arc::new(MockPlugin {
            name: "mock_plugin2".to_string(),
            capability: CapabilityType::Search,
            priority: 1,
        });

        let mut provider = StandardProvider::new(llm, vec![]);
        provider.register_plugin(plugin1).unwrap();
        provider.register_plugin(plugin2).unwrap();

        let context = ProviderContext::default();
        let request = create_valid_request();
        let response = provider.execute(&context, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_with_llm_plugin() {
        let llm = MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        };

        let plugin = Arc::new(MockPlugin {
            name: "mock_plugin".to_string(),
            capability: CapabilityType::Generate,
            priority: 0,
        });

        let mut provider = StandardProvider::new(llm, vec![]);
        provider.register_plugin(plugin).unwrap();

        let context = ProviderContext::default();
        let request = create_valid_request();
        let response = provider.execute(&context, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_with_llm_plugin_and_plugin() {
        let llm = MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        };

        let plugin1 = Arc::new(MockPlugin {
            name: "mock_plugin1".to_string(),
            capability: CapabilityType::Generate,
            priority: 0,
        });

        let plugin2 = Arc::new(MockPlugin {
            name: "mock_plugin2".to_string(),
            capability: CapabilityType::Search,
            priority: 1,
        });

        let mut provider = StandardProvider::new(llm, vec![]);
        provider.register_plugin(plugin1).unwrap();
        provider.register_plugin(plugin2).unwrap();

        let context = ProviderContext::default();
        let request = create_valid_request();
        let response = provider.execute(&context, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }
}

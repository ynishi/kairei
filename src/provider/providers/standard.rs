use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    config::ProviderConfig,
    provider::{
        capability::{Capabilities, CapabilityType, RequiredCapabilities, RequiresCapabilities},
        generator::generator::{Generator, PromptGenerator},
        llm::{LLMResponse, ProviderLLM},
        llms::simple_expert::SimpleExpertProviderLLM,
        plugin::{PluginContext, ProviderPlugin},
        provider::{Provider, ProviderSecret, Section},
        request::{ProviderContext, ProviderRequest, ProviderResponse},
        types::ProviderResult,
    },
};

pub struct StandardProvider {
    llm: Arc<dyn ProviderLLM>,
    plugins: Vec<Arc<dyn ProviderPlugin>>,
    generator: Arc<dyn Generator>,
}

impl Default for StandardProvider {
    fn default() -> Self {
        Self {
            llm: Arc::new(SimpleExpertProviderLLM::new("SimpleExpertProviderLLM")),
            plugins: Vec::new(),
            generator: Arc::new(PromptGenerator::new(None)),
        }
    }
}

#[async_trait]
impl Provider for StandardProvider {
    async fn initialize(
        &self,
        config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        let required = self.required_capabilities();
        let current = self.capabilities();

        required.unsupported(&current)?;

        Ok(())
    }

    async fn execute(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse> {
        {
            // 1. プラグインによるセクション生成
            let context = PluginContext {
                request: &request,
                configs: &request.config.plugin_configs,
            };
            let sections = self.generate_plugin_sections(&context).await?;

            // 2. プロンプトの生成
            let prompt = self.generator.generate(sections).await?;

            // 3. LLMの実行
            let llm_response = self.llm.send_message(&prompt, &request.config).await?;

            // 4. プラグインの後処理
            self.process_plugins_response(&context, &llm_response)
                .await?;

            // 5. レスポンスの構築
            Ok(ProviderResponse::from(llm_response))
        }
    }
    fn capabilities(&self) -> Capabilities {
        self.llm
            .capabilities()
            .or(self.plugins.iter().fold(Capabilities::default(), |acc, p| {
                acc.or(Capabilities::from(p.capability()))
            }))
    }

    fn name(&self) -> &str {
        self.llm.name()
    }

    async fn shutdown(&self) -> ProviderResult<()> {
        todo!()
    }
}

impl RequiresCapabilities for StandardProvider {
    fn required_capabilities(&self) -> RequiredCapabilities {
        RequiredCapabilities::new(vec![CapabilityType::Generate, CapabilityType::SystemPrompt])
    }
}

impl StandardProvider {
    pub fn new(llm: Arc<dyn ProviderLLM>) -> Self {
        Self {
            llm,
            plugins: Vec::new(),
            generator: Arc::new(PromptGenerator::new(None)),
        }
    }

    pub fn register_plugin(&mut self, plugin: Arc<dyn ProviderPlugin>) -> ProviderResult<()> {
        self.plugins.push(plugin);
        Ok(())
    }

    async fn generate_plugin_sections<'a>(
        &self,
        context: &PluginContext<'a>,
    ) -> ProviderResult<Vec<Section>> {
        let mut sections = Vec::new();

        let mut plugins = self.plugins.clone();
        plugins.sort_by_key(|p| p.priority());

        let llm_capabilities = self.llm.capabilities();

        for plugin in &plugins {
            if llm_capabilities.supports(&plugin.capability()) {
                continue;
            }
            let section = plugin.generate_section(context).await?;
            sections.push(section);
        }

        Ok(sections)
    }

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
    use crate::{
        config::ProviderConfig,
        provider::request::{ProviderContext, ProviderRequest},
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
    }

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

    // LLM -> Generate
    // Plugin -> Search
    // response has output
    #[tokio::test]
    async fn test_execute() {
        let llm = Arc::new(MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        });

        let plugin = Arc::new(MockPlugin {
            name: "mock_plugin".to_string(),
            capability: CapabilityType::Search,
            priority: 0,
        });

        let mut provider = StandardProvider::new(llm);
        provider.register_plugin(plugin).unwrap();

        let contxt = ProviderContext::default();
        let request = ProviderRequest::default();
        let response = provider.execute(&contxt, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }

    // LLM -> Generate
    // Plugin -> Search
    // response has output
    #[tokio::test]
    async fn test_execute_with_plugin() {
        let llm = Arc::new(MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        });

        let plugin = Arc::new(MockPlugin {
            name: "mock_plugin".to_string(),
            capability: CapabilityType::Search,
            priority: 0,
        });

        let mut provider = StandardProvider::new(llm);
        provider.register_plugin(plugin).unwrap();

        let contxt = ProviderContext::default();
        let request = ProviderRequest::default();
        let response = provider.execute(&contxt, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_with_multiple_plugins() {
        let llm = Arc::new(MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        });

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

        let mut provider = StandardProvider::new(llm);
        provider.register_plugin(plugin1).unwrap();
        provider.register_plugin(plugin2).unwrap();

        let contxt = ProviderContext::default();
        let request = ProviderRequest::default();
        let response = provider.execute(&contxt, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_with_llm_plugin() {
        let llm = Arc::new(MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        });

        let plugin = Arc::new(MockPlugin {
            name: "mock_plugin".to_string(),
            capability: CapabilityType::Generate,
            priority: 0,
        });

        let mut provider = StandardProvider::new(llm);
        provider.register_plugin(plugin).unwrap();

        let contxt = ProviderContext::default();
        let request = ProviderRequest::default();
        let response = provider.execute(&contxt, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_with_llm_plugin_and_plugin() {
        let llm = Arc::new(MockLLM {
            name: "mock_llm".to_string(),
            capabilities: Capabilities::from(CapabilityType::Generate),
        });

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

        let mut provider = StandardProvider::new(llm);
        provider.register_plugin(plugin1).unwrap();
        provider.register_plugin(plugin2).unwrap();

        let contxt = ProviderContext::default();
        let request = ProviderRequest::default();
        let response = provider.execute(&contxt, &request).await.unwrap();

        assert_eq!(response.output.len(), 0);
    }
}

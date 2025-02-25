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

/// # Provider Plugin Interface
///
/// The `ProviderPlugin` trait defines the extension points for provider functionality.
/// Plugins add specific capabilities to providers, such as memory management,
/// web search, policy enforcement, and more.
///
/// ## Plugin Lifecycle
///
/// Plugins follow a defined lifecycle:
/// 1. Registration - Plugin is registered with the provider
/// 2. Capability Declaration - Plugin declares its capabilities and requirements
/// 3. Section Generation - Plugin contributes to prompt generation
/// 4. Response Processing - Plugin processes LLM responses
///
/// ## Plugin Architecture
///
/// The plugin architecture enables:
/// - Modular extension of provider capabilities
/// - Separation of concerns between core LLM functionality and extensions
/// - Reusable components across different providers
/// - Dynamic capability negotiation
///
/// ## Implementation Example
///
/// ```ignore
/// # use kairei::provider::plugin::ProviderPlugin;
/// # use kairei::provider::plugin::PluginContext;
/// # use kairei::provider::provider::Section;
/// # use kairei::provider::capability::CapabilityType;
/// # use kairei::provider::types::ProviderResult;
/// # use kairei::provider::llm::LLMResponse;
/// # use async_trait::async_trait;
/// #
/// # struct MyPlugin {
/// #     // Plugin-specific fields
/// # }
/// #
/// # #[async_trait]
/// # impl ProviderPlugin for MyPlugin {
/// #     fn priority(&self) -> i32 {
/// #         10 // Higher priority plugins run first
/// #     }
/// #
/// #     fn capability(&self) -> CapabilityType {
/// #         CapabilityType::Memory // This plugin provides memory capability
/// #     }
/// #
/// #     async fn generate_section<'a>(&self, context: &PluginContext<'a>) -> ProviderResult<Section> {
/// #         // Generate a section for the prompt
/// #         let section = Section::new("Remember previous context: ...");
/// #         Ok(section)
/// #     }
/// #
/// #     async fn process_response<'a>(
/// #         &self,
/// #         _context: &PluginContext<'a>,
/// #         _response: &LLMResponse,
/// #     ) -> ProviderResult<()> {
/// #         // Process the LLM response
/// #         // For example, update memory with new information
/// #         Ok(())
/// #     }
/// # }
/// ```
#[async_trait]
#[mockall::automock]
pub trait ProviderPlugin: Send + Sync {
    /// Returns the plugin's priority.
    ///
    /// The priority determines the order in which plugins are executed.
    /// Higher priority plugins run before lower priority plugins.
    /// This is important for plugins that depend on each other's output.
    ///
    /// # Returns
    ///
    /// An integer representing the plugin's priority (higher values = higher priority)
    fn priority(&self) -> i32;

    /// Returns the capability provided by this plugin.
    ///
    /// Each plugin provides a specific capability to the provider.
    /// This method allows the system to match plugins with providers
    /// that require specific capabilities.
    ///
    /// # Returns
    ///
    /// A `CapabilityType` representing the plugin's primary capability
    fn capability(&self) -> CapabilityType;

    /// Generates a section for the prompt.
    ///
    /// This method is called during prompt generation to allow the plugin
    /// to contribute content to the prompt. For example, a memory plugin
    /// might add relevant context from previous interactions.
    ///
    /// # Parameters
    ///
    /// * `context` - The plugin context containing request information
    ///
    /// # Returns
    ///
    /// A `ProviderResult` containing either a `Section` or an error
    async fn generate_section<'a>(&self, context: &PluginContext<'a>) -> ProviderResult<Section>;

    /// Processes the LLM response.
    ///
    /// This method is called after the LLM has generated a response,
    /// allowing the plugin to process the response. For example, a memory
    /// plugin might store information from the response for future use.
    ///
    /// # Parameters
    ///
    /// * `context` - The plugin context containing request information
    /// * `response` - The LLM response to process
    ///
    /// # Returns
    ///
    /// A `ProviderResult` indicating success or failure
    async fn process_response<'a>(
        &self,
        context: &PluginContext<'a>,
        response: &LLMResponse,
    ) -> ProviderResult<()>;
}

/// # Plugin Context
///
/// The `PluginContext` struct provides plugins with access to the request
/// context, including the request itself, plugin configurations, and the
/// provider context.
///
/// ## Fields
///
/// * `request` - The provider request being processed
/// * `configs` - Plugin-specific configurations
/// * `context` - The provider execution context
#[derive(Clone, Serialize)]
pub struct PluginContext<'a> {
    pub request: &'a ProviderRequest,
    pub configs: &'a HashMap<String, PluginConfig>,
    pub context: &'a ProviderContext,
}

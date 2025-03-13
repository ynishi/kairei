//! Adapter for SharedMemoryCapability to ProviderPlugin
//!
//! This adapter allows SharedMemoryCapability implementations to be used
//! as ProviderPlugin instances without trait upcasting issues.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::provider::{
    capabilities::shared_memory::{Metadata, SharedMemoryCapability, SharedMemoryError},
    capability::CapabilityType,
    llm::LLMResponse,
    plugin::{PluginContext, ProviderPlugin},
    provider::Section,
    types::ProviderResult,
};

/// Adapter that wraps a SharedMemoryCapability and implements ProviderPlugin
pub struct SharedMemoryPluginAdapter {
    plugin: Arc<dyn SharedMemoryCapability>,
}

impl SharedMemoryPluginAdapter {
    /// Create a new adapter for a SharedMemoryCapability
    pub fn new(plugin: Arc<dyn SharedMemoryCapability>) -> Self {
        Self { plugin }
    }
}

#[async_trait]
impl ProviderPlugin for SharedMemoryPluginAdapter {
    fn priority(&self) -> i32 {
        // Delegate to the wrapped plugin
        self.plugin.priority()
    }

    fn capability(&self) -> CapabilityType {
        // Shared memory capability
        CapabilityType::SharedMemory
    }

    async fn generate_section<'a>(&self, context: &PluginContext<'a>) -> ProviderResult<Section> {
        // Delegate to the wrapped plugin
        self.plugin.generate_section(context).await
    }

    async fn process_response<'a>(
        &self,
        context: &PluginContext<'a>,
        response: &LLMResponse,
    ) -> ProviderResult<()> {
        // Delegate to the wrapped plugin
        self.plugin.process_response(context, response).await
    }
}

// Forward SharedMemoryCapability methods to the wrapped plugin
#[async_trait]
impl SharedMemoryCapability for SharedMemoryPluginAdapter {
    async fn get(&self, key: &str) -> Result<Value, SharedMemoryError> {
        self.plugin.get(key).await
    }

    async fn set(&self, key: &str, value: Value) -> Result<(), SharedMemoryError> {
        self.plugin.set(key, value).await
    }

    async fn delete(&self, key: &str) -> Result<(), SharedMemoryError> {
        self.plugin.delete(key).await
    }

    async fn exists(&self, key: &str) -> Result<bool, SharedMemoryError> {
        self.plugin.exists(key).await
    }

    async fn get_metadata(&self, key: &str) -> Result<Metadata, SharedMemoryError> {
        self.plugin.get_metadata(key).await
    }

    async fn list_keys(&self, pattern: &str) -> Result<Vec<String>, SharedMemoryError> {
        self.plugin.list_keys(pattern).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::config::plugins::SharedMemoryConfig;
    use crate::provider::plugins::shared_memory::InMemorySharedMemoryPlugin;

    #[tokio::test]
    async fn test_adapter_forwards_calls() {
        // Create a real plugin
        let config = SharedMemoryConfig::default();
        let plugin = Arc::new(InMemorySharedMemoryPlugin::new(config));

        // Create the adapter
        let adapter = SharedMemoryPluginAdapter::new(plugin.clone());

        // Test that the adapter forwards capability calls
        let test_key = "test_key";
        let test_value = serde_json::json!("test_value");

        // Set a value through the adapter
        adapter.set(test_key, test_value.clone()).await.unwrap();

        // Get the value through the adapter
        let retrieved = adapter.get(test_key).await.unwrap();
        assert_eq!(retrieved, test_value);

        // Check exists through the adapter
        assert!(adapter.exists(test_key).await.unwrap());

        // Delete through the adapter
        adapter.delete(test_key).await.unwrap();

        // Verify it's gone
        assert!(!adapter.exists(test_key).await.unwrap());
    }
}

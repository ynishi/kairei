//! Integration tests for SharedMemoryCapability
//!
//! These tests verify the integration of SharedMemoryCapability with the
//! Provider Registry and multiple providers.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;

use kairei_core::config::{PluginConfig, ProviderConfig, ProviderConfigs, SecretConfig};
use kairei_core::event_bus::EventBus;
use kairei_core::provider::capabilities::common::Capabilities;
use kairei_core::provider::config::plugins::SharedMemoryConfig;
use kairei_core::provider::provider::{Provider, ProviderSecret};
use kairei_core::provider::provider_registry::ProviderRegistry;
use kairei_core::provider::request::{ProviderContext, ProviderRequest, ProviderResponse};
use kairei_core::provider::types::ProviderResult;

/// Helper function to create a test provider registry
async fn create_test_registry() -> Arc<ProviderRegistry> {
    // Create event bus
    let event_bus = Arc::new(EventBus::new(20));

    // Create provider configs with HashMap
    let provider_configs = ProviderConfigs {
        providers: HashMap::new(),
        primary_provider: None,
    };

    // Create secret config
    let secret_config = SecretConfig::default();

    // Create registry
    let registry = ProviderRegistry::new(provider_configs, secret_config, event_bus).await;

    Arc::new(registry)
}

/// Helper function to create a provider config with shared memory
fn create_provider_config(name: &str, namespace: &str) -> ProviderConfig {
    let mut plugin_configs = std::collections::HashMap::new();

    // Add shared memory config
    let shared_memory_config = SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_secs(3600),
        namespace: namespace.to_string(),
    };

    plugin_configs.insert(
        "shared_memory".to_string(),
        PluginConfig::SharedMemory(shared_memory_config),
    );

    ProviderConfig {
        name: name.to_string(),
        provider_type: kairei_core::provider::provider::ProviderType::OpenAIChat,
        plugin_configs,
        ..Default::default()
    }
}

/// Helper function to create a test request
fn create_test_request() -> ProviderRequest {
    ProviderRequest::default()
}

/// Mock provider for testing
#[derive(Clone)]
struct MockProvider {
    name: String,
}

impl MockProvider {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn execute(
        &self,
        _context: &ProviderContext,
        _request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse> {
        Ok(ProviderResponse::default())
    }

    async fn capabilities(&self) -> Capabilities {
        Capabilities::default()
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

    async fn shutdown(&self) -> ProviderResult<()> {
        Ok(())
    }

    async fn health_check(&self) -> ProviderResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_provider_registry_integration() -> ProviderResult<()> {
    // Setup registry
    let registry = create_test_registry().await;

    // Create provider config with shared memory
    let provider_config = create_provider_config("test_provider", "test_namespace");

    // Create a mock provider
    let provider = Arc::new(MockProvider::new("test_provider"));
    let secret = ProviderSecret::default();

    // Register provider
    registry
        .register_provider_with("test_provider", &provider_config, &secret, provider)
        .await?;

    // Create shared memory config
    let shared_memory_config = SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_secs(3600),
        namespace: "test_namespace".to_string(),
    };

    // Get or create shared memory plugin
    let shared_memory = registry.get_or_create_shared_memory_plugin(&shared_memory_config);

    // Store value using the plugin
    shared_memory
        .set("registry_test", json!("test_value"))
        .await
        .unwrap();

    // Verify value exists
    let value = shared_memory.get("registry_test").await.unwrap();
    assert_eq!(value, json!("test_value"));

    // Verify plugin is cleaned up during shutdown
    registry.shutdown().await?;

    // After shutdown, the plugin should be removed
    let namespaces = registry.list_shared_memory_namespaces();
    assert!(
        namespaces.is_empty(),
        "Namespaces should be empty after shutdown"
    );

    Ok(())
}

#[tokio::test]
async fn test_multi_provider_sharing() -> ProviderResult<()> {
    // Setup registry
    let registry = create_test_registry().await;

    // Create two providers that share the same namespace
    let provider1_config = create_provider_config("provider1", "shared_namespace");
    let provider2_config = create_provider_config("provider2", "shared_namespace");

    // Create mock providers
    let provider1 = Arc::new(MockProvider::new("provider1"));
    let provider2 = Arc::new(MockProvider::new("provider2"));
    let secret = ProviderSecret::default();

    // Register both providers
    registry
        .register_provider_with("provider1", &provider1_config, &secret, provider1)
        .await?;
    registry
        .register_provider_with("provider2", &provider2_config, &secret, provider2)
        .await?;

    // Create shared memory config
    let shared_memory_config = SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_secs(3600),
        namespace: "shared_namespace".to_string(),
    };

    // Get or create shared memory plugin
    let shared_memory = registry.get_or_create_shared_memory_plugin(&shared_memory_config);

    // Provider 1 stores data
    shared_memory
        .set("shared_key", json!({"source": "provider1"}))
        .await
        .unwrap();

    // Get providers
    let provider1 = registry.get_provider("provider1").await?;
    let provider2 = registry.get_provider("provider2").await?;

    // Create test context and request
    let context = ProviderContext::default();
    let request = create_test_request();

    // Execute provider1
    let _ = provider1.provider.execute(&context, &request).await?;

    // Execute provider2 - should be able to access the shared data
    let _ = provider2.provider.execute(&context, &request).await?;

    // Verify data is still accessible
    let value = shared_memory.get("shared_key").await.unwrap();
    assert_eq!(value, json!({"source": "provider1"}));

    // Create shared memory config
    let shared_memory_config = SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_secs(3600),
        namespace: "shared_namespace".to_string(),
    };

    // Get the same plugin twice
    let plugin1 = registry.get_or_create_shared_memory_plugin(&shared_memory_config);
    let plugin2 = registry.get_or_create_shared_memory_plugin(&shared_memory_config);

    // They should be the same instance (Arc points to the same object)
    assert!(
        Arc::ptr_eq(&plugin1, &plugin2),
        "Both providers should share the same plugin instance"
    );

    Ok(())
}

#[tokio::test]
async fn test_namespace_isolation() -> ProviderResult<()> {
    // Setup registry
    let registry = create_test_registry().await;

    // Create two providers with different namespaces
    let provider1_config = create_provider_config("provider1", "namespace1");
    let provider2_config = create_provider_config("provider2", "namespace2");

    // Create mock providers
    let provider1 = Arc::new(MockProvider::new("provider1"));
    let provider2 = Arc::new(MockProvider::new("provider2"));
    let secret = ProviderSecret::default();

    // Register both providers
    registry
        .register_provider_with("provider1", &provider1_config, &secret, provider1)
        .await?;
    registry
        .register_provider_with("provider2", &provider2_config, &secret, provider2)
        .await?;

    // Create shared memory configs with different namespaces
    let shared_memory_config1 = SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_secs(3600),
        namespace: "namespace1".to_string(),
    };

    let shared_memory_config2 = SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_secs(3600),
        namespace: "namespace2".to_string(),
    };

    // Get shared memory plugins from registry
    let shared_memory1 = registry.get_or_create_shared_memory_plugin(&shared_memory_config1);
    let shared_memory2 = registry.get_or_create_shared_memory_plugin(&shared_memory_config2);

    // Provider 1 stores data
    shared_memory1
        .set("isolation_key", json!("provider1_data"))
        .await
        .unwrap();

    // Provider 2 stores data with the same key
    shared_memory2
        .set("isolation_key", json!("provider2_data"))
        .await
        .unwrap();

    // Verify data isolation
    let value1 = shared_memory1.get("isolation_key").await.unwrap();
    let value2 = shared_memory2.get("isolation_key").await.unwrap();

    assert_eq!(value1, json!("provider1_data"));
    assert_eq!(value2, json!("provider2_data"));

    // Verify they are different plugin instances
    assert!(
        !Arc::ptr_eq(&shared_memory1, &shared_memory2),
        "Plugins with different namespaces should be different instances"
    );

    Ok(())
}

#[tokio::test]
async fn test_plugin_registration_and_cleanup() -> ProviderResult<()> {
    // Setup registry
    let registry = create_test_registry().await;

    // Create multiple providers with different namespaces
    for i in 1..=5 {
        let namespace = format!("namespace{}", i);
        let provider_config = create_provider_config(&format!("provider{}", i), &namespace);
        let provider = Arc::new(MockProvider::new(&format!("provider{}", i)));
        let secret = ProviderSecret::default();

        registry
            .register_provider_with(
                &format!("provider{}", i),
                &provider_config,
                &secret,
                provider,
            )
            .await?;

        // Create shared memory config
        let shared_memory_config = SharedMemoryConfig {
            base: Default::default(),
            max_keys: 100,
            ttl: Duration::from_secs(3600),
            namespace: namespace.clone(),
        };

        // Create the plugin
        registry.get_or_create_shared_memory_plugin(&shared_memory_config);
    }

    // Verify all namespaces are registered
    let namespaces = registry.list_shared_memory_namespaces();
    assert_eq!(namespaces.len(), 5, "Should have 5 registered namespaces");

    // Verify each namespace has a plugin
    for i in 1..=5 {
        let namespace = format!("namespace{}", i);
        let shared_memory_config = SharedMemoryConfig {
            base: Default::default(),
            max_keys: 100,
            ttl: Duration::from_secs(3600),
            namespace: namespace.clone(),
        };

        let plugin = registry.get_or_create_shared_memory_plugin(&shared_memory_config);
        // Just verify the plugin exists - we can't easily create a dummy plugin for comparison
        assert!(
            plugin.exists("test_key").await.is_ok(),
            "Plugin for {} should exist",
            namespace
        );
    }

    // Shutdown registry
    registry.shutdown().await?;

    // Verify all plugins are cleaned up
    let namespaces = registry.list_shared_memory_namespaces();
    assert!(
        namespaces.is_empty(),
        "All namespaces should be cleaned up after shutdown"
    );

    Ok(())
}

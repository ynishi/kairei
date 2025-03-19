//! Integration tests for SistenceProvider
//!
//! These tests verify the functionality of SistenceProvider using SimpleExpertLLM
//! to test without requiring API access.

use std::collections::HashMap;
use std::sync::Arc;

use kairei_core::config::ProviderConfig;
use kairei_core::provider::capabilities::shared_memory::{
    SharedMemoryCapability, SharedMemoryError,
};
use kairei_core::provider::capabilities::will_action::{
    WillAction, WillActionContext, WillActionError, WillActionParams, WillActionResolver,
    WillActionResult, WillActionSignature,
};
use kairei_core::provider::capability::{Capabilities, CapabilityType};
use kairei_core::provider::llm::{LLMResponse, ProviderLLM};
use kairei_core::provider::llms::simple_expert::SimpleExpertProviderLLM;
use kairei_core::provider::plugin::{PluginContext, ProviderPlugin};
use kairei_core::provider::provider::{Provider, ProviderSecret, ProviderType, Section};
use kairei_core::provider::providers::sistence::{SistenceAgentContext, SistenceProvider};
use kairei_core::provider::request::{
    ProviderContext, ProviderRequest, ProviderResponse, RequestInput,
};
use kairei_core::provider::types::ProviderResult;
use kairei_core::timestamp::Timestamp;

use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::{Value, json};

/// Mock Shared Memory implementation for testing
struct MockSharedMemory {
    storage: DashMap<String, Value>,
}

impl MockSharedMemory {
    fn new() -> Self {
        Self {
            storage: DashMap::new(),
        }
    }
}

#[async_trait]
impl ProviderPlugin for MockSharedMemory {
    fn priority(&self) -> i32 {
        10
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::SharedMemory
    }

    async fn generate_section<'a>(&self, _context: &PluginContext<'a>) -> ProviderResult<Section> {
        Ok(Section::new(""))
    }

    async fn process_response<'a>(
        &self,
        _context: &PluginContext<'a>,
        _response: &LLMResponse,
    ) -> ProviderResult<()> {
        Ok(())
    }
}

#[async_trait]
impl SharedMemoryCapability for MockSharedMemory {
    async fn get(&self, key: &str) -> Result<Value, SharedMemoryError> {
        match self.storage.get(key) {
            Some(value) => Ok(value.clone()),
            None => Err(SharedMemoryError::KeyNotFound(key.to_string())),
        }
    }

    async fn set(&self, key: &str, value: Value) -> Result<(), SharedMemoryError> {
        self.storage.insert(key.to_string(), value);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), SharedMemoryError> {
        if self.storage.remove(key).is_some() {
            Ok(())
        } else {
            Err(SharedMemoryError::KeyNotFound(key.to_string()))
        }
    }

    async fn exists(&self, key: &str) -> Result<bool, SharedMemoryError> {
        Ok(self.storage.contains_key(key))
    }

    async fn get_metadata(
        &self,
        key: &str,
    ) -> Result<kairei_core::provider::capabilities::shared_memory::Metadata, SharedMemoryError>
    {
        if self.storage.contains_key(key) {
            Ok(kairei_core::provider::capabilities::shared_memory::Metadata::default())
        } else {
            Err(SharedMemoryError::KeyNotFound(key.to_string()))
        }
    }

    async fn list_keys(&self, _pattern: &str) -> Result<Vec<String>, SharedMemoryError> {
        Ok(self
            .storage
            .iter()
            .map(|entry| entry.key().clone())
            .collect())
    }
}

/// Mock Will Action implementation for testing
struct MockWillAction {
    name: String,
    execute_result: Option<WillActionResult>,
    execute_error: Option<WillActionError>,
}

impl MockWillAction {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            execute_result: None,
            execute_error: None,
        }
    }

    fn with_result(mut self, result: WillActionResult) -> Self {
        self.execute_result = Some(result);
        self
    }

    fn with_error(mut self, error: WillActionError) -> Self {
        self.execute_error = Some(error);
        self
    }
}

#[async_trait]
impl WillAction for MockWillAction {
    async fn execute(
        &self,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> WillActionResult {
        if let Some(error) = &self.execute_error {
            return WillActionResult::error(error.clone());
        }

        if let Some(result) = &self.execute_result {
            return result.clone();
        }

        // Default implementation
        WillActionResult::success(json!({
            "action": self.name,
            "params": params.named,
            "agent_id": context.agent_id,
            "timestamp": Timestamp::now().to_string(),
        }))
    }

    fn get_signature(&self) -> WillActionSignature {
        WillActionSignature {
            name: self.name.clone(),
            description: format!("Mock action for {}", self.name),
            parameters: vec![],
            return_type: "Object".to_string(),
            required_permissions: vec![self.name.clone()],
        }
    }
}

/// Mock Will Action Resolver for testing
struct MockWillActionResolver {
    actions: DashMap<String, Box<dyn WillAction + Send + Sync>>,
}

impl MockWillActionResolver {
    fn new() -> Self {
        Self {
            actions: DashMap::new(),
        }
    }

    fn with_action(self, name: &str, action: Box<dyn WillAction + Send + Sync>) -> Self {
        self.actions.insert(name.to_string(), action);
        self
    }
}

#[async_trait]
impl ProviderPlugin for MockWillActionResolver {
    fn priority(&self) -> i32 {
        10
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::Custom("will_action".to_string())
    }

    async fn generate_section<'a>(&self, _context: &PluginContext<'a>) -> ProviderResult<Section> {
        Ok(Section::new(""))
    }

    async fn process_response<'a>(
        &self,
        _context: &PluginContext<'a>,
        _response: &LLMResponse,
    ) -> ProviderResult<()> {
        Ok(())
    }
}

#[async_trait]
impl WillActionResolver for MockWillActionResolver {
    fn resolve(&self, action_name: &str) -> Option<Box<dyn WillAction>> {
        self.actions.get(action_name).map(|_action| {
            let action_clone: Box<dyn WillAction> = Box::new(MockWillAction::new(action_name));
            action_clone
        })
    }

    fn register(
        &mut self,
        action_name: &str,
        _action: Box<dyn WillAction>,
    ) -> Result<(), WillActionError> {
        // Convert to Box<dyn WillAction + Send + Sync> for internal storage
        let action_sync: Box<dyn WillAction + Send + Sync> =
            Box::new(MockWillAction::new(action_name));
        self.actions.insert(action_name.to_string(), action_sync);
        Ok(())
    }

    async fn execute(
        &self,
        action_name: &str,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> Result<WillActionResult, WillActionError> {
        if let Some(action) = self.actions.get(action_name) {
            Ok(action.execute(params, context).await)
        } else {
            Err(WillActionError::ActionNotFound(action_name.to_string()))
        }
    }

    fn list_actions(&self) -> Vec<String> {
        self.actions
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    fn get_action_signature(&self, action_name: &str) -> Option<WillActionSignature> {
        self.actions
            .get(action_name)
            .map(|action| action.get_signature())
    }
}

/// Wrapper for SimpleExpertProviderLLM that implements Provider
struct SimpleExpertProviderWrapper {
    llm: SimpleExpertProviderLLM,
}

impl SimpleExpertProviderWrapper {
    fn new(name: &str) -> Self {
        Self {
            llm: SimpleExpertProviderLLM::new(name),
        }
    }
}

#[async_trait]
impl Provider for SimpleExpertProviderWrapper {
    async fn execute(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse> {
        // Convert request to prompt and use the LLM
        let prompt = request.input.query.to_string();
        let llm_response = self.llm.send_message(&prompt, &context.config).await?;

        Ok(ProviderResponse {
            output: llm_response.content,
            metadata: kairei_core::provider::request::ResponseMetadata {
                timestamp: kairei_core::timestamp::Timestamp::now(),
            },
        })
    }

    async fn capabilities(&self) -> Capabilities {
        Capabilities::from(vec![CapabilityType::Generate])
    }

    fn name(&self) -> &str {
        &self.llm.name()
    }

    async fn initialize(
        &mut self,
        config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        self.llm.initialize(config, secret).await
    }

    async fn shutdown(&self) -> ProviderResult<()> {
        Ok(())
    }
}

/// Helper function to create a SimpleExpertProviderLLM with predefined responses
fn create_simple_expert_llm() -> Arc<SimpleExpertProviderWrapper> {
    Arc::new(SimpleExpertProviderWrapper::new("test_simple_expert"))
}

/// Helper function to create a test provider config
fn create_provider_config() -> ProviderConfig {
    let mut provider_specific = HashMap::new();
    provider_specific.insert(
        "notify".to_string(),
        json!("Notification sent successfully"),
    );
    provider_specific.insert("suggest".to_string(), json!("Here's a suggestion for you"));
    provider_specific.insert(
        "will_action".to_string(),
        json!("Will action executed successfully"),
    );

    ProviderConfig {
        name: "test_sistence".to_string(),
        provider_type: ProviderType::SimpleExpert,
        provider_specific,
        plugin_configs: HashMap::new(),
        ..Default::default()
    }
}

/// Helper function to create a test request
fn create_test_request(
    query: &str,
    parameters: HashMap<String, kairei_core::eval::expression::Value>,
) -> ProviderRequest {
    ProviderRequest {
        input: RequestInput {
            query: kairei_core::eval::expression::Value::String(query.to_string()),
            parameters,
        },
        state: Default::default(),
        config: Default::default(),
    }
}

/// Helper function to create a will action request
fn create_will_action_request(action: &str) -> ProviderRequest {
    let mut parameters = HashMap::new();
    parameters.insert(
        "will_action".to_string(),
        kairei_core::eval::expression::Value::String("true".to_string()),
    );
    parameters.insert(
        "message".to_string(),
        kairei_core::eval::expression::Value::String("Test message".to_string()),
    );

    create_test_request(action, parameters)
}

#[tokio::test]
async fn test_sistence_provider_initialization() {
    // Create components
    let llm_provider = create_simple_expert_llm();
    let shared_memory = Arc::new(MockSharedMemory::new());
    let will_action_resolver = Arc::new(MockWillActionResolver::new());

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        llm_provider,
        shared_memory,
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Test provider name
    assert_eq!(provider.name(), "test_sistence");

    // Test capabilities
    let capabilities = provider.capabilities().await;
    assert!(capabilities.supports(&CapabilityType::Custom("chat".to_string())));
    assert!(capabilities.supports(&CapabilityType::Custom("sistence".to_string())));
    assert!(capabilities.supports(&CapabilityType::Custom("will_action".to_string())));
}

#[tokio::test]
async fn test_sistence_provider_regular_request() {
    // Create components
    let llm_provider = create_simple_expert_llm();
    let shared_memory = Arc::new(MockSharedMemory::new());
    let will_action_resolver = Arc::new(MockWillActionResolver::new());

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        llm_provider,
        shared_memory,
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create context and request
    let context = ProviderContext {
        config: create_provider_config(),
        secret: ProviderSecret::default(),
    };

    let request = create_test_request("Hello world", HashMap::new());

    // Execute request
    // For SimpleExpertProviderLLM, we need to set up a response in the config
    let mut config = create_provider_config();
    config.provider_specific.insert(
        "Hello world".to_string(),
        json!("This is a test response"),
    );
    
    let context_with_config = ProviderContext {
        config,
        secret: ProviderSecret::default(),
    };
    
    let response = provider.execute(&context_with_config, &request).await.unwrap();

    // For regular requests, the provider should delegate to the underlying LLM
    assert!(!response.output.is_empty());
}

#[tokio::test]
async fn test_sistence_provider_will_action_request() {
    // Create components
    let llm_provider = create_simple_expert_llm();
    let shared_memory = Arc::new(MockSharedMemory::new());
    let will_action_resolver = Arc::new(MockWillActionResolver::new().with_action(
        "notify",
        Box::new(
            MockWillAction::new("notify").with_result(WillActionResult::success(json!({
                "message": "Notification sent",
                "status": "success"
            }))),
        ),
    ));

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        llm_provider,
        shared_memory.clone(),
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create context and request
    let context = ProviderContext {
        config: create_provider_config(),
        secret: ProviderSecret::default(),
    };

    let request = create_will_action_request("notify");

    // Execute request
    let response = provider.execute(&context, &request).await.unwrap();

    // Verify response
    let result: Value = serde_json::from_str(&response.output).unwrap();
    assert_eq!(result["success"], json!(true));
    assert_eq!(result["data"]["message"], json!("Notification sent"));

    // Verify agent context was stored in shared memory
    let keys = shared_memory.list_keys("*").await.unwrap();
    assert!(!keys.is_empty());

    // Get the agent context
    let agent_key = keys.first().unwrap();
    let agent_context_json = shared_memory.get(agent_key).await.unwrap();

    // Verify it's a valid SistenceAgentContext
    let agent_context: SistenceAgentContext = serde_json::from_value(agent_context_json).unwrap();
    assert!(!agent_context.agent_id.is_empty());
    assert!(!agent_context.interaction_history.is_empty());
}

#[tokio::test]
async fn test_sistence_provider_will_action_llm_fallback() {
    // Create components
    let llm_provider = create_simple_expert_llm();
    let shared_memory = Arc::new(MockSharedMemory::new());

    // Create resolver that will fail to resolve the action
    let will_action_resolver = Arc::new(MockWillActionResolver::new());

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        llm_provider,
        shared_memory.clone(),
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create context and request
    let context = ProviderContext {
        config: create_provider_config(),
        secret: ProviderSecret::default(),
    };

    let request = create_will_action_request("suggest");

    // Execute request
    let response = provider.execute(&context, &request).await.unwrap();

    // Verify response
    let result: Value = serde_json::from_str(&response.output).unwrap();
    assert_eq!(result["success"], json!(true));

    // Verify agent context was stored in shared memory
    let keys = shared_memory.list_keys("*").await.unwrap();
    assert!(!keys.is_empty());
}

#[tokio::test]
async fn test_sistence_provider_agent_context_persistence() {
    // Create components
    let llm_provider = create_simple_expert_llm();
    let shared_memory = Arc::new(MockSharedMemory::new());
    let will_action_resolver = Arc::new(
        MockWillActionResolver::new()
            .with_action("notify", Box::new(MockWillAction::new("notify"))),
    );

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        llm_provider,
        shared_memory.clone(),
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create context and request with proper config for SimpleExpertProviderLLM
    let mut config = create_provider_config();
    config.provider_specific.insert(
        "notify".to_string(),
        json!("{\"success\":true,\"data\":{\"message\":\"Notification sent\"}}"),
    );
    
    let context = ProviderContext {
        config,
        secret: ProviderSecret::default(),
    };

    // Execute first request
    let request1 = create_will_action_request("notify");
    let _ = provider.execute(&context, &request1).await.unwrap();

    // Execute second request
    let request2 = create_will_action_request("notify");
    let _ = provider.execute(&context, &request2).await.unwrap();

    // Get the agent context
    let keys = shared_memory.list_keys("*").await.unwrap();
    let agent_key = keys.first().unwrap();
    let agent_context_json = shared_memory.get(agent_key).await.unwrap();

    // Verify it's a valid SistenceAgentContext with multiple interactions
    let agent_context: SistenceAgentContext = serde_json::from_value(agent_context_json).unwrap();
    assert!(agent_context.interaction_history.len() >= 2);
}

#[tokio::test]
async fn test_sistence_provider_error_handling() {
    // Create components
    let llm_provider = create_simple_expert_llm();
    let shared_memory = Arc::new(MockSharedMemory::new());

    // Create resolver that will return an error
    let will_action_resolver = Arc::new(
        MockWillActionResolver::new().with_action(
            "notify",
            Box::new(
                MockWillAction::new("notify")
                    .with_error(WillActionError::ExecutionError("Test error".to_string())),
            ),
        ),
    );

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        llm_provider,
        shared_memory,
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create context and request
    let context = ProviderContext {
        config: create_provider_config(),
        secret: ProviderSecret::default(),
    };

    let request = create_will_action_request("notify");

    // Execute request - should fall back to LLM
    // For SimpleExpertProviderLLM, we need to set up a response in the config
    let mut config = create_provider_config();
    config.provider_specific.insert(
        "notify".to_string(),
        json!("{\"success\":true,\"data\":{\"message\":\"Handled by LLM fallback\"}}"),
    );
    
    let context_with_config = ProviderContext {
        config,
        secret: ProviderSecret::default(),
    };
    
    let response = provider.execute(&context_with_config, &request).await.unwrap();

    // Verify response
    let result: Value = serde_json::from_str(&response.output).unwrap();
    assert_eq!(result["success"], json!(true));
}

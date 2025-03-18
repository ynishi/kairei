//! Tests for the SistenceProvider implementation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::provider::capabilities::shared_memory::{SharedMemory, SharedMemoryError};
use crate::provider::capabilities::will_action::{
    ParameterSpec, WillAction, WillActionContext, WillActionError, WillActionParams,
    WillActionResolver, WillActionResult, WillActionSignature,
};
use crate::provider::capability::{Capabilities, CapabilityType};
use crate::provider::plugin::{PluginContext, ProviderPlugin};
use crate::provider::provider::{Provider, ProviderError, ProviderResult, ProviderSecret, Section};
use crate::provider::providers::sistence::{SistenceAgentContext, SistenceProvider};
use crate::provider::request::{ProviderContext, ProviderRequest, ProviderResponse};

// Mock implementations for testing

/// Mock Provider for testing SistenceProvider
#[derive(Clone)]
struct MockProvider {
    name: String,
    responses: Arc<Mutex<Vec<ProviderResponse>>>,
    requests: Arc<Mutex<Vec<ProviderRequest>>>,
}

impl MockProvider {
    fn new(name: &str, responses: Vec<ProviderResponse>) -> Self {
        Self {
            name: name.to_string(),
            responses: Arc::new(Mutex::new(responses)),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_requests(&self) -> Vec<ProviderRequest> {
        self.requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn execute(
        &self,
        _context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse> {
        // Store the request for later inspection
        self.requests.lock().unwrap().push(request.clone());

        // Return the next response or an error if none are available
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Err(ProviderError::ExecutionError("No mock responses available".to_string()))
        } else {
            Ok(responses.remove(0))
        }
    }

    async fn capabilities(&self) -> Capabilities {
        Capabilities::new(vec![
            CapabilityType::Chat,
            CapabilityType::Custom("will_action".to_string()),
        ])
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn initialize(
        &mut self,
        _config: &crate::provider::provider::ProviderConfig,
        _secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        Ok(())
    }

    async fn shutdown(&self) -> ProviderResult<()> {
        Ok(())
    }
}

/// Mock SharedMemory implementation for testing
struct MockSharedMemory {
    data: Arc<RwLock<HashMap<String, String>>>,
    get_error: Option<SharedMemoryError>,
    set_error: Option<SharedMemoryError>,
}

impl MockSharedMemory {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            get_error: None,
            set_error: None,
        }
    }

    fn with_get_error(mut self, error: SharedMemoryError) -> Self {
        self.get_error = Some(error);
        self
    }

    fn with_set_error(mut self, error: SharedMemoryError) -> Self {
        self.set_error = Some(error);
        self
    }

    fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.write().unwrap().insert(key.to_string(), value.to_string());
        self
    }
}

#[async_trait]
impl SharedMemory for MockSharedMemory {
    async fn get(&self, key: &str) -> Result<Option<String>, SharedMemoryError> {
        if let Some(error) = &self.get_error {
            return Err(error.clone());
        }
        
        let data = self.data.read().unwrap();
        Ok(data.get(key).cloned())
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), SharedMemoryError> {
        if let Some(error) = &self.set_error {
            return Err(error.clone());
        }
        
        let mut data = self.data.write().unwrap();
        data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, SharedMemoryError> {
        let mut data = self.data.write().unwrap();
        Ok(data.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool, SharedMemoryError> {
        let data = self.data.read().unwrap();
        Ok(data.contains_key(key))
    }
}

#[async_trait]
impl ProviderPlugin for MockSharedMemory {
    fn priority(&self) -> i32 {
        10
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::Custom("shared_memory".to_string())
    }

    async fn generate_section<'a>(&self, _context: &PluginContext<'a>) -> ProviderResult<Section> {
        Ok(Section::new(""))
    }

    async fn process_response<'a>(
        &self,
        _context: &PluginContext<'a>,
        _response: &crate::provider::llm::LLMResponse,
    ) -> ProviderResult<()> {
        Ok(())
    }
}

/// Mock NotifyAction for testing
#[derive(Debug, Clone)]
struct MockNotifyAction {
    execution_result: WillActionResult,
}

impl MockNotifyAction {
    fn new(success: bool) -> Self {
        let result = if success {
            WillActionResult::success(json!({
                "message": "Test notification",
                "priority": "high",
                "timestamp": Utc::now().to_rfc3339(),
            }))
        } else {
            WillActionResult::error(WillActionError::ExecutionError(
                "Failed to send notification".to_string(),
            ))
        };
        
        Self {
            execution_result: result,
        }
    }
}

#[async_trait]
impl WillAction for MockNotifyAction {
    async fn execute(
        &self,
        _params: WillActionParams,
        _context: &WillActionContext,
    ) -> WillActionResult {
        self.execution_result.clone()
    }

    fn get_signature(&self) -> WillActionSignature {
        WillActionSignature {
            name: "notify".to_string(),
            description: "Send a notification to users".to_string(),
            parameters: vec![
                ParameterSpec {
                    name: "message".to_string(),
                    param_type: "String".to_string(),
                    required: true,
                    description: "The notification message".to_string(),
                    default_value: None,
                },
            ],
            return_type: "NotificationResult".to_string(),
            required_permissions: vec!["notify".to_string()],
        }
    }
}

/// Mock WillActionResolver for testing
struct MockWillActionResolver {
    actions: HashMap<String, Box<dyn WillAction>>,
    resolve_error: Option<WillActionError>,
    execute_error: Option<WillActionError>,
}

impl MockWillActionResolver {
    fn new() -> Self {
        Self {
            actions: HashMap::new(),
            resolve_error: None,
            execute_error: None,
        }
    }

    fn with_action(mut self, name: &str, action: Box<dyn WillAction>) -> Self {
        self.actions.insert(name.to_string(), action);
        self
    }

    fn with_resolve_error(mut self, error: WillActionError) -> Self {
        self.resolve_error = Some(error);
        self
    }

    fn with_execute_error(mut self, error: WillActionError) -> Self {
        self.execute_error = Some(error);
        self
    }
}

#[async_trait]
impl WillActionResolver for MockWillActionResolver {
    fn resolve(&self, action_name: &str) -> Option<Box<dyn WillAction>> {
        if self.resolve_error.is_some() {
            return None;
        }
        
        self.actions.get(action_name).map(|action| {
            // Create a new instance since we can't clone Box<dyn WillAction>
            if action_name == "notify" {
                Box::new(MockNotifyAction::new(true)) as Box<dyn WillAction>
            } else {
                Box::new(MockNotifyAction::new(true)) as Box<dyn WillAction>
            }
        })
    }

    fn register(
        &mut self,
        action_name: &str,
        action: Box<dyn WillAction>,
    ) -> Result<(), WillActionError> {
        self.actions.insert(action_name.to_string(), action);
        Ok(())
    }

    async fn execute(
        &self,
        action_name: &str,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> Result<WillActionResult, WillActionError> {
        if let Some(error) = &self.execute_error {
            return Err(error.clone());
        }
        
        let action = self.resolve(action_name)
            .ok_or_else(|| WillActionError::ActionNotFound(action_name.to_string()))?;
            
        Ok(action.execute(params, context).await)
    }

    fn list_actions(&self) -> Vec<String> {
        self.actions.keys().cloned().collect()
    }

    fn get_action_signature(&self, action_name: &str) -> Option<WillActionSignature> {
        self.resolve(action_name).map(|action| action.get_signature())
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
        _response: &crate::provider::llm::LLMResponse,
    ) -> ProviderResult<()> {
        Ok(())
    }
}

// Tests for SistenceProvider

#[tokio::test]
async fn test_sistence_provider_initialization() {
    // Create mock components
    let base_provider = Arc::new(MockProvider::new(
        "mock_base",
        vec![ProviderResponse::new("Test response")],
    ));
    let shared_memory = Arc::new(MockSharedMemory::new());
    let will_action_resolver = Arc::new(
        MockWillActionResolver::new()
            .with_action("notify", Box::new(MockNotifyAction::new(true))),
    );

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        base_provider.clone(),
        shared_memory,
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Verify provider is created successfully
    assert_eq!(provider.name(), "test_sistence");
}

#[tokio::test]
async fn test_regular_request_delegation() {
    // Create mock components with expected response
    let expected_response = ProviderResponse::new("Delegated response");
    let base_provider = Arc::new(MockProvider::new(
        "mock_base",
        vec![expected_response.clone()],
    ));
    let shared_memory = Arc::new(MockSharedMemory::new());
    let will_action_resolver = Arc::new(MockWillActionResolver::new());

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        base_provider.clone(),
        shared_memory,
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create a regular request (non-will action)
    let context = ProviderContext::default();
    let request = ProviderRequest::new("Tell me about Rust programming");

    // Execute the request
    let response = provider.execute(&context, &request).await.unwrap();

    // Verify the response matches the expected response from the base provider
    assert_eq!(response.content, expected_response.content);

    // Verify the request was delegated to the base provider
    let requests = base_provider.get_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].content, request.content);
}

#[tokio::test]
async fn test_will_action_request_processing() {
    // Create mock components
    let base_provider = Arc::new(MockProvider::new(
        "mock_base",
        vec![ProviderResponse::new("Will action response")],
    ));
    let shared_memory = Arc::new(MockSharedMemory::new());
    let will_action_resolver = Arc::new(
        MockWillActionResolver::new()
            .with_action("notify", Box::new(MockNotifyAction::new(true))),
    );

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        base_provider.clone(),
        shared_memory.clone(),
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create a will action request
    let mut context = ProviderContext::default();
    context.agent_name = Some("test_agent".to_string());
    let request = ProviderRequest::new("I want to notify the user about something important");

    // Execute the request
    let response = provider.execute(&context, &request).await.unwrap();

    // Verify the response contains will action information
    assert!(response.content.contains("notification"));
}

#[tokio::test]
async fn test_agent_context_management() {
    // Create mock components
    let base_provider = Arc::new(MockProvider::new(
        "mock_base",
        vec![
            ProviderResponse::new("First response"),
            ProviderResponse::new("Second response"),
        ],
    ));
    let shared_memory = Arc::new(MockSharedMemory::new());
    let will_action_resolver = Arc::new(MockWillActionResolver::new());

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        base_provider.clone(),
        shared_memory.clone(),
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create a context with agent name
    let mut context = ProviderContext::default();
    context.agent_name = Some("test_agent".to_string());

    // First request
    let request1 = ProviderRequest::new("First message");
    let _ = provider.execute(&context, &request1).await.unwrap();

    // Second request with the same agent
    let request2 = ProviderRequest::new("Second message");
    let _ = provider.execute(&context, &request2).await.unwrap();

    // Verify that the agent context was stored in shared memory
    let agent_id = format!("agent:test_agent:{}", context.user_id.unwrap_or_default());
    let context_exists = shared_memory.exists(&agent_id).await.unwrap();
    assert!(context_exists);
}

#[tokio::test]
async fn test_error_handling_shared_memory() {
    // Create mock components with shared memory error
    let base_provider = Arc::new(MockProvider::new(
        "mock_base",
        vec![ProviderResponse::new("Test response")],
    ));
    let shared_memory = Arc::new(
        MockSharedMemory::new()
            .with_get_error(SharedMemoryError::StorageError("Test error".to_string())),
    );
    let will_action_resolver = Arc::new(MockWillActionResolver::new());

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        base_provider.clone(),
        shared_memory,
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create a context with agent name
    let mut context = ProviderContext::default();
    context.agent_name = Some("test_agent".to_string());
    let request = ProviderRequest::new("Test message");

    // Execute the request - should still work despite shared memory error
    let response = provider.execute(&context, &request).await.unwrap();

    // Verify we got a response despite the error
    assert!(!response.content.is_empty());
}

#[tokio::test]
async fn test_error_handling_will_action() {
    // Create mock components with will action error
    let base_provider = Arc::new(MockProvider::new(
        "mock_base",
        vec![ProviderResponse::new("Fallback response")],
    ));
    let shared_memory = Arc::new(MockSharedMemory::new());
    let will_action_resolver = Arc::new(
        MockWillActionResolver::new()
            .with_execute_error(WillActionError::ExecutionError("Test error".to_string())),
    );

    // Create SistenceProvider
    let provider = SistenceProvider::new(
        base_provider.clone(),
        shared_memory,
        will_action_resolver,
        "test_sistence".to_string(),
    );

    // Create a context with agent name
    let mut context = ProviderContext::default();
    context.agent_name = Some("test_agent".to_string());
    let request = ProviderRequest::new("I want to notify the user about something important");

    // Execute the request - should fall back to base provider
    let response = provider.execute(&context, &request).await.unwrap();

    // Verify we got the fallback response
    assert_eq!(response.content, "Fallback response");
}

#[tokio::test]
async fn test_agent_id_generation() {
    // Create a SistenceAgentContext
    let context = SistenceAgentContext::new("test_agent", "user123");
    
    // Verify the agent ID format
    assert!(context.agent_id.starts_with("agent:test_agent:user123"));
}

#[tokio::test]
async fn test_context_serialization() {
    // Create a SistenceAgentContext
    let mut context = SistenceAgentContext::new("test_agent", "user123");
    
    // Add some interaction history
    context.add_interaction("user", "Hello");
    context.add_interaction("assistant", "Hi there");
    
    // Add some memory
    context.memory.insert("key1".to_string(), json!("value1"));
    context.memory.insert("key2".to_string(), json!(42));
    
    // Serialize to JSON
    let json_str = serde_json::to_string(&context).unwrap();
    
    // Deserialize back
    let deserialized: SistenceAgentContext = serde_json::from_str(&json_str).unwrap();
    
    // Verify fields match
    assert_eq!(context.agent_id, deserialized.agent_id);
    assert_eq!(context.interaction_history.len(), deserialized.interaction_history.len());
    assert_eq!(context.memory.len(), deserialized.memory.len());
    assert_eq!(context.memory.get("key1"), deserialized.memory.get("key1"));
    assert_eq!(context.memory.get("key2"), deserialized.memory.get("key2"));
}

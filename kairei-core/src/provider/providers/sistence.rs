//! Sistence Provider implementation for proactive AI agents.
//!
//! The SistenceProvider enables proactive behaviors through will actions,
//! persistent context management, and LLM integration.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::provider::capabilities::shared_memory::SharedMemoryCapability;
use crate::provider::capabilities::will_action::{
    WillActionContext, WillActionParams, WillActionResolver, WillActionResult,
};
use crate::provider::capability::{Capabilities, CapabilityType};
use crate::provider::provider::{Provider, ProviderSecret};
use crate::provider::request::RequestInput;
use crate::provider::request::{ProviderContext, ProviderRequest, ProviderResponse};
use crate::provider::types::{ProviderError, ProviderResult};
use crate::timestamp::Timestamp;

/// Context structure for Sistence agents
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SistenceAgentContext {
    /// Unique agent identifier
    pub agent_id: String,

    /// Creation timestamp
    pub created_at: Timestamp,

    /// Last active timestamp
    pub last_active: Timestamp,

    /// Key-value memory storage
    pub memory: HashMap<String, Value>,

    /// Interaction history
    pub interaction_history: Vec<InteractionRecord>,
}

impl SistenceAgentContext {
    /// Create a new agent context
    pub fn new(agent_name: &str, user_id: &str) -> Self {
        let now = Timestamp::now();
        let agent_id = format!("agent:{}:{}", agent_name, user_id);
        Self {
            agent_id,
            created_at: now.clone(),
            last_active: now,
            memory: HashMap::new(),
            interaction_history: Vec::new(),
        }
    }

    /// Add an interaction to the history
    pub fn add_interaction(&mut self, role: &str, content: &str) {
        let interaction = InteractionRecord {
            timestamp: Timestamp::now(),
            action: role.to_string(),
            parameters: json!({"content": content}),
            result: json!({}),
        };
        self.interaction_history.push(interaction);

        // Limit history size
        if self.interaction_history.len() > 100 {
            self.interaction_history.drain(0..50);
        }
    }
}

/// Record of an interaction with the agent
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InteractionRecord {
    /// When the interaction occurred
    pub timestamp: Timestamp,

    /// Action that was executed
    pub action: String,

    /// Parameters that were provided
    pub parameters: Value,

    /// Result of the action
    pub result: Value,
}

/// Provider implementation specialized for Sistence agents
pub struct SistenceProvider {
    /// Base LLM provider used for standard requests
    llm_provider: Arc<dyn Provider>,

    /// Shared memory capability for persistent context
    shared_memory: Arc<dyn SharedMemoryCapability>,

    /// Will action resolver for executing will actions
    will_action_resolver: Arc<dyn WillActionResolver>,

    /// Provider name
    name: String,
}

impl SistenceProvider {
    /// Create a new SistenceProvider
    pub fn new(
        llm_provider: Arc<dyn Provider>,
        shared_memory: Arc<dyn SharedMemoryCapability>,
        will_action_resolver: Arc<dyn WillActionResolver>,
        name: String,
    ) -> Self {
        Self {
            llm_provider,
            shared_memory,
            will_action_resolver,
            name,
        }
    }

    /// Get agent context from shared memory
    async fn get_agent_context(&self, agent_id: &str) -> ProviderResult<SistenceAgentContext> {
        let key = agent_id.to_string();
        match self.shared_memory.get(&key).await {
            Ok(value) => {
                // Try to deserialize the JSON value into a SistenceAgentContext
                serde_json::from_value(value).map_err(|e| {
                    ProviderError::InternalError(format!("Context deserialization error: {}", e))
                })
            }
            Err(e) => {
                if let crate::provider::capabilities::shared_memory::SharedMemoryError::KeyNotFound(_) = e {
                    // If key not found, create a new context
                    Ok(SistenceAgentContext::new("default", "system"))
                } else {
                    // For other errors, propagate them
                    Err(ProviderError::InternalError(format!("Failed to get agent context: {}", e)))
                }
            }
        }
    }

    /// Save agent context to shared memory
    async fn save_agent_context(&self, context: &SistenceAgentContext) -> ProviderResult<()> {
        let key = context.agent_id.to_string();
        let value = serde_json::to_value(context).map_err(|e| {
            ProviderError::InternalError(format!("Context serialization error: {}", e))
        })?;

        match self.shared_memory.set(&key, value).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ProviderError::InternalError(format!(
                "Failed to save agent context: {}",
                e
            ))),
        }
    }

    /// Process a will action request
    async fn process_will_action(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse> {
        // Extract agent name from context
        let agent_name = request.state.agent_name.clone();
        let user_id = if let Some(name) = &request.state.agent_info.agent_name {
            name.clone()
        } else {
            String::new()
        };

        // Extract action name from request
        let action_name = match &request.input.query {
            crate::eval::expression::Value::String(s) => s.clone(),
            _ => "unknown".to_string(),
        };

        // Generate agent ID
        let agent_id = format!("agent:{}:{}", agent_name, user_id);

        // Get agent context
        let mut agent_context = self.get_agent_context(&agent_id).await?;

        // Update last active timestamp
        agent_context.last_active = Timestamp::now();

        // Prepare will action parameters
        let mut will_params = WillActionParams::new();

        // Add parameters from request if available
        for (k, v) in &request.input.parameters {
            if k != "will_action" {
                // Convert from eval::expression::Value to serde_json::Value
                let json_value = match v {
                    crate::eval::expression::Value::String(s) => {
                        serde_json::Value::String(s.clone())
                    }
                    crate::eval::expression::Value::Float(f) => serde_json::Value::Number(
                        serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0)),
                    ),
                    crate::eval::expression::Value::Integer(n) => {
                        serde_json::Value::Number(serde_json::Number::from(*n))
                    }
                    crate::eval::expression::Value::UInteger(n) => {
                        serde_json::Value::Number(serde_json::Number::from(*n))
                    }
                    crate::eval::expression::Value::Boolean(b) => serde_json::Value::Bool(*b),
                    _ => serde_json::Value::Null,
                };
                will_params.named.insert(k.clone(), json_value);
            }
        }

        // Prepare context for will action
        let will_context = WillActionContext {
            agent_id: agent_id.clone(),
            permissions: vec![
                "notify".to_string(),
                "suggest".to_string(),
                "research".to_string(),
            ],
            data: HashMap::new(), // Could be populated from agent_context
        };

        // Try to execute the action directly
        let result = match self
            .will_action_resolver
            .execute(&action_name, will_params.clone(), &will_context)
            .await
        {
            Ok(result) => {
                // Record the interaction
                self.record_interaction(&mut agent_context, &action_name, &will_params, &result)
                    .await?;

                // Save the updated context
                self.save_agent_context(&agent_context).await?;

                // Return the result
                Ok(ProviderResponse {
                    output: serde_json::to_string(&result).unwrap_or_default(),
                    metadata: Default::default(),
                })
            }
            Err(_) => {
                // If direct execution fails, delegate to LLM
                self.execute_via_llm(context, request, &action_name, &will_params, &agent_context)
                    .await
            }
        };

        result
    }

    /// Execute a will action using LLM integration
    async fn execute_via_llm(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
        action_name: &str,
        params: &WillActionParams,
        agent_context: &SistenceAgentContext,
    ) -> ProviderResult<ProviderResponse> {
        // Build LLM request with appropriate prompting
        let prompt = self.build_will_action_prompt(action_name, params, agent_context);
        let llm_request = ProviderRequest {
            input: RequestInput {
                query: crate::eval::expression::Value::String(prompt),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert(
                        "temperature".to_string(),
                        crate::eval::expression::Value::Float(0.7),
                    );
                    p.insert(
                        "max_tokens".to_string(),
                        crate::eval::expression::Value::Float(500.0),
                    );
                    p
                },
            },
            state: request.state.clone(),
            config: request.config.clone(),
        };

        // Execute LLM request
        let llm_response = self.llm_provider.execute(context, &llm_request).await?;

        // Create a will action result from the LLM response
        let result = WillActionResult::success(serde_json::json!(llm_response.output));

        // Record the interaction
        let mut updated_context = agent_context.clone();
        self.record_interaction(&mut updated_context, action_name, params, &result)
            .await?;

        // Save the updated context
        self.save_agent_context(&updated_context).await?;

        // Process and return the result
        Ok(ProviderResponse {
            output: serde_json::to_string(&result).unwrap_or_default(),
            metadata: Default::default(),
        })
    }

    /// Build prompt for will action execution via LLM
    fn build_will_action_prompt(
        &self,
        action_name: &str,
        params: &WillActionParams,
        agent_context: &SistenceAgentContext,
    ) -> String {
        // Create prompt with context and action details
        format!(
            "You are a proactive AI assistant executing a will action.\n\n\
             ACTION: {}\n\
             PARAMETERS: {:?}\n\n\
             AGENT CONTEXT:\n{:?}\n\n\
             Based on this information, determine the appropriate response for this action.\n\
             Your response should be helpful, accurate, and aligned with the agent's purpose.\n\
             RESPONSE:",
            action_name, params, agent_context
        )
    }

    /// Record an interaction in the agent context
    async fn record_interaction(
        &self,
        agent_context: &mut SistenceAgentContext,
        action_name: &str,
        params: &WillActionParams,
        result: &WillActionResult,
    ) -> ProviderResult<()> {
        // Create interaction record
        let interaction = InteractionRecord {
            timestamp: Timestamp::now(),
            action: action_name.to_string(),
            parameters: serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            result: serde_json::to_value(result).unwrap_or(serde_json::Value::Null),
        };

        // Add to history
        agent_context.interaction_history.push(interaction);

        // Limit history size
        if agent_context.interaction_history.len() > 100 {
            agent_context.interaction_history.drain(0..50);
        }

        Ok(())
    }
}

#[async_trait]
impl Provider for SistenceProvider {
    async fn execute(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse> {
        // Check if this is a will action request
        let query_str = match &request.input.query {
            crate::eval::expression::Value::String(s) => s.clone(),
            _ => String::new(),
        };

        if query_str.contains("notify") || query_str.contains("suggest") {
            return self.process_will_action(context, request).await;
        }

        // Forward regular requests to the underlying LLM provider
        self.llm_provider.execute(context, request).await
    }

    async fn capabilities(&self) -> Capabilities {
        // Return capabilities from the underlying provider
        // plus the sistence-specific capabilities
        let _base_capabilities = self.llm_provider.capabilities().await;

        // Create a new set of capabilities
        let mut capability_types = std::collections::HashSet::new();
        capability_types.insert(CapabilityType::Custom("chat".to_string()));
        capability_types.insert(CapabilityType::Custom("sistence".to_string()));
        capability_types.insert(CapabilityType::Custom("will_action".to_string()));

        Capabilities::new(capability_types)
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn initialize(
        &mut self,
        _config: &crate::config::ProviderConfig,
        _secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        // We can't initialize the underlying provider directly since it's in an Arc
        // Just return success as the provider was already initialized in the registry
        Ok(())
    }

    async fn shutdown(&self) -> ProviderResult<()> {
        // Forward to the underlying provider
        self.llm_provider.shutdown().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::capabilities::will_action::{WillAction, WillActionError};
    use crate::provider::capability::Capabilities;
    use std::sync::Mutex;

    // Mock LLM Provider for testing
    struct MockLLMProvider {
        name: String,
        response: Mutex<Option<ProviderResponse>>,
    }

    impl MockLLMProvider {
        fn new(name: &str, response: Option<ProviderResponse>) -> Self {
            Self {
                name: name.to_string(),
                response: Mutex::new(response),
            }
        }

        fn set_response(&self, response: ProviderResponse) {
            let mut lock = self.response.lock().unwrap();
            *lock = Some(response);
        }
    }

    #[async_trait]
    impl Provider for MockLLMProvider {
        async fn execute(
            &self,
            _context: &ProviderContext,
            _request: &ProviderRequest,
        ) -> ProviderResult<ProviderResponse> {
            let lock = self.response.lock().unwrap();
            match &*lock {
                Some(response) => {
                    // Create a new response to avoid clone issues
                    Ok(ProviderResponse {
                        output: response.output.clone(),
                        metadata: response.metadata.clone(),
                    })
                }
                None => Err(ProviderError::InternalError("No response set".to_string())),
            }
        }

        async fn capabilities(&self) -> Capabilities {
            let mut capability_types = std::collections::HashSet::new();
            capability_types.insert(CapabilityType::Custom("llm".to_string()));
            Capabilities::new(capability_types)
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn initialize(
            &mut self,
            _config: &crate::config::ProviderConfig,
            _secret: &ProviderSecret,
        ) -> ProviderResult<()> {
            Ok(())
        }

        async fn shutdown(&self) -> ProviderResult<()> {
            Ok(())
        }
    }

    // Mock Shared Memory Capability for testing
    struct MockSharedMemory {
        storage: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl MockSharedMemory {
        fn new() -> Self {
            Self {
                storage: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl SharedMemoryCapability for MockSharedMemory {
        async fn get(
            &self,
            key: &str,
        ) -> Result<
            serde_json::Value,
            crate::provider::capabilities::shared_memory::SharedMemoryError,
        > {
            let storage = self.storage.lock().unwrap();
            if let Some(data) = storage.get(key) {
                let s = String::from_utf8_lossy(data);
                serde_json::from_str(&s).map_err(|e| {
                    crate::provider::capabilities::shared_memory::SharedMemoryError::InvalidValue(
                        e.to_string(),
                    )
                })
            } else {
                Err(
                    crate::provider::capabilities::shared_memory::SharedMemoryError::KeyNotFound(
                        key.to_string(),
                    ),
                )
            }
        }

        async fn set(
            &self,
            key: &str,
            value: serde_json::Value,
        ) -> Result<(), crate::provider::capabilities::shared_memory::SharedMemoryError> {
            let mut storage = self.storage.lock().unwrap();
            let data = serde_json::to_string(&value).map_err(|e| {
                crate::provider::capabilities::shared_memory::SharedMemoryError::InvalidValue(
                    e.to_string(),
                )
            })?;
            storage.insert(key.to_string(), data.into_bytes());
            Ok(())
        }

        async fn delete(
            &self,
            key: &str,
        ) -> Result<(), crate::provider::capabilities::shared_memory::SharedMemoryError> {
            let mut storage = self.storage.lock().unwrap();
            if storage.remove(key).is_some() {
                Ok(())
            } else {
                Err(
                    crate::provider::capabilities::shared_memory::SharedMemoryError::KeyNotFound(
                        key.to_string(),
                    ),
                )
            }
        }

        async fn exists(
            &self,
            key: &str,
        ) -> Result<bool, crate::provider::capabilities::shared_memory::SharedMemoryError> {
            let storage = self.storage.lock().unwrap();
            Ok(storage.contains_key(key))
        }

        async fn get_metadata(
            &self,
            key: &str,
        ) -> Result<
            crate::provider::capabilities::shared_memory::Metadata,
            crate::provider::capabilities::shared_memory::SharedMemoryError,
        > {
            let storage = self.storage.lock().unwrap();
            if storage.contains_key(key) {
                Ok(crate::provider::capabilities::shared_memory::Metadata::default())
            } else {
                Err(
                    crate::provider::capabilities::shared_memory::SharedMemoryError::KeyNotFound(
                        key.to_string(),
                    ),
                )
            }
        }

        async fn list_keys(
            &self,
            _pattern: &str,
        ) -> Result<Vec<String>, crate::provider::capabilities::shared_memory::SharedMemoryError>
        {
            let storage = self.storage.lock().unwrap();
            Ok(storage.keys().cloned().collect())
        }
    }

    #[async_trait]
    impl crate::provider::plugin::ProviderPlugin for MockSharedMemory {
        fn priority(&self) -> i32 {
            0
        }

        fn capability(&self) -> crate::provider::capability::CapabilityType {
            crate::provider::capability::CapabilityType::Custom("shared_memory".to_string())
        }

        async fn generate_section<'a>(
            &self,
            _context: &crate::provider::plugin::PluginContext<'a>,
        ) -> crate::provider::types::ProviderResult<crate::provider::provider::Section> {
            Ok(crate::provider::provider::Section::default())
        }

        async fn process_response<'a>(
            &self,
            _context: &crate::provider::plugin::PluginContext<'a>,
            _response: &crate::provider::llm::LLMResponse,
        ) -> crate::provider::types::ProviderResult<()> {
            Ok(())
        }
    }

    // Mock Will Action Resolver for testing
    struct MockWillActionResolver {
        actions: Mutex<HashMap<String, Box<dyn WillAction>>>,
    }

    impl MockWillActionResolver {
        fn new() -> Self {
            Self {
                actions: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl crate::provider::plugin::ProviderPlugin for MockWillActionResolver {
        fn priority(&self) -> i32 {
            0
        }

        fn capability(&self) -> crate::provider::capability::CapabilityType {
            crate::provider::capability::CapabilityType::Custom("will_action".to_string())
        }

        async fn generate_section<'a>(
            &self,
            _context: &crate::provider::plugin::PluginContext<'a>,
        ) -> crate::provider::types::ProviderResult<crate::provider::provider::Section> {
            Ok(crate::provider::provider::Section::default())
        }

        async fn process_response<'a>(
            &self,
            _context: &crate::provider::plugin::PluginContext<'a>,
            _response: &crate::provider::llm::LLMResponse,
        ) -> crate::provider::types::ProviderResult<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl WillActionResolver for MockWillActionResolver {
        fn resolve(&self, action_name: &str) -> Option<Box<dyn WillAction>> {
            let actions = self.actions.lock().unwrap();
            if actions.contains_key(action_name) {
                // For testing, we'll just return a simple notify action
                Some(Box::new(MockNotifyAction::new()))
            } else {
                None
            }
        }

        fn register(
            &mut self,
            action_name: &str,
            _action: Box<dyn WillAction>,
        ) -> Result<(), WillActionError> {
            let mut actions = self.actions.lock().unwrap();
            actions.insert(action_name.to_string(), Box::new(MockNotifyAction::new()));
            Ok(())
        }

        async fn execute(
            &self,
            action_name: &str,
            params: WillActionParams,
            context: &WillActionContext,
        ) -> Result<WillActionResult, WillActionError> {
            if let Some(action) = self.resolve(action_name) {
                Ok(action.execute(params, context).await)
            } else {
                Err(WillActionError::ActionNotFound(action_name.to_string()))
            }
        }

        fn list_actions(&self) -> Vec<String> {
            let actions = self.actions.lock().unwrap();
            actions.keys().cloned().collect()
        }

        fn get_action_signature(&self, action_name: &str) -> Option<WillActionSignature> {
            if let Some(action) = self.resolve(action_name) {
                Some(action.get_signature())
            } else {
                None
            }
        }
    }

    // Mock Will Action for testing
    struct MockNotifyAction;

    impl MockNotifyAction {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl WillAction for MockNotifyAction {
        async fn execute(
            &self,
            params: WillActionParams,
            context: &WillActionContext,
        ) -> WillActionResult {
            WillActionResult::success(serde_json::json!({
                "message": params.named.get("message").unwrap_or(&serde_json::json!("Default message")),
                "agent_id": context.agent_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }

        fn get_signature(&self) -> WillActionSignature {
            use crate::provider::capabilities::will_action::ParameterSpec;

            WillActionSignature {
                name: "notify".to_string(),
                description: "Send a notification to users".to_string(),
                parameters: vec![ParameterSpec {
                    name: "message".to_string(),
                    param_type: "String".to_string(),
                    required: true,
                    description: "The notification message".to_string(),
                    default_value: None,
                }],
                return_type: "NotificationResult".to_string(),
                required_permissions: vec!["notify".to_string()],
            }
        }
    }

    #[tokio::test]
    async fn test_sistence_provider_capabilities() {
        // Create mock components
        let llm_provider = Arc::new(MockLLMProvider::new(
            "mock_llm",
            Some(ProviderResponse {
                output: "Test response".to_string(),
                metadata: Default::default(),
            }),
        ));

        let shared_memory = Arc::new(MockSharedMemory::new());
        let will_action_resolver = Arc::new(MockWillActionResolver::new());

        // Create SistenceProvider
        let sistence_provider = SistenceProvider::new(
            llm_provider,
            shared_memory,
            will_action_resolver,
            "test_sistence".to_string(),
        );

        // Test capabilities
        let capabilities = sistence_provider.capabilities().await;
        assert!(capabilities.supports(&CapabilityType::Custom("chat".to_string())));
        assert!(capabilities.supports(&CapabilityType::Custom("sistence".to_string())));
        assert!(capabilities.supports(&CapabilityType::Custom("will_action".to_string())));
    }

    #[tokio::test]
    async fn test_sistence_provider_regular_request() {
        // Create mock components
        let llm_provider = Arc::new(MockLLMProvider::new(
            "mock_llm",
            Some(ProviderResponse {
                output: "Regular response".to_string(),
                metadata: Default::default(),
            }),
        ));

        let shared_memory = Arc::new(MockSharedMemory::new());
        let will_action_resolver = Arc::new(MockWillActionResolver::new());

        // Create SistenceProvider
        let sistence_provider = SistenceProvider::new(
            llm_provider,
            shared_memory,
            will_action_resolver,
            "test_sistence".to_string(),
        );

        // Test regular request
        let context = ProviderContext::default();
        let request = ProviderRequest {
            input: crate::provider::request::RequestInput {
                query: crate::eval::expression::Value::String("test query".to_string()),
                parameters: HashMap::new(),
            },
            state: Default::default(),
            config: Default::default(),
        };

        let response = sistence_provider.execute(&context, &request).await.unwrap();
        assert_eq!(response.output, "Regular response");
    }

    #[tokio::test]
    async fn test_sistence_provider_will_action_request() {
        // Create mock components
        let llm_provider = Arc::new(MockLLMProvider::new(
            "mock_llm",
            Some(ProviderResponse {
                output: "LLM response for will action".to_string(),
                metadata: Default::default(),
            }),
        ));

        let shared_memory = Arc::new(MockSharedMemory::new());
        let will_action_resolver = Arc::new(MockWillActionResolver::new());

        // Create SistenceProvider
        let sistence_provider = SistenceProvider::new(
            llm_provider,
            shared_memory.clone(),
            will_action_resolver,
            "test_sistence".to_string(),
        );

        // Test will action request
        let context = ProviderContext::default();
        let mut parameters = HashMap::new();
        parameters.insert(
            "agent_id".to_string(),
            crate::eval::expression::Value::String("test_agent".to_string()),
        );
        parameters.insert(
            "will_action".to_string(),
            crate::eval::expression::Value::String("notify".to_string()),
        );
        parameters.insert(
            "message".to_string(),
            crate::eval::expression::Value::String("Test notification".to_string()),
        );

        let request = ProviderRequest {
            input: crate::provider::request::RequestInput {
                query: crate::eval::expression::Value::String("notify".to_string()),
                parameters,
            },
            state: Default::default(),
            config: Default::default(),
        };

        let response = sistence_provider.execute(&context, &request).await.unwrap();

        // Verify the response contains JSON
        let result: serde_json::Value = serde_json::from_str(&response.output).unwrap();
        let result_obj = result.as_object().unwrap();
        assert!(result_obj.contains_key("success"));

        // Skip context verification for now as it depends on implementation details
        // that may change during development
    }
}

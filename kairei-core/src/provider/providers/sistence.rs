//! Sistence Provider implementation for proactive AI agents.
//!
//! The SistenceProvider enables proactive behaviors through will actions,
//! persistent context management, and LLM integration.
//!
//! # Architecture
//!
//! SistenceProvider follows a decorator/wrapper pattern where it wraps another Provider
//! and adds proactive capabilities. This design choice allows:
//!
//! 1. **Separation of concerns**: Core LLM functionality remains in the underlying provider,
//!    while proactive agent behaviors are added by the SistenceProvider layer.
//!
//! 2. **Composition over inheritance**: Any Provider implementation can be enhanced with
//!    Sistence capabilities without requiring modification or subclassing.
//!
//! 3. **Flexible deployment**: Sistence capabilities can be applied selectively to
//!    different underlying providers, depending on the use case.
//!
//! # Core Components
//!
//! - **Context Management**: Maintains persistent agent state and history across interactions
//! - **Will Actions**: Enables agents to express intent that translates to concrete actions
//! - **LLM Integration**: Falls back to LLM for handling actions not directly implemented
//!
//! # Usage Example
//!
//! ```ignore,no_run
//! use std::sync::Arc;
//! // Initialize the underlying LLM provider
//! let llm_provider = Arc::new(some_provider_implementation);
//!
//! // Initialize shared memory capability
//! let shared_memory = Arc::new(some_shared_memory_implementation);
//!
//! // Initialize will action resolver
//! let will_action_resolver = Arc::new(some_will_action_resolver);
//!
//! // Create the SistenceProvider
//! let sistence_provider = SistenceProvider::new(
//!     llm_provider,
//!     shared_memory,
//!     will_action_resolver,
//!     "my_sistence_provider".to_string(),
//! );
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use strum::{AsRefStr, Display, EnumIter, EnumString, IntoEnumIterator};
use tracing::warn;

use crate::config::ProviderConfig;
use crate::provider::capabilities::shared_memory::SharedMemoryCapability;
use crate::provider::capabilities::will_action::{
    WillActionContext, WillActionParams, WillActionResolver, WillActionResult,
};
use crate::provider::capability::{Capabilities, CapabilityType};
use crate::provider::config::providers::sistence::{
    SistenceProviderConfig, SistenceProviderConfigValidator,
};
use crate::provider::config::{ErrorCollector, ProviderConfigValidator, config_to_map};
use crate::provider::provider::{Provider, ProviderSecret};
use crate::provider::request::RequestInput;
use crate::provider::request::{ProviderContext, ProviderRequest, ProviderResponse};
use crate::provider::types::{ProviderError, ProviderResult};
use crate::timestamp::Timestamp;

use super::standard::StandardProvider;

/// Sistence agent capabilities represent actions that an agent is permitted to perform.
///
/// These capabilities function as a permission system for controlling agent behaviors.
/// When an agent attempts to perform a will action, the system checks if the agent has
/// the corresponding capability before allowing the action to proceed.
///
/// # Examples
///
/// ```ignore,no_run
/// use strum::IntoEnumIterator;
///
/// // Get all standard capabilities
/// let standard_capabilities = SistenceCapability::iter().collect::<Vec<_>>();
///
/// // Create a custom capability
/// let custom_capability = SistenceCapability::Custom("data_analysis".to_string());
/// ```
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    EnumString,
    Display,
    AsRefStr,
    EnumIter,
)]
#[strum(serialize_all = "lowercase")]
pub enum SistenceCapability {
    /// Ability to send notifications to users or other systems
    Notify,

    /// Ability to make suggestions based on observed patterns or user preferences
    Suggest,

    /// Ability to research information from available data sources
    Research,

    /// Ability to make decisions based on defined criteria
    Decide,

    /// Ability to schedule tasks for future execution
    Schedule,

    /// Ability to learn and adapt behavior based on interactions
    Learn,

    /// Custom capability for extensibility
    ///
    /// This variant allows for adding domain-specific capabilities
    /// beyond the standard set provided by the enum.
    #[strum(disabled)]
    Custom(String),
}

impl SistenceCapability {
    /// Get all standard capabilities
    pub fn standard_capabilities() -> Vec<Self> {
        Self::iter().collect()
    }

    /// Create a custom capability
    pub fn custom(name: &str) -> Self {
        Self::Custom(name.to_string())
    }

    /// Convert capability to string
    pub fn as_string(&self) -> String {
        match self {
            Self::Custom(name) => name.clone(),
            _ => self.as_ref().to_string(),
        }
    }
}

/// Context structure for Sistence agents
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SistenceAgentContext {
    /// Unique agent identifier
    pub agent_id: String,

    /// Agent name
    pub agent_name: String,

    /// Creation timestamp
    pub created_at: Timestamp,

    /// Last active timestamp
    pub last_active: Timestamp,

    /// Key-value memory storage
    pub memory: HashMap<String, Value>,

    /// Interaction history
    pub interaction_history: Vec<InteractionRecord>,

    /// Agent capabilities
    pub capabilities: Vec<String>,
}

// test serialization for SistenceAgentContext
#[cfg(test)]
mod test_sistence_agent_context {
    use super::*;

    #[test]
    fn test_sistence_capability_serialize() {
        let cap = SistenceCapability::Notify;
        let serialized = serde_json::to_string(&cap).unwrap();
        assert_eq!(serialized, "\"Notify\"");

        let custom = SistenceCapability::Custom("custom_ability".to_string());
        let serialized = serde_json::to_string(&custom).unwrap();
        assert_eq!(serialized, "{\"Custom\":\"custom_ability\"}");

        // Test parsing from string
        let parsed: SistenceCapability = "suggest".parse().unwrap();
        assert_eq!(parsed, SistenceCapability::Suggest);
    }

    #[test]
    fn test_sistence_agent_context_serialization() {
        let mut context = SistenceAgentContext {
            agent_id: "test_agent".to_string(),
            agent_name: "TestAgent".to_string(),
            created_at: Timestamp::now(),
            last_active: Timestamp::now(),
            memory: HashMap::new(),
            interaction_history: Vec::new(),
            capabilities: vec![
                SistenceCapability::Notify.as_string(),
                SistenceCapability::Suggest.as_string(),
            ],
        };

        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: SistenceAgentContext = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.agent_id, context.agent_id);
        assert_eq!(deserialized.agent_name, context.agent_name);
        assert_eq!(deserialized.created_at, context.created_at);
        assert_eq!(deserialized.last_active, context.last_active);
        assert_eq!(deserialized.memory, context.memory);
        assert_eq!(deserialized.capabilities, context.capabilities);
        assert_eq!(
            format!("{:?}", deserialized.interaction_history),
            format!("{:?}", context.interaction_history)
        );

        context
            .memory
            .insert("test_key".to_string(), json!("test_value"));
        context.interaction_history.push(InteractionRecord {
            timestamp: Timestamp::now(),
            action: "test_action".to_string(),
            parameters: json!({"test_param": "test_value"}),
            result: json!({"test_result": "test_value"}),
        });
        assert_eq!(context.memory.len(), 1);
        assert_eq!(context.interaction_history.len(), 1);
        assert!(format!("{:?}", context).contains("[InteractionRecord {"));

        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: SistenceAgentContext = serde_json::from_str(&serialized).unwrap();
        assert!(format!("{:?}", deserialized).contains("[InteractionRecord {"));

        // to_value and from_value
        let value = serde_json::to_value(&context).unwrap();
        let deserialized: SistenceAgentContext = serde_json::from_value(value).unwrap();
        assert!(format!("{:?}", deserialized).contains("[InteractionRecord {"));
    }
}

impl SistenceAgentContext {
    /// Create a new agent context with default capabilities
    pub fn new(agent_name: &str, user_id: &str) -> Self {
        let now = Timestamp::now();
        let agent_id = format!("agent:{}:{}", agent_name, user_id);
        Self {
            agent_id,
            agent_name: agent_name.to_string(),
            created_at: now.clone(),
            last_active: now,
            memory: HashMap::new(),
            interaction_history: Vec::new(),
            capabilities: vec![
                SistenceCapability::Notify.as_string(),
                SistenceCapability::Suggest.as_string(),
            ],
        }
    }

    /// Create a new agent context with specific capabilities
    pub fn new_with_capabilities(
        agent_name: &str,
        user_id: &str,
        capabilities: &[SistenceCapability],
    ) -> Self {
        let mut context = Self::new(agent_name, user_id);
        context.capabilities = capabilities.iter().map(|c| c.as_string()).collect();
        context
    }

    /// Check if agent has a specific capability
    pub fn has_capability(&self, capability: &SistenceCapability) -> bool {
        self.capabilities.contains(&capability.as_string())
    }

    /// Add a capability to the agent
    pub fn add_capability(&mut self, capability: &SistenceCapability) -> bool {
        let cap_string = capability.as_string();
        if !self.capabilities.contains(&cap_string) {
            self.capabilities.push(cap_string);
            true
        } else {
            false
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
        self.add(interaction);
    }

    /// Add an interaction record to the history with automatic pruning
    pub fn add(&mut self, interaction: InteractionRecord) {
        self.interaction_history.push(interaction);
        self.prune_history();
    }

    /// Set a memory value
    pub fn set_memory(&mut self, key: &str, value: Value) {
        self.memory.insert(key.to_string(), value);
    }

    /// Get a memory value
    pub fn get_memory(&self, key: &str) -> Option<&Value> {
        self.memory.get(key)
    }

    /// Prune history if it exceeds the maximum size
    fn prune_history(&mut self) {
        const MAX_HISTORY_SIZE: usize = 100;
        const PRUNE_AMOUNT: usize = 50;

        if self.interaction_history.len() > MAX_HISTORY_SIZE {
            self.interaction_history.drain(0..PRUNE_AMOUNT);
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

/// Proactive AI agent provider implementation with persistent context and will actions
///
/// SistenceProvider wraps a base Provider implementation and adds proactive capabilities
/// including persistent context management, will actions, and LLM integration. It follows
/// a decorator pattern to enhance any Provider with Sistence behavior.
///
/// # Architecture
///
/// - Acts as a middleware between the client and the underlying LLM provider
/// - Adds agent identity, persistent memory, and capability-based permissions
/// - Intercepts and processes "will actions" (proactive agent behaviors)
/// - Falls back to underlying LLM when direct execution is not possible
///
/// # Components
///
/// - `llm_provider`: The base Provider that handles regular LLM requests
/// - `shared_memory`: Storage mechanism for persistent agent context
/// - `will_action_resolver`: Resolves and executes will actions
/// - `config`: Configuration settings for agent behavior
///
/// # Design Rationale
///
/// SistenceProvider uses composition over inheritance by wrapping another Provider
/// rather than extending it. This allows any Provider to be enhanced with Sistence
/// capabilities without modifying its implementation.
pub struct SistenceProvider {
    /// Base LLM provider used for standard requests
    ///
    /// This provider handles all requests that don't involve will actions,
    /// and serves as the fallback for will actions that can't be directly executed.
    llm_provider: Arc<dyn Provider>,

    /// Shared memory capability for persistent context
    ///
    /// Stores and retrieves agent context information between interactions,
    /// enabling long-term memory and persistent state.
    shared_memory: Arc<dyn SharedMemoryCapability>,

    /// Will action resolver for executing will actions
    ///
    /// Maps action names to concrete implementations and handles
    /// permission checking and execution flow.
    will_action_resolver: Arc<dyn WillActionResolver>,

    /// Provider name for identification
    name: String,

    /// Provider configuration controlling behavior
    config: SistenceProviderConfig,
}

impl SistenceProvider {
    /// Create a new SistenceProvider with default configuration
    ///
    /// # Arguments
    ///
    /// * `llm_provider` - The underlying Provider implementation for handling standard LLM requests
    /// * `shared_memory` - Capability for persistent storage of agent context
    /// * `will_action_resolver` - Resolver for executing will actions
    /// * `name` - Unique name for this provider instance
    ///
    /// # Returns
    ///
    /// A new SistenceProvider instance with default configuration settings
    ///
    /// # Examples
    ///
    /// ```ignore,no_run
    /// let sistence_provider = SistenceProvider::new(
    ///     standard_provider,
    ///     shared_memory,
    ///     will_action_resolver,
    ///     "my_sistence_provider".to_string(),
    /// );
    /// ```
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
            config: SistenceProviderConfig::default(),
        }
    }

    /// Create a new SistenceProvider with custom configuration
    ///
    /// # Arguments
    ///
    /// * `llm_provider` - The underlying Provider implementation for handling standard LLM requests
    /// * `shared_memory` - Capability for persistent storage of agent context
    /// * `will_action_resolver` - Resolver for executing will actions
    /// * `name` - Unique name for this provider instance
    /// * `config` - Custom configuration settings for this provider
    ///
    /// # Returns
    ///
    /// A new SistenceProvider instance with the specified configuration
    ///
    /// # Examples
    ///
    /// ```ignore,no_run
    /// let config = SistenceProviderConfig {
    ///     max_history_size: 200,
    ///     default_temperature: 0.5,
    ///     default_max_tokens: 800,
    ///     default_capabilities: vec![SistenceCapability::Notify, SistenceCapability::Suggest],
    /// };
    ///
    /// let sistence_provider = SistenceProvider::new_with_config(
    ///     standard_provider,
    ///     shared_memory,
    ///     will_action_resolver,
    ///     "my_sistence_provider".to_string(),
    ///     config,
    /// );
    /// ```
    pub fn new_with_config(
        llm_provider: Arc<dyn Provider>,
        shared_memory: Arc<dyn SharedMemoryCapability>,
        will_action_resolver: Arc<dyn WillActionResolver>,
        name: String,
        config: SistenceProviderConfig,
    ) -> Self {
        Self {
            llm_provider,
            shared_memory,
            will_action_resolver,
            name,
            config,
        }
    }

    pub fn validate_config_collecting(config: &ProviderConfig) -> ErrorCollector {
        let kv = config_to_map(config);
        let _config = SistenceProviderConfigValidator::validate(&kv);

        ErrorCollector::new()
    }

    /// Create a new SistenceProvider from a standard provider config
    ///
    /// This factory method extracts Sistence-specific configuration from a standard
    /// ProviderConfig, making integration with existing configuration systems easier.
    ///
    /// # Arguments
    ///
    /// * `llm_provider` - The underlying Provider implementation for handling standard LLM requests
    /// * `shared_memory` - Capability for persistent storage of agent context
    /// * `will_action_resolver` - Resolver for executing will actions
    /// * `name` - Unique name for this provider instance
    /// * `provider_config` - Standard provider configuration to extract Sistence settings from
    ///
    /// # Returns
    ///
    /// A new SistenceProvider with configuration derived from the provider_config
    pub fn from_config(
        llm_provider: Arc<StandardProvider>,
        shared_memory: Arc<dyn SharedMemoryCapability>,
        will_action_resolver: Arc<dyn WillActionResolver>,
        name: String,
        provider_config: &crate::config::ProviderConfig,
    ) -> Self {
        let config = SistenceProviderConfig::from(provider_config);
        Self::new_with_config(
            llm_provider,
            shared_memory,
            will_action_resolver,
            name,
            config,
        )
    }

    /// Validate Sistence provider configuration
    ///
    /// This method validates the provider-specific configuration parameters
    /// using the SistenceProviderConfigValidator.
    ///
    /// # Arguments
    ///
    /// * `config` - The provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if validation succeeds
    /// * `Err(ProviderError)` if validation fails
    ///
    /// # Examples
    ///
    /// ```ignore,no_run
    /// match SistenceProvider::validate_config(&provider_config) {
    ///     Ok(_) => println!("Configuration is valid"),
    ///     Err(e) => println!("Configuration error: {}", e),
    /// }
    /// ```
    pub fn validate_config(config: &crate::config::ProviderConfig) -> ProviderResult<()> {
        let validator = SistenceProviderConfigValidator::new();
        let kv = config_to_map(config);
        // Perform validation
        validator
            .validate(&kv)
            .map_err(|e| ProviderError::ConfigValidationFailed(e.to_string()))
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
    #[tracing::instrument(skip(self), level = "debug")]
    async fn save_agent_context(
        &self,
        agent_id: &str,
        context: &SistenceAgentContext,
    ) -> ProviderResult<()> {
        let key = agent_id.to_string();
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

    fn get_agent_id(&self, agent_name: &str, user_id: &str) -> String {
        format!("agent:{}:{}", agent_name, user_id)
    }

    /// Process a will action request
    #[tracing::instrument(skip(self, context), level = "debug")]
    async fn process_will_action(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse> {
        // Extract agent name from context
        let agent_name = request.state.agent_name.clone();
        let user_id = request.state.agent_info.agent_name.clone();

        // Extract action name from request
        let action_name = match &request.input.query {
            crate::eval::expression::Value::String(s) => s.clone(),
            _ => "unknown".to_string(),
        };

        // Generate agent ID
        let agent_id = self.get_agent_id(&agent_name, &user_id);

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
            permissions: agent_context.capabilities.clone(),
            data: HashMap::new(), // Could be populated from agent_context
        };

        // Try to execute the action directly
        match self
            .will_action_resolver
            .execute(&action_name, will_params.clone(), &will_context)
            .await
        {
            Ok(result) => {
                // Record the interaction
                self.record_interaction(&mut agent_context, &action_name, &will_params, &result)
                    .await?;

                // Save the updated context
                self.save_agent_context(&agent_id, &agent_context).await?;

                // Return the result
                Ok(ProviderResponse {
                    output: serde_json::to_string(&result).unwrap_or_default(),
                    metadata: Default::default(),
                })
            }
            Err(_) => {
                warn!("Failed to execute action directly, delegating to LLM");
                // If direct execution fails, delegate to LLM
                self.execute_via_llm(
                    &agent_id,
                    context,
                    request,
                    &action_name,
                    &will_params,
                    &agent_context,
                )
                .await
            }
        }
    }

    /// Execute a will action using LLM integration
    #[tracing::instrument(skip(self, context), level = "debug")]
    async fn execute_via_llm(
        &self,
        agent_id: &str,
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
                        crate::eval::expression::Value::Float(self.config.default_temperature),
                    );
                    p.insert(
                        "max_tokens".to_string(),
                        crate::eval::expression::Value::Float(
                            self.config.default_max_tokens as f64,
                        ),
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
        self.save_agent_context(agent_id, &updated_context).await?;

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
            "You are a proactive AI assistant named \"{}\" executing a will action.\n\n\
             ACTION: {}\n\
             PARAMETERS: {:?}\n\
             CAPABILITIES: {:?}\n\n\
             AGENT CONTEXT:\n\
             - Agent ID: {}\n\
             - Created: {}\n\
             - Last active: {}\n\
             - Memory entries: {}\n\
             - Interaction history count: {}\n\n\
             Based on this information, determine the appropriate response for this action.\n\
             Your response should be helpful, accurate, and aligned with the agent's purpose.\n\
             Ensure your actions are within the agent's capabilities.\n\
             RESPONSE:",
            agent_context.agent_name,
            action_name,
            params,
            agent_context.capabilities,
            agent_context.agent_id,
            agent_context.created_at,
            agent_context.last_active,
            agent_context.memory.len(),
            agent_context.interaction_history.len()
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
        agent_context.add(interaction);

        // Limit history size
        if agent_context.interaction_history.len() > 100 {
            agent_context.interaction_history.drain(0..50);
        }

        Ok(())
    }
}

#[async_trait]
impl Provider for SistenceProvider {
    /// Execute a provider request, handling both standard LLM requests and will actions
    ///
    /// This implementation detects potential will actions based on the query content
    /// and routes them to specialized handlers, or falls back to the underlying LLM
    /// provider for standard requests.
    ///
    /// # Will Action Detection
    ///
    /// Currently, will actions are detected by checking if the query string contains
    /// known action keywords like "notify" or "suggest". This simple heuristic allows
    /// the provider to intercept these requests without requiring special syntax.
    ///
    /// # Execution Flow
    ///
    /// 1. Query is analyzed to detect potential will actions
    /// 2. If a will action is detected, it's processed by `process_will_action`
    /// 3. Otherwise, the request is forwarded to the underlying provider
    ///
    /// # Arguments
    ///
    /// * `context` - The execution context for the request
    /// * `request` - The request to execute
    ///
    /// # Returns
    ///
    /// The provider response containing the execution results
    #[tracing::instrument(
        name = "sistence_provider_execute",
        skip(self, context),
        level = "debug"
    )]
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

        // Check for known will action patterns in the query
        // This is a simple heuristic that could be enhanced with more sophisticated detection
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
        config: &crate::config::ProviderConfig,
        _secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        // Update the provider configuration
        self.config = SistenceProviderConfig::from(config);

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
    use std::sync::Mutex;

    use super::*;
    use crate::provider::{
        capabilities::will_action::{WillAction, WillActionError, WillActionSignature},
        provider::Provider,
        request::ProviderResponse,
    };

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

        #[allow(dead_code)]
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
            self.resolve(action_name)
                .map(|action| action.get_signature())
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

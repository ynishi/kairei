//! Will Action capability for Provider Plugins.
//!
//! The WillActionCapability allows Sistence agents to express intent through
//! will actions that the system resolves into concrete actions. This enables
//! proactive behaviors and autonomous decision-making in agents.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

use crate::provider::plugin::ProviderPlugin;

/// Parameters for a will action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillActionParams {
    /// Named parameters for the action
    pub named: HashMap<String, Value>,

    /// Positional parameters for the action
    pub positional: Vec<Value>,
}

impl WillActionParams {
    /// Create a new empty parameter set
    pub fn new() -> Self {
        Self {
            named: HashMap::new(),
            positional: Vec::new(),
        }
    }

    /// Add a named parameter
    pub fn with_named(mut self, name: &str, value: Value) -> Self {
        self.named.insert(name.to_string(), value);
        self
    }

    /// Add a positional parameter
    pub fn with_positional(mut self, value: Value) -> Self {
        self.positional.push(value);
        self
    }
}

impl Default for WillActionParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for will action execution
#[derive(Debug, Clone)]
pub struct WillActionContext {
    /// Agent ID that initiated the action
    pub agent_id: String,

    /// Permissions for the action
    pub permissions: Vec<String>,

    /// Additional context data
    pub data: HashMap<String, Value>,
}

/// Result of a will action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillActionResult {
    /// Whether the action was successful
    pub success: bool,

    /// Result data if successful
    pub data: Option<Value>,

    /// Error information if unsuccessful
    pub error: Option<WillActionError>,
}

impl WillActionResult {
    /// Create a successful result
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error result
    pub fn error(error: WillActionError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Signature of a will action
#[derive(Debug, Clone)]
pub struct WillActionSignature {
    /// Name of the action
    pub name: String,

    /// Description of what the action does
    pub description: String,

    /// Parameter specifications
    pub parameters: Vec<ParameterSpec>,

    /// Return type specification
    pub return_type: String,

    /// Required permissions
    pub required_permissions: Vec<String>,
}

/// Parameter specification for a will action
#[derive(Debug, Clone)]
pub struct ParameterSpec {
    /// Name of the parameter
    pub name: String,

    /// Type of the parameter
    pub param_type: String,

    /// Whether the parameter is required
    pub required: bool,

    /// Description of the parameter
    pub description: String,

    /// Default value if not provided
    pub default_value: Option<Value>,
}

/// Will Action trait defining the interface for action implementations
#[async_trait]
pub trait WillAction: Send + Sync {
    /// Execute the action with the given parameters and context
    async fn execute(
        &self,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> WillActionResult;

    /// Get the signature of the action
    fn get_signature(&self) -> WillActionSignature;
}

/// Will Action Resolver capability for Provider Plugins
#[async_trait]
pub trait WillActionResolver: ProviderPlugin {
    /// Resolve an action by name
    ///
    /// # Arguments
    /// * `action_name` - The name of the action to resolve
    ///
    /// # Returns
    /// * `Some(Box<dyn WillAction>)` - The resolved action if found
    /// * `None` - If no action with the given name is registered
    fn resolve(&self, action_name: &str) -> Option<Box<dyn WillAction>>;

    /// Register an action implementation
    ///
    /// # Arguments
    /// * `action_name` - The name to register the action under
    /// * `action` - The action implementation
    ///
    /// # Returns
    /// * `Ok(())` - If registration was successful
    /// * `Err(WillActionError)` - If registration failed
    fn register(
        &mut self,
        action_name: &str,
        action: Box<dyn WillAction>,
    ) -> Result<(), WillActionError>;

    /// Execute an action by name with the given parameters and context
    ///
    /// # Arguments
    /// * `action_name` - The name of the action to execute
    /// * `params` - Parameters for the action
    /// * `context` - Execution context
    ///
    /// # Returns
    /// * `Ok(WillActionResult)` - The result of the action execution
    /// * `Err(WillActionError)` - If execution failed
    async fn execute(
        &self,
        action_name: &str,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> Result<WillActionResult, WillActionError>;

    /// List all registered actions
    ///
    /// # Returns
    /// * `Vec<String>` - Names of all registered actions
    fn list_actions(&self) -> Vec<String>;

    /// Get the signature of a registered action
    ///
    /// # Arguments
    /// * `action_name` - The name of the action
    ///
    /// # Returns
    /// * `Some(WillActionSignature)` - The signature if the action is registered
    /// * `None` - If no action with the given name is registered
    fn get_action_signature(&self, action_name: &str) -> Option<WillActionSignature>;
}

/// Errors that can occur during will action operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum WillActionError {
    #[error("Action not found: {0}")]
    ActionNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_will_action_params() {
        let params = WillActionParams::new()
            .with_named("name", json!("value"))
            .with_positional(json!(42));

        assert_eq!(params.named.get("name").unwrap(), &json!("value"));
        assert_eq!(params.positional[0], json!(42));
    }

    #[test]
    fn test_will_action_result() {
        let success_result = WillActionResult::success(json!({
            "key": "value"
        }));
        assert!(success_result.success);
        assert_eq!(success_result.data.unwrap(), json!({"key": "value"}));
        assert!(success_result.error.is_none());

        let error = WillActionError::InvalidParameters("test error".to_string());
        let error_result = WillActionResult::error(error);
        assert!(!error_result.success);
        assert!(error_result.data.is_none());
        assert!(error_result.error.is_some());
    }
}

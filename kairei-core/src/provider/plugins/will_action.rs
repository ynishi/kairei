//! Default implementation of the WillActionResolver capability.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
// Import serde_json for JSON handling in action implementations
use serde_json;

use crate::provider::capabilities::will_action::{
    ParameterSpec, WillAction, WillActionContext, WillActionError, WillActionParams,
    WillActionResolver, WillActionResult, WillActionSignature,
};
use crate::provider::capability::CapabilityType;
use crate::provider::config::plugins::WillActionConfig;
use crate::provider::llm::LLMResponse;
use crate::provider::plugin::{PluginContext, ProviderPlugin};
use crate::provider::provider::Section;
use crate::provider::types::ProviderResult;

/// Default implementation of the WillActionResolver capability
pub struct DefaultWillActionResolver {
    /// Configuration
    config: WillActionConfig,

    /// Registry of action implementations
    actions: Arc<RwLock<HashMap<String, Box<dyn WillAction>>>>,
}

impl DefaultWillActionResolver {
    /// Create a new DefaultWillActionResolver
    pub fn new(config: WillActionConfig) -> Self {
        Self {
            config,
            actions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize with built-in actions
    pub fn with_built_in_actions(mut self) -> Self {
        // Register built-in actions
        let notify_action = Box::new(NotifyAction::new());
        let suggest_action = Box::new(SuggestAction::new());
        let research_action = Box::new(ResearchAction::new());
        let decide_action = Box::new(DecideAction::new());
        let schedule_action = Box::new(ScheduleAction::new());

        // Register actions
        let _ = self.register("notify", notify_action);
        let _ = self.register("suggest", suggest_action);
        let _ = self.register("research", research_action);
        let _ = self.register("decide", decide_action);
        let _ = self.register("schedule", schedule_action);

        self
    }
}

#[async_trait]
impl ProviderPlugin for DefaultWillActionResolver {
    fn priority(&self) -> i32 {
        10 // Medium priority
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::Custom("will_action".to_string())
    }

    async fn generate_section<'a>(&self, _context: &PluginContext<'a>) -> ProviderResult<Section> {
        // Will Action resolver doesn't contribute to prompt generation
        Ok(Section::new(""))
    }

    async fn process_response<'a>(
        &self,
        _context: &PluginContext<'a>,
        _response: &LLMResponse,
    ) -> ProviderResult<()> {
        // Will Action resolver doesn't process LLM responses
        Ok(())
    }

    // Note: initialize method is not part of the ProviderPlugin trait
    // The initialization is handled in the constructor and with_built_in_actions method
}

#[async_trait]
impl WillActionResolver for DefaultWillActionResolver {
    fn resolve(&self, action_name: &str) -> Option<Box<dyn WillAction>> {
        let actions = self.actions.read().unwrap();
        actions.get(action_name).map(|_| {
            // Clone is not implemented for Box<dyn WillAction>, so we need to create a new box
            // In a real implementation, this would need a proper cloning mechanism
            match action_name {
                "notify" => Box::new(NotifyAction::new()) as Box<dyn WillAction>,
                "suggest" => Box::new(SuggestAction::new()) as Box<dyn WillAction>,
                "research" => Box::new(ResearchAction::new()) as Box<dyn WillAction>,
                "decide" => Box::new(DecideAction::new()) as Box<dyn WillAction>,
                "schedule" => Box::new(ScheduleAction::new()) as Box<dyn WillAction>,
                _ => Box::new(NotifyAction::new()) as Box<dyn WillAction>, // Default fallback
            }
        })
    }

    fn register(
        &mut self,
        action_name: &str,
        action: Box<dyn WillAction>,
    ) -> Result<(), WillActionError> {
        let mut actions = self.actions.write().unwrap();

        // Check if we've reached the maximum number of actions
        if actions.len() >= self.config.max_actions && !actions.contains_key(action_name) {
            return Err(WillActionError::ConfigurationError(
                "Maximum number of actions reached".to_string(),
            ));
        }

        actions.insert(action_name.to_string(), action);
        Ok(())
    }

    async fn execute(
        &self,
        action_name: &str,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> Result<WillActionResult, WillActionError> {
        // Resolve the action
        let action = self
            .resolve(action_name)
            .ok_or_else(|| WillActionError::ActionNotFound(action_name.to_string()))?;

        // Check permissions
        let signature = action.get_signature();
        for permission in &signature.required_permissions {
            if !context.permissions.contains(permission) {
                return Err(WillActionError::PermissionDenied(format!(
                    "Missing required permission: {}",
                    permission
                )));
            }
        }

        // Execute the action
        let result = action.execute(params, context).await;

        Ok(result)
    }

    fn list_actions(&self) -> Vec<String> {
        let actions = self.actions.read().unwrap();
        actions.keys().cloned().collect()
    }

    fn get_action_signature(&self, action_name: &str) -> Option<WillActionSignature> {
        let action = self.resolve(action_name)?;
        Some(action.get_signature())
    }
}

/// Notify action for sending notifications to users
#[derive(Debug, Clone)]
pub struct NotifyAction;

impl Default for NotifyAction {
    fn default() -> Self {
        Self::new()
    }
}

impl NotifyAction {
    /// Create a new NotifyAction
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WillAction for NotifyAction {
    async fn execute(
        &self,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> WillActionResult {
        // Extract message parameter
        let message = if let Some(message) = params.named.get("message") {
            message.as_str().unwrap_or_default().to_string()
        } else if !params.positional.is_empty() {
            params.positional[0]
                .as_str()
                .unwrap_or_default()
                .to_string()
        } else {
            return WillActionResult::error(WillActionError::InvalidParameters(
                "Missing required parameter: message".to_string(),
            ));
        };

        // Extract priority parameter
        let priority = params
            .named
            .get("priority")
            .and_then(|p| p.as_str())
            .unwrap_or("medium");

        // Placeholder implementation
        WillActionResult::success(serde_json::json!({
            "message": message,
            "priority": priority,
            "agent_id": context.agent_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
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
                ParameterSpec {
                    name: "priority".to_string(),
                    param_type: "String".to_string(),
                    required: false,
                    description: "Priority level (low, medium, high)".to_string(),
                    default_value: Some(serde_json::json!("medium")),
                },
            ],
            return_type: "NotificationResult".to_string(),
            required_permissions: vec!["notify".to_string()],
        }
    }
}

/// Suggest action for generating and presenting suggestions
#[derive(Debug, Clone)]
pub struct SuggestAction;

impl Default for SuggestAction {
    fn default() -> Self {
        Self::new()
    }
}

impl SuggestAction {
    /// Create a new SuggestAction
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WillAction for SuggestAction {
    async fn execute(
        &self,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> WillActionResult {
        // Extract options parameter
        let options = if let Some(options) = params.named.get("options") {
            options.clone()
        } else if !params.positional.is_empty() {
            params.positional[0].clone()
        } else {
            return WillActionResult::error(WillActionError::InvalidParameters(
                "Missing required parameter: options".to_string(),
            ));
        };

        // Placeholder implementation
        WillActionResult::success(serde_json::json!({
            "suggestions": options,
            "agent_id": context.agent_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    fn get_signature(&self) -> WillActionSignature {
        WillActionSignature {
            name: "suggest".to_string(),
            description: "Generate and present suggestions".to_string(),
            parameters: vec![ParameterSpec {
                name: "options".to_string(),
                param_type: "Array".to_string(),
                required: true,
                description: "Options to suggest from".to_string(),
                default_value: None,
            }],
            return_type: "SuggestionResult".to_string(),
            required_permissions: vec!["suggest".to_string()],
        }
    }
}

/// Research action for gathering information on a topic
#[derive(Debug, Clone)]
pub struct ResearchAction;

impl Default for ResearchAction {
    fn default() -> Self {
        Self::new()
    }
}

impl ResearchAction {
    /// Create a new ResearchAction
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WillAction for ResearchAction {
    async fn execute(
        &self,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> WillActionResult {
        // Extract topic parameter
        let topic = if let Some(topic) = params.named.get("topic") {
            topic.as_str().unwrap_or_default().to_string()
        } else if !params.positional.is_empty() {
            params.positional[0]
                .as_str()
                .unwrap_or_default()
                .to_string()
        } else {
            return WillActionResult::error(WillActionError::InvalidParameters(
                "Missing required parameter: topic".to_string(),
            ));
        };

        // Placeholder implementation
        WillActionResult::success(serde_json::json!({
            "topic": topic,
            "agent_id": context.agent_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "results": [],
        }))
    }

    fn get_signature(&self) -> WillActionSignature {
        WillActionSignature {
            name: "research".to_string(),
            description: "Gather information on a topic".to_string(),
            parameters: vec![ParameterSpec {
                name: "topic".to_string(),
                param_type: "String".to_string(),
                required: true,
                description: "The topic to research".to_string(),
                default_value: None,
            }],
            return_type: "ResearchResult".to_string(),
            required_permissions: vec!["research".to_string()],
        }
    }
}

/// Decide action for making decisions based on criteria
#[derive(Debug, Clone)]
pub struct DecideAction;

impl Default for DecideAction {
    fn default() -> Self {
        Self::new()
    }
}

impl DecideAction {
    /// Create a new DecideAction
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WillAction for DecideAction {
    async fn execute(
        &self,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> WillActionResult {
        // Extract options parameter
        let options = if let Some(options) = params.named.get("options") {
            options.clone()
        } else if !params.positional.is_empty() {
            params.positional[0].clone()
        } else {
            return WillActionResult::error(WillActionError::InvalidParameters(
                "Missing required parameter: options".to_string(),
            ));
        };

        // Placeholder implementation
        WillActionResult::success(serde_json::json!({
            "options": options,
            "agent_id": context.agent_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "decision": null,
        }))
    }

    fn get_signature(&self) -> WillActionSignature {
        WillActionSignature {
            name: "decide".to_string(),
            description: "Make decisions based on criteria".to_string(),
            parameters: vec![ParameterSpec {
                name: "options".to_string(),
                param_type: "Array".to_string(),
                required: true,
                description: "Options to choose from".to_string(),
                default_value: None,
            }],
            return_type: "DecisionResult".to_string(),
            required_permissions: vec!["decide".to_string()],
        }
    }
}

/// Schedule action for scheduling future tasks
#[derive(Debug, Clone)]
pub struct ScheduleAction;

impl Default for ScheduleAction {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleAction {
    /// Create a new ScheduleAction
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WillAction for ScheduleAction {
    async fn execute(
        &self,
        params: WillActionParams,
        context: &WillActionContext,
    ) -> WillActionResult {
        // Extract task parameter
        let task = if let Some(task) = params.named.get("task") {
            task.clone()
        } else if !params.positional.is_empty() {
            params.positional[0].clone()
        } else {
            return WillActionResult::error(WillActionError::InvalidParameters(
                "Missing required parameter: task".to_string(),
            ));
        };

        // Placeholder implementation
        WillActionResult::success(serde_json::json!({
            "task": task,
            "agent_id": context.agent_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "scheduled_id": format!("task_{}", uuid::Uuid::new_v4()),
        }))
    }

    fn get_signature(&self) -> WillActionSignature {
        WillActionSignature {
            name: "schedule".to_string(),
            description: "Schedule future tasks".to_string(),
            parameters: vec![ParameterSpec {
                name: "task".to_string(),
                param_type: "Object".to_string(),
                required: true,
                description: "Task to schedule".to_string(),
                default_value: None,
            }],
            return_type: "ScheduleResult".to_string(),
            required_permissions: vec!["schedule".to_string()],
        }
    }
}

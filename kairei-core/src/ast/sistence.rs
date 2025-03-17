use super::*;

/// Represents a Sistence agent definition in the KAIREI DSL.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SistenceAgentDef {
    /// Name of the Sistence agent
    pub name: String,
    /// Policy statements that guide the agent's behavior
    pub policies: Vec<Policy>,
    /// Lifecycle handlers for initialization and cleanup
    pub lifecycle: Option<LifecycleDef>,
    /// State definition for the agent
    pub state: Option<StateDef>,
    /// Observe handler for processing events
    pub observe: Option<ObserveDef>,
    /// Answer handler for responding to queries
    pub answer: Option<AnswerDef>,
    /// React handler for responding to events
    pub react: Option<ReactDef>,
    /// Sistence-specific configuration
    pub sistence_config: Option<SistenceConfig>,
}

/// Configuration for Sistence agent behavior
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SistenceConfig {
    /// Proactivity level (0.0 to 1.0)
    pub level: f64,
    /// Threshold for taking initiative (0.0 to 1.0)
    pub initiative_threshold: f64,
    /// Domains the agent can operate in
    pub domains: Vec<String>,
    /// Additional configuration parameters
    pub parameters: std::collections::HashMap<String, Literal>,
}

/// Represents a will action in the KAIREI DSL
#[derive(Debug, Clone, PartialEq)]
pub struct WillAction {
    /// The action to perform
    pub action: String,
    /// Parameters for the action
    pub parameters: Vec<Expression>,
    /// Optional target for the action
    pub target: Option<String>,
}

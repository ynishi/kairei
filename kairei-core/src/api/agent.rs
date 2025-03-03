//! Agent API for kairei-core
//!
//! This module defines the AgentApi trait for agent management operations.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::agent_registry::AgentError;
use crate::api::models::{AgentCreationRequest, AgentCreationResponse, AgentStatusDto};
use crate::event_bus::Value;
use crate::system::{SystemError, SystemResult};

/// Error type for agent operations
pub type AgentResult<T> = Result<T, AgentError>;

/// API for agent management operations
#[async_trait]
pub trait AgentApi: Send + Sync {
    /// Register a new agent from DSL code
    async fn register_agent_from_dsl(
        &self,
        request: AgentCreationRequest,
    ) -> Result<AgentCreationResponse, SystemError>;

    /// Start an agent
    async fn start_agent(&self, agent_name: &str) -> SystemResult<()>;

    /// Stop an agent
    async fn stop_agent(&self, agent_name: &str) -> SystemResult<()>;

    /// Restart an agent
    async fn restart_agent(&self, agent_name: &str) -> SystemResult<()>;

    /// Get agent status
    async fn get_agent_status(&self, agent_name: &str) -> Result<AgentStatusDto, SystemError>;

    /// Scale up agent instances
    async fn scale_up(
        &self,
        name: &str,
        count: usize,
        metadata: HashMap<String, Value>,
    ) -> Result<Vec<String>, SystemError>;

    /// Scale down agent instances
    async fn scale_down(
        &self,
        name: &str,
        count: usize,
        metadata: HashMap<String, Value>,
    ) -> SystemResult<()>;
}

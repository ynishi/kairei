//! State API for kairei-core
//!
//! This module defines the StateApi trait for state operations.

use async_trait::async_trait;

use crate::eval::expression;
use crate::system::SystemError;

/// API for state operations
#[async_trait]
pub trait StateApi: Send + Sync {
    /// Get agent state
    async fn get_agent_state(
        &self,
        agent_name: &str,
        key: &str,
    ) -> Result<expression::Value, SystemError>;
}

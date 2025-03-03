//! Integration module for kairei-http
//!
//! This module provides integration with kairei-core's API interfaces.

use std::sync::Arc;

use kairei_core::{
    api::{agent::AgentApi, event::EventApi, state::StateApi, system::SystemApi},
    system::System,
};

/// KaireiSystem provides a unified interface to kairei-core's API interfaces
pub struct KaireiSystem {
    /// System API for system-wide operations
    pub system_api: Arc<dyn SystemApi>,

    /// Agent API for agent management operations
    pub agent_api: Arc<dyn AgentApi>,

    /// Event API for event operations
    pub event_api: Arc<dyn EventApi>,

    /// State API for state operations
    pub state_api: Arc<dyn StateApi>,
}

impl KaireiSystem {
    /// Create a new KaireiSystem from a System instance
    pub fn new(system: Arc<System>) -> Self {
        Self {
            system_api: system.clone(),
            agent_api: system.clone(),
            event_api: system.clone(),
            state_api: system.clone(),
        }
    }
}

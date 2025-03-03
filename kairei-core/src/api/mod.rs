//! API module for kairei-core
//!
//! This module provides interfaces for external components to interact with the KAIREI system.
//! It defines traits for system, agent, event, and state operations.

pub mod agent;
pub mod event;
pub mod models;
pub mod state;
pub mod system;

// Re-export common types for convenience
pub use agent::AgentApi;
pub use event::EventApi;
pub use state::StateApi;
pub use system::SystemApi;

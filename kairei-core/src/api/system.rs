//! System API for kairei-core
//!
//! This module defines the SystemApi trait for system-wide operations.

use async_trait::async_trait;

use crate::api::models::SystemStatusDto;
use crate::system::SystemResult;

/// API for system-wide operations
#[async_trait]
pub trait SystemApi: Send + Sync {
    /// Get system-wide status information
    async fn get_system_status(&self) -> SystemResult<SystemStatusDto>;

    /// Start the system
    async fn start(&self) -> SystemResult<()>;

    /// Shutdown the system
    async fn shutdown(&self) -> SystemResult<()>;

    /// Emergency shutdown the system
    async fn emergency_shutdown(&self) -> SystemResult<()>;
}

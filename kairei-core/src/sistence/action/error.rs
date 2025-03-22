use chrono::Duration;

use crate::sistence::memory::error::SistenceMemoryError;

/// Error type for sistence action operations
#[derive(thiserror::Error, Debug)]
pub enum SistenceActionError {
    /// Memory-related error
    #[error("Memory error: {0}")]
    MemoryError(#[from] SistenceMemoryError),

    /// Execution timeout
    #[error("Execution timed out after {0:?}")]
    Timeout(Duration),

    /// Execution was cancelled
    #[error("Execution was cancelled")]
    Cancelled,

    /// Invalid execution ID
    #[error("Invalid execution ID: {0}")]
    InvalidExecutionId(String),

    /// No workspaces available
    #[error("No workspaces available")]
    NoWorkspacesAvailable,

    /// No agents available
    #[error("No agents available")]
    NoAgentsAvailable,

    /// Other errors
    #[error("Action error: {0}")]
    Other(String),
}

/// Result type for sistence action operations
pub type SistenceActionResult<T> = Result<T, SistenceActionError>;

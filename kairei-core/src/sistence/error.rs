use super::memory::error::SistenceMemoryError;
use super::space::error::SistenceWorkspaceError;

#[derive(Debug, thiserror::Error)]
pub enum SistenceError {
    #[error("Sistence memory error: {0}")]
    MemoryError(#[from] SistenceMemoryError),
    #[error("Sistence work space error: {0}")]
    SpaceError(#[from] SistenceWorkspaceError),
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type SistenceResult<T> = std::result::Result<T, SistenceError>;

#[derive(Debug, thiserror::Error)]
pub enum SistenceWorkspaceError {
    #[error("Workspace Internal error: {0}")]
    InternalError(String),
}

pub type SistenceWorkspaceResult<T> = std::result::Result<T, SistenceWorkspaceError>;

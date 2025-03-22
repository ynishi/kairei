#[derive(Debug, thiserror::Error)]
pub enum SistenceMemoryError {
    #[error("Recollection not found: ID: {0}")]
    RecollectionNotFound(String),
    #[error("Recollection already exists")]
    RecollectionAlreadyExists,
    #[error("Recollection not in workspace")]
    RecollectionNotInWorkspace,
    #[error("Recollection in workspace")]
    RecollectionInWorkspace,
    #[error("Workspace not found")]
    WorkspaceAlreadyExists,
    #[error("Workspace not found")]
    WorkspaceNotInSpace,
    #[error("Workspace in space")]
    WorkspaceInSpace,
    #[error("Workspace not in space")]
    SpaceAlreadyExists,
    #[error("Space not found")]
    SpaceNotFound,
    #[error("Space not in workspace")]
    SpaceInSpace,
    #[error("Space not in workspace")]
    SpaceNotInSpace,
    #[error("Space not in workspace")]
    SpaceNotInWorkspace,
    #[error("Space in workspace")]
    SpaceInWorkspace,
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type SistenceMemoryResult<T> = std::result::Result<T, SistenceMemoryError>;

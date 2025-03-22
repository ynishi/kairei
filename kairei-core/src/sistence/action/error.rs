#[derive(Debug, thiserror::Error)]
pub enum SistenceActionError {
    #[error("Action Internal error: {0}")]
    InternalError(String),
}

pub type SistenceActionResult<T> = std::result::Result<T, SistenceActionError>;

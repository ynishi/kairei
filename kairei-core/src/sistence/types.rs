use std::fmt;
use uuid::Uuid;

pub type WorkspaceId = String;
pub type RecollectionId = String;

/// Unique identifier for an execution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExecutionId(String);

impl Default for ExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionId {
    /// Creates a new random execution ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl fmt::Display for ExecutionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

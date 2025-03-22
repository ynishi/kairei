use crate::sistence::types::{RecollectionId, WorkspaceId};

/// Context for a workspace with memory capabilities
pub struct WorkspaceContext {
    /// ID of the workspace
    pub workspace_id: WorkspaceId,
    // Memory system for this workspace
    // pub memory: SistenceMemory,
    /// Name of the workspace
    pub name: String,
    /// Description of the workspace
    pub description: String,
    /// Recollections in this workspace
    pub recollections: Vec<RecollectionContext>,
}

/// Context for collaboration between agents in a workspace
pub struct CollaborationContext {
    /// ID of the workspace
    pub workspace_id: WorkspaceId,
    // Memory system for this collaboration
    // pub memory: SistenceMemory,
    // Participating in this collaboration
    // pub agents: Vec<AgentProxy>,
}

/// Context for a recollection
pub struct RecollectionContext {
    /// ID of the recollection
    pub id: RecollectionId,
    /// Content of the recollection
    pub content: String,
}

pub enum AgentProxy {
    // Agent ID
    // pub agent_id: String,
    // Agent name
    // pub name: String,
    // Agent role
    // pub role: String,
    // Agent capabilities
    // pub capabilities: Vec<Capability>,
}

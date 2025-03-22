use chrono::{DateTime, Utc};

use crate::sistence::types::WorkspaceId;

pub struct Workspace {
    id: WorkspaceId,
    name: String,
    parent_id: Option<WorkspaceId>,
    created_at: DateTime<Utc>,
    purpose: Option<String>,
    state: WorkspaceState,
}

pub enum WorkspaceState {
    Active,
    Merged {
        merged_into: WorkspaceId,
        merged_at: DateTime<Utc>,
    },
    Archived {
        archived_at: DateTime<Utc>,
    },
}

pub type WorkspaceInfo = Workspace;

pub enum MergeStrategy {
    /// Simple merge of content
    Simple,
    /// Merge with conflict resolution
    ConflictResolution,
    /// Merge with voting
    Voting,
}

pub struct MergeResult {
    pub merged_id: WorkspaceId,
    pub merged_at: DateTime<Utc>,
}

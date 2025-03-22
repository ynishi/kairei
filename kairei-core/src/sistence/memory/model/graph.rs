use chrono::{DateTime, Utc};

use crate::sistence::types::RecollectionId;

/// An edge connecting two recollections in the memory graph
pub struct RecollectionEdge {
    /// ID of the source recollection
    source_id: RecollectionId,
    /// ID of the target recollection
    target_id: RecollectionId,
    /// Type of relationship between the source and target
    relationship_type: RelationshipType,
    /// Additional metadata about this edge
    metadata: EdgeMetadata,
}

/// Types of relationships between recollections
pub enum RelationshipType {
    /// Derivation relationship (B is derived from A)
    DerivedFrom,
    /// Reference relationship (A references B)
    References,
    /// Contradiction relationship (A contradicts B)
    Contradicts,
    /// Support relationship (A supports B)
    Supports,
    /// Merge relationship (C is a merge of A and B)
    Merged,
    /// Supersede relationship (B supersedes A)
    Supersedes,
}

/// Metadata associated with an edge in the memory graph
pub struct EdgeMetadata {
    /// Timestamp when this edge was created
    created_at: DateTime<Utc>,
    /// Strength of the relationship (0.0-1.0)
    strength: f32,
    /// ID of the entity that created this edge
    creator_id: String,
}

pub struct ConflictInfo {
    /// IDs of the conflicting recollections
    recollection_ids: Vec<RecollectionId>,
    /// Type of conflict
    conflict_type: ConflictType,
    /// Timestamp when the conflict was detected
    detected_at: DateTime<Utc>,
}

pub enum ConflictType {
    /// Direct contradiction between recollections
    Contradiction,
    /// Inconsistency in the information provided by recollections
    Inconsistency,
    /// Ambiguity in the interpretation of recollections
    Ambiguity,
}

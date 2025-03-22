use crate::sistence::memory::error::SistenceMemoryResult;
use crate::sistence::memory::model::graph::{ConflictInfo, RecollectionEdge, RelationshipType};
use crate::sistence::memory::model::recollection::RecollectionEntry;
use crate::sistence::types::RecollectionId;

pub trait RecollectionGraphService {
    fn add_node(&self, entry: &RecollectionEntry) -> SistenceMemoryResult<()>;
    fn add_edge(&self, edge: RecollectionEdge) -> SistenceMemoryResult<()>;

    fn get_ancestors(
        &self,
        id: RecollectionId,
        depth: Option<usize>,
    ) -> SistenceMemoryResult<Vec<RecollectionId>>;
    fn get_related(
        &self,
        id: RecollectionId,
        relationship: Option<RelationshipType>,
    ) -> SistenceMemoryResult<Vec<RecollectionEdge>>;

    fn find_path(
        &self,
        from: RecollectionId,
        to: RecollectionId,
    ) -> SistenceMemoryResult<Vec<RecollectionEdge>>;
    fn detect_conflicts(&self, ids: &[RecollectionId]) -> SistenceMemoryResult<Vec<ConflictInfo>>;

    fn trace_origin(&self, id: RecollectionId) -> SistenceMemoryResult<Vec<RecollectionId>>;
}

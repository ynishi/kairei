use std::sync::Arc;

use async_trait::async_trait;

use crate::sistence::types::RecollectionId;

use super::{
    error::SistenceMemoryResult,
    model::{
        graph::RelationshipType,
        recollection::{
            Content, MergeStrategy, Recollection, RecollectionQuery, RecollectionSource,
        },
    },
    service::{
        graph::RecollectionGraphService, metadata::MetadataEnrichmentService,
        recollection::RecollectionService,
    },
};

#[async_trait]
pub trait SistenceMemory {
    async fn remember(
        &self,
        content: Content,
        source: RecollectionSource,
    ) -> SistenceMemoryResult<RecollectionId>;

    async fn recall(&self, query: RecollectionQuery) -> SistenceMemoryResult<Vec<Recollection>>;

    async fn recall_related(
        &self,
        id: RecollectionId,
        relationship_type: Option<RelationshipType>,
    ) -> SistenceMemoryResult<Vec<Recollection>>;

    async fn merge_recollections(
        &self,
        ids: &[RecollectionId],
        strategy: MergeStrategy,
    ) -> SistenceMemoryResult<RecollectionId>;
}

pub struct SistenceMemoryFacade {
    recollection_service: Arc<dyn RecollectionService>,
    metadata_service: Arc<dyn MetadataEnrichmentService>,
    graph_service: Arc<dyn RecollectionGraphService>,
}

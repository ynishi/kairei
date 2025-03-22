use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::sistence::types::RecollectionId;

use super::{
    error::SistenceMemoryResult,
    model::{
        graph::RelationshipType,
        recollection::{Content, MergeStrategy, RecollectionEntry, RecollectionSource},
    },
    service::{
        graph::RecollectionGraphService, metadata::MetadataEnrichmentService,
        recollection::RecollectionRepository, search::SearchEngine,
    },
};

#[async_trait]
pub trait SistenceMemory {
    async fn remember(
        &self,
        content: Content,
        source: RecollectionSource,
    ) -> SistenceMemoryResult<RecollectionId>;

    async fn recall(
        &self,
        query: &str,
        workspace_id: &str,
        context: Option<HashMap<String, String>>,
        top_k: usize,
        limit: usize,
    ) -> SistenceMemoryResult<Vec<RecollectionEntry>>;

    async fn recall_related(
        &self,
        id: RecollectionId,
        relationship_type: Option<RelationshipType>,
    ) -> SistenceMemoryResult<Vec<RecollectionEntry>>;

    async fn merge_recollections(
        &self,
        ids: &[RecollectionId],
        strategy: MergeStrategy,
    ) -> SistenceMemoryResult<RecollectionId>;
}

pub struct SistenceMemoryFacade {
    recollection_repository: Arc<dyn RecollectionRepository>,
    metadata_service: Arc<dyn MetadataEnrichmentService>,
    graph_service: Arc<dyn RecollectionGraphService>,
    search_engine: Arc<dyn SearchEngine>,
}

use crate::sistence::memory::error::SistenceMemoryResult;
use crate::sistence::{
    memory::model::{
        metadata::{MetadataType, RecollectionMetadata},
        recollection::RecollectionEntry,
    },
    types::RecollectionId,
};

pub trait MetadataEnrichmentService {
    fn generate_metadata(
        &self,
        entry: &RecollectionEntry,
    ) -> SistenceMemoryResult<RecollectionMetadata>;
    fn get_metadata(&self, entry_id: RecollectionId) -> SistenceMemoryResult<RecollectionMetadata>;
    fn update_usage_stats(
        &self,
        entry_id: RecollectionId,
        usage: String,
    ) -> SistenceMemoryResult<()>;
}

pub trait MetadataGenerator: Send + Sync {
    fn generator_name(&self) -> &str;
    fn supported_metadata_types(&self) -> Vec<MetadataType>;
    fn generate(&self, entry: &RecollectionEntry) -> SistenceMemoryResult<RecollectionMetadata>;
}

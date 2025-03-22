use crate::sistence::memory::error::SistenceMemoryResult;
use crate::sistence::{memory::model::recollection::RecollectionEntry, types::RecollectionId};

pub trait RecollectionRepository {
    fn store(&self, entry: &RecollectionEntry) -> SistenceMemoryResult<RecollectionId>;
    fn get(&self, id: RecollectionId) -> SistenceMemoryResult<RecollectionEntry>;
    fn get_batch(&self, ids: &[RecollectionId]) -> SistenceMemoryResult<Vec<RecollectionEntry>>;
    fn update(&self, entry: &RecollectionEntry) -> SistenceMemoryResult<()>;
    fn delete(&self, id: RecollectionId) -> SistenceMemoryResult<()>;
}

use crate::sistence::memory::error::SistenceMemoryResult;
use crate::sistence::memory::model::recollection::{Content, RecollectionQuery};
use crate::sistence::{memory::model::recollection::RecollectionEntry, types::RecollectionId};

pub trait RecollectionService {
    fn store(&self, entry: &RecollectionEntry) -> SistenceMemoryResult<RecollectionId>;
    fn retrieve(&self, id: RecollectionId) -> SistenceMemoryResult<RecollectionEntry>;
    fn update(&self, id: RecollectionId, content: Content) -> SistenceMemoryResult<RecollectionId>;
    fn archive(&self, id: RecollectionId) -> SistenceMemoryResult<()>;

    fn search(&self, query: RecollectionQuery) -> SistenceMemoryResult<Vec<RecollectionEntry>>;
    fn find_similar(
        &self,
        content: &Content,
        limit: usize,
    ) -> SistenceMemoryResult<Vec<RecollectionEntry>>;
}

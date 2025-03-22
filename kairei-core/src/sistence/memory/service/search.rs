use crate::sistence::{
    memory::{error::SistenceMemoryResult, model::search::SearchQuery},
    types::RecollectionId,
};

pub trait SearchEngine {
    fn search(&self, query: &SearchQuery) -> SistenceMemoryResult<Vec<RecollectionId>>;
}

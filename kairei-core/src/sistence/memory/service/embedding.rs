use crate::sistence::{memory::error::SistenceMemoryResult, types::RecollectionId};

pub trait EmbeddingService {
    fn generate_embedding(&self, text: &str) -> SistenceMemoryResult<Vec<f32>>;
    fn get_embedding(&self, entry_id: RecollectionId) -> SistenceMemoryResult<Vec<f32>>;
}

pub trait EmbeddingGenerator: Send + Sync {
    fn generator_name(&self) -> &str;
    fn generate(&self, text: &str) -> SistenceMemoryResult<Vec<f32>>;
}

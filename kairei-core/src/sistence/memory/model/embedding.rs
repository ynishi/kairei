use chrono::{DateTime, Utc};

use crate::sistence::types::RecollectionId;

// Structs
#[derive(Clone, Debug)]
pub struct Embedding {
    /// ID of the associated recollection entry
    entry_id: RecollectionId,
    /// Vector embeddings for semantic search
    embedding: Vec<f32>,
    /// Timestamp when this embedding was generated
    generated_at: DateTime<Utc>,
    /// Information about the generator that produced this embedding
    generator_info: EmbeddingGeneratorInfo,
}

// Enum (必要に応じて)
#[derive(Clone, Debug)]
pub enum EmbeddingType {
    SentenceTransformer,
}

#[derive(Clone, Debug)]
pub struct EmbeddingGeneratorInfo {
    /// Unique identifier for the generator
    generator_id: String,
    /// Type of generator (e.g., model name, algorithm)
    generator_type: String,
    /// Confidence score of this generator
    confidence: f32,
}

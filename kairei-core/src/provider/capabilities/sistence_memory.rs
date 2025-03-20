use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Memory item identifier type
pub type MemoryId = String;

use crate::provider::capabilities::shared_memory::SharedMemoryError;
use crate::provider::capabilities::storage::StorageError;
use crate::provider::plugin::ProviderPlugin;

/// Errors that can occur in the Sistence Memory system
#[derive(Debug, Error)]
pub enum SistenceMemoryError {
    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),

    #[error("Shared memory error: {0}")]
    SharedMemoryError(#[from] SharedMemoryError),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("LLM processing error: {0}")]
    LlmError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Content type of memory items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentType {
    /// Plain text content
    Text,
    /// JSON structured data
    Json,
    /// Binary data
    Binary,
    /// Mixed/composite content
    Mixed,
    /// Custom content type
    Custom(String),
}

/// Type of memory item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemType {
    /// General information
    Information,
    /// Decision or conclusion
    Decision,
    /// Specific event
    Event,
    /// Knowledge or fact
    Knowledge,
    /// Thought process
    Thought,
    /// Task or action
    Task,
    /// Conversation transcript
    Conversation,
    /// Custom type
    Custom(String),
}

/// Verification level of information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationLevel {
    /// Unverified information
    Unverified,
    /// Partially verified
    PartiallyVerified,
    /// Verified by system
    SystemVerified,
    /// Verified by human
    HumanVerified,
    /// Formally validated
    FormallyValidated,
}

/// Retention policy for memory items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetentionPolicy {
    /// Transient - can be removed when no longer needed
    Transient,
    /// Standard retention
    Standard,
    /// Important - retain longer than standard
    Important,
    /// Critical - never automatically remove
    Critical,
    /// Custom policy with specific rules
    Custom(String),
}

/// Type of query for contextual search
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryType {
    /// Simple keyword search
    Keyword,
    /// Semantic similarity search
    Semantic,
    /// Factual information retrieval
    Factual,
    /// Knowledge expansion
    Expansion,
    /// Decision support
    Decision,
}

/// Source of a memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Type of source (user, system, agent, etc.)
    pub source_type: String,
    /// Identifier of the source
    pub source_id: String,
    /// Additional source details
    pub details: Option<String>,
    /// Reliability score (0.0-1.0)
    pub reliability: f32,
}

/// Reference to another item or external source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// Type of reference
    pub ref_type: String,
    /// ID of referenced item or external identifier
    pub ref_id: String,
    /// Context of the reference
    pub context: Option<String>,
    /// Strength of the reference (0.0-1.0)
    pub strength: f32,
}

/// Record of an access to a memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessEvent {
    /// When the access occurred
    pub timestamp: SystemTime,
    /// Type of access (read, write, reference)
    pub access_type: String,
    /// Context in which the access occurred
    pub context_id: Option<String>,
    /// Accessor identity
    pub accessor_id: Option<String>,
}

/// Access statistics for a memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessStats {
    /// Total number of accesses
    pub access_count: u32,
    /// Time of last access
    pub last_accessed: Option<SystemTime>,
    /// Recent access events
    pub recent_accesses: Vec<AccessEvent>,
    /// Average access frequency
    pub access_frequency: f32,
}

/// Filters for search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Filter by item types
    pub item_types: Option<Vec<ItemType>>,
    /// Filter by topics
    pub topics: Option<Vec<String>>,
    /// Filter by source
    pub source: Option<Source>,
    /// Filter by time range (start)
    pub time_start: Option<SystemTime>,
    /// Filter by time range (end)
    pub time_end: Option<SystemTime>,
    /// Minimum importance score
    pub min_importance: Option<f32>,
    /// Custom filters as key-value pairs
    pub custom_filters: Option<HashMap<String, String>>,
}

/// Sistence profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SistenceProfile {
    /// Sistence instance name
    pub name: String,
    /// Specializations or expertise areas
    pub specializations: Vec<String>,
    /// Experience level
    pub experience_level: String,
    /// Personality traits
    pub personality_traits: Vec<String>,
    /// Value priorities (key=value name, value=priority 0-1)
    pub value_priorities: HashMap<String, f32>,
    /// Knowledge areas
    pub knowledge_areas: Vec<String>,
}

/// Search strategy policy for public API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchStrategy {
    /// Balance between recency and relevance
    Balanced,
    /// Focus on most recent items
    Recency,
    /// Focus on most relevant items
    Relevance,
    /// Focus on most important items
    Importance,
    /// Custom strategy
    Custom(String),
}

/// Context for memory search and retrieval operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchContext {
    /// Current session or conversation ID
    pub context_id: String,
    /// Current topics under discussion
    pub current_topics: Vec<String>,
    /// Recently accessed items
    pub recent_items: Vec<MemoryId>,

    /// Search text
    pub query_text: Option<String>,
    /// Type of query
    pub query_type: QueryType,
    /// Search strategy to use
    pub strategy: Option<SearchStrategy>,

    /// Participants in the current context
    pub participants: Vec<String>,
    /// Current activity or task
    pub current_activity: Option<String>,
    /// Temporal aspects of the context
    pub temporal_context: TemporalContext,

    /// Sistence agent profile
    pub sistence_profile: Option<SistenceProfile>,
    /// Current goals
    pub goals: Option<Vec<String>>,
    /// Conversation summary
    pub conversation_summary: Option<String>,
    /// Environmental factors
    pub environment_factors: Option<HashMap<String, String>>,
}

/// Structured result from memory retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredResult {
    /// Retrieved memory items
    pub items: Vec<MemoryItem>,
    /// Generated summary of items
    pub summary: Option<String>,
    /// Knowledge graph representation
    pub knowledge_graph: Option<KnowledgeNode>,

    /// Confidence in result relevance (0.0-1.0)
    pub confidence: f32,
    /// Context match score (0.0-1.0)
    pub context_match: f32,
    /// Query execution statistics
    pub execution_stats: Option<serde_json::Value>,
}

/// Knowledge node for graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    /// Node identifier
    pub id: String,
    /// Node label
    pub label: String,
    /// Node type
    pub node_type: String,
    /// Node properties
    pub properties: HashMap<String, String>,
    /// Connected nodes
    pub connections: Vec<KnowledgeConnection>,
}

/// Connection between knowledge nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConnection {
    /// Target node ID
    pub target_id: String,
    /// Relationship type
    pub relation_type: String,
    /// Connection strength (0.0-1.0)
    pub strength: f32,
    /// Direction (true = outgoing, false = incoming)
    pub is_outgoing: bool,
}

/// Temporal context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    /// Current timestamp reference
    pub current_time: Option<SystemTime>,
    /// Time focus (past, present, future)
    pub time_focus: Option<String>,
    /// Relevant time periods for context
    pub relevant_periods: Vec<(SystemTime, SystemTime)>,
    /// Historical context description
    pub historical_context: Option<String>,
}

/// Link between memory items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemLink {
    /// Source item ID
    pub source_id: MemoryId,
    /// Target item ID
    pub target_id: MemoryId,
    /// Relationship type
    pub relation_type: String,
    /// Link strength (0.0-1.0)
    pub strength: f32,
    /// When the link was created
    pub created_at: SystemTime,
    /// Additional link metadata
    pub metadata: Option<HashMap<String, String>>,
    pub is_bidirectional: bool,
    pub context: Option<String>,
}

/// Filters for commit log operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitLogFilters {
    /// Filter by operation types
    pub operations: Option<Vec<String>>,
    /// Filter by entity types
    pub entity_types: Option<Vec<String>>,
    /// Filter by time range (start)
    pub time_start: Option<SystemTime>,
    /// Filter by time range (end)
    pub time_end: Option<SystemTime>,
    /// Filter by source identifier
    pub source_id: Option<String>,
}

/// Memory system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of items
    pub total_items: usize,
    /// Items by type
    pub items_by_type: HashMap<String, usize>,
    /// Total storage used
    pub storage_used: usize,
    /// Average importance score
    pub avg_importance: f32,
    /// Access statistics
    pub access_stats: HashMap<String, usize>,
}

/// Topic distribution in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDistribution {
    /// Topics and their counts
    pub topics: HashMap<String, usize>,
    /// Topics by importance
    pub topics_by_importance: Vec<(String, f32)>,
    /// Topic relationships
    pub topic_relations: Vec<(String, String, f32)>,
}

/// Memory importance change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemImportanceChange {
    /// Item ID
    pub item_id: String,
    /// Previous importance score
    pub previous_score: f32,
    /// New importance score
    pub new_score: f32,
    /// Reason for change
    pub reason: String,
}

/// Distribution of importance scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceDistribution {
    /// Average base importance
    pub avg_base_importance: f32,
    /// Distribution by range
    pub distribution: HashMap<String, usize>,
    /// Top items by importance
    pub top_items: Vec<(String, f32)>,
    /// Recent changes
    pub recent_changes: Vec<ItemImportanceChange>,
}

/// Enhanced metadata generated by LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMetadata {
    /// Generated topics
    pub suggested_topics: Vec<String>,
    /// Generated tags
    pub suggested_tags: HashMap<String, String>,
    /// Suggested related items
    pub suggested_relations: Vec<String>,
    /// Extracted entities
    pub entities: Vec<String>,
    /// Confidence in suggestions (0.0-1.0)
    pub confidence: f32,
}

/// Statistics from indexing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Number of items indexed
    pub items_indexed: usize,
    /// Number of items skipped
    pub items_skipped: usize,
    /// Number of errors encountered
    pub errors: usize,
    /// Time taken for indexing
    pub duration: Duration,
}

/// Memory cleanup statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    /// Number of items removed
    pub items_removed: usize,
    /// Number of items kept
    pub items_kept: usize,
    /// Storage space reclaimed
    pub space_reclaimed: usize,
    /// Time taken for cleanup
    pub duration: Duration,
}

/// A memory item with content and metadata (public API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique identifier
    pub id: MemoryId,
    /// When the item was created
    pub created_at: SystemTime,
    /// When the item was last updated
    pub updated_at: SystemTime,

    /// The actual content of the memory item
    pub content: String,
    /// Content type (text, JSON, binary)
    pub content_type: ContentType,
    /// Structured content representation (if available)
    pub structured_content: Option<serde_json::Value>,

    /// Type of memory item
    pub item_type: ItemType,
    /// Topics or categories
    pub topics: Vec<String>,
    /// Custom tags
    pub tags: HashMap<String, String>,

    /// Source information
    pub source: Source,
    /// References to other items or external sources
    pub references: Vec<Reference>,
    /// Related items by ID
    pub related_items: Vec<MemoryId>,

    /// Simplified importance score for public API
    pub importance: ImportanceScore,
    /// Last access time
    pub last_accessed: Option<SystemTime>,
    /// Access count
    pub access_count: u32,

    /// Time-to-live for this item (None = permanent)
    pub ttl: Option<Duration>,
    /// Retention policy
    pub retention_policy: RetentionPolicy,
}

/// Memory item with contextual ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedMemoryItem {
    /// The memory item
    pub item: MemoryItem,
    /// Rank score for current context (0.0-1.0)
    pub rank_score: f32,
    /// Rank components breakdown
    pub rank_components: HashMap<String, f32>,
    /// Rank explanation
    pub rank_explanation: Option<String>,
}

/// Policy for importance evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportancePolicy {
    /// Standard balanced evaluation
    Standard,
    /// Focus on factual reliability
    FactualFocus,
    /// Focus on novelty and uniqueness
    NoveltyFocus,
    /// Focus on utility for current goals
    UtilityFocus,
    /// Custom weights
    Custom(String),
}

/// Simple importance metrics for public API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceScore {
    /// Overall importance score (0.0-1.0)
    pub score: f32,
    /// Base score independent of context
    pub base_score: f32,
    /// Context-dependent score component
    pub context_score: Option<f32>,
    /// Brief reason for importance
    pub reason: Option<String>,
    /// When evaluated
    pub evaluated_at: SystemTime,
}

/// Types of importance evaluators (internal API)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportanceEvaluatorType {
    /// Based on intrinsic properties
    Intrinsic,
    /// Based on usage patterns
    Usage,
    /// Based on network analysis
    Network,
    /// Based on context
    Contextual,
    /// Based on emotional factors
    Emotional,
    /// Combined evaluator
    Combined,
}

/// The SistenceMemoryCapability defines the interface for the middle layer
/// of the 3-layer memory architecture. This layer focuses on metadata enrichment,
/// context-aware search, and connecting the fast working memory with the
/// persistent commit log.
#[async_trait]
pub trait SistenceMemoryCapability: ProviderPlugin + Send + Sync {
    // === Basic CRUD Operations ===

    /// Store a new memory item with rich metadata
    async fn store(&self, item: MemoryItem) -> Result<MemoryId, SistenceMemoryError>;

    /// Retrieve a memory item by ID
    async fn retrieve(&self, id: &MemoryId) -> Result<MemoryItem, SistenceMemoryError>;

    /// Update an existing memory item
    async fn update(&self, item: MemoryItem) -> Result<(), SistenceMemoryError>;

    /// Delete a memory item
    async fn delete(&self, id: &MemoryId) -> Result<(), SistenceMemoryError>;

    /// Check if a memory item exists
    async fn exists(&self, id: &MemoryId) -> Result<bool, SistenceMemoryError>;

    /// Update importance score for a memory item
    async fn update_importance(
        &self,
        id: &MemoryId,
        policy: Option<ImportancePolicy>,
    ) -> Result<ImportanceScore, SistenceMemoryError>;

    // === Search Operations ===

    /// Search for memory items
    async fn search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>, SistenceMemoryError>;

    /// Search with context awareness
    async fn search_with_context(
        &self,
        query: &str,
        context: SearchContext,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>, SistenceMemoryError>;

    /// Find related items
    async fn find_related(
        &self,
        item_id: &MemoryId,
        max_results: Option<usize>,
    ) -> Result<Vec<MemoryItem>, SistenceMemoryError>;

    /// Get items relevant to the current context
    async fn get_relevant_for_context(
        &self,
        context: SearchContext,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>, SistenceMemoryError>;

    // === Metadata Management ===

    /// Add topics to an item
    async fn add_topics(
        &self,
        item_id: &MemoryId,
        topics: Vec<String>,
    ) -> Result<(), SistenceMemoryError>;

    /// Add tags to an item
    async fn add_tags(
        &self,
        item_id: &MemoryId,
        tags: HashMap<String, String>,
    ) -> Result<(), SistenceMemoryError>;

    /// Link items together
    async fn link_items(
        &self,
        source_id: &MemoryId,
        target_ids: Vec<MemoryId>,
        relation_type: Option<String>,
    ) -> Result<(), SistenceMemoryError>;

    // === Working Memory Integration ===

    /// Index items from working memory
    async fn index_from_working_memory(
        &self,
        namespace: &str,
        pattern: Option<&str>,
    ) -> Result<IndexStats, SistenceMemoryError>;

    /// Promote an item from working memory
    async fn promote_from_working_memory(
        &self,
        key: &str,
        namespace: &str,
    ) -> Result<String, SistenceMemoryError>;

    /// Store item to working memory
    async fn store_to_working_memory(
        &self,
        item_id: &MemoryId,
        namespace: &str,
        key: &str,
    ) -> Result<(), SistenceMemoryError>;

    // === LLM-Enhanced Functions ===

    /// Enhance metadata using lightweight LLM
    async fn enhance_metadata(
        &self,
        item_id: &MemoryId,
    ) -> Result<EnhancedMetadata, SistenceMemoryError>;

    /// Build optimized context for LLM from memory items
    async fn build_context(
        &self,
        context: SearchContext,
        max_tokens: usize,
        strategy: Option<SearchStrategy>,
    ) -> Result<String, SistenceMemoryError>;

    // === Management Functions ===

    /// Clean up expired items
    async fn cleanup_expired(&self) -> Result<CleanupStats, SistenceMemoryError>;

    /// Get memory statistics
    async fn get_stats(&self) -> Result<MemoryStats, SistenceMemoryError>;
}

/// Basic metadata for memory items (public API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicMetadata {
    /// Type of memory item
    pub item_type: String,
    /// Topics or categories
    pub topics: Vec<String>,
    /// Custom tags
    pub tags: HashMap<String, String>,
    /// Source information
    pub source: Option<String>,
    /// Time-to-live for this item (None = permanent)
    pub ttl: Option<Duration>,
}

/// Interface for importance evaluation functionality (public API)
#[async_trait]
pub trait ImportanceEvaluator: Send + Sync {
    /// Evaluate importance for an item
    fn evaluate_importance(&self, item: &MemoryItem) -> ImportanceScore;

    /// Evaluate contextual importance
    fn evaluate_contextual_importance(
        &self,
        item: &MemoryItem,
        context: &SearchContext,
    ) -> ImportanceScore;

    /// Recalculate importance for all items
    async fn recalculate_all_importance(
        &self,
    ) -> Result<ImportanceDistribution, SistenceMemoryError>;

    /// Update importance for a specific item
    async fn update_item_importance(
        &self,
        item_id: &MemoryId,
        policy: Option<ImportancePolicy>,
    ) -> Result<ImportanceScore, SistenceMemoryError>;
}

//! RelevantMemoryCapability provides the internal implementation layer of the 3-layer memory architecture.
//!
//! This capability implements a stateless metadata-enriched memory layer that sits between
//! the fast SharedMemory (working memory) and the persistent CommitLog storage.
//! It focuses on rich metadata management, context-aware search, and connecting
//! information across different memory layers.
//!
//! # Key Features
//!
//! - Rich metadata management with multiple classification dimensions
//! - Context-aware search using lightweight LLMs
//! - Importance evaluation with both intrinsic and contextual factors
//! - Seamless integration with SharedMemory and persistent storage layers
//!
//! # Implementation Notes
//!
//! This is the internal implementation layer that provides rich functionality
//! for the simpler public SistenceMemoryCapability interface.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::provider::capabilities::sistence_memory::{
    ContentType, EnhancedMetadata, ImportanceDistribution, ImportanceEvaluatorType, ItemType,
    KnowledgeNode, MemoryId, RetentionPolicy, SearchContext, SearchFilters, SistenceMemoryError,
    Source, StructuredResult, VerificationLevel,
};
use crate::provider::plugin::ProviderPlugin;

use super::sistence_memory::ItemLink;

/// Strategy for building optimized context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextStrategy {
    /// Focus on most recent items
    Recency,
    /// Focus on most important items
    Importance,
    /// Balance between recency and importance
    Balanced,
    /// Focus on items most relevant to current query
    QueryRelevance,
    /// Focus on items most relevant to current activity
    ActivityRelevance,
    /// Custom strategy with weights
    Custom(HashMap<String, f32>),
}

/// Temporal focus for context building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeFocus {
    /// Focus on past events
    Past,
    /// Focus on current situation
    Present,
    /// Focus on future possibilities
    Future,
    /// Compare across time periods
    Comparative,
}

/// Options for metadata enhancement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementOptions {
    /// Generate additional topics
    pub enhance_topics: bool,
    /// Generate additional tags
    pub enhance_tags: bool,
    /// Extract entities
    pub extract_entities: bool,
    /// Suggest relationships
    pub suggest_relations: bool,
    /// Generate a summary of the content
    pub generate_summary: bool,
    /// Extract key points from the content
    pub extract_key_points: bool,
    /// Analyze sentiment of the content
    pub analyze_sentiment: bool,
    /// Update the item with enhanced metadata
    pub update_item: bool,
    /// Confidence threshold for suggestions (0.0-1.0)
    pub confidence_threshold: f32,
    /// Maximum number of suggestions per category
    pub max_suggestions: usize,
}

/// Format options for working memory integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkingMemoryFormat {
    /// Simple key-value format
    Simple,
    /// JSON with basic metadata
    BasicJson,
    /// JSON with extended metadata
    RichJson,
    /// Custom format with specific fields
    Custom(Vec<String>),
}

/// Level of metadata enhancement when integrating from other layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnhancementLevel {
    /// Minimal processing, preserve as-is
    Minimal,
    /// Basic metadata enrichment
    Basic,
    /// Standard metadata enrichment
    Standard,
    /// Full metadata enrichment with LLM
    Complete,
}

/// Detailed memory item for internal use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedMemoryItem {
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
    pub references: Vec<DetailedReference>,
    /// Related items by ID
    pub related_items: Vec<MemoryId>,

    /// Multi-dimensional importance evaluation
    pub importance: DetailedImportanceEvaluation,
    /// Access statistics
    pub access_stats: DetailedAccessStats,

    /// Time-to-live for this item (None = permanent)
    pub ttl: Option<Duration>,
    /// Retention policy
    pub retention_policy: RetentionPolicy,
}

/// Reference to another item or external source (internal API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedReference {
    /// Type of reference
    pub ref_type: String,
    /// ID of referenced item or external identifier
    pub ref_id: String,
    /// Context of the reference
    pub context: Option<String>,
    /// Strength of the reference (0.0-1.0)
    pub strength: f32,
    /// When the reference was created
    pub created_at: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Multi-dimensional importance evaluation (internal API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedImportanceEvaluation {
    /// Base score independent of context (0.0-1.0)
    pub base_score: f32,
    /// Context-dependent score (0.0-1.0)
    pub context_score: f32,
    /// Intrinsic metrics components
    pub intrinsic_components: IntrinsicMetrics,
    /// Usage statistics components
    pub usage_components: DetailedAccessStats,
    /// Reference network components
    pub reference_components: ReferenceNetwork,
    /// Contextual relevance components
    pub contextual_components: Option<ContextualRelevance>,
    /// Emotional factors components
    pub emotional_components: Option<EmotionalFactors>,
    /// When evaluation was performed
    pub evaluated_at: SystemTime,
    /// Context of evaluation
    pub evaluation_context: Option<String>,
}

/// Intrinsic importance metrics - relatively stable aspects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrinsicMetrics {
    /// First occurrence time
    pub first_occurrence: SystemTime,
    /// Creation context
    pub creation_context: String,
    /// Source reliability (0.0-1.0)
    pub source_reliability: f32,
    /// Verification level
    pub verification_level: VerificationLevel,
    /// Criticality - how critical this information is (0.0-1.0)
    pub criticality: f32,
    /// Novelty - how unique or new this information is (0.0-1.0)
    pub novelty: f32,
    /// Permanence - long-term value (0.0-1.0)
    pub permanence: f32,
    /// Scope breadth - how broad the impact is (0.0-1.0)
    pub scope_breadth: f32,
}

/// Reference network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceNetwork {
    /// Number of times this item is referenced
    pub reference_count: u32,
    /// Diversity of reference sources (0.0-1.0)
    pub reference_diversity: f32,
    /// Overall strength of citations (0.0-1.0)
    pub citation_strength: f32,
    /// Centrality in the knowledge network (0.0-1.0)
    pub network_centrality: f32,
}

/// Context-dependent relevance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualRelevance {
    /// Match with current topics (0.0-1.0)
    pub topic_match: f32,
    /// Temporal relevance to current context (0.0-1.0)
    pub temporal_relevance: f32,
    /// Relevance to agent goals (0.0-1.0)
    pub agent_relevance: f32,
    /// Relevance to current query (0.0-1.0)
    pub query_relevance: f32,
}

/// Emotional factors affecting importance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalFactors {
    /// Emotional intensity (0.0-1.0)
    pub emotional_intensity: f32,
    /// Sentiment (-1.0 to 1.0)
    pub sentiment: f32,
    /// Personal significance (0.0-1.0)
    pub personal_significance: f32,
    /// Social resonance (0.0-1.0)
    pub social_resonance: f32,
}

/// Detailed access statistics for a memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedAccessStats {
    /// Total number of accesses
    pub access_count: u32,
    /// Time of last access
    pub last_accessed: Option<SystemTime>,
    /// Recent access events
    pub recent_accesses: Vec<DetailedAccessEvent>,
    /// Average access frequency
    pub access_frequency: f32,
    /// Access patterns analysis
    pub pattern_analysis: Option<AccessPatternAnalysis>,
}

/// Detailed record of an access to a memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedAccessEvent {
    /// When the access occurred
    pub timestamp: SystemTime,
    /// Type of access (read, write, reference)
    pub access_type: String,
    /// Context in which the access occurred
    pub context_id: Option<String>,
    /// Accessor identity
    pub accessor_id: Option<String>,
    /// Additional access metadata
    pub metadata: HashMap<String, String>,
    /// Related queries or operations
    pub related_operation: Option<String>,
}

/// Access pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPatternAnalysis {
    /// Most common access time of day
    pub common_time: String,
    /// Periodicity of access (0.0-1.0)
    pub periodicity: f32,
    /// Clustered access pattern (0.0-1.0)
    pub clustering: f32,
    /// Recency-weighted frequency
    pub recency_weighted_frequency: f32,
    /// Associated context patterns
    pub context_patterns: HashMap<String, f32>,
}

/// Request context for a stateless memory service request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Relevant instance ID
    pub instance_id: String,
    /// Relevant profile information
    pub profile: RelevantProfile,
    /// Current operational context
    pub current_context: CurrentContext,
    /// Request-specific metadata
    pub request_metadata: HashMap<String, String>,
    /// Request timestamp
    pub timestamp: SystemTime,
}

/// Relevant instance profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantProfile {
    /// Instance name
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
    pub knowledge_areas: Vec<KnowledgeArea>,
}

/// Knowledge area with expertise level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeArea {
    /// Area name
    pub name: String,
    /// Expertise level (0-1)
    pub expertise_level: f32,
    /// Related concepts
    pub related_concepts: Vec<String>,
}

/// Current operational context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentContext {
    /// Current activity description
    pub current_activity: Option<String>,
    /// Current goals
    pub current_goals: Vec<String>,
    /// Conversation summary
    pub conversation_summary: Option<String>,
    /// Recent topics discussed
    pub recent_topics: Vec<String>,
    /// Recent memory items accessed
    pub recent_items: Vec<String>,
    /// Session duration
    pub session_duration: Duration,
    /// Environmental factors
    pub environment_factors: HashMap<String, String>,
}

/// The RelevantMemoryCapability defines the internal interface for the richly detailed
/// middle layer of the 3-layer memory architecture. This is where the complex
/// metadata enrichment, contextual relevance scoring, and memory optimization happens.
///
/// Unlike the more streamlined SistenceMemoryCapability public interface, this trait
/// exposes the full richness of the underlying implementation.
#[async_trait]
pub trait RelevantMemoryCapability: ProviderPlugin + Send + Sync {
    // === Core Memory Operations ===

    /// Store a new memory item with rich metadata
    async fn store_memory_item(
        &self,
        item: DetailedMemoryItem,
    ) -> Result<MemoryId, SistenceMemoryError>;

    /// Retrieve a specific memory item by ID with full metadata
    async fn retrieve_memory_item(
        &self,
        id: &MemoryId,
    ) -> Result<DetailedMemoryItem, SistenceMemoryError>;

    /// Update an existing memory item preserving all metadata
    async fn update_memory_item(&self, item: DetailedMemoryItem)
    -> Result<(), SistenceMemoryError>;

    /// Delete a memory item and its references
    async fn delete_memory_item(&self, id: &MemoryId) -> Result<(), SistenceMemoryError>;

    // === Advanced Search Operations ===

    /// Search for memory items with detailed relevance scoring
    async fn search_with_relevance(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        context: Option<SearchContext>,
        max_results: usize,
        min_relevance: Option<f32>,
    ) -> Result<Vec<(DetailedMemoryItem, f32, HashMap<String, f32>)>, SistenceMemoryError>;

    /// Search with structured result including knowledge graph generation
    async fn contextual_search(
        &self,
        query: &str,
        context: SearchContext,
    ) -> Result<StructuredResult, SistenceMemoryError>;

    /// Get memory items relevant to a specific context even without a query
    async fn get_context_relevant(
        &self,
        context: SearchContext,
        max_items: usize,
        min_relevance: f32,
    ) -> Result<StructuredResult, SistenceMemoryError>;

    // === Item Relationship Management ===

    /// Build a semantic graph of related items
    async fn build_relationship_graph(
        &self,
        starting_item_ids: Vec<String>,
        max_depth: usize,
        min_relationship_strength: f32,
    ) -> Result<KnowledgeNode, SistenceMemoryError>;

    /// Find items semantically related to the given item
    async fn find_semantically_related(
        &self,
        item_id: &MemoryId,
        max_results: usize,
        min_similarity: f32,
    ) -> Result<Vec<(DetailedMemoryItem, f32)>, SistenceMemoryError>;

    /// Create explicit links between memory items with specified relationship type
    async fn create_item_links(&self, links: Vec<ItemLink>) -> Result<(), SistenceMemoryError>;

    /// Get all links for a specific item
    async fn get_all_item_links(
        &self,
        item_id: &MemoryId,
        include_incoming: bool,
        include_outgoing: bool,
    ) -> Result<Vec<ItemLink>, SistenceMemoryError>;

    // === Context Building & Optimization ===

    /// Generate an optimized prompt context from memory items
    async fn generate_optimized_context(
        &self,
        context: SearchContext,
        max_tokens: usize,
        context_strategy: ContextStrategy,
        include_metadata: bool,
    ) -> Result<String, SistenceMemoryError>;

    /// Build temporal context for a specific time range or focus
    async fn build_temporal_context(
        &self,
        time_focus: TimeFocus,
        time_range: Option<(SystemTime, SystemTime)>,
        related_topics: Option<Vec<String>>,
    ) -> Result<crate::provider::capabilities::sistence_memory::TemporalContext, SistenceMemoryError>;

    // === Metadata Enhancement ===

    /// Enhance metadata for a memory item using lightweight LLM
    async fn enhance_item_metadata(
        &self,
        item_id: &MemoryId,
        enhancement_options: EnhancementOptions,
    ) -> Result<EnhancedMetadata, SistenceMemoryError>;

    /// Update importance evaluation for a specific item
    async fn reevaluate_importance(
        &self,
        item_id: &MemoryId,
        context: Option<SearchContext>,
    ) -> Result<DetailedImportanceEvaluation, SistenceMemoryError>;

    // === Integration with Other Layers ===

    /// Convert a memory item to working memory compatible format
    async fn prepare_for_working_memory(
        &self,
        item_id: &MemoryId,
        format: WorkingMemoryFormat,
    ) -> Result<serde_json::Value, SistenceMemoryError>;

    /// Process and integrate a working memory item into relevant memory
    async fn process_from_working_memory(
        &self,
        key: &str,
        namespace: &str,
        enhancement_level: EnhancementLevel,
    ) -> Result<String, SistenceMemoryError>;

    /// Export memory item to commit log compatible format
    async fn prepare_for_commit_log(
        &self,
        item_id: &MemoryId,
        include_details: bool,
    ) -> Result<serde_json::Value, SistenceMemoryError>;

    /// Process and integrate a commit log entry into relevant memory
    async fn process_from_commit_log(
        &self,
        entry_id: &str,
        enhancement_level: EnhancementLevel,
    ) -> Result<String, SistenceMemoryError>;
}

/// Interface for advanced importance evaluation functionality
#[async_trait]
pub trait DetailedImportanceEvaluator: Send + Sync {
    /// Calculate base importance score independent of context
    fn calculate_base_importance(&self, item: &DetailedMemoryItem) -> f32;

    /// Calculate context-dependent importance
    fn calculate_contextual_importance(
        &self,
        item: &DetailedMemoryItem,
        context: &SearchContext,
    ) -> f32;

    /// Recalculate importance for all items
    async fn recalculate_all_importance(
        &self,
    ) -> Result<ImportanceDistribution, SistenceMemoryError>;

    /// Update importance for a specific item
    async fn update_item_importance(
        &self,
        item_id: &MemoryId,
        evaluators: Vec<ImportanceEvaluatorType>,
    ) -> Result<DetailedImportanceEvaluation, SistenceMemoryError>;

    /// Evaluate first impression metrics
    fn evaluate_first_impression(&self, item: &DetailedMemoryItem) -> f32;

    /// Evaluate network centrality
    fn evaluate_network_centrality(&self, item_id: &MemoryId) -> f32;

    /// Evaluate access patterns
    fn evaluate_access_patterns(&self, item_id: &MemoryId) -> f32;

    /// Evaluate contextual relevance using lightweight LLM
    async fn evaluate_contextual_relevance(
        &self,
        item: &DetailedMemoryItem,
        context: &SearchContext,
    ) -> f32;
}

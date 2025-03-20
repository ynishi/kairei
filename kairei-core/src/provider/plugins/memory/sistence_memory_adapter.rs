use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use serde_json::Value;
use tracing;
use uuid::Uuid;

use crate::provider::{
    capabilities::common::CapabilityType,
    llm::LLMResponse,
    plugin::{PluginContext, ProviderPlugin},
    provider::Section,
    types::ProviderResult,
};

use crate::provider::capabilities::relevant_memory::{
    ContextStrategy, DetailedAccessStats, DetailedImportanceEvaluation, DetailedMemoryItem,
    DetailedReference, EnhancementLevel, EnhancementOptions, RelevantMemoryCapability,
    WorkingMemoryFormat,
};
use crate::provider::capabilities::sistence_memory::{
    CleanupStats, EnhancedMetadata, ImportancePolicy, ImportanceScore, IndexStats, ItemLink,
    MemoryId, MemoryItem, MemoryStats, Reference, SearchContext, SearchFilters, SearchStrategy,
    SistenceMemoryCapability, SistenceMemoryError,
};
use crate::provider::plugins::memory::sistence_memory_plugin::SistenceMemoryConfig;

/// Context information for the SistenceMemory system
pub struct SistenceContext {
    /// Unique identifier for this context
    pub id: String,
    /// Name of the context
    pub name: String,
    /// Description of the context
    pub description: String,
    /// Additional data associated with the context
    pub data: Value,
}

impl Default for SistenceContext {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "Default Context".to_string(),
            description: "Default context for SistenceMemory".to_string(),
            data: Value::Null,
        }
    }
}

/// Adapter that implements the SistenceMemoryCapability trait
/// by delegating to a RelevantMemoryCapability implementation.
///
/// This adapter provides the simplified public API while leveraging
/// the rich functionality of the internal implementation.
pub struct SistenceMemoryAdapter {
    /// The underlying RelevantMemoryCapability implementation
    relevant_memory: Arc<dyn RelevantMemoryCapability>,
    /// Configuration for the adapter
    _config: SistenceMemoryConfig,
    /// Context information
    _context: SistenceContext,
    /// Default enhancement level for metadata operations
    default_enhancement: EnhancementLevel,
    /// Default context strategy for search operations
    default_context_strategy: ContextStrategy,
    /// Default number of results for search operations
    default_result_limit: usize,
}

impl SistenceMemoryAdapter {
    /// Create a new SistenceMemoryAdapter
    pub fn new(
        relevant_memory: Arc<dyn RelevantMemoryCapability>,
        config: SistenceMemoryConfig,
        context: Option<SistenceContext>,
    ) -> Self {
        Self {
            relevant_memory,
            _config: config,
            _context: context.unwrap_or_default(),
            default_enhancement: EnhancementLevel::Standard,
            default_context_strategy: ContextStrategy::Balanced,
            default_result_limit: 20,
        }
    }

    /// Helper method to convert errors to SistenceMemoryError
    ///
    /// This method leverages the From trait implementations for SistenceMemoryError
    /// to provide a consistent way to handle errors throughout the adapter.
    fn convert_error<T, E: Into<SistenceMemoryError>>(
        &self,
        result: Result<T, E>,
    ) -> Result<T, SistenceMemoryError> {
        result.map_err(Into::into)
    }

    /// Create a new SistenceMemoryAdapter with custom defaults
    pub fn with_defaults(
        relevant_memory: Arc<dyn RelevantMemoryCapability>,
        config: SistenceMemoryConfig,
        enhancement: EnhancementLevel,
        strategy: ContextStrategy,
        limit: usize,
    ) -> Self {
        Self {
            relevant_memory,
            _config: config,
            default_enhancement: enhancement,
            default_context_strategy: strategy,
            default_result_limit: limit,
            _context: SistenceContext::default(),
        }
    }

    // === Enhanced Policy Management ===

    /// Creates a custom importance policy with specified weights for different evaluator types.
    ///
    /// This allows fine-grained control over how importance scores are calculated, giving
    /// different weights to intrinsic properties, usage patterns, network analysis, etc.
    ///
    /// # Arguments
    /// * `weights` - HashMap mapping evaluator types to their relative weights (0.0-1.0)
    ///
    /// # Returns
    /// A custom ImportancePolicy that can be used with update_importance
    pub fn create_custom_importance_policy(
        &self,
        _weights: HashMap<
            crate::provider::capabilities::sistence_memory::ImportanceEvaluatorType,
            f32,
        >,
    ) -> crate::provider::capabilities::sistence_memory::ImportancePolicy {
        unimplemented!("Custom policy creation is not yet implemented")
    }

    /// Dynamically adjusts importance policy weights based on metadata and context.
    ///
    /// # Arguments
    /// * `base_policy` - The starting policy to adjust
    /// * `context` - The search context to consider for adjustments
    ///
    /// # Returns
    /// An adjusted policy with weights optimized for the given context
    pub fn adapt_importance_policy(
        &self,
        _base_policy: crate::provider::capabilities::sistence_memory::ImportancePolicy,
        _context: &crate::provider::capabilities::sistence_memory::SearchContext,
    ) -> crate::provider::capabilities::sistence_memory::ImportancePolicy {
        unimplemented!("Dynamic policy adaptation is not yet implemented")
    }

    // === Batch Operations ===

    /// Store multiple memory items in a single batch operation.
    ///
    /// This is more efficient than calling store() multiple times as it can
    /// optimize the underlying storage operations and reduce network calls.
    ///
    /// # Arguments
    /// * `items` - Vector of MemoryItems to store
    ///
    /// # Returns
    /// Vector of MemoryIds for the stored items, in the same order as the input
    #[tracing::instrument(level = "debug", skip(self, items), err)]
    pub async fn batch_store(
        &self,
        items: Vec<crate::provider::capabilities::sistence_memory::MemoryItem>,
    ) -> Result<
        Vec<crate::provider::capabilities::sistence_memory::MemoryId>,
        crate::provider::capabilities::sistence_memory::SistenceMemoryError,
    > {
        let mut results = Vec::with_capacity(items.len());
        let mut first_error = None;

        // Process each item and collect results
        for item in items {
            // Convert to internal format
            let detailed_item = self.simple_to_detailed(item);

            // Store the item using the error conversion helper
            match self.convert_error(self.relevant_memory.store_memory_item(detailed_item).await) {
                Ok(id) => results.push(id),
                Err(err) => {
                    // Store the first error encountered
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                    // Continue processing other items
                }
            }
        }

        // Return results or the first error encountered
        if let Some(err) = first_error {
            if results.is_empty() {
                // If no items were stored successfully, return the error
                Err(err)
            } else {
                // If some items were stored successfully, log the error but return the successful IDs
                // In a real implementation, we might want to return a more complex result type
                // that includes both successful IDs and errors
                eprintln!("Warning: Some items failed to store: {}", err);
                Ok(results)
            }
        } else {
            Ok(results)
        }
    }

    /// Retrieve multiple memory items in a single batch operation.
    ///
    /// # Arguments
    /// * `ids` - Vector of MemoryIds to retrieve
    ///
    /// # Returns
    /// HashMap mapping requested ids to their items (missing items are omitted)
    #[tracing::instrument(level = "debug", skip(self, ids), err)]
    pub async fn batch_retrieve(
        &self,
        ids: Vec<&crate::provider::capabilities::sistence_memory::MemoryId>,
    ) -> Result<
        HashMap<
            crate::provider::capabilities::sistence_memory::MemoryId,
            crate::provider::capabilities::sistence_memory::MemoryItem,
        >,
        crate::provider::capabilities::sistence_memory::SistenceMemoryError,
    > {
        let mut results = HashMap::with_capacity(ids.len());
        let mut first_error: Option<SistenceMemoryError> = None;

        // Process each ID and collect results
        for id in ids {
            // Retrieve the item using the error conversion helper
            match self.convert_error(self.relevant_memory.retrieve_memory_item(id).await) {
                Ok(detailed_item) => {
                    // Convert to simplified format and add to results
                    let simple_item = self.detailed_to_simple(detailed_item);
                    results.insert(id.clone(), simple_item);
                }
                Err(SistenceMemoryError::NotFound(_)) => {
                    // Skip items that don't exist
                    continue;
                }
                Err(err) => {
                    // Store the first non-NotFound error encountered
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                    // Continue processing other items
                }
            }
        }

        // Return results or the first error encountered
        if let Some(err) = first_error {
            if results.is_empty() {
                // If no items were retrieved successfully, return the error
                Err(err)
            } else {
                // If some items were retrieved successfully, log the error but return the successful items
                eprintln!("Warning: Some items failed to retrieve: {}", err);
                Ok(results)
            }
        } else {
            Ok(results)
        }
    }

    // === Metrics and Telemetry ===

    /// Collects and returns detailed metrics about adapter operations.
    ///
    /// This includes conversion stats, cache hits/misses, and performance metrics
    /// to help optimize and debug the adapter.
    ///
    /// # Returns
    /// A detailed metrics report structure
    #[allow(dead_code)]
    pub fn get_adapter_metrics(&self) -> HashMap<String, serde_json::Value> {
        unimplemented!("Adapter metrics collection is not yet implemented")
    }

    /// Records an event in the adapter's internal telemetry system.
    ///
    /// # Arguments
    /// * `event_type` - Type of event being recorded
    /// * `details` - Additional event details
    #[allow(dead_code)]
    pub fn record_telemetry_event(&self, _event_type: &str, _details: HashMap<String, String>) {
        unimplemented!("Telemetry event recording is not yet implemented")
    }

    // === Caching Strategy ===

    /// Configures the caching behavior of the adapter.
    ///
    /// # Arguments
    /// * `max_items` - Maximum number of items to keep in the cache
    /// * `ttl` - Time-to-live for cached items
    /// * `strategy` - Caching strategy (e.g., LRU, MRU, etc.)
    #[allow(dead_code)]
    pub fn configure_cache(&self, _max_items: usize, _ttl: std::time::Duration, _strategy: &str) {
        unimplemented!("Cache configuration is not yet implemented")
    }

    /// Invalidates specific items from the cache.
    ///
    /// # Arguments
    /// * `ids` - IDs of items to invalidate (if None, invalidates all)
    #[allow(dead_code)]
    pub fn invalidate_cache(
        &self,
        _ids: Option<Vec<&crate::provider::capabilities::sistence_memory::MemoryId>>,
    ) {
        unimplemented!("Cache invalidation is not yet implemented")
    }

    /// Convert a DetailedMemoryItem to a public MemoryItem
    fn detailed_to_simple(&self, detailed: DetailedMemoryItem) -> MemoryItem {
        MemoryItem {
            id: detailed.id,
            created_at: detailed.created_at,
            updated_at: detailed.updated_at,
            content: detailed.content,
            content_type: detailed.content_type,
            structured_content: detailed.structured_content,
            item_type: detailed.item_type,
            topics: detailed.topics,
            tags: detailed.tags,
            source: detailed.source,
            references: self.convert_references(detailed.references),
            related_items: detailed.related_items,
            importance: self.convert_importance(detailed.importance),
            last_accessed: detailed.access_stats.last_accessed,
            access_count: detailed.access_stats.access_count,
            ttl: detailed.ttl,
            retention_policy: detailed.retention_policy,
        }
    }

    /// Convert a public MemoryItem to a DetailedMemoryItem
    fn simple_to_detailed(&self, simple: MemoryItem) -> DetailedMemoryItem {
        // Default creation time for new components
        let now = SystemTime::now();
        let first_occurrence = simple.created_at;

        // Clone values that will be used multiple times
        let source_clone = simple.source.clone();
        let references_clone = simple.references.clone();
        let _references_len = references_clone.len();

        DetailedMemoryItem {
            id: simple.id,
            created_at: simple.created_at,
            updated_at: simple.updated_at,
            content: simple.content,
            content_type: simple.content_type,
            structured_content: simple.structured_content,
            item_type: simple.item_type,
            topics: simple.topics,
            tags: simple.tags,
            source: source_clone.clone(),
            references: references_clone.iter().map(|r| self.convert_to_detailed_reference(r.clone(), now)).collect(),
            related_items: simple.related_items,
            importance: DetailedImportanceEvaluation {
                base_score: simple.importance.base_score,
                context_score: simple.importance.context_score.unwrap_or(0.0),
                intrinsic_components: crate::provider::capabilities::relevant_memory::IntrinsicMetrics {
                    first_occurrence,
                    creation_context: "Converted from public API".to_string(),
                    source_reliability: simple.source.reliability,
                    verification_level: crate::provider::capabilities::sistence_memory::VerificationLevel::Unverified,
                    criticality: 0.5, // Default value
                    novelty: 0.5,     // Default value
                    permanence: 0.5,   // Default value
                    scope_breadth: 0.5 // Default value
                },
                usage_components: DetailedAccessStats {
                    access_count: simple.access_count,
                    last_accessed: simple.last_accessed,
                    recent_accesses: Vec::new(), // No detailed access history in public item
                    access_frequency: 0.0,       // Will be calculated later
                    pattern_analysis: None,      // No pattern analysis available
                },
                reference_components: crate::provider::capabilities::relevant_memory::ReferenceNetwork {
                    reference_count: simple.references.len() as u32,
                    reference_diversity: 0.0, // Will be calculated later
                    citation_strength: 0.0,   // Will be calculated later
                    network_centrality: 0.0,  // Will be calculated later
                },
                contextual_components: None, // No contextual components available
                emotional_components: None,  // No emotional components available
                evaluated_at: simple.importance.evaluated_at,
                evaluation_context: None,    // No evaluation context available
            },
            access_stats: DetailedAccessStats {
                access_count: simple.access_count,
                last_accessed: simple.last_accessed,
                recent_accesses: Vec::new(), // No detailed access history in public item
                access_frequency: 0.0,       // Will be calculated later
                pattern_analysis: None,      // No pattern analysis available
            },
            ttl: simple.ttl,
            retention_policy: simple.retention_policy,
        }
    }

    /// Convert detailed references to simplified references
    fn convert_references(&self, detailed_refs: Vec<DetailedReference>) -> Vec<Reference> {
        detailed_refs
            .into_iter()
            .map(|r| Reference {
                ref_type: r.ref_type,
                ref_id: r.ref_id,
                context: r.context,
                strength: r.strength,
            })
            .collect()
    }

    /// Convert a standard reference to a detailed reference
    fn convert_to_detailed_reference(
        &self,
        reference: Reference,
        creation_time: SystemTime,
    ) -> DetailedReference {
        DetailedReference {
            ref_type: reference.ref_type,
            ref_id: reference.ref_id,
            context: reference.context,
            strength: reference.strength,
            created_at: creation_time,
            metadata: HashMap::new(), // No detailed metadata available
        }
    }

    /// Convert a DetailedImportanceEvaluation to a simplified ImportanceScore
    fn convert_importance(&self, detailed: DetailedImportanceEvaluation) -> ImportanceScore {
        // Use a weighted calculation for the overall score
        let overall_score = detailed.base_score * 0.7 + detailed.context_score * 0.3;

        ImportanceScore {
            score: overall_score,
            base_score: detailed.base_score,
            context_score: Some(detailed.context_score),
            reason: None, // Would generate from components in a real implementation
            evaluated_at: detailed.evaluated_at,
        }
    }

    /// Enhanced conversion of importance metrics that includes more sophisticated analysis
    /// of the detailed components to generate meaningful reasons and scores.
    ///
    /// # Arguments
    /// * `detailed` - The detailed importance evaluation
    /// * `include_reason` - Whether to generate a human-readable reason
    ///
    /// # Returns
    /// A more nuanced importance score with detailed reasoning
    #[allow(dead_code)]
    fn enhanced_convert_importance(
        &self,
        _detailed: &DetailedImportanceEvaluation,
        _include_reason: bool,
    ) -> ImportanceScore {
        unimplemented!("Enhanced importance conversion is not yet implemented")
    }

    /// Generates an explanation of why a particular item has its importance score.
    ///
    /// This is useful for debugging and for providing transparency to users about
    /// why certain memories are considered important.
    ///
    /// # Arguments
    /// * `item_id` - The memory item to explain
    ///
    /// # Returns
    /// A structured explanation of the item's importance factors
    #[tracing::instrument(level = "debug", skip(self), err)]
    pub async fn explain_importance(
        &self,
        item_id: &crate::provider::capabilities::sistence_memory::MemoryId,
    ) -> Result<
        HashMap<String, serde_json::Value>,
        crate::provider::capabilities::sistence_memory::SistenceMemoryError,
    > {
        unimplemented!("Importance explanation is not yet implemented")
    }
}

#[async_trait]
impl ProviderPlugin for SistenceMemoryAdapter {
    fn priority(&self) -> i32 {
        10 // ポリシーは早めに適用
    }

    #[tracing::instrument(skip(self, _context))]
    async fn generate_section<'a>(&self, _context: &PluginContext<'a>) -> ProviderResult<Section> {
        todo!()
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::SistenceMemory
    }

    async fn process_response<'a>(
        &self,
        _context: &PluginContext<'a>,
        _response: &LLMResponse,
    ) -> ProviderResult<()> {
        Ok(())
    }
}

#[async_trait]
impl SistenceMemoryCapability for SistenceMemoryAdapter {
    // === Basic CRUD Operations ===

    #[tracing::instrument(level = "debug", skip(self, item), err)]
    async fn store(&self, item: MemoryItem) -> Result<MemoryId, SistenceMemoryError> {
        // Convert to internal format
        let detailed_item = self.simple_to_detailed(item);

        // Delegate to internal implementation using the error conversion helper
        self.convert_error(self.relevant_memory.store_memory_item(detailed_item).await)
    }

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn retrieve(&self, id: &MemoryId) -> Result<MemoryItem, SistenceMemoryError> {
        // Retrieve detailed item and convert any errors
        let detailed_item =
            self.convert_error(self.relevant_memory.retrieve_memory_item(id).await)?;

        // Convert to simplified format
        Ok(self.detailed_to_simple(detailed_item))
    }

    #[tracing::instrument(level = "debug", skip(self, item), err)]
    async fn update(&self, item: MemoryItem) -> Result<(), SistenceMemoryError> {
        // Convert to internal format
        let detailed_item = self.simple_to_detailed(item);

        // Delegate to internal implementation using the error conversion helper
        self.convert_error(self.relevant_memory.update_memory_item(detailed_item).await)
    }

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn delete(&self, id: &MemoryId) -> Result<(), SistenceMemoryError> {
        // Simply delegate to internal implementation with error conversion
        self.convert_error(self.relevant_memory.delete_memory_item(id).await)
    }

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn exists(&self, id: &MemoryId) -> Result<bool, SistenceMemoryError> {
        // Try to retrieve and convert to a boolean result
        match self.convert_error(self.relevant_memory.retrieve_memory_item(id).await) {
            Ok(_) => Ok(true),
            Err(SistenceMemoryError::NotFound(_)) => Ok(false),
            Err(err) => Err(err),
        }
    }

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn update_importance(
        &self,
        id: &MemoryId,
        policy: Option<ImportancePolicy>,
    ) -> Result<ImportanceScore, SistenceMemoryError> {
        // Retrieve the item
        let mut item = self.retrieve(id).await?;

        // Apply policy to adjust importance
        let base_score = item.importance.base_score;
        let context_score = item.importance.context_score.unwrap_or(0.0);

        // Adjust scores based on policy
        let (new_base, new_context) = match policy {
            Some(ImportancePolicy::Standard) => (base_score, context_score),
            Some(ImportancePolicy::FactualFocus) => (base_score * 1.2, context_score * 0.8),
            Some(ImportancePolicy::NoveltyFocus) => (base_score * 1.1, context_score * 1.1),
            Some(ImportancePolicy::UtilityFocus) => (base_score * 0.9, context_score * 1.3),
            Some(ImportancePolicy::Custom(_)) => (base_score, context_score),
            None => (base_score, context_score),
        };

        // Ensure scores are in valid range
        let new_base = new_base.clamp(0.0, 1.0);
        let new_context = new_context.clamp(0.0, 1.0);

        // Create new importance score
        let new_score = ImportanceScore {
            score: new_base * 0.7 + new_context * 0.3,
            base_score: new_base,
            context_score: Some(new_context),
            reason: Some(format!("Updated with policy: {:?}", policy)),
            evaluated_at: SystemTime::now(),
        };

        // Update the item with new importance
        item.importance = new_score.clone();
        self.update(item).await?;

        Ok(new_score)
    }

    // === Search Operations ===

    #[tracing::instrument(level = "debug", skip(self, filters), err)]
    async fn search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>, SistenceMemoryError> {
        // Delegate to the detailed search method with error conversion
        let results = self.convert_error(
            self.relevant_memory
                .search_with_relevance(
                    query,
                    filters,
                    None, // No context for basic search
                    limit.unwrap_or(self.default_result_limit),
                    None, // No minimum relevance
                )
                .await,
        )?;

        // Convert results to simplified format
        Ok(results
            .into_iter()
            .map(|(item, _, _)| self.detailed_to_simple(item))
            .collect())
    }

    #[tracing::instrument(level = "debug", skip(self, context), err)]
    async fn search_with_context(
        &self,
        query: &str,
        context: SearchContext,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>, SistenceMemoryError> {
        // Use structured result for context-aware search with error conversion
        let result =
            self.convert_error(self.relevant_memory.contextual_search(query, context).await)?;

        // Apply limit if provided
        let mut items = result.items;
        if let Some(limit) = limit {
            items.truncate(limit);
        }

        Ok(items)
    }

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn find_related(
        &self,
        item_id: &MemoryId,
        max_results: Option<usize>,
    ) -> Result<Vec<MemoryItem>, SistenceMemoryError> {
        // Use semantic relatedness search with error conversion
        let results = self.convert_error(
            self.relevant_memory
                .find_semantically_related(
                    item_id,
                    max_results.unwrap_or(self.default_result_limit),
                    0.0, // No minimum similarity for basic API
                )
                .await,
        )?;

        // Convert results to simplified format
        Ok(results
            .into_iter()
            .map(|(item, _)| self.detailed_to_simple(item))
            .collect())
    }

    #[tracing::instrument(level = "debug", skip(self, context), err)]
    async fn get_relevant_for_context(
        &self,
        context: SearchContext,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>, SistenceMemoryError> {
        // Use context relevance function with error conversion
        let result = self.convert_error(
            self.relevant_memory
                .get_context_relevant(
                    context,
                    limit.unwrap_or(self.default_result_limit),
                    0.0, // No minimum relevance for basic API
                )
                .await,
        )?;

        Ok(result.items)
    }

    // === Metadata Management ===

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn add_topics(
        &self,
        item_id: &MemoryId,
        topics: Vec<String>,
    ) -> Result<(), SistenceMemoryError> {
        // Retrieve the item with error conversion
        let mut item =
            self.convert_error(self.relevant_memory.retrieve_memory_item(item_id).await)?;

        // Add topics without duplicates
        for topic in topics {
            if !item.topics.contains(&topic) {
                item.topics.push(topic);
            }
        }

        // Update the item with error conversion
        self.convert_error(self.relevant_memory.update_memory_item(item).await)
    }

    #[tracing::instrument(level = "debug", skip(self, tags), err)]
    async fn add_tags(
        &self,
        item_id: &MemoryId,
        tags: HashMap<String, String>,
    ) -> Result<(), SistenceMemoryError> {
        // Retrieve the item with error conversion
        let mut item =
            self.convert_error(self.relevant_memory.retrieve_memory_item(item_id).await)?;

        // Add or update tags
        for (key, value) in tags {
            item.tags.insert(key, value);
        }

        // Update the item with error conversion
        self.convert_error(self.relevant_memory.update_memory_item(item).await)
    }

    #[tracing::instrument(level = "debug", skip(self, target_ids), err)]
    async fn link_items(
        &self,
        source_id: &MemoryId,
        target_ids: Vec<MemoryId>,
        relation_type: Option<String>,
    ) -> Result<(), SistenceMemoryError> {
        // Create links
        let now = SystemTime::now();
        let relation = relation_type.unwrap_or_else(|| "related".to_string());

        let links = target_ids
            .into_iter()
            .map(|target_id| {
                ItemLink {
                    source_id: source_id.clone(),
                    target_id,
                    relation_type: relation.clone(),
                    strength: 1.0, // Default full strength for basic API
                    created_at: now,
                    metadata: Some(HashMap::new()),
                    is_bidirectional: false, // Default unidirectional
                    context: None,
                }
            })
            .collect();

        // Create links using internal implementation with error conversion
        self.convert_error(self.relevant_memory.create_item_links(links).await)
    }

    // === Working Memory Integration ===

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn index_from_working_memory(
        &self,
        namespace: &str,
        pattern: Option<&str>,
    ) -> Result<IndexStats, SistenceMemoryError> {
        let pattern_str = pattern.unwrap_or("*");
        let _ids = self.convert_error(
            self.relevant_memory
                .process_from_working_memory(
                    pattern_str,
                    namespace,
                    self.default_enhancement.clone(),
                )
                .await,
        )?;

        // A real implementation would track successful/failed operations
        Ok(IndexStats {
            items_indexed: 1, // Placeholder - would be actual count
            items_skipped: 0,
            errors: 0,
            duration: Duration::from_millis(1),
        })
    }

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn promote_from_working_memory(
        &self,
        key: &str,
        namespace: &str,
    ) -> Result<String, SistenceMemoryError> {
        // Directly delegate to internal implementation with error conversion
        self.convert_error(
            self.relevant_memory
                .process_from_working_memory(key, namespace, self.default_enhancement.clone())
                .await,
        )
    }

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn store_to_working_memory(
        &self,
        item_id: &MemoryId,
        namespace: &str,
        key: &str,
    ) -> Result<(), SistenceMemoryError> {
        // Prepare the item for working memory with error conversion
        let _value = self.convert_error(
            self.relevant_memory
                .prepare_for_working_memory(item_id, WorkingMemoryFormat::RichJson)
                .await,
        )?;

        // In a real implementation, we would store this in shared memory
        // For now, just return success
        Ok(())
    }

    // === LLM-Enhanced Functions ===

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn enhance_metadata(
        &self,
        item_id: &MemoryId,
    ) -> Result<EnhancedMetadata, SistenceMemoryError> {
        // Create standard enhancement options
        let options = EnhancementOptions {
            enhance_topics: true,
            enhance_tags: true,
            extract_entities: true,
            suggest_relations: true,
            generate_summary: true,
            extract_key_points: true,
            analyze_sentiment: true,
            update_item: true,
            confidence_threshold: 0.6,
            max_suggestions: 10,
        };

        // Delegate to internal implementation with error conversion
        self.convert_error(
            self.relevant_memory
                .enhance_item_metadata(item_id, options)
                .await,
        )
    }

    #[tracing::instrument(level = "debug", skip(self, context), err)]
    async fn build_context(
        &self,
        context: SearchContext,
        max_tokens: usize,
        strategy: Option<SearchStrategy>,
    ) -> Result<String, SistenceMemoryError> {
        // Map the search strategy to context strategy
        let context_strategy = match strategy {
            Some(SearchStrategy::Balanced) => self.default_context_strategy.clone(),
            Some(SearchStrategy::Recency) => ContextStrategy::Recency,
            Some(SearchStrategy::Relevance) => ContextStrategy::QueryRelevance,
            Some(SearchStrategy::Importance) => ContextStrategy::Importance,
            Some(SearchStrategy::Custom(_name)) => ContextStrategy::Custom(HashMap::new()), // Would parse custom strategy
            None => self.default_context_strategy.clone(),
        };

        // Generate context using internal implementation with error conversion
        self.convert_error(
            self.relevant_memory
                .generate_optimized_context(
                    context,
                    max_tokens,
                    context_strategy,
                    false, // Don't include metadata by default
                )
                .await,
        )
    }

    // === Management Functions ===

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn cleanup_expired(&self) -> Result<CleanupStats, SistenceMemoryError> {
        // In a real implementation, would have a dedicated cleanup method
        // For now, return placeholder stats
        Ok(CleanupStats {
            items_removed: 0,
            items_kept: 0,
            space_reclaimed: 0,
            duration: Duration::from_secs(0),
        })
    }

    #[tracing::instrument(level = "debug", skip(self), err)]
    async fn get_stats(&self) -> Result<MemoryStats, SistenceMemoryError> {
        // In a real implementation, would collect stats from internal store
        // For now, return placeholder stats
        Ok(MemoryStats {
            total_items: 0,
            items_by_type: HashMap::new(),
            storage_used: 0,
            avg_importance: 0.0,
            access_stats: HashMap::new(),
        })
    }
}

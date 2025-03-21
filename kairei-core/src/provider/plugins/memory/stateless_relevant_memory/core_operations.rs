// Core operations for the StatelessRelevantMemory implementation

use std::collections::HashMap;
use std::time::SystemTime;

use tracing::{debug, info};
use uuid::Uuid;

use crate::provider::capabilities::relevant_memory::DetailedMemoryItem;
use crate::provider::capabilities::sistence_memory::*;
use crate::provider::capabilities::sistence_storage::SistenceStorageService;
use crate::provider::capabilities::storage::StorageError;

use super::StatelessRelevantMemory;

impl StatelessRelevantMemory {
    /// Internal implementation: Store a memory item using basic storage backend
    #[tracing::instrument(level = "debug", skip(self, item), fields(item_id = %item.id), err)]
    pub(crate) async fn store_to_basic_storage(
        &self,
        item: &DetailedMemoryItem,
    ) -> Result<(), SistenceMemoryError> {
        debug!(
            "Using basic storage backend to store memory item {}",
            item.id
        );
        // Basic storage backend implementation
        let item_key = format!("memory_items/{}", item.id);

        // Serialize the item
        let item_json = serde_json::to_value(item).map_err(|e| {
            SistenceMemoryError::SerializationError(format!(
                "Failed to serialize memory item: {}",
                e
            ))
        })?;

        // Create metadata
        let metadata = crate::provider::capabilities::shared_memory::Metadata::default();

        // Create value with metadata
        let data = crate::provider::capabilities::storage::ValueWithMetadata {
            value: item_json,
            metadata,
            expiry: None,
        };

        // Save to storage
        self.storage
            .save_key("default", &item_key, &data)
            .await
            .map_err(|e| SistenceMemoryError::StorageError(e))?;

        // Update indexes
        self.update_indexes(item);

        Ok(())
    }

    /// Internal implementation: Retrieve a memory item using basic storage backend
    #[tracing::instrument(level = "debug", skip(self), err)]
    pub(crate) async fn retrieve_from_basic_storage(
        &self,
        id: &str,
    ) -> Result<DetailedMemoryItem, SistenceMemoryError> {
        debug!("Using basic storage backend to retrieve memory item {}", id);

        // Try to get from memory index first (local cache)
        if let Some(item) = self.memory_index.get(id) {
            debug!("Found memory item {} in memory index", id);
            return Ok(item.clone());
        }

        // Construct the storage key
        let item_key = format!("memory_items/{}", id);

        // Load data from storage
        let data = self
            .storage
            .load("default")
            .await
            .map_err(|e| SistenceMemoryError::StorageError(e))?;

        // Find the item
        let item_data = data
            .get(&item_key)
            .ok_or_else(|| SistenceMemoryError::NotFound(id.to_string()))?;

        // Deserialize the item
        let item: DetailedMemoryItem =
            serde_json::from_value(item_data.value.clone()).map_err(|e| {
                SistenceMemoryError::SerializationError(format!(
                    "Failed to deserialize memory item: {}",
                    e
                ))
            })?;

        // Update indexes
        self.update_indexes(&item);

        Ok(item)
    }

    /// Update memory indexes with the given item
    #[tracing::instrument(level = "debug", skip(self, item), fields(item_id = %item.id))]
    pub fn update_indexes(&self, item: &DetailedMemoryItem) {
        // Add to memory index
        self.memory_index.insert(item.id.clone(), item.clone());

        // Update topic index
        for topic in &item.topics {
            let mut entry = self.topic_index.entry(topic.clone()).or_default();
            if !entry.contains(&item.id) {
                entry.push(item.id.clone());
            }
        }

        // Update tag index
        for (key, value) in &item.tags {
            let tag_key = format!("{}:{}", key, value);
            let mut entry = self.tag_index.entry(tag_key).or_default();
            if !entry.contains(&item.id) {
                entry.push(item.id.clone());
            }
        }
    }

    /// Remove an item from the memory indexes
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_from_indexes(&self, id: &str) {
        if let Some((_, item)) = self.memory_index.remove(id) {
            // Remove from topic index
            for topic in &item.topics {
                if let Some(mut entry) = self.topic_index.get_mut(topic) {
                    entry.retain(|item_id| item_id != id);
                }
            }

            // Remove from tag index
            for (key, value) in &item.tags {
                let tag_key = format!("{}:{}", key, value);
                if let Some(mut entry) = self.tag_index.get_mut(&tag_key) {
                    entry.retain(|item_id| item_id != id);
                }
            }
        }
    }

    /// Convert a DetailedMemoryItem to a public MemoryItem
    #[tracing::instrument(level = "debug", skip(self, detailed), fields(item_id = %detailed.id))]
    pub fn detailed_to_memory_item(
        &self,
        detailed: DetailedMemoryItem,
        relevance_score: Option<f32>,
    ) -> MemoryItem {
        let importance = ImportanceScore {
            score: detailed.importance.base_score,
            base_score: detailed.importance.base_score,
            context_score: Some(detailed.importance.context_score),
            reason: None, // Would generate in a real implementation
            evaluated_at: detailed.importance.evaluated_at,
        };

        // Convert references
        let references = detailed
            .references
            .into_iter()
            .map(|r| Reference {
                ref_type: r.ref_type,
                ref_id: r.ref_id,
                context: r.context,
                strength: r.strength,
            })
            .collect();

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
            references,
            related_items: detailed.related_items,
            importance,
            last_accessed: detailed.access_stats.last_accessed,
            access_count: detailed.access_stats.access_count,
            ttl: detailed.ttl,
            retention_policy: detailed.retention_policy,
        }
    }

    /// Create a knowledge graph node for a query
    pub fn create_query_knowledge_node(&self, query: &str, context_id: &str) -> KnowledgeNode {
        KnowledgeNode {
            id: format!("query-{}", Uuid::new_v4()),
            label: query.to_string(),
            node_type: "query".to_string(),
            properties: HashMap::from([
                ("query".to_string(), query.to_string()),
                ("context_id".to_string(), context_id.to_string()),
                (
                    "timestamp".to_string(),
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                ),
            ]),
            connections: Vec::new(),
        }
    }

    /// Calculate confidence from search results
    pub fn calculate_confidence(
        &self,
        search_results: &[(DetailedMemoryItem, f32, HashMap<String, f32>)],
    ) -> f32 {
        if search_results.is_empty() {
            return 0.0;
        }

        let top_relevance_sum: f32 = search_results
            .iter()
            .take(3.min(search_results.len()))
            .map(|(_, relevance, _)| *relevance)
            .sum();

        top_relevance_sum / 3.min(search_results.len()) as f32
    }
}

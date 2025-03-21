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

    // The DetailedMemoryItem to MemoryItem conversion has been moved to the From trait implementation
    // in the sistence_memory.rs file, which is a more idiomatic Rust approach.

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

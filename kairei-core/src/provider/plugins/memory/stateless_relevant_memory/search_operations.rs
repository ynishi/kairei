// Search and relevance operations for the StatelessRelevantMemory implementation

use std::collections::HashMap;

use serde_json::json;
use tracing::debug;

use crate::provider::capabilities::relevant_memory::DetailedMemoryItem;
use crate::provider::capabilities::sistence_memory::*;

use super::StatelessRelevantMemory;
use super::utility_functions::{
    calculate_reference_similarity, calculate_tag_similarity, calculate_text_similarity,
    calculate_topic_match,
};
use crate::provider::capabilities::relevant_memory::RelevantMemoryCapability;

type SearchResultType = Vec<(DetailedMemoryItem, f32, HashMap<String, f32>)>;

impl StatelessRelevantMemory {
    // === Advanced Search Operations ===

    #[tracing::instrument(level = "debug", skip(self, filters, _context), err)]
    pub async fn search_with_relevance(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        _context: Option<SearchContext>,
        max_results: usize,
        min_relevance: Option<f32>,
    ) -> Result<SearchResultType, SistenceMemoryError> {
        // Simple implementation - in a real system this would use more sophisticated search
        let mut results = Vec::new();

        // Score and filter items
        for item_ref in self.memory_index.iter() {
            let item = item_ref.value().clone();

            // Apply filters if provided
            if let Some(filters) = &filters {
                // Filter by item type
                if let Some(item_types) = &filters.item_types {
                    if !item_types.contains(&item.item_type) {
                        continue;
                    }
                }

                // Filter by topics
                if let Some(topics) = &filters.topics {
                    if !topics.iter().any(|t| item.topics.contains(t)) {
                        continue;
                    }
                }

                // Filter by time range
                if let Some(time_start) = filters.time_start {
                    if item.created_at < time_start {
                        continue;
                    }
                }

                if let Some(time_end) = filters.time_end {
                    if item.created_at > time_end {
                        continue;
                    }
                }

                // Filter by importance
                if let Some(min_importance) = filters.min_importance {
                    if item.importance.base_score < min_importance {
                        continue;
                    }
                }
            }

            let query_lowercase = query.to_lowercase();
            let content_lowercase = item.content.to_lowercase();

            // Calculate relevance score - simplistic implementation
            let content_match = if item.content.to_lowercase().contains(&query_lowercase) {
                0.8
            } else {
                // Check if any tokens match
                let query_tokens: Vec<&str> = query_lowercase.split_whitespace().collect();
                let content_tokens: Vec<&str> = content_lowercase.split_whitespace().collect();

                let matching_tokens = query_tokens
                    .iter()
                    .filter(|&qt| content_tokens.contains(qt))
                    .count();

                if matching_tokens > 0 {
                    0.5 * (matching_tokens as f32 / query_tokens.len() as f32)
                } else {
                    0.0
                }
            };

            // Check topics match
            let topic_match = item
                .topics
                .iter()
                .any(|t| query_lowercase.contains(&t.to_lowercase()))
                as u8 as f32
                * 0.5;

            // Calculate final relevance
            let relevance = content_match.max(topic_match);

            // Skip if below minimum relevance
            if let Some(min_rel) = min_relevance {
                if relevance < min_rel {
                    continue;
                }
            }

            // Create components map
            let mut components = HashMap::new();
            components.insert("content_match".to_string(), content_match);
            components.insert("topic_match".to_string(), topic_match);

            // Add to results
            results.push((item, relevance, components));
        }

        // Sort by relevance
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit
        if results.len() > max_results {
            results.truncate(max_results);
        }

        Ok(results)
    }

    #[tracing::instrument(level = "debug", skip(self, context), fields(query = %query, context_id = %context.context_id), err)]
    pub async fn contextual_search(
        &self,
        query: &str,
        context: SearchContext,
    ) -> Result<StructuredResult, SistenceMemoryError> {
        debug!(
            "Performing contextual search with query: '{}', context ID: {}",
            query, context.context_id
        );

        // First, get relevant items with relevance scores
        let max_results = 20; // Configurable parameter
        let min_relevance = 0.3; // Configurable threshold

        // Prepare filters based on context
        let filters = SearchFilters {
            item_types: None,
            topics: Some(context.current_topics.clone()),
            source: None,
            time_start: None,
            time_end: None,
            min_importance: Some(0.1), // Low threshold for comprehensive results
            custom_filters: None,
        };

        // Perform the relevance-based search
        let search_results = self
            .search_with_relevance(
                query,
                Some(filters),
                Some(context.clone()),
                max_results,
                Some(min_relevance),
            )
            .await?;

        // Convert detailed items to standard MemoryItems using From trait
        let memory_items: Vec<MemoryItem> = search_results
            .iter()
            .map(|(item, relevance, _)| {
                // Use From trait implementation to convert DetailedMemoryItem to MemoryItem
                MemoryItem::from(item.clone())
                // Note: relevance score was previously passed but not used in detailed_to_memory_item
            })
            .collect();

        // Create a simple knowledge graph if we have results
        let knowledge_graph = if !search_results.is_empty() {
            Some(
                self.create_simple_knowledge_graph(&search_results, query, &context)
                    .await?,
            )
        } else {
            None
        };

        // Calculate overall confidence based on top relevance scores
        let confidence = if search_results.is_empty() {
            0.0
        } else {
            let top_relevance_sum: f32 = search_results
                .iter()
                .take(3.min(search_results.len()))
                .map(|(_, relevance, _)| *relevance)
                .sum();

            top_relevance_sum / (3.min(search_results.len()) as f32)
        };

        // Create a structured result
        let result = StructuredResult {
            items: memory_items,
            summary: None, // Would generate with LLM in a real implementation
            knowledge_graph,
            confidence,
            context_match: 0.7, // Placeholder - would calculate from context
            execution_stats: Some(json!({
                "total_items_considered": self.memory_index.len(),
                "matching_items": search_results.len(),
                "query": query,
                "context_id": context.context_id
            })),
        };

        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self, context), fields(context_id = %context.context_id, max_items = %max_items), err)]
    pub async fn get_context_relevant(
        &self,
        context: SearchContext,
        max_items: usize,
        min_relevance: f32,
    ) -> Result<StructuredResult, SistenceMemoryError> {
        debug!(
            "Getting context-relevant items for context ID: {}",
            context.context_id
        );

        // This is similar to contextual_search but doesn't require a specific query
        // Instead, it uses the context itself to find relevant items

        // Collect all memory items with contextual relevance scores
        let mut relevance_scores = Vec::new();

        for item_ref in self.memory_index.iter() {
            let item = item_ref.value().clone();

            // Calculate relevance based on:
            // 1. Topic match with current context
            let topic_match = calculate_topic_match(&item.topics, &context.current_topics);

            // 2. Recency factor - more recent items have higher relevance
            let recency = super::utility_functions::calculate_recency_factor(&item.created_at);

            // 3. Importance score
            let importance = item.importance.base_score;

            // 4. Activity relevance if available
            let activity_relevance = if let Some(activity) = &context.current_activity {
                if item
                    .content
                    .to_lowercase()
                    .contains(&activity.to_lowercase())
                {
                    0.8
                } else {
                    0.2
                }
            } else {
                0.5 // Neutral if no activity specified
            };

            // Combine factors - adjust weights as needed
            let relevance = (topic_match * 0.4)
                + (recency * 0.2)
                + (importance * 0.3)
                + (activity_relevance * 0.1);

            // Skip items below minimum relevance
            if relevance >= min_relevance {
                // Create component map for detailed relevance information
                let mut components = HashMap::new();
                components.insert("topic_match".to_string(), topic_match);
                components.insert("recency".to_string(), recency);
                components.insert("importance".to_string(), importance);
                components.insert("activity_relevance".to_string(), activity_relevance);

                relevance_scores.push((item, relevance, components));
            }
        }

        // Sort by relevance
        relevance_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        if relevance_scores.len() > max_items {
            relevance_scores.truncate(max_items);
        }

        // Convert to MemoryItems using From trait
        let memory_items: Vec<MemoryItem> = relevance_scores
            .iter()
            .map(|(item, _, _)| {
                // Use From trait implementation to convert DetailedMemoryItem to MemoryItem
                MemoryItem::from(item.clone())
            })
            .collect();

        // Calculate overall confidence
        let confidence = if relevance_scores.is_empty() {
            0.0
        } else {
            let avg_relevance: f32 = relevance_scores
                .iter()
                .map(|(_, relevance, _)| *relevance)
                .sum::<f32>()
                / relevance_scores.len() as f32;

            avg_relevance
        };

        // Create a structured result
        let result = StructuredResult {
            items: memory_items,
            summary: None,         // Would generate with LLM in a real implementation
            knowledge_graph: None, // Would build if needed
            confidence,
            context_match: confidence, // Same as confidence for this method
            execution_stats: Some(json!({
                "total_items": self.memory_index.len(),
                "relevant_items": relevance_scores.len(),
                "context_id": context.context_id,
                "min_relevance_threshold": min_relevance
            })),
        };

        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self), fields(item_id = %item_id, max_results = %max_results), err)]
    pub async fn find_semantically_related(
        &self,
        item_id: &MemoryId,
        max_results: usize,
        min_similarity: f32,
    ) -> Result<Vec<(DetailedMemoryItem, f32)>, SistenceMemoryError> {
        debug!(
            "Finding semantically related items for item ID: {}",
            item_id
        );

        // Get the source item
        let source_item = self.retrieve_memory_item(item_id).await?;

        // Calculate similarity scores for all items
        let mut similarity_scores = Vec::new();

        for item_ref in self.memory_index.iter() {
            let item = item_ref.value().clone();

            // Skip the source item itself
            if item.id == *item_id {
                continue;
            }

            // Calculate similarity based on multiple factors:

            // 1. Content similarity
            let content_similarity = calculate_text_similarity(&source_item.content, &item.content);

            // 2. Topic similarity
            let topic_similarity = super::utility_functions::calculate_set_similarity(
                &source_item.topics,
                &item.topics,
            );

            // 3. Reference similarity
            let reference_similarity = calculate_reference_similarity(&source_item, &item);

            // 4. Tag similarity
            let tag_similarity = calculate_tag_similarity(&source_item.tags, &item.tags);

            // Combine factors - adjust weights as needed
            let similarity = (content_similarity * 0.4)
                + (topic_similarity * 0.3)
                + (reference_similarity * 0.2)
                + (tag_similarity * 0.1);

            // Skip items below minimum similarity
            if similarity >= min_similarity {
                similarity_scores.push((item, similarity));
            }
        }

        // Sort by similarity
        similarity_scores
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        if similarity_scores.len() > max_results {
            similarity_scores.truncate(max_results);
        }

        Ok(similarity_scores)
    }
}

// Context optimization operations for the StatelessRelevantMemory implementation

use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};

use serde_json::json;
use tracing::debug;

use crate::provider::capabilities::relevant_memory::{ContextStrategy, TimeFocus};
use crate::provider::capabilities::sistence_memory::*;

use super::StatelessRelevantMemory;
// Removed unused import: RelevantMemoryCapability

impl StatelessRelevantMemory {
    /// Generate an optimized context for LLM prompts
    #[tracing::instrument(level = "debug", skip(self, context), fields(strategy = ?context_strategy), err)]
    pub async fn generate_optimized_context(
        &self,
        context: SearchContext,
        max_tokens: usize,
        context_strategy: ContextStrategy,
        include_metadata: bool,
    ) -> Result<String, SistenceMemoryError> {
        debug!(
            "Generating optimized context with strategy: {:?}",
            context_strategy
        );

        // Get relevant items
        let relevant_items = self
            .get_context_relevant(
                context.clone(),
                50,  // Get more items than we need
                0.2, // Low threshold to get more candidates
            )
            .await?
            .items;

        // Apply strategy to select and format items
        let mut selected_items = Vec::new();
        let mut current_tokens = 0;
        let avg_tokens_per_char = 0.25; // Rough estimate

        match context_strategy {
            ContextStrategy::Recency => {
                // Sort by recency
                let mut items = relevant_items;
                items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

                // Take items until we hit the token limit
                for item in items {
                    let estimated_tokens =
                        (item.content.len() as f32 * avg_tokens_per_char) as usize;

                    if current_tokens + estimated_tokens <= max_tokens {
                        selected_items.push(item);
                        current_tokens += estimated_tokens;
                    } else {
                        break;
                    }
                }
            }
            ContextStrategy::QueryRelevance => {
                // Items are already sorted by relevance
                let items = relevant_items;

                // Take items until we hit the token limit
                for item in items {
                    let estimated_tokens =
                        (item.content.len() as f32 * avg_tokens_per_char) as usize;

                    if current_tokens + estimated_tokens <= max_tokens {
                        selected_items.push(item);
                        current_tokens += estimated_tokens;
                    } else {
                        break;
                    }
                }
            }
            ContextStrategy::Balanced => {
                // Mix of recency and relevance
                let items = relevant_items;

                // Create a balanced score
                let mut scored_items: Vec<(MemoryItem, f32)> = items
                    .into_iter()
                    .map(|item| {
                        // Calculate recency score (0-1)
                        let now = SystemTime::now();
                        let age = now
                            .duration_since(item.created_at)
                            .unwrap_or(Duration::from_secs(0))
                            .as_secs() as f32;
                        let recency_score = 1.0 / (1.0 + (age / 86400.0)); // Decay over days

                        // Use importance as relevance score
                        let relevance_score = item.importance.score;

                        // Balanced score
                        let score = (recency_score * 0.4) + (relevance_score * 0.6);

                        (item, score)
                    })
                    .collect();

                // Sort by balanced score
                scored_items
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                // Take items until we hit the token limit
                for (item, _) in scored_items {
                    let estimated_tokens =
                        (item.content.len() as f32 * avg_tokens_per_char) as usize;

                    if current_tokens + estimated_tokens <= max_tokens {
                        selected_items.push(item);
                        current_tokens += estimated_tokens;
                    } else {
                        break;
                    }
                }
            }
            ContextStrategy::ActivityRelevance => {
                // Group by topics
                let mut topic_groups: HashMap<String, Vec<MemoryItem>> = HashMap::new();

                for item in relevant_items {
                    for topic in &item.topics {
                        topic_groups
                            .entry(topic.clone())
                            .or_default()
                            .push(item.clone());
                    }
                }

                // Take top items from each topic
                let mut added_ids = HashSet::new();

                // Sort topics by number of items (most to least)
                let mut topics: Vec<_> = topic_groups.keys().collect();
                topics.sort_by(|a, b| {
                    topic_groups
                        .get(*b)
                        .unwrap()
                        .len()
                        .cmp(&topic_groups.get(*a).unwrap().len())
                });

                // Round-robin through topics
                let mut i = 0;
                while current_tokens < max_tokens && !topics.is_empty() {
                    let topic_idx = i % topics.len();
                    let topic = topics[topic_idx].clone();

                    let mut items = topic_groups.get(&topic).unwrap().clone();

                    // Sort items in this topic by relevance
                    items.sort_by(|a, b| {
                        b.importance
                            .score
                            .partial_cmp(&a.importance.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    // Take the top item we haven't added yet
                    if let Some(pos) = items.iter().position(|item| !added_ids.contains(&item.id)) {
                        let item = items.remove(pos);
                        let estimated_tokens =
                            (item.content.len() as f32 * avg_tokens_per_char) as usize;

                        if current_tokens + estimated_tokens <= max_tokens {
                            added_ids.insert(item.id.clone());
                            selected_items.push(item);
                            current_tokens += estimated_tokens;
                        }
                    }

                    // If no more items for this topic, remove it
                    if items.is_empty() || items.iter().all(|item| added_ids.contains(&item.id)) {
                        topics.remove(topic_idx);
                    } else {
                        i += 1;
                    }
                }
            }
            _ => todo!(),
        }

        // Format selected items into a context string
        let mut context_str = String::new();

        for (i, item) in selected_items.iter().enumerate() {
            // Add separator
            if i > 0 {
                context_str.push_str("\n---\n");
            }

            // Add metadata if requested
            if include_metadata {
                context_str.push_str(&format!("ID: {}\n", item.id));
                context_str.push_str(&format!("Type: {:?}\n", item.item_type));
                context_str.push_str(&format!("Topics: {}\n", item.topics.join(", ")));
                context_str.push_str(&format!("Created: {:?}\n", item.created_at));
                context_str.push_str(&format!("Importance: {}\n", item.importance.score));
                context_str.push('\n');
            }

            // Add content
            context_str.push_str(&item.content);
        }

        Ok(context_str)
    }

    /// Build a temporal context for time-based analysis
    #[tracing::instrument(level = "debug", skip(self), fields(focus = ?time_focus), err)]
    pub async fn build_temporal_context(
        &self,
        time_focus: TimeFocus,
        time_range: Option<(SystemTime, SystemTime)>,
        related_topics: Option<Vec<String>>,
    ) -> Result<TemporalContext, SistenceMemoryError> {
        debug!("Building temporal context with focus: {:?}", time_focus);

        // Get all items within the time range
        let mut items = Vec::new();

        for item_ref in self.memory_index.iter() {
            let item = item_ref.value().clone();

            // Apply time range filter if provided
            if let Some((start, end)) = time_range {
                if item.created_at < start || item.created_at > end {
                    continue;
                }
            }

            // Apply topic filter if provided
            if let Some(topics) = &related_topics {
                if !item.topics.iter().any(|t| topics.contains(t)) {
                    continue;
                }
            }

            items.push(item);
        }

        // Sort items by creation time
        items.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Apply focus strategy
        let focused_items = match time_focus {
            TimeFocus::Present => {
                // Take most recent items
                let count = 10.min(items.len());
                items.into_iter().rev().take(count).collect()
            }
            TimeFocus::Past => {
                // Take distributed samples across the time range
                if items.len() <= 10 {
                    items
                } else {
                    let step = items.len() / 10;
                    let mut result = Vec::new();

                    for i in (0..items.len()).step_by(step) {
                        result.push(items[i].clone());

                        if result.len() >= 10 {
                            break;
                        }
                    }

                    result
                }
            }
            TimeFocus::Comparative => {
                // Take items with highest importance
                let mut important_items = items;
                important_items.sort_by(|a, b| {
                    b.importance
                        .base_score
                        .partial_cmp(&a.importance.base_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                important_items.into_iter().take(10).collect()
            }
            _ => items,
        };

        // Convert to MemoryItems
        let memory_items: Vec<MemoryItem> = focused_items
            .into_iter()
            .map(|item| self.detailed_to_memory_item(item, None))
            .collect();

        // Create temporal context
        let temporal_context = TemporalContext {
            current_time: Some(SystemTime::now()),
            time_focus: Some(match time_focus {
                TimeFocus::Past => "past".to_string(),
                TimeFocus::Present => "present".to_string(),
                TimeFocus::Future => "future".to_string(),
                TimeFocus::Comparative => "comparative".to_string(),
            }),
            relevant_periods: Vec::new(),
            historical_context: Some(json!(memory_items).to_string()),
        };

        Ok(temporal_context)
    }
}

// Utility functions for the StatelessRelevantMemory implementation

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::provider::capabilities::relevant_memory::DetailedMemoryItem;

/// Calculate similarity between two sets of strings
pub fn calculate_set_similarity(set1: &[String], set2: &[String]) -> f32 {
    if set1.is_empty() && set2.is_empty() {
        return 0.0;
    }

    let set1_len = set1.len();
    let set2_len = set2.len();

    // Count common elements
    let common_count = set1.iter().filter(|s| set2.contains(s)).count();

    // Jaccard similarity: intersection / union
    let union_size = set1_len + set2_len - common_count;
    if union_size == 0 {
        return 0.0;
    }

    common_count as f32 / union_size as f32
}

/// Calculate similarity between two text strings
pub fn calculate_text_similarity(text1: &str, text2: &str) -> f32 {
    if text1.is_empty() && text2.is_empty() {
        return 1.0;
    }

    if text1.is_empty() || text2.is_empty() {
        return 0.0;
    }

    let text1_lower = text1.to_lowercase();
    let text2_lower = text2.to_lowercase();

    // Split into tokens
    let tokens1: Vec<&str> = text1_lower.split_whitespace().collect();

    let tokens2: Vec<&str> = text2_lower.split_whitespace().collect();

    // Calculate tokens similarity using Jaccard similarity
    let common_tokens = tokens1.iter().filter(|t| tokens2.contains(t)).count();

    let union_size = tokens1.len() + tokens2.len() - common_tokens;
    if union_size == 0 {
        return 0.0;
    }

    common_tokens as f32 / union_size as f32
}

/// Calculate topic match between item topics and context topics
pub fn calculate_topic_match(item_topics: &[String], context_topics: &[String]) -> f32 {
    if context_topics.is_empty() {
        return 0.5; // Neutral score if no context topics
    }

    if item_topics.is_empty() {
        return 0.0; // No match if item has no topics
    }

    // Count how many context topics are in the item topics
    let matching_topics = context_topics
        .iter()
        .filter(|t| item_topics.contains(t))
        .count();

    // Normalize by context topics count
    (matching_topics as f32) / (context_topics.len() as f32)
}

/// Calculate recency factor based on creation time
pub fn calculate_recency_factor(created_at: &SystemTime) -> f32 {
    // Get duration since creation
    let now = SystemTime::now();
    let duration_since = now
        .duration_since(*created_at)
        .unwrap_or(Duration::from_secs(0));

    // Convert to hours
    let hours_old = duration_since.as_secs() as f32 / 3600.0;

    // Decay function: 1.0 for very recent items, approaching 0.0 for old items
    // Half-life of 1 week (168 hours)
    let half_life = 168.0;

    0.5f32.powf(hours_old / half_life)
}

/// Calculate similarity based on reference relationships
pub fn calculate_reference_similarity(
    item1: &DetailedMemoryItem,
    item2: &DetailedMemoryItem,
) -> f32 {
    // Check direct references
    let direct_ref1 = item1.references.iter().any(|r| r.ref_id == item2.id);

    let direct_ref2 = item2.references.iter().any(|r| r.ref_id == item1.id);

    if direct_ref1 && direct_ref2 {
        // Bidirectional reference
        return 1.0;
    } else if direct_ref1 || direct_ref2 {
        // Unidirectional reference
        return 0.8;
    }

    // Check common references
    let refs1: Vec<&String> = item1.references.iter().map(|r| &r.ref_id).collect();

    let refs2: Vec<&String> = item2.references.iter().map(|r| &r.ref_id).collect();

    let common_refs = refs1.iter().filter(|r| refs2.contains(r)).count();

    if common_refs > 0 {
        // Items share references
        let max_possible = refs1.len().max(refs2.len());
        if max_possible == 0 {
            return 0.0;
        }

        0.6 * (common_refs as f32 / max_possible as f32)
    } else {
        // No reference relationship
        0.0
    }
}

/// Calculate similarity between tag maps
pub fn calculate_tag_similarity(
    tags1: &HashMap<String, String>,
    tags2: &HashMap<String, String>,
) -> f32 {
    if tags1.is_empty() && tags2.is_empty() {
        return 0.0; // No tags to compare
    }

    // Check for exact key-value matches
    let mut exact_matches = 0;

    for (key, value) in tags1 {
        if let Some(other_value) = tags2.get(key) {
            if value == other_value {
                exact_matches += 1;
            }
        }
    }

    // Calculate union size (keys in either map)
    let all_keys = tags1
        .keys()
        .chain(tags2.keys())
        .collect::<std::collections::HashSet<_>>();

    let union_size = all_keys.len();
    if union_size == 0 {
        return 0.0;
    }

    // Similarity is the proportion of exact matches to all possible matches
    exact_matches as f32 / union_size as f32
}

/// Extract a meaningful label for a knowledge node from a memory item
pub fn get_item_label(item: &DetailedMemoryItem) -> String {
    // Try different strategies to get a good label

    // Strategy 1: Check if there's a title tag
    if let Some(title) = item.tags.get("title") {
        return title.clone();
    }

    // Strategy 2: Check if there's a name tag
    if let Some(name) = item.tags.get("name") {
        return name.clone();
    }

    // Strategy 3: Use first line of content (if it's short)
    let first_line = item.content.lines().next().unwrap_or("").trim();
    if first_line.len() <= 50 && !first_line.is_empty() {
        return first_line.to_string();
    }

    // Strategy 4: Use the first sentence (if it's short)
    let first_sentence_end = item.content.find('.').unwrap_or(item.content.len().min(50));
    let first_sentence = item.content[..first_sentence_end].trim();
    if first_sentence.len() <= 50 && !first_sentence.is_empty() {
        return format!("{}...", first_sentence);
    }

    // Strategy 5: Use ID with type and topics if the content is too long
    let type_str = format!("{:?}", item.item_type).to_lowercase();
    let topics_str = if !item.topics.is_empty() {
        format!(" about {}", item.topics.join(", "))
    } else {
        String::new()
    };

    format!("{} {}{}", type_str, item.id, topics_str)
}

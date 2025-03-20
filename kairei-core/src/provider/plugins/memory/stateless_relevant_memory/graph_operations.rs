// Graph-related operations for the StatelessRelevantMemory implementation

use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use tracing::debug;
use uuid::Uuid;

use crate::provider::capabilities::relevant_memory::DetailedMemoryItem;
use crate::provider::capabilities::sistence_memory::*;

use super::StatelessRelevantMemory;
use super::utility_functions::get_item_label;
use crate::provider::capabilities::relevant_memory::RelevantMemoryCapability;

impl StatelessRelevantMemory {
    /// Create a simple knowledge graph from search results
    #[tracing::instrument(level = "debug", skip(self, results, context), fields(query = %query, result_count = %results.len()), err)]
    pub async fn create_simple_knowledge_graph(
        &self,
        results: &[(DetailedMemoryItem, f32, HashMap<String, f32>)],
        query: &str,
        context: &SearchContext,
    ) -> Result<KnowledgeNode, SistenceMemoryError> {
        // Create central node based on query
        let root_node = KnowledgeNode {
            id: format!("query-{}", Uuid::new_v4()),
            label: query.to_string(),
            node_type: "query".to_string(),
            properties: HashMap::from([
                ("query".to_string(), query.to_string()),
                ("context_id".to_string(), context.context_id.clone()),
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
        };

        // Build connections to result items
        let mut connections = Vec::new();

        for (item, relevance, _components) in results.iter() {
            // Create connection to each result
            let connection = KnowledgeConnection {
                target_id: item.id.clone(),
                relation_type: "result".to_string(),
                strength: *relevance,
                is_outgoing: true,
            };

            connections.push(connection);

            // In a real implementation, we would also find connections between results
            // but for simplicity, we'll just create a star topology
        }

        // Create a complete graph
        let mut graph = root_node;
        graph.connections = connections;

        Ok(graph)
    }

    /// Build a relationship graph starting from specified items
    #[tracing::instrument(level = "debug", skip(self), fields(starting_items = ?starting_item_ids, max_depth = %max_depth), err)]
    pub async fn build_relationship_graph(
        &self,
        starting_item_ids: Vec<String>,
        max_depth: usize,
        min_relationship_strength: f32,
    ) -> Result<KnowledgeNode, SistenceMemoryError> {
        debug!(
            "Building relationship graph from {} items with max depth {}",
            starting_item_ids.len(),
            max_depth
        );

        if starting_item_ids.is_empty() {
            return Err(SistenceMemoryError::InvalidInput(
                "No starting items provided".to_string(),
            ));
        }

        // Create a central node representing the graph
        let graph_id = Uuid::new_v4().to_string();
        let mut root_node = KnowledgeNode {
            id: graph_id.clone(),
            label: "Knowledge Graph".to_string(),
            node_type: "graph_root".to_string(),
            properties: HashMap::from([
                (
                    "created_at".to_string(),
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                ),
                ("starting_items".to_string(), starting_item_ids.join(", ")),
                ("max_depth".to_string(), max_depth.to_string()),
                (
                    "min_strength".to_string(),
                    min_relationship_strength.to_string(),
                ),
            ]),
            connections: Vec::new(),
        };

        // Track processed items to avoid cycles
        let mut processed_items = HashSet::new();

        // Track nodes to process
        let mut nodes_to_process = Vec::new();

        // Add starting items
        for id in &starting_item_ids {
            if let Ok(item) = self.retrieve_memory_item(id).await {
                // Create node for this item
                let node = KnowledgeNode {
                    id: item.id.clone(),
                    label: get_item_label(&item),
                    node_type: format!("{:?}", item.item_type).to_lowercase(),
                    properties: HashMap::from([
                        (
                            "created_at".to_string(),
                            item.created_at
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                                .to_string(),
                        ),
                        (
                            "importance".to_string(),
                            item.importance.base_score.to_string(),
                        ),
                        ("topics".to_string(), item.topics.join(", ")),
                    ]),
                    connections: Vec::new(),
                };

                // Add connection from root to this node
                let connection = KnowledgeConnection {
                    target_id: item.id.clone(),
                    relation_type: "starting_item".to_string(),
                    strength: 1.0,
                    is_outgoing: true,
                };

                root_node.connections.push(connection);

                // Add to processing queue
                nodes_to_process.push((item.clone(), node, 1)); // Depth 1
                processed_items.insert(item.id.clone());
            }
        }

        // Process nodes up to max depth
        while let Some((item, mut node, depth)) = nodes_to_process.pop() {
            if depth >= max_depth {
                continue; // Don't process further at max depth
            }

            // Process references
            for reference in &item.references {
                if reference.strength < min_relationship_strength {
                    continue; // Skip weak references
                }

                // Add connection
                let connection = KnowledgeConnection {
                    target_id: reference.ref_id.clone(),
                    relation_type: reference.ref_type.clone(),
                    strength: reference.strength,
                    is_outgoing: true,
                };

                node.connections.push(connection);

                // Process referenced item if not already processed
                if !processed_items.contains(&reference.ref_id) {
                    if let Ok(ref_item) = self.retrieve_memory_item(&reference.ref_id).await {
                        // Create node for referenced item
                        let ref_node = KnowledgeNode {
                            id: ref_item.id.clone(),
                            label: get_item_label(&ref_item),
                            node_type: format!("{:?}", ref_item.item_type).to_lowercase(),
                            properties: HashMap::from([
                                (
                                    "created_at".to_string(),
                                    ref_item
                                        .created_at
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs()
                                        .to_string(),
                                ),
                                (
                                    "importance".to_string(),
                                    ref_item.importance.base_score.to_string(),
                                ),
                                ("topics".to_string(), ref_item.topics.join(", ")),
                            ]),
                            connections: Vec::new(),
                        };

                        // Add to processing queue
                        nodes_to_process.push((ref_item.clone(), ref_node, depth + 1));
                        processed_items.insert(ref_item.id.clone());
                    }
                }
            }
        }

        Ok(root_node)
    }
}

// Link-related operations for the StatelessRelevantMemory implementation

use std::collections::HashMap;
use std::time::SystemTime;

use tracing::debug;

use crate::provider::capabilities::relevant_memory::DetailedReference;
use crate::provider::capabilities::sistence_memory::*;

use super::StatelessRelevantMemory;
use crate::provider::capabilities::relevant_memory::RelevantMemoryCapability;

impl StatelessRelevantMemory {
    /// Create links between memory items
    #[tracing::instrument(level = "debug", skip(self, links), fields(link_count = %links.len()), err)]
    pub async fn create_item_links(&self, links: Vec<ItemLink>) -> Result<(), SistenceMemoryError> {
        debug!("Creating {} item links", links.len());

        for link in links {
            // Get source item
            let mut source_item = self.retrieve_memory_item(&link.source_id).await?;

            // Add reference to source item
            let reference = DetailedReference {
                ref_id: link.target_id.clone(),
                ref_type: link.relation_type.clone(),
                context: link.context.clone(),
                strength: link.strength,
                created_at: SystemTime::now(),
                metadata: HashMap::new(),
            };

            // Check if reference already exists
            let already_exists = source_item
                .references
                .iter()
                .any(|r| r.ref_id == reference.ref_id && r.ref_type == reference.ref_type);

            if !already_exists {
                source_item.references.push(reference);

                // Update source item
                self.update_memory_item(source_item).await?;
            }

            // If bidirectional, add reverse reference
            if link.is_bidirectional {
                // Get target item
                let mut target_item = self.retrieve_memory_item(&link.target_id).await?;

                // Add reference to target item
                let reverse_reference = DetailedReference {
                    ref_id: link.source_id.clone(),
                    ref_type: format!("reverse_{}", link.relation_type),
                    context: link.context.clone(),
                    strength: link.strength,
                    created_at: SystemTime::now(),
                    metadata: HashMap::new(),
                };

                // Check if reverse reference already exists
                let reverse_already_exists = target_item.references.iter().any(|r| {
                    r.ref_id == reverse_reference.ref_id && r.ref_type == reverse_reference.ref_type
                });

                if !reverse_already_exists {
                    target_item.references.push(reverse_reference);

                    // Update target item
                    self.update_memory_item(target_item).await?;
                }
            }
        }

        Ok(())
    }

    /// Get all links for a memory item
    #[tracing::instrument(level = "debug", skip(self), fields(item_id = %item_id, incoming = %include_incoming, outgoing = %include_outgoing), err)]
    pub async fn get_all_item_links(
        &self,
        item_id: &MemoryId,
        include_incoming: bool,
        include_outgoing: bool,
    ) -> Result<Vec<ItemLink>, SistenceMemoryError> {
        debug!("Getting links for item ID: {}", item_id);

        let mut links = Vec::new();

        // Get outgoing links (references from this item)
        if include_outgoing {
            let item = self.retrieve_memory_item(item_id).await?;

            for reference in &item.references {
                let link = ItemLink {
                    source_id: item_id.clone(),
                    target_id: reference.ref_id.clone(),
                    relation_type: reference.ref_type.clone(),
                    strength: reference.strength,
                    created_at: SystemTime::now(),
                    context: reference.context.clone(),
                    metadata: None,
                    is_bidirectional: false,
                };

                links.push(link);
            }
        }

        // Get incoming links (references to this item)
        if include_incoming {
            // This is inefficient in a real implementation - would use an index
            for item_ref in self.memory_index.iter() {
                let item = item_ref.value();

                // Skip the item itself
                if item.id == *item_id {
                    continue;
                }

                // Check if this item references our target
                for reference in &item.references {
                    if reference.ref_id == *item_id {
                        let link = ItemLink {
                            source_id: item.id.clone(),
                            target_id: item_id.clone(),
                            relation_type: reference.ref_type.clone(),
                            strength: reference.strength,
                            context: reference.context.clone(),
                            created_at: reference.created_at,
                            metadata: None,
                            is_bidirectional: false,
                        };

                        links.push(link);
                    }
                }
            }
        }

        // Check for bidirectional links
        if include_incoming && include_outgoing {
            for mut link in links.clone().into_iter() {
                // Find if there's a reverse link
                let has_reverse = links
                    .iter()
                    .any(|l| l.source_id == link.target_id && l.target_id == link.source_id);

                if has_reverse {
                    link.is_bidirectional = true;
                }
            }
        }

        Ok(links)
    }
}

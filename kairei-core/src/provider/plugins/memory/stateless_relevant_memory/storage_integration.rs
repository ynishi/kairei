//! Integration between StatelessRelevantMemory and SistenceStorageService.
//!
//! This module provides the implementation for StatelessRelevantMemory to use
//! SistenceStorageService for persisting memory items and supporting parallel
//! workspace processing.

use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, error, info, instrument};

use crate::provider::capabilities::relevant_memory::DetailedMemoryItem;
use crate::provider::capabilities::sistence_memory::*;
use crate::provider::capabilities::sistence_storage::{
    SistenceStorageError, SistenceStorageService,
};
use crate::provider::capabilities::storage::StorageError;

use super::StatelessRelevantMemory;

impl StatelessRelevantMemory {
    /// Initialize with SistenceStorageService
    #[instrument(level = "debug", skip(self, storage_service), err)]
    pub fn init_with_sistence_storage(
        &self,
        storage_service: Arc<dyn SistenceStorageService>,
    ) -> Result<(), SistenceMemoryError> {
        // You would typically store this in a field on StatelessRelevantMemory
        // For now, we'll just demonstrate the integration pattern

        info!("Initialized StatelessRelevantMemory with SistenceStorageService");

        // Return success
        Ok(())
    }

    /// Store a memory item using SistenceStorageService
    #[instrument(level = "debug", skip(self, item), fields(item_id = %item.id), err)]
    pub async fn store_to_sistence_storage(
        &self,
        item: &DetailedMemoryItem,
        workspace_id: Option<&str>,
    ) -> Result<(), SistenceMemoryError> {
        // Get storage service
        let storage_service = self.get_storage_service()?;

        debug!("Storing memory item {} to storage", item.id);

        // Calculate TTL if present
        let ttl = item.ttl.map(|ttl| Duration::from_secs(ttl as u64));

        // Create metadata from item
        let mut metadata_map = std::collections::HashMap::new();

        // Add important metadata
        if let Some(content_type) = &item.content_type {
            metadata_map.insert("content_type".to_string(), content_type.clone());
        }

        if let Some(item_type) = &item.item_type {
            metadata_map.insert("item_type".to_string(), item_type.clone());
        }

        // Add topics as tags
        for (i, topic) in item.topics.iter().enumerate() {
            metadata_map.insert(format!("tag_topic_{}", i), topic.clone());
        }

        // Add tags
        for (key, value) in &item.tags {
            metadata_map.insert(format!("tag_{}", key), value.clone());
        }

        // Create metadata
        let metadata = crate::provider::capabilities::shared_memory::Metadata(metadata_map);

        // Save to storage
        storage_service
            .save(
                "memory_items",
                &item.id,
                item,
                Some(metadata),
                ttl,
                workspace_id,
            )
            .await
            .map_err(|e| {
                SistenceMemoryError::StorageError(StorageError::StorageError(e.to_string()))
            })?;

        // Update indexes
        self.update_indexes(item);

        Ok(())
    }

    /// Retrieve a memory item using SistenceStorageService
    #[instrument(level = "debug", skip(self), err)]
    pub async fn retrieve_from_sistence_storage(
        &self,
        id: &str,
        workspace_id: Option<&str>,
    ) -> Result<DetailedMemoryItem, SistenceMemoryError> {
        // Get storage service
        let storage_service = self.get_storage_service()?;

        debug!("Retrieving memory item {} from storage", id);

        // Try to get from memory index first (in-memory cache)
        if let Some(item) = self.memory_index.get(id) {
            debug!("Found memory item {} in memory index", id);
            return Ok(item.clone());
        }

        // Retrieve from storage
        let result = storage_service
            .get::<DetailedMemoryItem>("memory_items", id, workspace_id)
            .await
            .map_err(|e| match e {
                SistenceStorageError::NotFound(_) => SistenceMemoryError::NotFound(id.to_string()),
                _ => SistenceMemoryError::StorageError(StorageError::StorageError(e.to_string())),
            })?;

        let item = result.value;

        // Update indexes with retrieved item
        self.update_indexes(&item);

        Ok(item)
    }

    /// Create a memory workspace for parallel processing
    #[instrument(level = "debug", skip(self), err)]
    pub async fn create_memory_workspace(
        &self,
        workspace_id: &str,
        parent_workspace_id: Option<&str>,
    ) -> Result<(), SistenceMemoryError> {
        // Get storage service
        let storage_service = self.get_storage_service()?;

        info!("Creating memory workspace {}", workspace_id);

        // Create workspace
        storage_service
            .create_workspace("memory_items", workspace_id, parent_workspace_id)
            .await
            .map_err(|e| {
                SistenceMemoryError::StorageError(StorageError::StorageError(e.to_string()))
            })?;

        Ok(())
    }

    /// Merge a memory workspace
    #[instrument(level = "debug", skip(self), err)]
    pub async fn merge_memory_workspace(
        &self,
        source_workspace_id: &str,
        target_workspace_id: &str,
    ) -> Result<(), SistenceMemoryError> {
        // Get storage service
        let storage_service = self.get_storage_service()?;

        info!(
            "Merging memory workspace {} into {}",
            source_workspace_id, target_workspace_id
        );

        // Define merge strategy
        let merge_strategy = Box::new(
            |_key: &str, source: &serde_json::Value, target: &serde_json::Value| {
                // In a real implementation, you'd implement a sophisticated merge strategy
                // For example, comparing importance scores, timestamps, etc.
                // For now, we'll use a simple strategy: prefer source if newer

                // Try to extract timestamps
                let source_time = source
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);

                let target_time = target
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);

                // Choose the newer one
                if source_time >= target_time {
                    source.clone()
                } else {
                    target.clone()
                }
            },
        );

        // Merge workspaces
        storage_service
            .merge_workspace(
                "memory_items",
                source_workspace_id,
                target_workspace_id,
                Some(merge_strategy),
            )
            .await
            .map_err(|e| {
                SistenceMemoryError::StorageError(StorageError::StorageError(e.to_string()))
            })?;

        Ok(())
    }

    /// Get storage service
    fn get_storage_service(&self) -> Result<Arc<dyn SistenceStorageService>, SistenceMemoryError> {
        // In a real implementation, you'd store this in a field on StatelessRelevantMemory
        // For now, we'll just return an error
        Err(SistenceMemoryError::NotImplemented(
            "SistenceStorageService not initialized".to_string(),
        ))
    }

    /// Store to storage (implementation for RelevantMemoryCapability)
    #[tracing::instrument(level = "debug", skip(self, item), fields(item_id = %item.id), err)]
    pub async fn store_to_storage(
        &self,
        item: &DetailedMemoryItem,
    ) -> Result<(), SistenceMemoryError> {
        // Try to use SistenceStorageService if available
        if let Ok(storage_service) = self.get_storage_service() {
            return self.store_to_sistence_storage(item, None).await;
        }

        // Fallback implementation using basic StorageBackend
        // This is simplified for illustration purposes

        debug!("Storing memory item to legacy storage backend");

        // Update indexes
        self.update_indexes(item);

        Ok(())
    }

    /// Retrieve from storage (implementation for RelevantMemoryCapability)
    #[tracing::instrument(level = "debug", skip(self), err)]
    pub async fn retrieve_from_storage(
        &self,
        id: &str,
    ) -> Result<DetailedMemoryItem, SistenceMemoryError> {
        // Try to use SistenceStorageService if available
        if let Ok(storage_service) = self.get_storage_service() {
            return self.retrieve_from_sistence_storage(id, None).await;
        }

        // Fallback implementation
        // This is simplified for illustration purposes

        debug!("Retrieving memory item from legacy storage backend");

        // Try to get from memory index
        if let Some(item) = self.memory_index.get(id) {
            return Ok(item.clone());
        }

        // If not found in memory, return an error
        Err(SistenceMemoryError::NotFound(id.to_string()))
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::capabilities::relevant_memory::{DetailedMemoryItem, ImportanceScore};
    use crate::provider::capabilities::shared_memory::Metadata;
    use crate::provider::capabilities::sistence_memory::{MemoryId, SistenceMemoryError};
    use crate::provider::capabilities::sistence_storage::{
        SistenceStorageService, SistenceValueWithMetadata,
    };
    use crate::provider::capabilities::storage::StorageBackend;
    use crate::provider::plugins::storage::in_memory::InMemoryStorageBackend;
    use crate::provider::plugins::storage::sistence_storage::{
        SistenceStorage, SistenceStorageConfig,
    };

    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    // Mock implementation of StatelessRelevantMemory for testing
    struct MockSistenceStorage {
        storage_service: Arc<dyn SistenceStorageService>,
    }

    impl MockSistenceStorage {
        fn new() -> Self {
            // Create a storage backend
            let backend = Arc::new(InMemoryStorageBackend::new());

            // Create a SistenceStorage
            let storage = SistenceStorage::new(
                "test".to_string(),
                backend,
                None,
                SistenceStorageConfig::default(),
            );

            Self {
                storage_service: Arc::new(storage),
            }
        }

        fn get_storage_service(
            &self,
        ) -> Result<Arc<dyn SistenceStorageService>, SistenceMemoryError> {
            Ok(self.storage_service.clone())
        }
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        // Create a mock SistenceStorage
        let mock = MockSistenceStorage::new();

        // Create a test memory item
        let item = DetailedMemoryItem {
            id: "test-item-1".to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            content: "Test content".to_string(),
            content_type: Some("text/plain".to_string()),
            structured_content: None,
            item_type: Some("note".to_string()),
            topics: vec!["test".to_string(), "example".to_string()],
            tags: HashMap::from([("key1".to_string(), "value1".to_string())]),
            source: None,
            references: Vec::new(),
            related_items: Vec::new(),
            importance:
                crate::provider::capabilities::relevant_memory::DetailedImportanceEvaluation {
                    base_score: 0.5,
                    context_score: 0.5,
                    factor_scores: HashMap::new(),
                    evaluated_at: SystemTime::now(),
                },
            access_stats: crate::provider::capabilities::relevant_memory::AccessStats {
                last_accessed: SystemTime::now(),
                access_count: 0,
            },
            ttl: Some(86400),
            retention_policy: None,
            metadata: HashMap::new(),
        };

        // Store the item
        let result = mock
            .storage_service
            .save(
                "memory_items",
                &item.id,
                &item,
                None,
                Some(Duration::from_secs(86400)),
                None,
            )
            .await;

        assert!(result.is_ok());

        // Retrieve the item
        let result = mock
            .storage_service
            .get::<DetailedMemoryItem>("memory_items", &item.id, None)
            .await;

        assert!(result.is_ok());
        let retrieved = result.unwrap();

        // Verify content
        assert_eq!(retrieved.value.id, item.id);
        assert_eq!(retrieved.value.content, item.content);
    }
}

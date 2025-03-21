//! Basic implementation of SistenceStorageService.
//!
//! This module provides a foundational implementation of the SistenceStorageService
//! trait that wraps an existing StorageBackend and adds advanced features
//! like versioning, TTL handling, event-based persistence, and workspace isolation.

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::provider::capabilities::shared_memory::Metadata;
use crate::provider::capabilities::sistence_storage::{
    BatchResult, OrderBy, PaginationInfo, QueryOptions, SistenceStorageError,
    SistenceStorageService, SistenceValueWithMetadata, StorageEvent, StorageEventType,
};
use crate::provider::capabilities::storage::{StorageBackend, StorageError, ValueWithMetadata};
use crate::provider::event::event_bus::EventBus;

/// Basic implementation of SistenceStorageService
pub struct SistenceStorage {
    /// Unique identifier
    id: String,

    /// Underlying storage backend
    storage: Arc<dyn StorageBackend>,

    /// Event bus (for event-based persistence)
    event_bus: Option<Arc<EventBus>>,

    /// Enable versioning
    enable_versioning: bool,

    /// Default TTL value (seconds)
    default_ttl: Option<Duration>,

    /// Use cache
    use_cache: bool,

    /// Item cache (key -> version mapping for optimistic locking)
    version_cache: Arc<DashMap<String, u64>>,

    /// Batch operation size limit
    batch_size_limit: usize,

    /// Workspace metadata
    workspace_metadata: Arc<DashMap<String, HashMap<String, String>>>,
}

impl SistenceStorage {
    /// Create a new SistenceStorage instance
    pub fn new(
        id: String,
        storage: Arc<dyn StorageBackend>,
        event_bus: Option<Arc<EventBus>>,
        config: SistenceStorageConfig,
    ) -> Self {
        Self {
            id,
            storage,
            event_bus,
            enable_versioning: config.enable_versioning,
            default_ttl: config.default_ttl,
            use_cache: config.use_cache,
            version_cache: Arc::new(DashMap::new()),
            batch_size_limit: config.batch_size_limit.unwrap_or(100),
            workspace_metadata: Arc::new(DashMap::new()),
        }
    }

    /// Create a full storage key
    fn make_storage_key(&self, namespace: &str, key: &str, workspace_id: Option<&str>) -> String {
        match workspace_id {
            Some(ws_id) => format!("{}/{}/{}", namespace, ws_id, key),
            None => format!("{}/main/{}", namespace, key),
        }
    }

    /// Generate a new version
    fn generate_version(&self) -> u64 {
        let uuid = Uuid::new_v4();
        let bytes = uuid.as_bytes();
        let mut version = 0u64;

        // Use the first 8 bytes as a u64
        for i in 0..8 {
            version = (version << 8) | (bytes[i] as u64);
        }

        version
    }

    /// Update version cache
    fn update_version_cache(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
        version: u64,
    ) {
        if self.use_cache {
            let cache_key = self.make_storage_key(namespace, key, workspace_id);
            self.version_cache.insert(cache_key, version);
        }
    }

    /// Get cached version
    fn get_cached_version(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Option<u64> {
        if self.use_cache {
            let cache_key = self.make_storage_key(namespace, key, workspace_id);
            self.version_cache.get(&cache_key).map(|v| *v)
        } else {
            None
        }
    }

    /// Convert SistenceValueWithMetadata to storage ValueWithMetadata
    fn to_storage_value<T: Serialize>(
        &self,
        value: &T,
        metadata: Option<&Metadata>,
        ttl: Option<Duration>,
        workspace_id: Option<&str>,
        version: Option<u64>,
    ) -> Result<ValueWithMetadata, SistenceStorageError> {
        let now = SystemTime::now();
        let version = version.unwrap_or_else(|| self.generate_version());

        // Create metadata structure with version, timestamps, and workspace
        let mut metadata_map = metadata.cloned().unwrap_or_default().0;

        // Add system metadata
        metadata_map.insert("version".to_string(), version.to_string());
        metadata_map.insert(
            "created_at".to_string(),
            now.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        );
        metadata_map.insert(
            "updated_at".to_string(),
            now.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        );

        if let Some(ws_id) = workspace_id {
            metadata_map.insert("workspace_id".to_string(), ws_id.to_string());
        }

        // Serialize the value
        let serialized = serde_json::to_value(value)
            .map_err(|e| SistenceStorageError::SerializationError(e.to_string()))?;

        Ok(ValueWithMetadata {
            value: serialized,
            metadata: Metadata(metadata_map),
            expiry: ttl.map(|d| std::time::Instant::now() + d),
        })
    }

    /// Convert storage ValueWithMetadata to SistenceValueWithMetadata
    fn from_storage_value<T: DeserializeOwned>(
        &self,
        key: &str,
        value_with_metadata: ValueWithMetadata,
    ) -> Result<SistenceValueWithMetadata<T>, SistenceStorageError> {
        let metadata = value_with_metadata.metadata;

        // Extract version
        let version = metadata
            .0
            .get("version")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Extract timestamps
        let created_at = metadata
            .0
            .get("created_at")
            .and_then(|v| v.parse::<u64>().ok())
            .map(|secs| SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
            .unwrap_or_else(SystemTime::now);

        let updated_at = metadata
            .0
            .get("updated_at")
            .and_then(|v| v.parse::<u64>().ok())
            .map(|secs| SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
            .unwrap_or_else(SystemTime::now);

        // Extract workspace ID
        let workspace_id = metadata.0.get("workspace_id").cloned();

        // Extract TTL (if present in metadata)
        let ttl = metadata
            .0
            .get("ttl")
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs);

        // Extract tags (keys starting with "tag_")
        let mut tags = HashMap::new();
        for (k, v) in &metadata.0 {
            if k.starts_with("tag_") {
                let tag_key = k[4..].to_string(); // Remove "tag_" prefix
                tags.insert(tag_key, v.clone());
            }
        }

        // Deserialize the value
        let value = serde_json::from_value(value_with_metadata.value).map_err(|e| {
            SistenceStorageError::DeserializationError(format!(
                "Failed to deserialize key {}: {}",
                key, e
            ))
        })?;

        Ok(SistenceValueWithMetadata {
            value,
            metadata,
            created_at,
            updated_at,
            ttl,
            workspace_id,
            version,
            tags,
        })
    }

    /// Publish an event to the event bus
    async fn emit_event(&self, event: StorageEvent) -> Result<(), SistenceStorageError> {
        if let Some(event_bus) = &self.event_bus {
            let event_json = serde_json::to_value(&event)
                .map_err(|e| SistenceStorageError::SerializationError(e.to_string()))?;

            event_bus
                .publish("storage_events", &event_json)
                .await
                .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;
        }

        Ok(())
    }
}

/// SistenceStorage configuration
#[derive(Debug, Clone)]
pub struct SistenceStorageConfig {
    /// Enable versioning
    pub enable_versioning: bool,

    /// Default TTL value (seconds)
    pub default_ttl: Option<Duration>,

    /// Use cache
    pub use_cache: bool,

    /// Batch operation size limit
    pub batch_size_limit: Option<usize>,
}

impl Default for SistenceStorageConfig {
    fn default() -> Self {
        Self {
            enable_versioning: true,
            default_ttl: Some(Duration::from_secs(86400 * 30)), // 30 days
            use_cache: true,
            batch_size_limit: Some(100),
        }
    }
}

impl Clone for SistenceStorage {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            storage: self.storage.clone_backend(),
            event_bus: self.event_bus.clone(),
            enable_versioning: self.enable_versioning,
            default_ttl: self.default_ttl,
            use_cache: self.use_cache,
            version_cache: self.version_cache.clone(),
            batch_size_limit: self.batch_size_limit,
            workspace_metadata: self.workspace_metadata.clone(),
        }
    }
}

#[async_trait]
impl SistenceStorageService for SistenceStorage {
    #[instrument(level = "debug", skip(self))]
    fn clone_service(&self) -> Box<dyn SistenceStorageService> {
        Box::new(self.clone())
    }

    #[instrument(level = "debug", skip(self))]
    async fn is_available(&self) -> bool {
        self.storage.is_available().await
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn list_namespaces(&self) -> Result<Vec<String>, SistenceStorageError> {
        // This is a simplified implementation - in a real system,
        // you'd need to implement a way to track namespaces

        // For now, return a hardcoded list of common namespaces
        Ok(vec![
            "memory_items".to_string(),
            "system".to_string(),
            "user_data".to_string(),
        ])
    }

    #[instrument(level = "debug", skip(self, value), err)]
    async fn save<T: Serialize + Send + Sync>(
        &self,
        namespace: &str,
        key: &str,
        value: T,
        metadata: Option<Metadata>,
        ttl: Option<Duration>,
        workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError> {
        let storage_key = self.make_storage_key(namespace, key, workspace_id);
        let version = self.generate_version();

        // Convert to storage value
        let storage_value = self.to_storage_value(
            &value,
            metadata.as_ref(),
            ttl.or(self.default_ttl),
            workspace_id,
            Some(version),
        )?;

        // Save to storage
        self.storage
            .save_key("data", &storage_key, &storage_value)
            .await
            .map_err(|e| match e {
                StorageError::AccessDenied(s) => SistenceStorageError::PermissionError(s),
                StorageError::InvalidPath(s) => SistenceStorageError::StorageError(s),
                StorageError::SerializationError(s) => SistenceStorageError::SerializationError(s),
                StorageError::StorageError(s) => SistenceStorageError::StorageError(s),
                _ => SistenceStorageError::StorageError(e.to_string()),
            })?;

        // Update version cache
        self.update_version_cache(namespace, key, workspace_id, version);

        // Emit event
        let event = StorageEvent {
            id: Uuid::new_v4().to_string(),
            event_type: StorageEventType::Create,
            key: key.to_string(),
            namespace: namespace.to_string(),
            timestamp: SystemTime::now(),
            data: None,
            workspace_id: workspace_id.map(|s| s.to_string()),
        };

        self.emit_event(event).await?;

        Ok(())
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn get<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Result<SistenceValueWithMetadata<T>, SistenceStorageError> {
        let storage_key = self.make_storage_key(namespace, key, workspace_id);

        // Try to load from storage
        let mut data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        // Look for the key
        let value_with_metadata = data
            .remove(&storage_key)
            .ok_or_else(|| SistenceStorageError::NotFound(key.to_string()))?;

        // Convert to SistenceValueWithMetadata
        let result = self.from_storage_value(key, value_with_metadata)?;

        // Update cache
        self.update_version_cache(namespace, key, workspace_id, result.version);

        Ok(result)
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn exists(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Result<bool, SistenceStorageError> {
        let storage_key = self.make_storage_key(namespace, key, workspace_id);

        // Try to load from storage
        let data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        // Check if key exists
        Ok(data.contains_key(&storage_key))
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn delete(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError> {
        let storage_key = self.make_storage_key(namespace, key, workspace_id);

        // Delete from storage
        self.storage
            .delete_key("data", &storage_key)
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        // Remove from cache
        if self.use_cache {
            self.version_cache.remove(&storage_key);
        }

        // Emit event
        let event = StorageEvent {
            id: Uuid::new_v4().to_string(),
            event_type: StorageEventType::Delete,
            key: key.to_string(),
            namespace: namespace.to_string(),
            timestamp: SystemTime::now(),
            data: None,
            workspace_id: workspace_id.map(|s| s.to_string()),
        };

        self.emit_event(event).await?;

        Ok(())
    }

    #[instrument(level = "debug", skip(self, new_value), err)]
    async fn update_if<T: Serialize + Send + Sync>(
        &self,
        namespace: &str,
        key: &str,
        expected_version: u64,
        new_value: T,
        new_metadata: Option<Metadata>,
        workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError> {
        let storage_key = self.make_storage_key(namespace, key, workspace_id);

        // Try to load from storage
        let mut data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        // Look for the key
        let value_with_metadata = data
            .get(&storage_key)
            .ok_or_else(|| SistenceStorageError::NotFound(key.to_string()))?
            .clone();

        // Extract current version
        let current_version = value_with_metadata
            .metadata
            .0
            .get("version")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Check version
        if current_version != expected_version {
            return Err(SistenceStorageError::VersionMismatch(
                expected_version,
                current_version,
            ));
        }

        // Generate new version
        let new_version = self.generate_version();

        // Convert to storage value
        let storage_value = self.to_storage_value(
            &new_value,
            new_metadata.as_ref(),
            None, // Keep existing TTL
            workspace_id,
            Some(new_version),
        )?;

        // Update in storage
        data.insert(storage_key.clone(), storage_value);

        self.storage
            .save("data", &data)
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        // Update version cache
        self.update_version_cache(namespace, key, workspace_id, new_version);

        // Emit event
        let event = StorageEvent {
            id: Uuid::new_v4().to_string(),
            event_type: StorageEventType::Update,
            key: key.to_string(),
            namespace: namespace.to_string(),
            timestamp: SystemTime::now(),
            data: None,
            workspace_id: workspace_id.map(|s| s.to_string()),
        };

        self.emit_event(event).await?;

        Ok(())
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn batch_get<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        namespace: &str,
        keys: &[String],
        workspace_id: Option<&str>,
    ) -> Result<
        HashMap<String, Result<SistenceValueWithMetadata<T>, SistenceStorageError>>,
        SistenceStorageError,
    > {
        // Check batch size limit
        if keys.len() > self.batch_size_limit {
            return Err(SistenceStorageError::StorageError(format!(
                "Batch size exceeds limit: {} > {}",
                keys.len(),
                self.batch_size_limit
            )));
        }

        // Try to load from storage
        let data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        // Process each key
        let mut results = HashMap::new();
        for key in keys {
            let storage_key = self.make_storage_key(namespace, key, workspace_id);

            let result = match data.get(&storage_key) {
                Some(value_with_metadata) => {
                    // Convert to SistenceValueWithMetadata
                    match self.from_storage_value(key, value_with_metadata.clone()) {
                        Ok(result) => {
                            // Update cache
                            self.update_version_cache(namespace, key, workspace_id, result.version);
                            Ok(result)
                        }
                        Err(e) => Err(e),
                    }
                }
                None => Err(SistenceStorageError::NotFound(key.to_string())),
            };

            results.insert(key.clone(), result);
        }

        Ok(results)
    }

    // Implementation for other methods...
    // To keep this example concise, we'll implement just these core methods
    // and leave the rest as placeholders for now

    #[instrument(level = "debug", skip(self), err)]
    async fn batch_save<T: Serialize + Send + Sync>(
        &self,
        _namespace: &str,
        _items: &HashMap<String, (T, Option<Metadata>, Option<Duration>)>,
        _workspace_id: Option<&str>,
    ) -> Result<BatchResult, SistenceStorageError> {
        // Placeholder implementation
        Ok(BatchResult {
            success_count: 0,
            failures: HashMap::new(),
        })
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn batch_delete(
        &self,
        _namespace: &str,
        _keys: &[String],
        _workspace_id: Option<&str>,
    ) -> Result<BatchResult, SistenceStorageError> {
        // Placeholder implementation
        Ok(BatchResult {
            success_count: 0,
            failures: HashMap::new(),
        })
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn list_keys(
        &self,
        _namespace: &str,
        _options: Option<QueryOptions>,
    ) -> Result<(Vec<String>, PaginationInfo), SistenceStorageError> {
        // Placeholder implementation
        Ok((
            Vec::new(),
            PaginationInfo {
                next_start_after: None,
                total_count: Some(0),
            },
        ))
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn query<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        _namespace: &str,
        _options: QueryOptions,
    ) -> Result<(Vec<(String, SistenceValueWithMetadata<T>)>, PaginationInfo), SistenceStorageError>
    {
        // Placeholder implementation
        Ok((
            Vec::new(),
            PaginationInfo {
                next_start_after: None,
                total_count: Some(0),
            },
        ))
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn create_workspace(
        &self,
        _namespace: &str,
        _workspace_id: &str,
        _parent_workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError> {
        // Placeholder implementation
        Ok(())
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn delete_workspace(
        &self,
        _namespace: &str,
        _workspace_id: &str,
    ) -> Result<(), SistenceStorageError> {
        // Placeholder implementation
        Ok(())
    }

    #[instrument(level = "debug", skip(self, conflict_resolution), err)]
    async fn merge_workspace(
        &self,
        _namespace: &str,
        _source_workspace_id: &str,
        _target_workspace_id: &str,
        _conflict_resolution: Option<
            Box<
                dyn FnMut(&str, &serde_json::Value, &serde_json::Value) -> serde_json::Value
                    + Send
                    + Sync,
            >,
        >,
    ) -> Result<BatchResult, SistenceStorageError> {
        // Placeholder implementation
        Ok(BatchResult {
            success_count: 0,
            failures: HashMap::new(),
        })
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn publish_event(&self, event: StorageEvent) -> Result<(), SistenceStorageError> {
        self.emit_event(event).await
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn get_events(
        &self,
        _namespace: &str,
        _key: Option<&str>,
        _start_time: Option<SystemTime>,
        _end_time: Option<SystemTime>,
        _limit: Option<usize>,
    ) -> Result<Vec<StorageEvent>, SistenceStorageError> {
        // Placeholder implementation
        Ok(Vec::new())
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn rebuild_from_events<T: for<'de> Deserialize<'de> + Serialize + Send + Sync>(
        &self,
        _namespace: &str,
        _key: &str,
        _to_timestamp: Option<SystemTime>,
    ) -> Result<Option<SistenceValueWithMetadata<T>>, SistenceStorageError> {
        // Placeholder implementation
        Ok(None)
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::capabilities::storage::StorageBackend;
    use crate::provider::plugins::storage::in_memory::InMemoryStorageBackend;

    #[tokio::test]
    async fn test_basic_operations() {
        // Create a storage backend
        let backend = Arc::new(InMemoryStorageBackend::new());

        // Create a SistenceStorage
        let storage = SistenceStorage::new(
            "test".to_string(),
            backend,
            None,
            SistenceStorageConfig::default(),
        );

        // Test save and get
        storage
            .save("test", "key1", "value1", None, None, None)
            .await
            .unwrap();

        let result: SistenceValueWithMetadata<String> =
            storage.get("test", "key1", None).await.unwrap();
        assert_eq!(result.value, "value1");

        // Test exists
        let exists = storage.exists("test", "key1", None).await.unwrap();
        assert!(exists);

        let exists = storage.exists("test", "nonexistent", None).await.unwrap();
        assert!(!exists);

        // Test delete
        storage.delete("test", "key1", None).await.unwrap();

        let exists = storage.exists("test", "key1", None).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_versioning() {
        // Create a storage backend
        let backend = Arc::new(InMemoryStorageBackend::new());

        // Create a SistenceStorage
        let storage = SistenceStorage::new(
            "test".to_string(),
            backend,
            None,
            SistenceStorageConfig::default(),
        );

        // Save initial value
        storage
            .save("test", "key1", "value1", None, None, None)
            .await
            .unwrap();

        // Get the value and extract version
        let result: SistenceValueWithMetadata<String> =
            storage.get("test", "key1", None).await.unwrap();
        let version = result.version;

        // Update with correct version
        storage
            .update_if("test", "key1", version, "value2", None, None)
            .await
            .unwrap();

        // Try to update with incorrect version
        let result = storage
            .update_if("test", "key1", version, "value3", None, None)
            .await;
        assert!(result.is_err());

        // Get current value
        let result: SistenceValueWithMetadata<String> =
            storage.get("test", "key1", None).await.unwrap();
        assert_eq!(result.value, "value2");
    }
}

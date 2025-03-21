//! Basic implementation of SistenceStorageService.
//!
//! This module provides a foundational implementation of the SistenceStorageService
//! trait that wraps an existing StorageBackend and adds advanced features
//! like versioning, TTL handling, event-based persistence, and workspace isolation.

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, instrument, warn};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::provider::capabilities::shared_memory::Metadata;

// Import from sistence_storage.rs
use crate::provider::capabilities::sistence_storage::{
    BatchResult, OrderBy, PaginationInfo, QueryOptions, SistenceStorageError,
    SistenceStorageService, SistenceValueWithMetadata, StorageEvent, StorageEventType,
};
use crate::provider::capabilities::storage::{StorageBackend, StorageError, ValueWithMetadata};
// TODO: Add event bus integration when it's available
// Uncomment when event_bus is available
// use crate::provider::event::event_bus::EventBus;

/// Basic implementation of SistenceStorageService
pub struct SistenceStorage {
    /// Unique identifier
    id: String,

    /// Underlying storage backend
    storage: Arc<dyn StorageBackend>,

    /// Event bus (for event-based persistence)
    // TODO: Uncomment when event_bus is available
    // event_bus: Option<Arc<EventBus>>,

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
        // event_bus: Option<Arc<EventBus>>,
        config: SistenceStorageConfig,
    ) -> Self {
        Self {
            id,
            storage,
            // event_bus: None,
            enable_versioning: config.enable_versioning,
            default_ttl: config.default_ttl,
            use_cache: config.use_cache,
            version_cache: Arc::new(DashMap::new()),
            batch_size_limit: config.batch_size_limit.unwrap_or(100),
            workspace_metadata: Arc::new(DashMap::new()),
        }
    }
    
    /// Generic save method for any serializable type
    pub async fn save<T: Serialize>(
        &self,
        namespace: &str,
        key: &str,
        value: &T,
        metadata: Option<Metadata>,
        ttl: Option<Duration>,
        workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError> {
        // Convert value to JSON
        let json_value = serde_json::to_value(value)
            .map_err(|e| SistenceStorageError::SerializationError(e.to_string()))?;
        
        // Use save_json implementation
        self.save_json(namespace, key, &json_value, metadata, ttl, workspace_id).await
    }

    /// Generic get method for any deserializable type
    pub async fn get<T: DeserializeOwned>(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Result<SistenceValueWithMetadata<T>, SistenceStorageError> {
        // Get as JSON first
        let json_result = self.get_json(namespace, key, workspace_id).await?;
        
        // Deserialize to requested type
        let value = serde_json::from_value(json_result.value.clone())
            .map_err(|e| SistenceStorageError::DeserializationError(format!(
                "Failed to deserialize key {}: {}", key, e
            )))?;
        
        // Return with same metadata but typed value
        Ok(SistenceValueWithMetadata {
            value,
            metadata: json_result.metadata,
            created_at: json_result.created_at,
            updated_at: json_result.updated_at,
            ttl: json_result.ttl,
            workspace_id: json_result.workspace_id,
            version: json_result.version,
            tags: json_result.tags,
        })
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
        let mut metadata_map = metadata.cloned().map(|m| m.tags).unwrap_or_default();

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

        // Create metadata using the From trait
        let metadata = Metadata::from(metadata_map);

        Ok(ValueWithMetadata {
            value: serialized,
            metadata,
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
            .tags
            .get("version")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Extract timestamps
        let created_at = metadata
            .tags
            .get("created_at")
            .and_then(|v| v.parse::<u64>().ok())
            .map(|secs| SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
            .unwrap_or_else(SystemTime::now);

        let updated_at = metadata
            .tags
            .get("updated_at")
            .and_then(|v| v.parse::<u64>().ok())
            .map(|secs| SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
            .unwrap_or_else(SystemTime::now);

        // Extract workspace ID
        let workspace_id = metadata.tags.get("workspace_id").cloned();

        // Extract TTL (if present in metadata)
        let ttl = metadata
            .tags
            .get("ttl")
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs);

        // Extract tags (keys starting with "tag_")
        let mut tags = HashMap::new();
        for (k, v) in &metadata.tags {
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
    async fn emit_event(&self, _event: StorageEvent) -> Result<(), SistenceStorageError> {
        // TODO: Implement when event bus is available
        /*
        if let Some(event_bus) = &self.event_bus {
            let event_json = serde_json::to_value(&event)
                .map_err(|e| SistenceStorageError::SerializationError(e.to_string()))?;
            
            event_bus
                .publish("storage_events", &event_json)
                .await
                .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;
        }
        */
        
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
            storage: self.storage.clone(),
            // event_bus: None,
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

    // === Key-based operations (new non-generic methods) ===

    #[instrument(level = "debug", skip(self, value), err)]
    async fn save_string(
        &self,
        namespace: &str,
        key: &str,
        value: &str,
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

    #[instrument(level = "debug", skip(self, value), err)]
    async fn save_json(
        &self,
        namespace: &str,
        key: &str,
        value: &Value,
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
    async fn get_string(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Result<SistenceValueWithMetadata<String>, SistenceStorageError> {
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
        let result = self.from_storage_value::<String>(key, value_with_metadata)?;

        // Update cache
        self.update_version_cache(namespace, key, workspace_id, result.version);

        Ok(result)
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn get_json(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Result<SistenceValueWithMetadata<Value>, SistenceStorageError> {
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
        let result = self.from_storage_value::<Value>(key, value_with_metadata)?;

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
    async fn update_string_if(
        &self,
        namespace: &str,
        key: &str,
        expected_version: u64,
        new_value: &str,
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
            .tags
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

    #[instrument(level = "debug", skip(self, new_value), err)]
    async fn update_json_if(
        &self,
        namespace: &str,
        key: &str,
        expected_version: u64,
        new_value: &Value,
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
            .tags
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
    async fn batch_get_strings(
        &self,
        namespace: &str,
        keys: &[String],
        workspace_id: Option<&str>,
    ) -> Result<HashMap<String, Result<SistenceValueWithMetadata<String>, SistenceStorageError>>, SistenceStorageError> {
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
                    match self.from_storage_value::<String>(key, value_with_metadata.clone()) {
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

    #[instrument(level = "debug", skip(self), err)]
    async fn batch_get_json(
        &self,
        namespace: &str,
        keys: &[String],
        workspace_id: Option<&str>,
    ) -> Result<HashMap<String, Result<SistenceValueWithMetadata<Value>, SistenceStorageError>>, SistenceStorageError> {
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
                    match self.from_storage_value::<Value>(key, value_with_metadata.clone()) {
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

    #[instrument(level = "debug", skip(self), err)]
    async fn batch_save_strings(
        &self,
        namespace: &str,
        items: &HashMap<String, (String, Option<Metadata>, Option<Duration>)>,
        workspace_id: Option<&str>,
    ) -> Result<BatchResult, SistenceStorageError> {
        // Check batch size limit
        if items.len() > self.batch_size_limit {
            return Err(SistenceStorageError::StorageError(format!(
                "Batch size exceeds limit: {} > {}",
                items.len(),
                self.batch_size_limit
            )));
        }

        // Try to load existing data
        let mut data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        let mut success_count = 0;
        let mut failures = HashMap::<String, SistenceStorageError>::new();

        // Process each item
        for (key, (value, metadata, ttl)) in items {
            let storage_key = self.make_storage_key(namespace, key, workspace_id);
            let version = self.generate_version();

            // Try to convert to storage value
            match self.to_storage_value(
                value,
                metadata.as_ref(),
                ttl.or(self.default_ttl),
                workspace_id,
                Some(version),
            ) {
                Ok(storage_value) => {
                    // Add to data
                    data.insert(storage_key, storage_value);
                    
                    // Update version cache
                    self.update_version_cache(namespace, key, workspace_id, version);
                    
                    success_count += 1;
                }
                Err(e) => {
                    failures.insert(key.clone(), e);
                }
            }
        }

        // Save all to storage
        if success_count > 0 {
            self.storage
                .save("data", &data)
                .await
                .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;
        }

        // Return batch result
        Ok(BatchResult {
            success_count,
            failures,
        })
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn batch_save_json(
        &self,
        namespace: &str,
        items: &HashMap<String, (Value, Option<Metadata>, Option<Duration>)>,
        workspace_id: Option<&str>,
    ) -> Result<BatchResult, SistenceStorageError> {
        // Check batch size limit
        if items.len() > self.batch_size_limit {
            return Err(SistenceStorageError::StorageError(format!(
                "Batch size exceeds limit: {} > {}",
                items.len(),
                self.batch_size_limit
            )));
        }

        // Try to load existing data
        let mut data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        let mut success_count = 0;
        let mut failures = HashMap::<String, SistenceStorageError>::new();

        // Process each item
        for (key, (value, metadata, ttl)) in items {
            let storage_key = self.make_storage_key(namespace, key, workspace_id);
            let version = self.generate_version();

            // Try to convert to storage value
            match self.to_storage_value(
                value,
                metadata.as_ref(),
                ttl.or(self.default_ttl),
                workspace_id,
                Some(version),
            ) {
                Ok(storage_value) => {
                    // Add to data
                    data.insert(storage_key, storage_value);
                    
                    // Update version cache
                    self.update_version_cache(namespace, key, workspace_id, version);
                    
                    success_count += 1;
                }
                Err(e) => {
                    failures.insert(key.clone(), e);
                }
            }
        }

        // Save all to storage
        if success_count > 0 {
            self.storage
                .save("data", &data)
                .await
                .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;
        }

        // Return batch result
        Ok(BatchResult {
            success_count,
            failures,
        })
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn batch_delete(
        &self,
        namespace: &str,
        keys: &[String],
        workspace_id: Option<&str>,
    ) -> Result<BatchResult, SistenceStorageError> {
        // Check batch size limit
        if keys.len() > self.batch_size_limit {
            return Err(SistenceStorageError::StorageError(format!(
                "Batch size exceeds limit: {} > {}",
                keys.len(),
                self.batch_size_limit
            )));
        }

        let mut success_count = 0;
        let mut failures = HashMap::new();

        // Process each key
        for key in keys {
            let storage_key = self.make_storage_key(namespace, key, workspace_id);
            
            // Try to delete
            match self.storage.delete_key("data", &storage_key).await {
                Ok(_) => {
                    // Remove from cache
                    if self.use_cache {
                        self.version_cache.remove(&storage_key);
                    }
                    success_count += 1;
                }
                Err(e) => {
                    failures.insert(
                        key.clone(),
                        SistenceStorageError::StorageError(e.to_string()),
                    );
                }
            }
        }

        // Return batch result
        Ok(BatchResult {
            success_count,
            failures,
        })
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn list_keys(
        &self,
        namespace: &str,
        options: Option<QueryOptions>,
    ) -> Result<(Vec<String>, PaginationInfo), SistenceStorageError> {
        // Load data from storage
        let data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        // Extract options
        let options = options.unwrap_or_default();
        let prefix = options.prefix.as_deref();
        let workspace_id = options.workspace_id.as_deref();
        let limit = options.limit.unwrap_or(100);
        let start_after = options.start_after.as_deref();

        // Process namespace and workspace
        let namespace_prefix = match workspace_id {
            Some(ws_id) => format!("{}/{}/", namespace, ws_id),
            None => format!("{}/main/", namespace),
        };

        // Filter keys that match namespace and options
        let mut matching_keys: Vec<String> = data
            .keys()
            .filter(|k| {
                k.starts_with(&namespace_prefix)
                    && (prefix.is_none() || k.contains(prefix.unwrap()))
                    && (start_after.is_none() || k.as_str() > start_after.unwrap())
            })
            .map(|k| {
                // Extract just the key part (remove namespace and workspace)
                k[namespace_prefix.len()..].to_string()
            })
            .collect();

        // Sort keys (basic sorting for now)
        matching_keys.sort();

        // Apply limit
        let has_more = matching_keys.len() > limit;
        if has_more {
            matching_keys.truncate(limit);
        }

        // Prepare pagination info
        let pagination_info = PaginationInfo {
            next_start_after: if has_more {
                matching_keys.last().cloned()
            } else {
                None
            },
            total_count: Some(data.keys().filter(|k| k.starts_with(&namespace_prefix)).count()),
        };

        Ok((matching_keys, pagination_info))
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn query_strings(
        &self,
        namespace: &str,
        options: QueryOptions,
    ) -> Result<(Vec<(String, SistenceValueWithMetadata<String>)>, PaginationInfo), SistenceStorageError> {
        // Load data from storage
        let data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        // Extract options
        let prefix = options.prefix.as_deref();
        let workspace_id = options.workspace_id.as_deref();
        let limit = options.limit.unwrap_or(100);
        let start_after = options.start_after.as_deref();
        let tags = options.tags.as_ref();

        // Process namespace and workspace
        let namespace_prefix = match workspace_id {
            Some(ws_id) => format!("{}/{}/", namespace, ws_id),
            None => format!("{}/main/", namespace),
        };

        // Filter and process items
        let mut matching_items = Vec::new();

        // First-pass filtering
        for (storage_key, value_with_metadata) in &data {
            if !storage_key.starts_with(&namespace_prefix) {
                continue;
            }

            // Extract key part
            let key = &storage_key[namespace_prefix.len()..];
            
            // Apply filters
            if (prefix.is_some() && !key.contains(prefix.unwrap()))
                || (start_after.is_some() && key <= start_after.unwrap())
            {
                continue;
            }
            
            // Tags filter (if specified)
            if let Some(tag_filters) = tags {
                // Check metadata for matching tags
                let has_all_tags = tag_filters.iter().all(|(tag_key, tag_value)| {
                    let metadata_tag_key = format!("tag_{}", tag_key);
                    value_with_metadata.metadata.tags.get(&metadata_tag_key)
                        .map_or(false, |v| v == tag_value)
                });
                
                if !has_all_tags {
                    continue;
                }
            }

            // Attempt to convert to String value
            match self.from_storage_value::<String>(key, value_with_metadata.clone()) {
                Ok(value) => {
                    matching_items.push((key.to_string(), value));
                }
                Err(_) => {
                    // Skip items that can't be deserialized as strings
                    continue;
                }
            }
        }

        // Sort based on ordering option
        if let Some(order_by) = &options.order_by {
            match order_by {
                OrderBy::CreatedAsc => {
                    matching_items.sort_by(|a, b| a.1.created_at.cmp(&b.1.created_at));
                }
                OrderBy::CreatedDesc => {
                    matching_items.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
                }
                OrderBy::UpdatedAsc => {
                    matching_items.sort_by(|a, b| a.1.updated_at.cmp(&b.1.updated_at));
                }
                OrderBy::UpdatedDesc => {
                    matching_items.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at));
                }
                OrderBy::KeyAsc => {
                    matching_items.sort_by(|a, b| a.0.cmp(&b.0));
                }
                OrderBy::KeyDesc => {
                    matching_items.sort_by(|a, b| b.0.cmp(&a.0));
                }
            }
        }

        // Apply limit
        let has_more = matching_items.len() > limit;
        if has_more {
            matching_items.truncate(limit);
        }

        // Prepare pagination info
        let pagination_info = PaginationInfo {
            next_start_after: if has_more {
                matching_items.last().map(|(k, _)| k.clone())
            } else {
                None
            },
            total_count: Some(matching_items.len()),
        };

        Ok((matching_items, pagination_info))
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn query_json(
        &self,
        namespace: &str,
        options: QueryOptions,
    ) -> Result<(Vec<(String, SistenceValueWithMetadata<Value>)>, PaginationInfo), SistenceStorageError> {
        // Load data from storage
        let data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;

        // Extract options
        let prefix = options.prefix.as_deref();
        let workspace_id = options.workspace_id.as_deref();
        let limit = options.limit.unwrap_or(100);
        let start_after = options.start_after.as_deref();
        let tags = options.tags.as_ref();

        // Process namespace and workspace
        let namespace_prefix = match workspace_id {
            Some(ws_id) => format!("{}/{}/", namespace, ws_id),
            None => format!("{}/main/", namespace),
        };

        // Filter and process items
        let mut matching_items = Vec::new();

        // First-pass filtering
        for (storage_key, value_with_metadata) in &data {
            if !storage_key.starts_with(&namespace_prefix) {
                continue;
            }

            // Extract key part
            let key = &storage_key[namespace_prefix.len()..];
            
            // Apply filters
            if (prefix.is_some() && !key.contains(prefix.unwrap()))
                || (start_after.is_some() && key <= start_after.unwrap())
            {
                continue;
            }
            
            // Tags filter (if specified)
            if let Some(tag_filters) = tags {
                // Check metadata for matching tags
                let has_all_tags = tag_filters.iter().all(|(tag_key, tag_value)| {
                    let metadata_tag_key = format!("tag_{}", tag_key);
                    value_with_metadata.metadata.tags.get(&metadata_tag_key)
                        .map_or(false, |v| v == tag_value)
                });
                
                if !has_all_tags {
                    continue;
                }
            }

            // Convert to Value
            match self.from_storage_value::<Value>(key, value_with_metadata.clone()) {
                Ok(value) => {
                    matching_items.push((key.to_string(), value));
                }
                Err(_) => {
                    // Skip items that can't be deserialized
                    continue;
                }
            }
        }

        // Sort based on ordering option
        if let Some(order_by) = &options.order_by {
            match order_by {
                OrderBy::CreatedAsc => {
                    matching_items.sort_by(|a, b| a.1.created_at.cmp(&b.1.created_at));
                }
                OrderBy::CreatedDesc => {
                    matching_items.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
                }
                OrderBy::UpdatedAsc => {
                    matching_items.sort_by(|a, b| a.1.updated_at.cmp(&b.1.updated_at));
                }
                OrderBy::UpdatedDesc => {
                    matching_items.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at));
                }
                OrderBy::KeyAsc => {
                    matching_items.sort_by(|a, b| a.0.cmp(&b.0));
                }
                OrderBy::KeyDesc => {
                    matching_items.sort_by(|a, b| b.0.cmp(&a.0));
                }
            }
        }

        // Apply limit
        let has_more = matching_items.len() > limit;
        if has_more {
            matching_items.truncate(limit);
        }

        // Prepare pagination info
        let pagination_info = PaginationInfo {
            next_start_after: if has_more {
                matching_items.last().map(|(k, _)| k.clone())
            } else {
                None
            },
            total_count: Some(matching_items.len()),
        };

        Ok((matching_items, pagination_info))
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn create_workspace(
        &self,
        namespace: &str,
        workspace_id: &str,
        parent_workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError> {
        // Insert workspace metadata
        let mut metadata = HashMap::new();
        metadata.insert("created_at".to_string(), 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string());
                
        if let Some(parent) = parent_workspace_id {
            metadata.insert("parent_workspace".to_string(), parent.to_string());
        }
        
        self.workspace_metadata.insert(
            format!("{}/{}", namespace, workspace_id),
            metadata,
        );
        
        // Emit workspace creation event
        let event = StorageEvent {
            id: Uuid::new_v4().to_string(),
            event_type: StorageEventType::WorkspaceChange,
            key: workspace_id.to_string(),
            namespace: namespace.to_string(),
            timestamp: SystemTime::now(),
            data: Some(serde_json::json!({
                "action": "create",
                "parent": parent_workspace_id
            })),
            workspace_id: Some(workspace_id.to_string()),
        };
        
        self.emit_event(event).await?;
        
        Ok(())
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn delete_workspace(
        &self,
        namespace: &str,
        workspace_id: &str,
    ) -> Result<(), SistenceStorageError> {
        // Remove workspace metadata
        self.workspace_metadata.remove(&format!("{}/{}", namespace, workspace_id));
        
        // Load all data from storage
        let data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;
            
        // Filter keys belonging to this workspace
        let workspace_prefix = format!("{}/{}/", namespace, workspace_id);
        let workspace_keys: Vec<String> = data
            .keys()
            .filter(|k| k.starts_with(&workspace_prefix))
            .cloned()
            .collect();
            
        // Delete each key
        for key in workspace_keys {
            let _ = self.storage.delete_key("data", &key).await;
        }
        
        // Emit workspace deletion event
        let event = StorageEvent {
            id: Uuid::new_v4().to_string(),
            event_type: StorageEventType::WorkspaceChange,
            key: workspace_id.to_string(),
            namespace: namespace.to_string(),
            timestamp: SystemTime::now(),
            data: Some(serde_json::json!({
                "action": "delete"
            })),
            workspace_id: Some(workspace_id.to_string()),
        };
        
        self.emit_event(event).await?;
        
        Ok(())
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn merge_workspace(
        &self,
        namespace: &str,
        source_workspace_id: &str,
        target_workspace_id: &str,
        resolve_conflicts: bool,
    ) -> Result<BatchResult, SistenceStorageError> {
        // Load all data from storage
        let mut data = self
            .storage
            .load("data")
            .await
            .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;
            
        // Find source workspace keys and values
        let source_prefix = format!("{}/{}/", namespace, source_workspace_id);
        let target_prefix = format!("{}/{}/", namespace, target_workspace_id);
        
        let mut success_count = 0;
        let mut failures = HashMap::new();
        
        // First, collect all source items in a separate collection
        let mut source_items = Vec::new();
        for (source_key, source_value) in data.iter() {
            if source_key.starts_with(&source_prefix) {
                // Extract the key part (without namespace and workspace)
                let key_part = &source_key[source_prefix.len()..];
                let target_key = format!("{}{}", target_prefix, key_part);
                source_items.push((source_key.clone(), target_key, key_part.to_string(), source_value.clone()));
            }
        }
        
        // Now process the collected items
        for (_, target_key, key_part, source_value) in source_items {
            // Check if target already has this key
            let conflict = data.contains_key(&target_key);
            
            if conflict && !resolve_conflicts {
                // Record conflict error
                failures.insert(
                    key_part.to_string(),
                    SistenceStorageError::ConflictError(format!(
                        "Key conflict: {} exists in both workspaces",
                        key_part
                    )),
                );
                continue;
            }
            
            // Copy to target
            data.insert(target_key, source_value);
            success_count += 1;
        }
        
        // Save changes
        if success_count > 0 {
            self.storage
                .save("data", &data)
                .await
                .map_err(|e| SistenceStorageError::StorageError(e.to_string()))?;
        }
        
        // Emit merge event
        let event = StorageEvent {
            id: Uuid::new_v4().to_string(),
            event_type: StorageEventType::WorkspaceChange,
            key: format!("{}->{}", source_workspace_id, target_workspace_id),
            namespace: namespace.to_string(),
            timestamp: SystemTime::now(),
            data: Some(serde_json::json!({
                "action": "merge",
                "source": source_workspace_id,
                "target": target_workspace_id,
                "success_count": success_count,
                "failure_count": failures.len()
            })),
            workspace_id: Some(target_workspace_id.to_string()),
        };
        
        self.emit_event(event).await?;
        
        Ok(BatchResult {
            success_count,
            failures,
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
        // TODO: In a real implementation, events would be stored and retrieved from storage
        // For now, return an empty list
        Ok(Vec::new())
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

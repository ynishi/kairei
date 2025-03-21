//! Storage service capability for Sistence Memory.
//!
//! The SistenceStorageService trait defines an enhanced interface for storage operations
//! tailored specifically for the StatelessRelevantMemory system. It extends beyond the
//! basic StorageBackend trait to provide features like metadata management, workspace
//! isolation, event-based operations, and query capabilities.
//!
//! # Key Features
//!
//! - Generic type support for type-safe storage and retrieval
//! - Workspace isolation for parallel processing
//! - Event-based persistence for tracking changes
//! - Enhanced query capabilities with metadata filtering
//! - Batch operations for efficiency
//! - Versioning and conflict resolution
//!
//! # Usage Example
//!
//! ```no_run
//! use kairei_core::provider::capabilities::sistence_storage::{SistenceStorageService, SistenceValueWithMetadata};
//! use std::time::Duration;
//!
//! # async fn example<T: SistenceStorageService>(storage: &T) -> Result<(), Box<dyn std::error::Error>> {
//! // Save data with metadata
//! let data = "Important memory item";
//! storage.save("memory_items", "item1", data, None, Some(Duration::from_secs(86400)), None).await?;
//!
//! // Retrieve data
//! let retrieved: SistenceValueWithMetadata<String> = storage.get("memory_items", "item1", None).await?;
//! println!("Retrieved value: {}", retrieved.value);
//!
//! // Create a workspace for parallel processing
//! storage.create_workspace("memory_items", "workspace1", None).await?;
//!
//! // Work with isolated data
//! storage.save("memory_items", "item2", "Workspace-specific data", None, None, Some("workspace1")).await?;
//!
//! // Merge workspace back to main
//! storage.merge_workspace("memory_items", "workspace1", "main", None).await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

use crate::provider::capabilities::shared_memory::Metadata;

/// Memory item value container with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SistenceValueWithMetadata<T> {
    /// The stored value
    pub value: T,

    /// Metadata about the value
    pub metadata: Metadata,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Last update timestamp
    pub updated_at: SystemTime,

    /// Optional time-to-live
    pub ttl: Option<Duration>,

    /// Workspace ID (for parallel processing)
    pub workspace_id: Option<String>,

    /// Version information
    pub version: u64,

    /// Tags (for indexing and filtering)
    pub tags: HashMap<String, String>,
}

/// Error types for SistenceStorageService
#[derive(Debug, Error, Clone)]
pub enum SistenceStorageError {
    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Conflict error: {0}")]
    ConflictError(String),

    #[error("Version mismatch: expected={0}, actual={1}")]
    VersionMismatch(u64, u64),

    #[error("Permission error: {0}")]
    PermissionError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Workspace error: {0}")]
    WorkspaceError(String),
}

/// Query options for SistenceStorageService
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Filter by tags
    pub tags: Option<HashMap<String, String>>,

    /// Filter by key prefix
    pub prefix: Option<String>,

    /// Filter by workspace ID
    pub workspace_id: Option<String>,

    /// Maximum results to return
    pub limit: Option<usize>,

    /// Start after this key (for pagination)
    pub start_after: Option<String>,

    /// Order by field
    pub order_by: Option<OrderBy>,
}

/// Order options for queries
#[derive(Debug, Clone)]
pub enum OrderBy {
    /// Created timestamp (ascending)
    CreatedAsc,
    /// Created timestamp (descending)
    CreatedDesc,
    /// Updated timestamp (ascending)
    UpdatedAsc,
    /// Updated timestamp (descending)
    UpdatedDesc,
    /// Key (ascending)
    KeyAsc,
    /// Key (descending)
    KeyDesc,
}

/// Pagination information for query results
#[derive(Debug, Clone)]
pub struct PaginationInfo {
    /// Next start key for pagination (None if no more results)
    pub next_start_after: Option<String>,
    /// Total count (if provided by backend)
    pub total_count: Option<usize>,
}

/// Result of batch operations
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Number of successful operations
    pub success_count: usize,
    /// Map of failed operations (key and error)
    pub failures: HashMap<String, SistenceStorageError>,
}

/// Event types for event-based persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageEventType {
    /// Item created
    Create,
    /// Item updated
    Update,
    /// Item deleted
    Delete,
    /// Workspace change (merge, etc.)
    WorkspaceChange,
}

/// Storage event for event-based persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: StorageEventType,
    /// Target key
    pub key: String,
    /// Namespace
    pub namespace: String,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event data (optional)
    pub data: Option<serde_json::Value>,
    /// Workspace ID (optional)
    pub workspace_id: Option<String>,
}

/// SistenceStorageService trait
///
/// Advanced storage service for StatelessRelevantMemory.
/// Supports key-based access, metadata, event-based persistence, and concurrent workspaces.
#[async_trait]
pub trait SistenceStorageService: Send + Sync {
    /// Clone the service
    fn clone_service(&self) -> Box<dyn SistenceStorageService>;

    /// Check if service is available
    async fn is_available(&self) -> bool;

    /// List available namespaces
    async fn list_namespaces(&self) -> Result<Vec<String>, SistenceStorageError>;

    /// === Key-based CRUD operations ===

    /// Save value to key
    async fn save<T: Serialize + Send + Sync>(
        &self,
        namespace: &str,
        key: &str,
        value: T,
        metadata: Option<Metadata>,
        ttl: Option<Duration>,
        workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError>;

    /// Get value from key
    async fn get<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Result<SistenceValueWithMetadata<T>, SistenceStorageError>;

    /// Check if key exists
    async fn exists(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Result<bool, SistenceStorageError>;

    /// Delete key
    async fn delete(
        &self,
        namespace: &str,
        key: &str,
        workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError>;

    /// Update if version matches (optimistic locking)
    async fn update_if<T: Serialize + Send + Sync>(
        &self,
        namespace: &str,
        key: &str,
        expected_version: u64,
        new_value: T,
        new_metadata: Option<Metadata>,
        workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError>;

    /// === Batch operations ===

    /// Get multiple keys in one operation
    async fn batch_get<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        namespace: &str,
        keys: &[String],
        workspace_id: Option<&str>,
    ) -> Result<
        HashMap<String, Result<SistenceValueWithMetadata<T>, SistenceStorageError>>,
        SistenceStorageError,
    >;

    /// Save multiple keys in one operation
    async fn batch_save<T: Serialize + Send + Sync>(
        &self,
        namespace: &str,
        items: &HashMap<String, (T, Option<Metadata>, Option<Duration>)>,
        workspace_id: Option<&str>,
    ) -> Result<BatchResult, SistenceStorageError>;

    /// Delete multiple keys in one operation
    async fn batch_delete(
        &self,
        namespace: &str,
        keys: &[String],
        workspace_id: Option<&str>,
    ) -> Result<BatchResult, SistenceStorageError>;

    /// === Query operations ===

    /// List keys
    async fn list_keys(
        &self,
        namespace: &str,
        options: Option<QueryOptions>,
    ) -> Result<(Vec<String>, PaginationInfo), SistenceStorageError>;

    /// Query items
    async fn query<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        namespace: &str,
        options: QueryOptions,
    ) -> Result<(Vec<(String, SistenceValueWithMetadata<T>)>, PaginationInfo), SistenceStorageError>;

    /// === Workspace management ===

    /// Create a new workspace
    async fn create_workspace(
        &self,
        namespace: &str,
        workspace_id: &str,
        parent_workspace_id: Option<&str>,
    ) -> Result<(), SistenceStorageError>;

    /// Delete a workspace
    async fn delete_workspace(
        &self,
        namespace: &str,
        workspace_id: &str,
    ) -> Result<(), SistenceStorageError>;

    /// Merge workspace
    async fn merge_workspace(
        &self,
        namespace: &str,
        source_workspace_id: &str,
        target_workspace_id: &str,
        conflict_resolution: Option<
            Box<
                dyn FnMut(&str, &serde_json::Value, &serde_json::Value) -> serde_json::Value
                    + Send
                    + Sync,
            >,
        >,
    ) -> Result<BatchResult, SistenceStorageError>;

    /// === Event-based operations ===

    /// Publish an event
    async fn publish_event(&self, event: StorageEvent) -> Result<(), SistenceStorageError>;

    /// Get events
    async fn get_events(
        &self,
        namespace: &str,
        key: Option<&str>,
        start_time: Option<SystemTime>,
        end_time: Option<SystemTime>,
        limit: Option<usize>,
    ) -> Result<Vec<StorageEvent>, SistenceStorageError>;

    /// Rebuild state from events
    async fn rebuild_from_events<T: for<'de> Deserialize<'de> + Serialize + Send + Sync>(
        &self,
        namespace: &str,
        key: &str,
        to_timestamp: Option<SystemTime>,
    ) -> Result<Option<SistenceValueWithMetadata<T>>, SistenceStorageError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define MockSistenceStorage for testing
    #[derive(Clone)]
    struct MockSistenceStorage {}

    #[async_trait]
    impl SistenceStorageService for MockSistenceStorage {
        fn clone_service(&self) -> Box<dyn SistenceStorageService> {
            Box::new(self.clone())
        }

        async fn is_available(&self) -> bool {
            true
        }

        async fn list_namespaces(&self) -> Result<Vec<String>, SistenceStorageError> {
            Ok(vec!["test".to_string()])
        }

        async fn save<T: Serialize + Send + Sync>(
            &self,
            _namespace: &str,
            _key: &str,
            _value: T,
            _metadata: Option<Metadata>,
            _ttl: Option<Duration>,
            _workspace_id: Option<&str>,
        ) -> Result<(), SistenceStorageError> {
            Ok(())
        }

        async fn get<T: for<'de> Deserialize<'de> + Send + Sync>(
            &self,
            _namespace: &str,
            _key: &str,
            _workspace_id: Option<&str>,
        ) -> Result<SistenceValueWithMetadata<T>, SistenceStorageError> {
            Err(SistenceStorageError::NotFound(
                "Mock not implemented".to_string(),
            ))
        }

        async fn exists(
            &self,
            _namespace: &str,
            _key: &str,
            _workspace_id: Option<&str>,
        ) -> Result<bool, SistenceStorageError> {
            Ok(false)
        }

        async fn delete(
            &self,
            _namespace: &str,
            _key: &str,
            _workspace_id: Option<&str>,
        ) -> Result<(), SistenceStorageError> {
            Ok(())
        }

        async fn update_if<T: Serialize + Send + Sync>(
            &self,
            _namespace: &str,
            _key: &str,
            _expected_version: u64,
            _new_value: T,
            _new_metadata: Option<Metadata>,
            _workspace_id: Option<&str>,
        ) -> Result<(), SistenceStorageError> {
            Ok(())
        }

        async fn batch_get<T: for<'de> Deserialize<'de> + Send + Sync>(
            &self,
            _namespace: &str,
            _keys: &[String],
            _workspace_id: Option<&str>,
        ) -> Result<
            HashMap<String, Result<SistenceValueWithMetadata<T>, SistenceStorageError>>,
            SistenceStorageError,
        > {
            Ok(HashMap::new())
        }

        async fn batch_save<T: Serialize + Send + Sync>(
            &self,
            _namespace: &str,
            _items: &HashMap<String, (T, Option<Metadata>, Option<Duration>)>,
            _workspace_id: Option<&str>,
        ) -> Result<BatchResult, SistenceStorageError> {
            Ok(BatchResult {
                success_count: 0,
                failures: HashMap::new(),
            })
        }

        async fn batch_delete(
            &self,
            _namespace: &str,
            _keys: &[String],
            _workspace_id: Option<&str>,
        ) -> Result<BatchResult, SistenceStorageError> {
            Ok(BatchResult {
                success_count: 0,
                failures: HashMap::new(),
            })
        }

        async fn list_keys(
            &self,
            _namespace: &str,
            _options: Option<QueryOptions>,
        ) -> Result<(Vec<String>, PaginationInfo), SistenceStorageError> {
            Ok((
                Vec::new(),
                PaginationInfo {
                    next_start_after: None,
                    total_count: Some(0),
                },
            ))
        }

        async fn query<T: for<'de> Deserialize<'de> + Send + Sync>(
            &self,
            _namespace: &str,
            _options: QueryOptions,
        ) -> Result<
            (Vec<(String, SistenceValueWithMetadata<T>)>, PaginationInfo),
            SistenceStorageError,
        > {
            Ok((
                Vec::new(),
                PaginationInfo {
                    next_start_after: None,
                    total_count: Some(0),
                },
            ))
        }

        async fn create_workspace(
            &self,
            _namespace: &str,
            _workspace_id: &str,
            _parent_workspace_id: Option<&str>,
        ) -> Result<(), SistenceStorageError> {
            Ok(())
        }

        async fn delete_workspace(
            &self,
            _namespace: &str,
            _workspace_id: &str,
        ) -> Result<(), SistenceStorageError> {
            Ok(())
        }

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
            Ok(BatchResult {
                success_count: 0,
                failures: HashMap::new(),
            })
        }

        async fn publish_event(&self, _event: StorageEvent) -> Result<(), SistenceStorageError> {
            Ok(())
        }

        async fn get_events(
            &self,
            _namespace: &str,
            _key: Option<&str>,
            _start_time: Option<SystemTime>,
            _end_time: Option<SystemTime>,
            _limit: Option<usize>,
        ) -> Result<Vec<StorageEvent>, SistenceStorageError> {
            Ok(Vec::new())
        }

        async fn rebuild_from_events<T: for<'de> Deserialize<'de> + Serialize + Send + Sync>(
            &self,
            _namespace: &str,
            _key: &str,
            _to_timestamp: Option<SystemTime>,
        ) -> Result<Option<SistenceValueWithMetadata<T>>, SistenceStorageError> {
            Ok(None)
        }
    }

    #[test]
    fn test_error_display() {
        let error = SistenceStorageError::NotFound("test_key".to_string());
        assert!(error.to_string().contains("not found"));

        let error = SistenceStorageError::VersionMismatch(1, 2);
        assert!(error.to_string().contains("expected=1, actual=2"));
    }
}

//! Storage backend trait definition for persistent data storage.
//!
//! This module defines the `StorageBackend` trait that abstracts storage backend
//! operations. This trait is implemented by different storage providers (GCP Storage,
//! Local File System, etc.) and is used by the PersistentSharedMemoryPlugin to
//! interact with the underlying storage.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

use crate::provider::capabilities::shared_memory::Metadata;

/// Errors that can occur during storage backend operations.
///
/// This enum defines the various error conditions that can occur
/// when working with storage backends.
#[derive(Debug, Error, Clone)]
pub enum StorageError {
    /// The requested namespace was not found.
    #[error("Namespace not found: {0}")]
    NamespaceNotFound(String),

    /// The requested key was not found.
    #[error("Key not found: {0} in namespace {1}")]
    KeyNotFound(String, String),

    /// The storage backend is not available.
    #[error("Storage backend not available: {0}")]
    BackendUnavailable(String),

    /// Authentication error with the storage backend.
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Permission error with the storage backend.
    #[error("Permission error: {0}")]
    PermissionError(String),

    /// Error during serialization or deserialization.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Network error during communication with the storage backend.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Generic storage error.
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// A storable value with associated metadata for persistence.
///
/// This structure is similar to the internal ValueWithMetadata in the
/// shared_memory plugin, but is designed for storage backends to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorableValue {
    /// The stored JSON value
    pub value: Value,

    /// Metadata about the value
    pub metadata: Metadata,

    /// Optional expiration time as seconds since epoch
    /// This is converted from Instant for serialization purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_timestamp: Option<i64>,
}

/// Trait that abstracts storage backend operations.
///
/// This trait is implemented by different storage providers (GCP Storage,
/// Local File System, etc.) and is used by the PersistentSharedMemoryPlugin
/// to interact with the underlying storage.
///
/// # Thread Safety
///
/// All implementations of this trait must be thread-safe and can be
/// shared between multiple tasks or threads. This is enforced by the
/// `Send + Sync` trait bounds.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Load all data for a namespace
    ///
    /// # Arguments
    /// * `namespace` - The namespace to load data from
    ///
    /// # Returns
    /// * `Ok(HashMap<String, StorableValue>)` - The loaded data
    /// * `Err(StorageError)` - If loading fails
    ///
    /// # Expected Behavior
    ///
    /// - Should load all key-value pairs for the specified namespace
    /// - Should handle non-existent namespaces gracefully (return empty HashMap)
    /// - Should deserialize stored data into the correct format
    /// - Should preserve metadata (creation time, modification time, etc.)
    async fn load(&self, namespace: &str) -> Result<HashMap<String, StorableValue>, StorageError>;

    /// Save all data for a namespace
    ///
    /// # Arguments
    /// * `namespace` - The namespace to save data to
    /// * `data` - The data to save
    ///
    /// # Returns
    /// * `Ok(())` - If saving succeeds
    /// * `Err(StorageError)` - If saving fails
    ///
    /// # Expected Behavior
    ///
    /// - Should save all key-value pairs for the specified namespace
    /// - Should overwrite existing data if the namespace already exists
    /// - Should serialize data in a format that can be deserialized by `load`
    /// - Should preserve metadata
    async fn save(
        &self,
        namespace: &str,
        data: &HashMap<String, StorableValue>,
    ) -> Result<(), StorageError>;

    /// Save a single key
    ///
    /// # Arguments
    /// * `namespace` - The namespace to save the key to
    /// * `key` - The key to save
    /// * `value` - The value to save
    ///
    /// # Returns
    /// * `Ok(())` - If saving succeeds
    /// * `Err(StorageError)` - If saving fails
    ///
    /// # Expected Behavior
    ///
    /// - Should save a single key-value pair
    /// - Should overwrite the key if it already exists
    /// - Should create the namespace if it doesn't exist
    /// - Should be more efficient than calling `save` for a single key
    async fn save_key(
        &self,
        namespace: &str,
        key: &str,
        value: &StorableValue,
    ) -> Result<(), StorageError>;

    /// Delete a single key
    ///
    /// # Arguments
    /// * `namespace` - The namespace to delete the key from
    /// * `key` - The key to delete
    ///
    /// # Returns
    /// * `Ok(())` - If deletion succeeds
    /// * `Err(StorageError)` - If deletion fails
    ///
    /// # Expected Behavior
    ///
    /// - Should delete a single key-value pair
    /// - Should handle non-existent keys gracefully (no error)
    /// - Should not affect other keys in the namespace
    async fn delete_key(&self, namespace: &str, key: &str) -> Result<(), StorageError>;

    /// Check if the backend is available
    ///
    /// # Returns
    /// * `true` - If the backend is available
    /// * `false` - If the backend is not available
    ///
    /// # Expected Behavior
    ///
    /// - Should return `true` if the backend is available and operational
    /// - Should return `false` if the backend is not available (e.g., network issues, authentication failures)
    /// - Should not throw exceptions or errors
    async fn is_available(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    

    // Mock implementation of StorageBackend for testing
    struct MockStorageBackend {
        available: bool,
        data: HashMap<String, HashMap<String, StorableValue>>,
    }

    #[async_trait]
    impl StorageBackend for MockStorageBackend {
        async fn load(
            &self,
            namespace: &str,
        ) -> Result<HashMap<String, StorableValue>, StorageError> {
            if !self.available {
                return Err(StorageError::BackendUnavailable(
                    "Mock backend unavailable".to_string(),
                ));
            }

            Ok(self.data.get(namespace).cloned().unwrap_or_default())
        }

        async fn save(
            &self,
            _namespace: &str,
            _data: &HashMap<String, StorableValue>,
        ) -> Result<(), StorageError> {
            if !self.available {
                return Err(StorageError::BackendUnavailable(
                    "Mock backend unavailable".to_string(),
                ));
            }

            Ok(())
        }

        async fn save_key(
            &self,
            _namespace: &str,
            _key: &str,
            _value: &StorableValue,
        ) -> Result<(), StorageError> {
            if !self.available {
                return Err(StorageError::BackendUnavailable(
                    "Mock backend unavailable".to_string(),
                ));
            }

            Ok(())
        }

        async fn delete_key(&self, _namespace: &str, _key: &str) -> Result<(), StorageError> {
            if !self.available {
                return Err(StorageError::BackendUnavailable(
                    "Mock backend unavailable".to_string(),
                ));
            }

            Ok(())
        }

        async fn is_available(&self) -> bool {
            self.available
        }
    }

    #[test]
    fn test_storable_value_serialization() {
        let metadata = Metadata::default();
        let value = serde_json::json!({"name": "test", "value": 42});

        let storable = StorableValue {
            value: value.clone(),
            metadata: metadata.clone(),
            expiry_timestamp: Some(1234567890),
        };

        let serialized = serde_json::to_string(&storable).unwrap();
        let deserialized: StorableValue = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.value, value);
        assert_eq!(deserialized.expiry_timestamp, Some(1234567890));
    }
}

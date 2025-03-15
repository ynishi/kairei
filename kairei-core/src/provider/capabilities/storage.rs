//! Storage capability for Provider Plugins.
//!
//! The StorageBackend trait defines a common interface for different storage backends
//! that can be used by the PersistentSharedMemoryPlugin to persist data. This enables
//! the shared memory system to store data in various backends like local file systems,
//! cloud storage services, or other persistent storage solutions.
//!
//! # Key Features
//!
//! - Common interface for different storage backends
//! - Namespace-based data organization
//! - Asynchronous operations for non-blocking I/O
//! - Error handling with specific error types
//! - Backend availability checking
//!
//! # Usage Example
//!
//! ```no_run
//! use kairei_core::provider::capabilities::storage::{StorageBackend, ValueWithMetadata};
//! use std::collections::HashMap;
//!
//! # async fn example<T: StorageBackend>(backend: &T) -> Result<(), Box<dyn std::error::Error>> {
//! // Load data from storage
//! let data = backend.load("my_namespace").await?;
//!
//! // Save data to storage
//! let mut data_to_save = HashMap::new();
//! // ... populate data_to_save ...
//! backend.save("my_namespace", &data_to_save).await?;
//!
//! // Check if backend is available
//! if backend.is_available().await {
//!     println!("Storage backend is available");
//! }
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::provider::capabilities::shared_memory::Metadata;
use serde_json::Value;
use std::time::Instant;

/// A thread-safe value container with expiration support
///
/// This structure stores a value along with its metadata and
/// optional expiration time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueWithMetadata {
    /// The stored JSON value
    pub value: Value,

    /// Metadata about the value
    pub metadata: Metadata,

    /// Optional expiration time (None means no expiration)
    #[serde(skip)]
    pub expiry: Option<Instant>,
}

/// Storage backend capability for Provider Plugins
///
/// This trait defines the interface for storage operations
/// that allow persistent storage of shared memory data.
///
/// # Thread Safety
///
/// All methods in this trait are designed to be thread-safe and can be
/// called concurrently from multiple tasks or threads. Implementations
/// must ensure proper synchronization.
///
/// # Error Handling
///
/// Operations return `Result<T, StorageError>` to indicate success or failure.
/// Specific error variants provide detailed information about what went wrong.
///
/// # Namespace Isolation
///
/// Data is isolated by namespace, which allows multiple applications or
/// components to use the same storage backend without collisions.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Load all data for a namespace
    ///
    /// # Arguments
    /// * `namespace` - The namespace to load data from
    ///
    /// # Returns
    /// * `Ok(HashMap<String, ValueWithMetadata>)` - The loaded data
    /// * `Err(StorageError)` - If loading fails
    ///
    /// # Notes
    ///
    /// - If the namespace doesn't exist, an empty HashMap should be returned
    /// - Implementations should handle deserialization of stored data
    async fn load(&self, namespace: &str) -> Result<HashMap<String, ValueWithMetadata>, StorageError>;
    
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
    /// # Notes
    ///
    /// - If the namespace already exists, it should be overwritten
    /// - Implementations should handle serialization of data
    async fn save(&self, namespace: &str, data: &HashMap<String, ValueWithMetadata>) -> Result<(), StorageError>;
    
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
    /// # Notes
    ///
    /// - If the key already exists, it should be overwritten
    /// - If the namespace doesn't exist, it should be created
    /// - This method should be more efficient than calling `save` for a single key
    async fn save_key(&self, namespace: &str, key: &str, value: &ValueWithMetadata) -> Result<(), StorageError>;
    
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
    /// # Notes
    ///
    /// - If the key doesn't exist, this should not be considered an error
    /// - This operation should not affect other keys in the namespace
    async fn delete_key(&self, namespace: &str, key: &str) -> Result<(), StorageError>;
    
    /// Check if the backend is available
    ///
    /// # Returns
    /// * `true` - If the backend is available
    /// * `false` - If the backend is not available
    ///
    /// # Notes
    ///
    /// - This method should not throw exceptions or errors
    /// - It should return `false` if the backend is not available for any reason
    /// - Examples of unavailability: network issues, authentication failures, etc.
    async fn is_available(&self) -> bool;
}

/// Errors that can occur during storage operations
///
/// This enum defines the various error conditions that can occur
/// when working with the StorageBackend.
#[derive(Debug, Error, Clone)]
pub enum StorageError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Sync error: {0}")]
    SyncError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockStorageBackend {}

    #[async_trait]
    impl StorageBackend for MockStorageBackend {
        async fn load(&self, _namespace: &str) -> Result<HashMap<String, ValueWithMetadata>, StorageError> {
            todo!()
        }
        
        async fn save(&self, _namespace: &str, _data: &HashMap<String, ValueWithMetadata>) -> Result<(), StorageError> {
            todo!()
        }
        
        async fn save_key(&self, _namespace: &str, _key: &str, _value: &ValueWithMetadata) -> Result<(), StorageError> {
            todo!()
        }
        
        async fn delete_key(&self, _namespace: &str, _key: &str) -> Result<(), StorageError> {
            todo!()
        }
        
        async fn is_available(&self) -> bool {
            todo!()
        }
    }

    #[test]
    fn test_storage_error() {
        let error = StorageError::FileNotFound("test.txt".to_string());
        assert!(error.to_string().contains("File not found"));
    }
}

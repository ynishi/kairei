use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

use crate::provider::plugin::ProviderPlugin;

/// Shared Memory capability for Provider Plugins
///
/// This trait defines the interface for shared memory operations
/// that allow data sharing between different agents and providers.
#[async_trait]
pub trait SharedMemoryCapability: ProviderPlugin {
    /// Retrieve a value by key
    ///
    /// # Arguments
    /// * `key` - The unique identifier for the value
    ///
    /// # Returns
    /// * `Ok(Value)` - The stored value if found
    /// * `Err(MemoryError::KeyNotFound)` - If the key doesn't exist
    async fn get(&self, key: &str) -> Result<Value, MemoryError>;

    /// Store a value with the specified key
    ///
    /// # Arguments
    /// * `key` - The unique identifier for the value
    /// * `value` - The value to store
    ///
    /// # Returns
    /// * `Ok(())` - If storage was successful
    /// * `Err(MemoryError)` - If storage failed
    async fn set(&self, key: &str, value: Value) -> Result<(), MemoryError>;

    /// Delete a value by key
    ///
    /// # Arguments
    /// * `key` - The unique identifier for the value to delete
    ///
    /// # Returns
    /// * `Ok(())` - If deletion was successful
    /// * `Err(MemoryError::KeyNotFound)` - If the key doesn't exist
    async fn delete(&self, key: &str) -> Result<(), MemoryError>;

    /// Check if a key exists
    ///
    /// # Arguments
    /// * `key` - The unique identifier to check
    ///
    /// # Returns
    /// * `Ok(true)` - If the key exists
    /// * `Ok(false)` - If the key doesn't exist
    async fn exists(&self, key: &str) -> Result<bool, MemoryError>;

    /// Get metadata about a stored value
    ///
    /// # Arguments
    /// * `key` - The unique identifier for the value
    ///
    /// # Returns
    /// * `Ok(Metadata)` - The metadata for the stored value
    /// * `Err(MemoryError::KeyNotFound)` - If the key doesn't exist
    async fn get_metadata(&self, key: &str) -> Result<Metadata, MemoryError>;

    /// List keys matching a pattern
    ///
    /// # Arguments
    /// * `pattern` - Pattern to match keys against (e.g., "user_*")
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of matching keys
    /// * `Err(MemoryError)` - If listing failed
    async fn list_keys(&self, pattern: &str) -> Result<Vec<String>, MemoryError>;
}

/// Metadata associated with stored values in shared memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// When the value was created
    pub created_at: DateTime<Utc>,

    /// When the value was last modified
    pub last_modified: DateTime<Utc>,

    /// Content type of the value (e.g., "text/plain", "application/json")
    pub content_type: String,

    /// Size of the value in bytes
    pub size: usize,

    /// Additional metadata as key-value pairs
    pub tags: HashMap<String, String>,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            last_modified: now,
            content_type: "application/json".to_string(),
            size: 0,
            tags: HashMap::new(),
        }
    }
}

/// Errors that can occur during shared memory operations
#[derive(Debug, Error, Clone)]
pub enum MemoryError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Pattern parsing error: {0}")]
    PatternError(String),
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_metadata_default() {
        let metadata = Metadata::default();
        assert_eq!(metadata.content_type, "application/json");
        assert_eq!(metadata.size, 0);
        assert_eq!(metadata.tags.len(), 0);
    }
}

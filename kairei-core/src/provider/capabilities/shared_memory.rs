//! Shared Memory capability for Provider Plugins.
//!
//! The SharedMemoryCapability allows different agents and providers to share data and state
//! in a thread-safe, high-performance way. This enables sophisticated multi-agent communication
//! patterns and stateful interactions without requiring direct message passing.
//!
//! # Key Features
//!
//! - Thread-safe data sharing across providers and agents
//! - Fast key-value operations (sub-millisecond performance)
//! - Support for JSON data structures
//! - TTL-based automatic expiration
//! - Pattern-based key listing
//! - Rich metadata for stored values
//! - Namespace isolation for multi-tenant applications
//!
//! # Usage Example
//!
//! ```no_run
//! use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
//! use kairei_core::provider::config::plugins::SharedMemoryConfig;
//! use serde_json::json;
//! use std::time::Duration;
//!
//! # async fn example(provider_registry: &kairei_core::provider::provider_registry::ProviderRegistry) -> Result<(), Box<dyn std::error::Error>> {
//! // Get shared memory plugin from provider registry
//! let shared_memory_config = SharedMemoryConfig {
//!     base: Default::default(),
//!     max_keys: 100,
//!     ttl: Duration::from_secs(3600),
//!     namespace: "my_namespace".to_string(),
//! };
//!
//! let shared_memory = provider_registry.get_or_create_shared_memory_plugin(&shared_memory_config);
//!
//! // Store a value
//! shared_memory.set("user_123", json!({"name": "Alice", "role": "admin"})).await?;
//!
//! // Retrieve a value
//! let user_data = shared_memory.get("user_123").await?;
//! println!("User name: {}", user_data["name"]);
//!
//! // Check if a key exists
//! if shared_memory.exists("user_123").await? {
//!     println!("User exists!");
//! }
//!
//! // Delete a value
//! shared_memory.delete("user_123").await?;
//! # Ok(())
//! # }
//! ```

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
///
/// # Thread Safety
///
/// All methods in this trait are designed to be thread-safe and can be
/// called concurrently from multiple tasks or threads. Implementations
/// must ensure proper synchronization.
///
/// # Error Handling
///
/// Operations return `Result<T, SharedMemoryError>` to indicate success or failure.
/// Specific error variants provide detailed information about what went wrong.
///
/// # Namespace Isolation
///
/// Keys are isolated by namespace, which is configured when creating the plugin.
/// This allows multiple applications or components to use the same shared memory
/// system without key collisions.
#[async_trait]
pub trait SharedMemoryCapability: ProviderPlugin {
    /// Retrieve a value by key
    ///
    /// # Arguments
    /// * `key` - The unique identifier for the value
    ///
    /// # Returns
    /// * `Ok(Value)` - The stored value if found
    /// * `Err(SharedMemoryError::KeyNotFound)` - If the key doesn't exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
    /// # use serde_json::json;
    /// # async fn example(shared_memory: &impl SharedMemoryCapability) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_data = shared_memory.get("user_123").await?;
    /// println!("User name: {}", user_data["name"]);
    /// # Ok(())
    /// # }
    /// ```
    async fn get(&self, key: &str) -> Result<Value, SharedMemoryError>;

    /// Store a value with the specified key
    ///
    /// # Arguments
    /// * `key` - The unique identifier for the value
    /// * `value` - The value to store
    ///
    /// # Returns
    /// * `Ok(())` - If storage was successful
    /// * `Err(SharedMemoryError)` - If storage failed
    ///
    /// # Notes
    ///
    /// - If the key already exists, its value will be updated
    /// - The TTL will be reset for existing keys
    /// - The operation may fail if the maximum capacity is reached
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
    /// # use serde_json::json;
    /// # async fn example(shared_memory: &impl SharedMemoryCapability) -> Result<(), Box<dyn std::error::Error>> {
    /// shared_memory.set("user_123", json!({"name": "Alice", "role": "admin"})).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn set(&self, key: &str, value: Value) -> Result<(), SharedMemoryError>;

    /// Delete a value by key
    ///
    /// # Arguments
    /// * `key` - The unique identifier for the value to delete
    ///
    /// # Returns
    /// * `Ok(())` - If deletion was successful
    /// * `Err(SharedMemoryError::KeyNotFound)` - If the key doesn't exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
    /// # async fn example(shared_memory: &impl SharedMemoryCapability) -> Result<(), Box<dyn std::error::Error>> {
    /// shared_memory.delete("user_123").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete(&self, key: &str) -> Result<(), SharedMemoryError>;

    /// Check if a key exists
    ///
    /// # Arguments
    /// * `key` - The unique identifier to check
    ///
    /// # Returns
    /// * `Ok(true)` - If the key exists
    /// * `Ok(false)` - If the key doesn't exist
    ///
    /// # Notes
    ///
    /// This method will automatically handle expired keys, returning `false`
    /// for keys that have exceeded their TTL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
    /// # async fn example(shared_memory: &impl SharedMemoryCapability) -> Result<(), Box<dyn std::error::Error>> {
    /// if shared_memory.exists("user_123").await? {
    ///     println!("User exists!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn exists(&self, key: &str) -> Result<bool, SharedMemoryError>;

    /// Get metadata about a stored value
    ///
    /// # Arguments
    /// * `key` - The unique identifier for the value
    ///
    /// # Returns
    /// * `Ok(Metadata)` - The metadata for the stored value
    /// * `Err(SharedMemoryError::KeyNotFound)` - If the key doesn't exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
    /// # async fn example(shared_memory: &impl SharedMemoryCapability) -> Result<(), Box<dyn std::error::Error>> {
    /// let metadata = shared_memory.get_metadata("document_456").await?;
    /// println!("Document size: {} bytes", metadata.size);
    /// println!("Created at: {}", metadata.created_at);
    /// println!("Last modified: {}", metadata.last_modified);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_metadata(&self, key: &str) -> Result<Metadata, SharedMemoryError>;

    /// List keys matching a pattern
    ///
    /// # Arguments
    /// * `pattern` - Pattern to match keys against (e.g., "user_*")
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of matching keys
    /// * `Err(SharedMemoryError)` - If listing failed
    ///
    /// # Pattern Syntax
    ///
    /// The pattern syntax follows glob patterns:
    /// - `*` matches any sequence of characters
    /// - `?` matches any single character
    /// - `[abc]` matches any character in the set
    /// - `[a-z]` matches any character in the range
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
    /// # use serde_json::json;
    /// # async fn example(shared_memory: &impl SharedMemoryCapability) -> Result<(), Box<dyn std::error::Error>> {
    /// // List all user keys
    /// let user_keys = shared_memory.list_keys("user_*").await?;
    /// for key in user_keys {
    ///     let user = shared_memory.get(&key).await?;
    ///     println!("User: {}", user["name"]);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn list_keys(&self, pattern: &str) -> Result<Vec<String>, SharedMemoryError>;
}

/// Metadata associated with stored values in shared memory
///
/// This structure provides information about stored values, including
/// creation and modification times, content type, size, and custom tags.
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
///
/// This enum defines the various error conditions that can occur
/// when working with the SharedMemoryCapability.
#[derive(Debug, Error, Clone)]
pub enum SharedMemoryError {
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

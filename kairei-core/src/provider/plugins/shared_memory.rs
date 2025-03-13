//! Shared Memory plugin implementation.
//!
//! This module provides the reference implementation of the SharedMemoryCapability
//! trait using in-memory storage. It offers thread-safe, high-performance shared
//! memory operations with features like TTL-based expiration, capacity limits,
//! and pattern-based key listing.
//!
//! # Example
//!
//! ```no_run
//! use kairei_core::provider::plugins::shared_memory::InMemorySharedMemoryPlugin;
//! use kairei_core::provider::config::plugins::SharedMemoryConfig;
//! use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
//! use serde_json::json;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a shared memory plugin
//! let plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
//!     base: Default::default(),
//!     max_keys: 100,
//!     ttl: Duration::from_secs(3600),
//!     namespace: "my_namespace".to_string(),
//! });
//!
//! // Store and retrieve values
//! plugin.set("user_123", json!({"name": "Alice"})).await?;
//! let user = plugin.get("user_123").await?;
//! println!("User: {}", user["name"]);
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use glob::Pattern;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

use crate::provider::capabilities::shared_memory::{
    Metadata, SharedMemoryCapability, SharedMemoryError,
};
use crate::provider::capability::CapabilityType;
use crate::provider::config::plugins::SharedMemoryConfig;
use crate::provider::llm::LLMResponse;
use crate::provider::plugin::{PluginContext, ProviderPlugin};
use crate::provider::provider::Section;
use crate::provider::types::ProviderResult;

/// A thread-safe value container with expiration support
///
/// This structure stores a value along with its metadata and
/// optional expiration time.
struct ValueWithMetadata {
    /// The stored JSON value
    value: Value,

    /// Metadata about the value
    metadata: Metadata,

    /// Optional expiration time (None means no expiration)
    expiry: Option<Instant>,
}

/// Reference implementation of SharedMemoryCapability using in-memory storage
///
/// This plugin provides a high-performance, thread-safe implementation of
/// the SharedMemoryCapability trait using in-memory storage. It supports
/// all features of the SharedMemoryCapability interface, including:
///
/// - Thread-safe concurrent access
/// - TTL-based automatic expiration
/// - Capacity limits
/// - Pattern-based key listing
/// - Rich metadata
///
/// # Thread Safety
///
/// The implementation uses DashMap for thread-safe concurrent access,
/// allowing multiple tasks or threads to safely interact with the
/// shared memory simultaneously.
///
/// # Performance Characteristics
///
/// - Get/Set/Delete operations: O(1) average time complexity
/// - Exists operation: O(1) average time complexity
/// - List keys operation: O(n) where n is the number of keys
///
/// # Memory Usage
///
/// Memory usage is proportional to:
/// - Number of keys
/// - Size of stored values
/// - Metadata overhead (approximately 100 bytes per key)
pub struct InMemorySharedMemoryPlugin {
    /// Thread-safe map for storing values
    data: Arc<DashMap<String, ValueWithMetadata>>,
    /// Configuration for the shared memory
    config: SharedMemoryConfig,
}

impl InMemorySharedMemoryPlugin {
    /// Create a new instance with the given configuration
    ///
    /// # Arguments
    /// * `config` - Configuration for the shared memory plugin
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kairei_core::provider::plugins::shared_memory::InMemorySharedMemoryPlugin;
    /// # use kairei_core::provider::config::plugins::SharedMemoryConfig;
    /// # use std::time::Duration;
    /// let plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
    ///     base: Default::default(),
    ///     max_keys: 100,
    ///     ttl: Duration::from_secs(3600),
    ///     namespace: "my_namespace".to_string(),
    /// });
    /// ```
    pub fn new(config: SharedMemoryConfig) -> Self {
        Self {
            data: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Calculate expiry instant based on TTL
    fn calculate_expiry(&self) -> Option<Instant> {
        if self.config.ttl.as_millis() > 0 {
            Some(Instant::now() + self.config.ttl)
        } else {
            None
        }
    }

    /// Check if we've reached the maximum capacity
    fn check_capacity(&self) -> Result<(), SharedMemoryError> {
        if self.config.max_keys > 0 {
            // First, remove all expired keys atomically
            let now = Instant::now();
            self.data.retain(|_, value| {
                if let Some(expiry) = value.expiry {
                    now < expiry
                } else {
                    true
                }
            });

            // Now check capacity after cleanup
            if self.data.len() >= self.config.max_keys {
                return Err(SharedMemoryError::StorageError(format!(
                    "Maximum capacity reached ({} keys)",
                    self.config.max_keys
                )));
            }
        }
        Ok(())
    }

    /// Validate key format
    fn validate_key(&self, key: &str) -> Result<(), SharedMemoryError> {
        if key.is_empty() {
            return Err(SharedMemoryError::InvalidKey("Key cannot be empty".into()));
        }
        // Add any other key validation logic here
        Ok(())
    }
}

#[async_trait]
impl ProviderPlugin for InMemorySharedMemoryPlugin {
    fn priority(&self) -> i32 {
        100 // High priority
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::SharedMemory
    }

    async fn generate_section<'a>(&self, _context: &PluginContext<'a>) -> ProviderResult<Section> {
        // Shared memory plugin doesn't generate prompt sections
        Ok(Section::default())
    }

    async fn process_response<'a>(
        &self,
        _context: &PluginContext<'a>,
        _response: &LLMResponse,
    ) -> ProviderResult<()> {
        // Nothing to process after response
        Ok(())
    }
}

#[async_trait]
impl SharedMemoryCapability for InMemorySharedMemoryPlugin {
    async fn get(&self, key: &str) -> Result<Value, SharedMemoryError> {
        let now = Instant::now();

        // Try to remove the key if it's expired
        let expired = self
            .data
            .remove_if(key, |_, value| {
                if let Some(expiry) = value.expiry {
                    now >= expiry
                } else {
                    false
                }
            })
            .is_some();

        if expired {
            return Err(SharedMemoryError::KeyNotFound(key.to_string()));
        }

        // If not expired, get the value
        if let Some(entry) = self.data.get(key) {
            Ok(entry.value.clone())
        } else {
            Err(SharedMemoryError::KeyNotFound(key.to_string()))
        }
    }

    async fn set(&self, key: &str, value: Value) -> Result<(), SharedMemoryError> {
        // Validate key
        self.validate_key(key)?;

        // Check capacity
        self.check_capacity()?;

        // Calculate size
        let size = serde_json::to_string(&value)
            .map_err(|e| SharedMemoryError::InvalidValue(e.to_string()))?
            .len();

        // Create metadata
        let now = Utc::now();
        let metadata = if let Some(existing) = self.data.get(key) {
            Metadata {
                created_at: existing.metadata.created_at,
                last_modified: now,
                content_type: "application/json".to_string(),
                size,
                tags: existing.metadata.tags.clone(),
            }
        } else {
            Metadata {
                created_at: now,
                last_modified: now,
                content_type: "application/json".to_string(),
                size,
                tags: Default::default(),
            }
        };

        // Create value container
        let value_with_metadata = ValueWithMetadata {
            value,
            metadata,
            expiry: self.calculate_expiry(),
        };

        // Store value
        self.data.insert(key.to_string(), value_with_metadata);

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), SharedMemoryError> {
        if self.data.remove(key).is_some() {
            Ok(())
        } else {
            Err(SharedMemoryError::KeyNotFound(key.to_string()))
        }
    }

    async fn exists(&self, key: &str) -> Result<bool, SharedMemoryError> {
        // Use a single atomic operation to check and remove if expired
        let now = Instant::now();

        // Try to remove the key if it's expired
        let expired = self
            .data
            .remove_if(key, |_, value| {
                if let Some(expiry) = value.expiry {
                    now >= expiry
                } else {
                    false
                }
            })
            .is_some();

        if expired {
            return Ok(false);
        }

        // If not expired, check if it exists
        Ok(self.data.contains_key(key))
    }

    async fn get_metadata(&self, key: &str) -> Result<Metadata, SharedMemoryError> {
        let now = Instant::now();

        // Try to remove the key if it's expired
        let expired = self
            .data
            .remove_if(key, |_, value| {
                if let Some(expiry) = value.expiry {
                    now >= expiry
                } else {
                    false
                }
            })
            .is_some();

        if expired {
            return Err(SharedMemoryError::KeyNotFound(key.to_string()));
        }

        // If not expired, get the metadata
        if let Some(entry) = self.data.get(key) {
            Ok(entry.metadata.clone())
        } else {
            Err(SharedMemoryError::KeyNotFound(key.to_string()))
        }
    }

    async fn list_keys(&self, pattern: &str) -> Result<Vec<String>, SharedMemoryError> {
        if pattern.is_empty() {
            return Err(SharedMemoryError::PatternError(
                "Pattern cannot be empty".into(),
            ));
        }

        // Compile pattern
        let glob_pattern =
            Pattern::new(pattern).map_err(|e| SharedMemoryError::PatternError(e.to_string()))?;

        // First, remove all expired keys atomically
        let now = Instant::now();
        self.data.retain(|_, value| {
            if let Some(expiry) = value.expiry {
                now < expiry
            } else {
                true
            }
        });

        // Then collect matching keys
        let result: Vec<String> = self
            .data
            .iter()
            .filter_map(|entry| {
                let key = entry.key();
                if glob_pattern.matches(key) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use tokio::time::sleep;

    fn create_test_plugin() -> InMemorySharedMemoryPlugin {
        InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
            base: Default::default(),
            max_keys: 100,
            ttl: Duration::from_secs(3600),
            namespace: "test".to_string(),
        })
    }

    #[tokio::test]
    async fn test_basic_operations() {
        let plugin = create_test_plugin();

        // Test set and get
        let value = json!({"test": "value"});
        plugin.set("test_key", value.clone()).await.unwrap();

        let retrieved = plugin.get("test_key").await.unwrap();
        assert_eq!(retrieved, value);

        // Test exists
        assert!(plugin.exists("test_key").await.unwrap());
        assert!(!plugin.exists("nonexistent").await.unwrap());

        // Test delete
        plugin.delete("test_key").await.unwrap();
        assert!(!plugin.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_metadata() {
        let plugin = create_test_plugin();

        let value = json!("metadata_test");
        plugin.set("meta_key", value).await.unwrap();

        let metadata = plugin.get_metadata("meta_key").await.unwrap();
        assert_eq!(metadata.content_type, "application/json");
        assert!(metadata.size > 0);
    }

    // Test for TTL expiration
    #[tokio::test]
    async fn test_ttl_expiration() {
        let plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
            base: Default::default(),
            max_keys: 100,
            ttl: Duration::from_millis(10), // Extremely short TTL for testing
            namespace: "test".to_string(),
        });

        plugin.set("expiring_key", json!("test")).await.unwrap();
        assert!(plugin.exists("expiring_key").await.unwrap());

        // Wait for expiration - use much longer sleep to ensure expiration
        sleep(Duration::from_millis(500)).await;

        // Key should be gone - exists() will automatically handle expired keys
        assert!(!plugin.exists("expiring_key").await.unwrap());
    }

    // Add a more reliable test for expiration logic
    #[tokio::test]
    async fn test_expiration_logic() {
        let plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
            base: Default::default(),
            max_keys: 100,
            ttl: Duration::from_secs(3600), // Long TTL
            namespace: "test".to_string(),
        });

        // Set a key
        plugin.set("test_key", json!("test")).await.unwrap();

        // Manually modify the expiry to be in the past
        if let Some(mut entry) = plugin.data.get_mut("test_key") {
            entry.expiry = Some(Instant::now() - Duration::from_secs(10)); // 10 seconds in the past
        }

        // Now the key should be reported as not existing
        assert!(!plugin.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_pattern_matching() {
        let plugin = create_test_plugin();

        plugin.set("user_1", json!(1)).await.unwrap();
        plugin.set("user_2", json!(2)).await.unwrap();
        plugin.set("admin_1", json!(3)).await.unwrap();

        let user_keys = plugin.list_keys("user_*").await.unwrap();
        assert_eq!(user_keys.len(), 2);
        assert!(user_keys.contains(&"user_1".to_string()));
        assert!(user_keys.contains(&"user_2".to_string()));
    }

    #[tokio::test]
    async fn test_capacity_limits() {
        let plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
            base: Default::default(),
            max_keys: 2,
            ttl: Duration::from_secs(3600),
            namespace: "test".to_string(),
        });

        // Fill to capacity
        plugin.set("key1", json!(1)).await.unwrap();
        plugin.set("key2", json!(2)).await.unwrap();

        // Should fail when capacity is reached
        let result = plugin.set("key3", json!(3)).await;
        assert!(matches!(result, Err(SharedMemoryError::StorageError(_))));
    }

    #[tokio::test]
    async fn test_error_cases() -> ProviderResult<()> {
        let plugin = create_test_plugin();

        // Test empty key
        let result = plugin.set("", json!("test")).await;
        assert!(
            matches!(result, Err(SharedMemoryError::InvalidKey(_))),
            "Empty key should return InvalidKey error"
        );

        // Test key not found
        let result = plugin.get("nonexistent").await;
        assert!(
            matches!(result, Err(SharedMemoryError::KeyNotFound(_))),
            "Nonexistent key should return KeyNotFound error"
        );

        // Test delete nonexistent key
        let result = plugin.delete("nonexistent").await;
        assert!(
            matches!(result, Err(SharedMemoryError::KeyNotFound(_))),
            "Deleting nonexistent key should return KeyNotFound error"
        );

        // Test invalid pattern
        let result = plugin.list_keys("[invalid-pattern").await;
        assert!(
            matches!(result, Err(SharedMemoryError::PatternError(_))),
            "Invalid pattern should return PatternError"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_ttl_edge_cases() -> ProviderResult<()> {
        // Create plugin with zero TTL (no expiration)
        let unlimited_plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
            base: Default::default(),
            max_keys: 100,
            ttl: Duration::from_secs(0), // No expiration
            namespace: "test_unlimited".to_string(),
        });

        // Set a key
        unlimited_plugin
            .set("unlimited_key", json!("value"))
            .await
            .unwrap();

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Key should still exist
        assert!(unlimited_plugin.exists("unlimited_key").await.unwrap());

        // Create plugin with very short TTL
        let short_ttl_plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
            base: Default::default(),
            max_keys: 100,
            ttl: Duration::from_millis(50), // Very short TTL
            namespace: "test_short".to_string(),
        });

        // Set a key
        short_ttl_plugin
            .set("short_key", json!("value"))
            .await
            .unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Key should be gone
        assert!(!short_ttl_plugin.exists("short_key").await.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_metadata_edge_cases() -> ProviderResult<()> {
        let plugin = create_test_plugin();

        // Test metadata for nonexistent key
        let result = plugin.get_metadata("nonexistent").await;
        assert!(
            matches!(result, Err(SharedMemoryError::KeyNotFound(_))),
            "Metadata for nonexistent key should return KeyNotFound error"
        );

        // Test metadata after setting and updating
        plugin.set("meta_key", json!("initial")).await.unwrap();
        let initial_metadata = plugin.get_metadata("meta_key").await.unwrap();

        // Update value
        tokio::time::sleep(Duration::from_millis(10)).await;
        plugin.set("meta_key", json!("updated_data")).await.unwrap();
        let updated_metadata = plugin.get_metadata("meta_key").await.unwrap();

        // created_at should be the same
        assert_eq!(
            initial_metadata.created_at, updated_metadata.created_at,
            "created_at should not change on update"
        );

        // last_modified should be updated
        assert!(
            updated_metadata.last_modified > initial_metadata.last_modified,
            "last_modified should be updated"
        );

        // size should reflect the new value
        assert_ne!(
            initial_metadata.size, updated_metadata.size,
            "size should change when value changes"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_empty_pattern() -> ProviderResult<()> {
        let plugin = create_test_plugin();

        // Add some keys
        plugin.set("test1", json!(1)).await.unwrap();
        plugin.set("test2", json!(2)).await.unwrap();

        // Test empty pattern
        let result = plugin.list_keys("").await;
        assert!(
            matches!(result, Err(SharedMemoryError::PatternError(_))),
            "Empty pattern should return PatternError"
        );

        // Test wildcard pattern
        let keys = plugin.list_keys("*").await.unwrap();
        assert!(!keys.is_empty(), "Wildcard pattern should return all keys");

        Ok(())
    }
}

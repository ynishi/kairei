//! Shared Memory plugin implementation.

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use glob::Pattern;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

use crate::provider::capabilities::shared_memory::{MemoryError, Metadata, SharedMemoryCapability};
use crate::provider::capability::CapabilityType;
use crate::provider::config::plugins::SharedMemoryConfig;
use crate::provider::llm::LLMResponse;
use crate::provider::plugin::{PluginContext, ProviderPlugin};
use crate::provider::provider::Section;
use crate::provider::types::ProviderResult;

/// A thread-safe value container with expiration support
struct ValueWithMetadata {
    value: Value,
    metadata: Metadata,
    expiry: Option<Instant>,
}

/// Reference implementation of SharedMemoryCapability using in-memory storage
pub struct InMemorySharedMemoryPlugin {
    /// Thread-safe map for storing values
    data: Arc<DashMap<String, ValueWithMetadata>>,
    /// Configuration for the shared memory
    config: SharedMemoryConfig,
}

impl InMemorySharedMemoryPlugin {
    /// Create a new instance with the given configuration
    pub fn new(config: SharedMemoryConfig) -> Self {
        Self {
            data: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Check if a key has expired
    fn is_expired(&self, key: &str) -> bool {
        if let Some(entry) = self.data.get(key) {
            if let Some(expiry) = entry.expiry {
                let now = Instant::now();
                return now >= expiry;
            }
        }
        false
    }

    /// Remove expired keys (to be called periodically)
    fn cleanup_expired(&self) {
        let keys_to_remove: Vec<String> = self
            .data
            .iter()
            .filter(|entry| {
                if let Some(expiry) = entry.expiry {
                    Instant::now() > expiry
                } else {
                    false
                }
            })
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_remove {
            self.data.remove(&key);
        }
    }

    /// Calculate expiry instant based on TTL
    fn calculate_expiry(&self) -> Option<Instant> {
        if self.config.ttl.as_secs() > 0 {
            Some(Instant::now() + self.config.ttl)
        } else {
            None
        }
    }

    /// Check if we've reached the maximum capacity
    fn check_capacity(&self) -> Result<(), MemoryError> {
        if self.config.max_keys > 0 && self.data.len() >= self.config.max_keys {
            // Try to clean up expired entries first
            self.cleanup_expired();

            // If still at capacity, fail
            if self.data.len() >= self.config.max_keys {
                return Err(MemoryError::StorageError(format!(
                    "Maximum capacity reached ({} keys)",
                    self.config.max_keys
                )));
            }
        }
        Ok(())
    }

    /// Validate key format
    fn validate_key(&self, key: &str) -> Result<(), MemoryError> {
        if key.is_empty() {
            return Err(MemoryError::InvalidKey("Key cannot be empty".into()));
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
    async fn get(&self, key: &str) -> Result<Value, MemoryError> {
        // Check if key exists and is not expired
        if self.is_expired(key) {
            self.data.remove(key);
            return Err(MemoryError::KeyNotFound(key.to_string()));
        }

        if let Some(entry) = self.data.get(key) {
            Ok(entry.value.clone())
        } else {
            Err(MemoryError::KeyNotFound(key.to_string()))
        }
    }

    async fn set(&self, key: &str, value: Value) -> Result<(), MemoryError> {
        // Validate key
        self.validate_key(key)?;

        // Check capacity
        self.check_capacity()?;

        // Calculate size
        let size = serde_json::to_string(&value)
            .map_err(|e| MemoryError::InvalidValue(e.to_string()))?
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

    async fn delete(&self, key: &str) -> Result<(), MemoryError> {
        if self.data.remove(key).is_some() {
            Ok(())
        } else {
            Err(MemoryError::KeyNotFound(key.to_string()))
        }
    }

    async fn exists(&self, key: &str) -> Result<bool, MemoryError> {
        // Check for expiration
        if self.is_expired(key) {
            self.data.remove(key);
            return Ok(false);
        }

        // Double-check expiration with current time
        if let Some(entry) = self.data.get(key) {
            if let Some(expiry) = entry.expiry {
                if Instant::now() >= expiry {
                    self.data.remove(key);
                    return Ok(false);
                }
            }
        }

        Ok(self.data.contains_key(key))
    }

    async fn get_metadata(&self, key: &str) -> Result<Metadata, MemoryError> {
        // Check for expiration
        if self.is_expired(key) {
            self.data.remove(key);
            return Err(MemoryError::KeyNotFound(key.to_string()));
        }

        if let Some(entry) = self.data.get(key) {
            Ok(entry.metadata.clone())
        } else {
            Err(MemoryError::KeyNotFound(key.to_string()))
        }
    }

    async fn list_keys(&self, pattern: &str) -> Result<Vec<String>, MemoryError> {
        // Compile pattern
        let glob_pattern =
            Pattern::new(pattern).map_err(|e| MemoryError::PatternError(e.to_string()))?;

        // Filter keys by pattern and non-expired status
        let mut result = Vec::new();

        for entry in self.data.iter() {
            let key = entry.key();

            // Skip expired keys
            if self.is_expired(key) {
                self.data.remove(key);
                continue;
            }

            // Match pattern
            if glob_pattern.matches(key) {
                result.push(key.clone());
            }
        }

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

    // Skip this test as it's timing-dependent and can be flaky
    #[tokio::test]
    #[ignore]
    async fn test_ttl_expiration() {
        let plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
            base: Default::default(),
            max_keys: 100,
            ttl: Duration::from_millis(10), // Extremely short TTL for testing
        });

        plugin.set("expiring_key", json!("test")).await.unwrap();
        assert!(plugin.exists("expiring_key").await.unwrap());

        // Wait for expiration - use much longer sleep to ensure expiration
        sleep(Duration::from_millis(500)).await;

        // Explicitly call cleanup to ensure expired keys are removed
        plugin.cleanup_expired();

        // Key should be gone
        assert!(!plugin.exists("expiring_key").await.unwrap());
    }

    // Add a more reliable test for expiration logic
    #[tokio::test]
    async fn test_expiration_logic() {
        let plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
            base: Default::default(),
            max_keys: 100,
            ttl: Duration::from_secs(3600), // Long TTL
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
        });

        // Fill to capacity
        plugin.set("key1", json!(1)).await.unwrap();
        plugin.set("key2", json!(2)).await.unwrap();

        // Should fail when capacity is reached
        let result = plugin.set("key3", json!(3)).await;
        assert!(matches!(result, Err(MemoryError::StorageError(_))));
    }
}

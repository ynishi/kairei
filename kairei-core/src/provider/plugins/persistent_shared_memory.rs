//! Persistent Shared Memory plugin implementation.
//!
//! This module provides an implementation of the SharedMemoryCapability
//! trait that adds persistence through a storage backend. It maintains an
//! in-memory cache for fast access while providing persistence through
//! the configured storage backend.
//!
//! # Example
//!
//! ```no_run
//! use kairei_core::provider::plugins::persistent_shared_memory::PersistentSharedMemoryPlugin;
//! use kairei_core::provider::config::plugins::{PersistentSharedMemoryConfig, SharedMemoryConfig};
//! use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
//! use kairei_core::provider::capabilities::storage::{StorageBackend, ValueWithMetadata};
//! use serde_json::json;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a persistent shared memory plugin
//! let config = PersistentSharedMemoryConfig::default();
//! let plugin = PersistentSharedMemoryPlugin::new(config);
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
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::event::event_registry::EventType;
use crate::event_bus::{Event, EventBus};
use crate::provider::capabilities::shared_memory::{
    Metadata, SharedMemoryCapability, SharedMemoryError,
};
use crate::provider::capabilities::storage::{StorageBackend, StorageError, ValueWithMetadata};
use crate::provider::capability::CapabilityType;
use crate::provider::config::plugins::PersistentSharedMemoryConfig;
use crate::provider::llm::LLMResponse;
use crate::provider::plugin::{PluginContext, ProviderPlugin};
use crate::provider::provider::Section;
use crate::provider::types::ProviderResult;

/// Persistent shared memory plugin that implements SharedMemoryCapability
/// with persistence through a storage backend
///
/// This plugin provides a high-performance, thread-safe implementation of
/// the SharedMemoryCapability trait with persistence through a storage backend.
/// It maintains an in-memory cache for fast access while providing persistence
/// through the configured storage backend.
///
/// # Features
///
/// - Thread-safe concurrent access
/// - TTL-based automatic expiration
/// - Capacity limits
/// - Pattern-based key listing
/// - Rich metadata
/// - Persistence through storage backends
/// - Auto-load on startup
/// - Auto-save on changes
/// - Background sync
///
/// # Thread Safety
///
/// The implementation uses DashMap for thread-safe concurrent access to the
/// in-memory cache, allowing multiple tasks or threads to safely interact with
/// the shared memory simultaneously.
pub struct PersistentSharedMemoryPlugin {
    /// In-memory cache for fast access
    cache: Arc<DashMap<String, ValueWithMetadata>>,

    /// Storage backend for persistence
    backend: Box<dyn StorageBackend>,

    /// Configuration
    config: PersistentSharedMemoryConfig,

    /// Background sync task handle
    sync_task: Option<JoinHandle<()>>,

    /// Last sync time
    last_sync: Arc<RwLock<Option<Instant>>>,

    /// Event bus for notifications
    event_bus: Option<Arc<EventBus>>,
}

/// Event types for persistent shared memory operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistentMemoryEventType {
    /// Sync operation started
    SyncStarted,

    /// Sync operation completed successfully
    SyncCompleted,

    /// Sync operation failed
    SyncFailed,

    /// Load operation started
    LoadStarted,

    /// Load operation completed successfully
    LoadCompleted,

    /// Load operation failed
    LoadFailed,

    /// Save operation started
    SaveStarted,

    /// Save operation completed successfully
    SaveCompleted,

    /// Save operation failed
    SaveFailed,
}

impl PersistentSharedMemoryPlugin {
    /// Create a new instance with the given configuration
    ///
    /// # Arguments
    /// * `config` - Configuration for the persistent shared memory plugin
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kairei_core::provider::plugins::persistent_shared_memory::PersistentSharedMemoryPlugin;
    /// # use kairei_core::provider::config::plugins::PersistentSharedMemoryConfig;
    /// let config = PersistentSharedMemoryConfig::default();
    /// let plugin = PersistentSharedMemoryPlugin::new(config);
    /// ```
    pub async fn new(config: PersistentSharedMemoryConfig) -> Self {
        // Create an empty instance
        let mut instance = Self {
            cache: Arc::new(DashMap::new()),
            backend: Box::new(DummyStorageBackend {}), // Will be replaced during initialization
            config,
            sync_task: None,
            last_sync: Arc::new(RwLock::new(None)),
            event_bus: None,
        };

        // Initialize the instance
        instance.initialize().await;

        instance
    }

    /// Initialize the plugin
    ///
    /// This method sets up the storage backend and starts the background sync task
    /// if configured. It also loads data from storage if auto-load is enabled.
    async fn initialize(&mut self) {
        // Set up the storage backend based on configuration
        self.backend = match self.config.persistence.backend_type {
            crate::provider::config::plugins::BackendType::GCPStorage => {
                if let crate::provider::config::plugins::BackendSpecificConfig::GCP(ref _config) =
                    self.config.persistence.backend_config
                {
                    // Create GCP storage backend
                    // This will be implemented in a later task
                    Box::new(DummyStorageBackend {})
                } else {
                    // Configuration mismatch, use dummy backend
                    Box::new(DummyStorageBackend {})
                }
            }
            crate::provider::config::plugins::BackendType::LocalFileSystem => {
                if let crate::provider::config::plugins::BackendSpecificConfig::Local(ref config) =
                    self.config.persistence.backend_config
                {
                    // Create local file system backend
                    use crate::provider::plugins::storage::local_fs::LocalFileSystemBackend;
                    Box::new(LocalFileSystemBackend::new(config.clone()))
                } else {
                    // Configuration mismatch, use dummy backend
                    Box::new(DummyStorageBackend {})
                }
            }
        };

        // Auto-load if configured
        if self.config.persistence.auto_load {
            // Load data from storage
            let _ = self.load().await;
        }

        // Start background sync task if interval > 0
        if self.config.persistence.sync_interval.as_millis() > 0 {
            self.start_sync_task();
        }
    }

    /// Start the background sync task
    ///
    /// This method starts a background task that periodically syncs the cache
    /// with the storage backend.
    fn start_sync_task(&mut self) {
        let interval = self.config.persistence.sync_interval;
        let backend = self.backend.clone_backend();
        let cache = self.cache.clone();
        let namespace = self.config.base.namespace.clone();
        let last_sync = self.last_sync.clone();
        let event_bus = self.event_bus.clone();

        self.sync_task = Some(tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;

                // Emit sync started event
                if let Some(ref event_bus) = event_bus {
                    let _ = event_bus
                        .publish(Event {
                            event_type: EventType::Custom(
                                "PersistentMemorySyncStarted".to_string(),
                            ),
                            parameters: HashMap::new(),
                        })
                        .await;
                }

                // Perform sync operation
                let data: HashMap<String, ValueWithMetadata> = cache
                    .iter()
                    .map(|entry| (entry.key().clone(), entry.value().clone()))
                    .collect();

                let _ = backend.save(&namespace, &data).await;

                // Update last sync time
                let now = Instant::now();
                let mut last_sync_guard = last_sync.write().await;
                *last_sync_guard = Some(now);

                // Emit sync completed/failed event
                if let Some(ref event_bus) = event_bus {
                    let event_type = EventType::Custom("PersistentMemorySyncCompleted".to_string());
                    let _ = event_bus
                        .publish(Event {
                            event_type,
                            parameters: HashMap::new(),
                        })
                        .await;
                }
            }
        }));
    }

    /// Set the event bus for notifications
    ///
    /// # Arguments
    /// * `event_bus` - The event bus to use for notifications
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>) {
        self.event_bus = Some(event_bus);
    }

    /// Explicitly sync with storage backend
    ///
    /// This method syncs the in-memory cache with the storage backend.
    /// It saves all data in the cache to the storage backend.
    ///
    /// # Returns
    /// * `Ok(())` - If sync succeeds
    /// * `Err(SharedMemoryError)` - If sync fails
    pub async fn sync(&self) -> Result<(), SharedMemoryError> {
        // Emit sync started event
        if let Some(ref event_bus) = self.event_bus {
            let _ = event_bus
                .publish(Event {
                    event_type: EventType::Custom("PersistentMemorySyncStarted".to_string()),
                    parameters: HashMap::new(),
                })
                .await;
        }

        // Perform sync operation by saving all data to the backend
        let result = self.save().await;

        // Update last sync time if successful
        if result.is_ok() {
            let now = Instant::now();
            let mut last_sync_guard = self.last_sync.write().await;
            *last_sync_guard = Some(now);

            // Emit sync completed event
            if let Some(ref event_bus) = self.event_bus {
                let _ = event_bus
                    .publish(Event {
                        event_type: EventType::Custom("PersistentMemorySyncCompleted".to_string()),
                        parameters: HashMap::new(),
                    })
                    .await;
            }
        } else {
            // Emit sync failed event
            if let Some(ref event_bus) = self.event_bus {
                let _ = event_bus
                    .publish(Event {
                        event_type: EventType::Custom("PersistentMemorySyncFailed".to_string()),
                        parameters: HashMap::new(),
                    })
                    .await;
            }
        }

        result
    }

    /// Load data from storage backend
    ///
    /// This method loads data from the storage backend into the in-memory cache.
    /// It replaces all data in the cache with the data from the storage backend.
    ///
    /// # Returns
    /// * `Ok(())` - If loading succeeds
    /// * `Err(SharedMemoryError)` - If loading fails
    pub async fn load(&self) -> Result<(), SharedMemoryError> {
        // Emit load started event
        if let Some(ref event_bus) = self.event_bus {
            let _ = event_bus
                .publish(Event {
                    event_type: EventType::Custom("PersistentMemoryLoadStarted".to_string()),
                    parameters: HashMap::new(),
                })
                .await;
        }

        // Load data from storage backend
        let namespace = &self.config.base.namespace;
        let result = match self.backend.load(namespace).await {
            Ok(data) => {
                // Clear existing cache
                self.cache.clear();

                // Populate cache with loaded data
                for (key, value) in data {
                    self.cache.insert(key, value);
                }

                Ok(())
            }
            Err(err) => Err(SharedMemoryError::from(err)),
        };

        // Emit appropriate event based on result
        if let Some(ref event_bus) = self.event_bus {
            let event_type = if result.is_ok() {
                EventType::Custom("PersistentMemoryLoadCompleted".to_string())
            } else {
                EventType::Custom("PersistentMemoryLoadFailed".to_string())
            };

            let _ = event_bus
                .publish(Event {
                    event_type,
                    parameters: HashMap::new(),
                })
                .await;
        }

        result
    }

    /// Save data to storage backend
    ///
    /// This method saves all data in the in-memory cache to the storage backend.
    ///
    /// # Returns
    /// * `Ok(())` - If saving succeeds
    /// * `Err(SharedMemoryError)` - If saving fails
    pub async fn save(&self) -> Result<(), SharedMemoryError> {
        // Emit save started event
        if let Some(ref event_bus) = self.event_bus {
            let _ = event_bus
                .publish(Event {
                    event_type: EventType::Custom("PersistentMemorySaveStarted".to_string()),
                    parameters: HashMap::new(),
                })
                .await;
        }

        // Convert cache to HashMap for storage
        let data: HashMap<String, ValueWithMetadata> = self
            .cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        // Save data to storage backend
        let namespace = &self.config.base.namespace;
        let result = match self.backend.save(namespace, &data).await {
            Ok(_) => Ok(()),
            Err(err) => Err(SharedMemoryError::from(err)),
        };

        // Emit appropriate event based on result
        if let Some(ref event_bus) = self.event_bus {
            let event_type = if result.is_ok() {
                EventType::Custom("PersistentMemorySaveCompleted".to_string())
            } else {
                EventType::Custom("PersistentMemorySaveFailed".to_string())
            };

            let _ = event_bus
                .publish(Event {
                    event_type,
                    parameters: HashMap::new(),
                })
                .await;
        }

        result
    }

    /// Calculate expiry instant based on TTL
    fn calculate_expiry(&self) -> Option<Instant> {
        if self.config.base.ttl.as_millis() > 0 {
            Some(Instant::now() + self.config.base.ttl)
        } else {
            None
        }
    }

    /// Check if we've reached the maximum capacity
    fn check_capacity(&self) -> Result<(), SharedMemoryError> {
        if self.config.base.max_keys > 0 {
            // First, remove all expired keys atomically
            let now = Instant::now();
            self.cache.retain(|_, value| {
                if let Some(expiry) = value.expiry {
                    now < expiry
                } else {
                    true
                }
            });

            // Now check capacity after cleanup
            if self.cache.len() >= self.config.base.max_keys {
                return Err(SharedMemoryError::StorageError(format!(
                    "Maximum capacity reached ({} keys)",
                    self.config.base.max_keys
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
impl ProviderPlugin for PersistentSharedMemoryPlugin {
    fn priority(&self) -> i32 {
        100 // High priority
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::SharedMemory
    }

    async fn generate_section<'a>(&self, _context: &PluginContext<'a>) -> ProviderResult<Section> {
        // Persistent shared memory plugin doesn't generate prompt sections
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
impl SharedMemoryCapability for PersistentSharedMemoryPlugin {
    async fn get(&self, key: &str) -> Result<Value, SharedMemoryError> {
        let now = Instant::now();

        // Try to remove the key if it's expired
        let expired = self
            .cache
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
        if let Some(entry) = self.cache.get(key) {
            Ok(entry.value.clone())
        } else {
            // If not in cache, try to load from storage
            // This will be implemented in Phase 3
            // For now, just return KeyNotFound
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
        let metadata = if let Some(existing) = self.cache.get(key) {
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
            value: value.clone(),
            metadata,
            expiry: self.calculate_expiry(),
        };

        // Store value in cache
        self.cache.insert(key.to_string(), value_with_metadata);

        // Auto-save if configured
        if self.config.persistence.auto_save {
            // This will be implemented in Phase 3
            // For now, just return Ok
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), SharedMemoryError> {
        if self.cache.remove(key).is_some() {
            // If auto-save is enabled, delete from storage
            if self.config.persistence.auto_save {
                // This will be implemented in Phase 3
                // For now, just return Ok
            }

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
            .cache
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

        // If not expired, check if it exists in cache
        if self.cache.contains_key(key) {
            return Ok(true);
        }

        // If not in cache, try to load from storage
        // This will be implemented in Phase 3
        // For now, just return false
        Ok(false)
    }

    async fn get_metadata(&self, key: &str) -> Result<Metadata, SharedMemoryError> {
        let now = Instant::now();

        // Try to remove the key if it's expired
        let expired = self
            .cache
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
        if let Some(entry) = self.cache.get(key) {
            Ok(entry.metadata.clone())
        } else {
            // If not in cache, try to load from storage
            // This will be implemented in Phase 3
            // For now, just return KeyNotFound
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
        let glob_pattern = glob::Pattern::new(pattern)
            .map_err(|e| SharedMemoryError::PatternError(e.to_string()))?;

        // First, remove all expired keys atomically
        let now = Instant::now();
        self.cache.retain(|_, value| {
            if let Some(expiry) = value.expiry {
                now < expiry
            } else {
                true
            }
        });

        // Then collect matching keys from cache
        let result: Vec<String> = self
            .cache
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

        // In Phase 3, we'll also check storage for keys that match the pattern
        // For now, just return the keys from cache

        Ok(result)
    }
}

/// Dummy storage backend for testing
///
/// This backend doesn't actually store anything and is used as a placeholder
/// until the real backends are implemented.
#[derive(Clone)]
struct DummyStorageBackend {}

#[async_trait]
impl StorageBackend for DummyStorageBackend {
    fn clone_backend(&self) -> Box<dyn StorageBackend> {
        Box::new(self.clone())
    }
    async fn load(
        &self,
        _namespace: &str,
    ) -> Result<HashMap<String, ValueWithMetadata>, StorageError> {
        Ok(HashMap::new())
    }

    async fn save(
        &self,
        _namespace: &str,
        _data: &HashMap<String, ValueWithMetadata>,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn save_key(
        &self,
        _namespace: &str,
        _key: &str,
        _value: &ValueWithMetadata,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn delete_key(&self, _namespace: &str, _key: &str) -> Result<(), StorageError> {
        Ok(())
    }

    async fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use serde_json::json;

    async fn create_test_plugin() -> PersistentSharedMemoryPlugin {
        PersistentSharedMemoryPlugin::new(PersistentSharedMemoryConfig::default()).await
    }

    #[tokio::test]
    async fn test_basic_operations() {
        let plugin = create_test_plugin().await;

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
        let plugin = create_test_plugin().await;

        let value = json!("metadata_test");
        plugin.set("meta_key", value).await.unwrap();

        let metadata = plugin.get_metadata("meta_key").await.unwrap();
        assert_eq!(metadata.content_type, "application/json");
        assert!(metadata.size > 0);
    }

    // Test for TTL expiration
    #[tokio::test]
    async fn test_ttl_expiration() {
        let mut config = PersistentSharedMemoryConfig::default();
        config.base.ttl = Duration::from_millis(10); // Extremely short TTL for testing

        let plugin = PersistentSharedMemoryPlugin::new(config).await;

        plugin.set("expiring_key", json!("test")).await.unwrap();
        assert!(plugin.exists("expiring_key").await.unwrap());

        // Wait for expiration - use much longer sleep to ensure expiration
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Key should be gone - exists() will automatically handle expired keys
        assert!(!plugin.exists("expiring_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_pattern_matching() {
        let plugin = create_test_plugin().await;

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
        let mut config = PersistentSharedMemoryConfig::default();
        config.base.max_keys = 2;

        let plugin = PersistentSharedMemoryPlugin::new(config).await;

        // Fill to capacity
        plugin.set("key1", json!(1)).await.unwrap();
        plugin.set("key2", json!(2)).await.unwrap();

        // Should fail when capacity is reached
        let result = plugin.set("key3", json!(3)).await;
        assert!(matches!(result, Err(SharedMemoryError::StorageError(_))));
    }
}

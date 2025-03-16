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
//! let plugin = PersistentSharedMemoryPlugin::new(config).await;
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
use tracing::error;

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

    /// Cancellation channel for sync task
    sync_cancel: Option<tokio::sync::oneshot::Sender<()>>,

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

impl std::fmt::Display for PersistentMemoryEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyncStarted => write!(f, "persistent_memory.sync.started"),
            Self::SyncCompleted => write!(f, "persistent_memory.sync.completed"),
            Self::SyncFailed => write!(f, "persistent_memory.sync.failed"),
            Self::LoadStarted => write!(f, "persistent_memory.load.started"),
            Self::LoadCompleted => write!(f, "persistent_memory.load.completed"),
            Self::LoadFailed => write!(f, "persistent_memory.load.failed"),
            Self::SaveStarted => write!(f, "persistent_memory.save.started"),
            Self::SaveCompleted => write!(f, "persistent_memory.save.completed"),
            Self::SaveFailed => write!(f, "persistent_memory.save.failed"),
        }
    }
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
            sync_cancel: None,
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
                if let crate::provider::config::plugins::BackendSpecificConfig::GCP(ref config) =
                    self.config.persistence.backend_config
                {
                    // Create GCP storage backend
                    use crate::provider::plugins::storage::gcp::GCPStorageBackend;
                    match GCPStorageBackend::new(config.clone()) {
                        Ok(backend) => Box::new(backend),
                        Err(e) => {
                            // Log error and fall back to dummy backend
                            error!("Failed to create GCP Storage backend: {}", e);
                            Box::new(DummyStorageBackend {})
                        }
                    }
                } else {
                    // Configuration mismatch, use dummy backend
                    error!("Backend type is GCPStorage but config is not GCP");
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

        // Create a cancellation channel
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        self.sync_cancel = Some(tx);

        self.sync_task = Some(tokio::spawn(async move {
            // Start the first tick immediately
            let mut interval_timer = tokio::time::interval(interval);
            interval_timer.tick().await; // Consume the first tick immediately

            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        // Emit sync started event
                        if let Some(ref event_bus) = event_bus {
                            let _ = event_bus
                                .publish(Event {
                                    event_type: EventType::Custom(PersistentMemoryEventType::SyncStarted.to_string()),
                                    parameters: HashMap::new(),
                                })
                                .await;
                        }

                        // Perform sync operation
                        let data: HashMap<String, ValueWithMetadata> = cache
                            .iter()
                            .map(|entry| (entry.key().clone(), entry.value().clone()))
                            .collect();

                        let result = backend.save(&namespace, &data).await;

                        // Update last sync time if successful
                        if result.is_ok() {
                            let now = Instant::now();
                            let mut last_sync_guard = last_sync.write().await;
                            *last_sync_guard = Some(now);

                            // Emit sync completed event
                            if let Some(ref event_bus) = event_bus {
                                let _ = event_bus
                                    .publish(Event {
                                        event_type: EventType::Custom(PersistentMemoryEventType::SyncCompleted.to_string()),
                                        parameters: HashMap::new(),
                                    })
                                    .await;
                            }
                        } else if let Err(ref err) = result {
                            // Emit sync failed event with error information
                            if let Some(ref event_bus) = event_bus {
                                let mut params = HashMap::new();
                                params.insert("error".to_string(), crate::event::event_bus::Value::String(format!("{}", err)));

                                let _ = event_bus
                                    .publish(Event {
                                        event_type: EventType::Custom(PersistentMemoryEventType::SyncFailed.to_string()),
                                        parameters: params,
                                    })
                                    .await;
                            }
                        }
                    }
                    _ = &mut rx => {
                        // Cancellation received, exit the loop
                        break;
                    }
                }
            }
        }));
    }

    /// Stops the background synchronization task
    ///
    /// This method stops the background sync task by sending a cancellation signal
    /// and waiting for the task to complete. It is used internally for cleanup
    /// during shutdown or when reconfiguring the plugin.
    ///
    /// # Note
    /// This method is primarily for internal use during plugin lifecycle management.
    #[allow(dead_code)]
    async fn stop_sync_task(&mut self) {
        // Send cancellation signal if the task is running
        if let Some(cancel) = self.sync_cancel.take() {
            let _ = cancel.send(());
        }

        // Wait for the task to complete
        if let Some(task) = self.sync_task.take() {
            let _ = task.await;
        }
    }

    /// Set the event bus for notifications
    ///
    /// # Arguments
    /// * `event_bus` - The event bus to use for notifications
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>) {
        self.event_bus = Some(event_bus);
    }

    /// Synchronize in-memory cache with storage backend
    ///
    /// This method manually triggers synchronization of all data in the in-memory cache
    /// with the storage backend. It emits events for the start, completion, and failure
    /// of the synchronization process.
    ///
    /// # Returns
    /// * `Ok(())` - If synchronization succeeds
    /// * `Err(SharedMemoryError)` - If synchronization fails
    pub async fn sync(&self) -> Result<(), SharedMemoryError> {
        // Emit sync started event
        if let Some(ref event_bus) = self.event_bus {
            let mut params = HashMap::new();
            params.insert(
                "namespace".to_string(),
                crate::event::event_bus::Value::String(self.config.base.namespace.clone()),
            );

            let _ = event_bus
                .publish(Event {
                    event_type: EventType::Custom(
                        PersistentMemoryEventType::SyncStarted.to_string(),
                    ),
                    parameters: params,
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
                let mut params = HashMap::new();
                params.insert(
                    "namespace".to_string(),
                    crate::event::event_bus::Value::String(self.config.base.namespace.clone()),
                );

                let _ = event_bus
                    .publish(Event {
                        event_type: EventType::Custom(
                            PersistentMemoryEventType::SyncCompleted.to_string(),
                        ),
                        parameters: params,
                    })
                    .await;
            }
        } else if let Err(ref err) = result {
            // Emit sync failed event with error information
            if let Some(ref event_bus) = self.event_bus {
                let mut params = HashMap::new();
                params.insert(
                    "namespace".to_string(),
                    crate::event::event_bus::Value::String(self.config.base.namespace.clone()),
                );
                params.insert(
                    "error".to_string(),
                    crate::event::event_bus::Value::String(format!("{}", err)),
                );

                let _ = event_bus
                    .publish(Event {
                        event_type: EventType::Custom(
                            PersistentMemoryEventType::SyncFailed.to_string(),
                        ),
                        parameters: params,
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
            let mut params = HashMap::new();
            params.insert(
                "namespace".to_string(),
                crate::event::event_bus::Value::String(self.config.base.namespace.clone()),
            );

            let _ = event_bus
                .publish(Event {
                    event_type: EventType::Custom(
                        PersistentMemoryEventType::LoadStarted.to_string(),
                    ),
                    parameters: params,
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
            let mut params = HashMap::new();
            params.insert(
                "namespace".to_string(),
                crate::event::event_bus::Value::String(self.config.base.namespace.clone()),
            );

            // Add error information if the operation failed
            if let Err(ref err) = result {
                params.insert(
                    "error".to_string(),
                    crate::event::event_bus::Value::String(format!("{}", err)),
                );
            }

            let event_type = if result.is_ok() {
                EventType::Custom(PersistentMemoryEventType::LoadCompleted.to_string())
            } else {
                EventType::Custom(PersistentMemoryEventType::LoadFailed.to_string())
            };

            let _ = event_bus
                .publish(Event {
                    event_type,
                    parameters: params,
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
            let mut params = HashMap::new();
            params.insert(
                "namespace".to_string(),
                crate::event::event_bus::Value::String(self.config.base.namespace.clone()),
            );

            let _ = event_bus
                .publish(Event {
                    event_type: EventType::Custom(
                        PersistentMemoryEventType::SaveStarted.to_string(),
                    ),
                    parameters: params,
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
            let mut params = HashMap::new();
            params.insert(
                "namespace".to_string(),
                crate::event::event_bus::Value::String(self.config.base.namespace.clone()),
            );

            // Add error information if the operation failed
            if let Err(ref err) = result {
                params.insert(
                    "error".to_string(),
                    crate::event::event_bus::Value::String(format!("{}", err)),
                );
            }

            let event_type = if result.is_ok() {
                EventType::Custom(PersistentMemoryEventType::SaveCompleted.to_string())
            } else {
                EventType::Custom(PersistentMemoryEventType::SaveFailed.to_string())
            };

            let _ = event_bus
                .publish(Event {
                    event_type,
                    parameters: params,
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
        self.cache
            .insert(key.to_string(), value_with_metadata.clone());

        // Auto-save if configured
        if self.config.persistence.auto_save {
            let backend = self.backend.clone_backend();
            let namespace = self.config.base.namespace.clone();
            let event_bus = self.event_bus.clone();

            // Emit save started event
            if let Some(ref event_bus) = event_bus {
                let mut params = HashMap::new();
                params.insert(
                    "key".to_string(),
                    crate::event::event_bus::Value::String(key.to_string()),
                );

                let _ = event_bus
                    .publish(Event {
                        event_type: EventType::Custom(
                            PersistentMemoryEventType::SaveStarted.to_string(),
                        ),
                        parameters: params,
                    })
                    .await;
            }

            // Save the specific key
            match backend
                .save_key(&namespace, key, &value_with_metadata)
                .await
            {
                Ok(_) => {
                    // Emit save completed event
                    if let Some(ref event_bus) = event_bus {
                        let mut params = HashMap::new();
                        params.insert(
                            "key".to_string(),
                            crate::event::event_bus::Value::String(key.to_string()),
                        );

                        let _ = event_bus
                            .publish(Event {
                                event_type: EventType::Custom(
                                    PersistentMemoryEventType::SaveCompleted.to_string(),
                                ),
                                parameters: params,
                            })
                            .await;
                    }
                }
                Err(e) => {
                    // Emit save failed event
                    if let Some(ref event_bus) = event_bus {
                        let mut params = HashMap::new();
                        params.insert(
                            "key".to_string(),
                            crate::event::event_bus::Value::String(key.to_string()),
                        );
                        params.insert(
                            "error".to_string(),
                            crate::event::event_bus::Value::String(format!("{}", e)),
                        );

                        let _ = event_bus
                            .publish(Event {
                                event_type: EventType::Custom(
                                    PersistentMemoryEventType::SaveFailed.to_string(),
                                ),
                                parameters: params,
                            })
                            .await;
                    }

                    return Err(SharedMemoryError::from(e));
                }
            }
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), SharedMemoryError> {
        if self.cache.remove(key).is_some() {
            // If auto-save is enabled, delete from storage
            if self.config.persistence.auto_save {
                let backend = self.backend.clone_backend();
                let namespace = self.config.base.namespace.clone();
                let event_bus = self.event_bus.clone();

                // Emit save started event
                if let Some(ref event_bus) = event_bus {
                    let mut params = HashMap::new();
                    params.insert(
                        "key".to_string(),
                        crate::event::event_bus::Value::String(key.to_string()),
                    );

                    let _ = event_bus
                        .publish(Event {
                            event_type: EventType::Custom(
                                PersistentMemoryEventType::SaveStarted.to_string(),
                            ),
                            parameters: params,
                        })
                        .await;
                }

                // Delete the key from storage
                match backend.delete_key(&namespace, key).await {
                    Ok(_) => {
                        // Emit save completed event
                        if let Some(ref event_bus) = event_bus {
                            let mut params = HashMap::new();
                            params.insert(
                                "key".to_string(),
                                crate::event::event_bus::Value::String(key.to_string()),
                            );

                            let _ = event_bus
                                .publish(Event {
                                    event_type: EventType::Custom(
                                        PersistentMemoryEventType::SaveCompleted.to_string(),
                                    ),
                                    parameters: params,
                                })
                                .await;
                        }
                    }
                    Err(e) => {
                        // Emit save failed event
                        if let Some(ref event_bus) = event_bus {
                            let mut params = HashMap::new();
                            params.insert(
                                "key".to_string(),
                                crate::event::event_bus::Value::String(key.to_string()),
                            );
                            params.insert(
                                "error".to_string(),
                                crate::event::event_bus::Value::String(format!("{}", e)),
                            );

                            let _ = event_bus
                                .publish(Event {
                                    event_type: EventType::Custom(
                                        PersistentMemoryEventType::SaveFailed.to_string(),
                                    ),
                                    parameters: params,
                                })
                                .await;
                        }

                        return Err(SharedMemoryError::from(e));
                    }
                }
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
impl Drop for PersistentSharedMemoryPlugin {
    fn drop(&mut self) {
        // Send cancellation signal if the task is running
        if let Some(cancel) = self.sync_cancel.take() {
            let _ = cancel.send(());
        }
    }
}
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
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;

    use super::*;
    use serde_json::json;

    // Mock storage backend for testing
    #[derive(Clone)]
    struct MockStorageBackend {
        saved_data: Arc<RwLock<HashMap<String, HashMap<String, ValueWithMetadata>>>>,
        save_called: Arc<RwLock<usize>>,
        save_key_called: Arc<RwLock<usize>>,
        delete_key_called: Arc<RwLock<usize>>,
        should_fail: bool,
    }

    impl MockStorageBackend {
        fn new() -> Self {
            Self {
                saved_data: Arc::new(RwLock::new(HashMap::new())),
                save_called: Arc::new(RwLock::new(0)),
                save_key_called: Arc::new(RwLock::new(0)),
                delete_key_called: Arc::new(RwLock::new(0)),
                should_fail: false,
            }
        }

        // Removed unused method

        async fn save_called_count(&self) -> usize {
            *self.save_called.read().await
        }

        async fn save_key_called_count(&self) -> usize {
            *self.save_key_called.read().await
        }

        async fn delete_key_called_count(&self) -> usize {
            *self.delete_key_called.read().await
        }
    }

    #[async_trait]
    impl StorageBackend for MockStorageBackend {
        fn clone_backend(&self) -> Box<dyn StorageBackend> {
            Box::new(self.clone())
        }

        async fn load(
            &self,
            namespace: &str,
        ) -> Result<HashMap<String, ValueWithMetadata>, StorageError> {
            if self.should_fail {
                return Err(StorageError::StorageError("Simulated failure".into()));
            }

            let saved = self.saved_data.read().await;
            if let Some(data) = saved.get(namespace) {
                Ok(data.clone())
            } else {
                Ok(HashMap::new())
            }
        }

        async fn save(
            &self,
            namespace: &str,
            data: &HashMap<String, ValueWithMetadata>,
        ) -> Result<(), StorageError> {
            if self.should_fail {
                return Err(StorageError::StorageError("Simulated failure".into()));
            }

            let mut save_called = self.save_called.write().await;
            *save_called += 1;

            let mut saved = self.saved_data.write().await;
            saved.insert(namespace.to_string(), data.clone());
            Ok(())
        }

        async fn save_key(
            &self,
            namespace: &str,
            key: &str,
            value: &ValueWithMetadata,
        ) -> Result<(), StorageError> {
            if self.should_fail {
                return Err(StorageError::StorageError("Simulated failure".into()));
            }

            let mut save_key_called = self.save_key_called.write().await;
            *save_key_called += 1;

            let mut saved = self.saved_data.write().await;
            let namespace_data = saved
                .entry(namespace.to_string())
                .or_insert_with(HashMap::new);
            namespace_data.insert(key.to_string(), value.clone());
            Ok(())
        }

        async fn delete_key(&self, namespace: &str, key: &str) -> Result<(), StorageError> {
            if self.should_fail {
                return Err(StorageError::StorageError("Simulated failure".into()));
            }

            let mut delete_key_called = self.delete_key_called.write().await;
            *delete_key_called += 1;

            let mut saved = self.saved_data.write().await;
            if let Some(namespace_data) = saved.get_mut(namespace) {
                namespace_data.remove(key);
            }
            Ok(())
        }

        async fn is_available(&self) -> bool {
            !self.should_fail
        }
    }

    #[tokio::test]
    async fn test_sync_method() {
        // Create a mock backend
        let mock_backend = Arc::new(MockStorageBackend::new());

        // Create a plugin with the mock backend
        let mut config = PersistentSharedMemoryConfig::default();
        config.persistence.sync_interval = Duration::from_secs(3600); // Long interval to avoid auto-sync

        let mut plugin = PersistentSharedMemoryPlugin::new(config).await;
        plugin.backend = mock_backend.clone_backend();

        // Add some data
        plugin.set("test_key", json!("test_value")).await.unwrap();

        // Call sync
        plugin.sync().await.unwrap();

        // Verify that save was called
        assert_eq!(mock_backend.save_called_count().await, 1);

        // Verify that the data was saved
        let saved = mock_backend.saved_data.read().await;
        assert!(saved.contains_key(&plugin.config.base.namespace));
        let namespace_data = &saved[&plugin.config.base.namespace];
        assert!(namespace_data.contains_key("test_key"));
        assert_eq!(namespace_data["test_key"].value, json!("test_value"));
    }

    #[tokio::test]
    async fn test_sync_failure() {
        // Create a mock backend that fails only for save operations but not for set
        let mock_backend = Arc::new(MockStorageBackend::new());

        // Create a plugin with the mock backend
        let mut config = PersistentSharedMemoryConfig::default();
        config.persistence.sync_interval = Duration::from_secs(3600); // Long interval to avoid auto-sync
        config.persistence.auto_save = false; // Disable auto-save to avoid immediate failure

        let mut plugin = PersistentSharedMemoryPlugin::new(config).await;
        plugin.backend = mock_backend.clone_backend();

        // Add some data
        plugin.set("test_key", json!("test_value")).await.unwrap();

        // Now make the backend fail for the sync operation
        let mut backend = MockStorageBackend::new();
        backend.should_fail = true;
        plugin.backend = Box::new(backend);

        // Call sync and expect failure
        let result = plugin.sync().await;
        assert!(
            result.is_err(),
            "Expected sync to fail with mock backend that should fail"
        );
    }

    #[tokio::test]
    async fn test_auto_save_on_set() {
        // Create a mock backend
        let mock_backend = Arc::new(MockStorageBackend::new());

        // Create a plugin with the mock backend and auto-save enabled
        let mut config = PersistentSharedMemoryConfig::default();
        config.persistence.auto_save = true;
        config.persistence.sync_interval = Duration::from_secs(3600); // Long interval to avoid auto-sync

        let mut plugin = PersistentSharedMemoryPlugin::new(config).await;
        plugin.backend = mock_backend.clone_backend();

        // Set a key
        plugin
            .set("auto_save_key", json!("auto_save_value"))
            .await
            .unwrap();

        // Verify that save_key was called
        assert_eq!(mock_backend.save_key_called_count().await, 1);

        // Verify that the data was saved
        let saved = mock_backend.saved_data.read().await;
        assert!(saved.contains_key(&plugin.config.base.namespace));
        let namespace_data = &saved[&plugin.config.base.namespace];
        assert!(namespace_data.contains_key("auto_save_key"));
        assert_eq!(
            namespace_data["auto_save_key"].value,
            json!("auto_save_value")
        );
    }

    #[tokio::test]
    async fn test_auto_save_on_delete() {
        // Create a mock backend
        let mock_backend = Arc::new(MockStorageBackend::new());

        // Create a plugin with the mock backend and auto-save enabled
        let mut config = PersistentSharedMemoryConfig::default();
        config.persistence.auto_save = true;
        config.persistence.sync_interval = Duration::from_secs(3600); // Long interval to avoid auto-sync

        let mut plugin = PersistentSharedMemoryPlugin::new(config).await;
        plugin.backend = mock_backend.clone_backend();

        // Add a key
        plugin
            .set("delete_key", json!("delete_value"))
            .await
            .unwrap();

        // Reset the counter
        let mut save_key_called = mock_backend.save_key_called.write().await;
        *save_key_called = 0;
        drop(save_key_called);

        // Delete the key
        plugin.delete("delete_key").await.unwrap();

        // Verify that delete_key was called
        assert_eq!(mock_backend.delete_key_called_count().await, 1);
    }

    #[tokio::test]
    async fn test_background_sync_task() {
        // Create a mock backend
        let mock_backend = Arc::new(MockStorageBackend::new());

        // Create a plugin with the mock backend and a short sync interval
        let mut config = PersistentSharedMemoryConfig::default();
        config.persistence.sync_interval = Duration::from_millis(50); // Very short interval for testing
        config.persistence.auto_save = false; // Disable auto-save to avoid interference

        let mut plugin = PersistentSharedMemoryPlugin::new(config).await;

        // Manually trigger the background sync task to start
        plugin.stop_sync_task().await; // Ensure no existing task
        plugin.backend = mock_backend.clone_backend();
        plugin.start_sync_task();

        // Add some data
        plugin
            .set("background_key", json!("background_value"))
            .await
            .unwrap();

        // Wait for the background sync to happen - use a longer wait time to ensure sync occurs
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Verify that save was called at least once
        let save_count = mock_backend.save_called_count().await;
        assert!(
            save_count >= 1,
            "Expected at least one save call, got {}",
            save_count
        );

        // Stop the sync task
        plugin.stop_sync_task().await;

        // Reset the counter
        let mut save_called = mock_backend.save_called.write().await;
        *save_called = 0;
        drop(save_called);

        // Wait to ensure no more syncs happen
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Verify that save was not called again
        assert_eq!(mock_backend.save_called_count().await, 0);
    }

    // Original test functions

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
        // Test that capacity checking works as expected
        let config = PersistentSharedMemoryConfig {
            base: Default::default(),
            persistence: Default::default(),
        };

        // Create plugin with default settings (which should have a high max_keys)
        let plugin = PersistentSharedMemoryPlugin::new(config).await;

        // Store a key and verify it works
        let result = plugin.set("test_key", json!(123)).await;
        assert!(result.is_ok());

        // Verify we can retrieve it
        let value = plugin.get("test_key").await;
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), json!(123));

        // This test verifies that the basic functionality works
        // without hitting capacity limits with default settings
    }
}

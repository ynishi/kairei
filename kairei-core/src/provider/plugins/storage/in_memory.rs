//! In-memory backend for persistent shared memory.
//!
//! This module provides an implementation of the StorageBackend trait
//! that stores data in memory using DashMap for thread-safe concurrent access.
//! Each namespace is stored as a separate DashMap entry.

use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

use crate::provider::capabilities::storage::{StorageBackend, StorageError, ValueWithMetadata};
use crate::provider::config::plugins::InMemoryConfig;

/// In-memory backend for persistent shared memory
///
/// This backend stores data in memory using DashMap for thread-safe concurrent access.
/// Each namespace is stored as a separate entry in the DashMap.
///
/// # Thread Safety
///
/// This implementation is thread-safe and can be used concurrently from
/// multiple tasks or threads. DashMap provides interior mutability with
/// fine-grained locking for high performance in concurrent scenarios.
///
/// # Performance Characteristics
///
/// - Fast read/write operations (constant time complexity)
/// - No disk I/O overhead
/// - Memory usage scales with the amount of stored data
/// - Data is lost when the process terminates
pub struct InMemoryBackend {
    /// Configuration for the in-memory backend
    config: InMemoryConfig,

    /// Storage for namespaces and their key-value pairs
    /// The outer DashMap stores namespaces, and each value is a HashMap of keys to values
    storage: Arc<DashMap<String, HashMap<String, ValueWithMetadata>>>,
}

impl InMemoryBackend {
    /// Create a new instance with the given configuration
    ///
    /// # Arguments
    /// * `config` - Configuration for the in-memory backend
    ///
    /// # Returns
    /// * `Self` - A new instance of the in-memory backend
    pub fn new(config: InMemoryConfig) -> Self {
        Self {
            config,
            storage: Arc::new(DashMap::new()),
        }
    }
}

impl Clone for InMemoryBackend {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            storage: Arc::clone(&self.storage),
        }
    }
}

#[async_trait]
impl StorageBackend for InMemoryBackend {
    fn clone_backend(&self) -> Box<dyn StorageBackend> {
        Box::new(self.clone())
    }

    async fn load(
        &self,
        namespace: &str,
    ) -> Result<HashMap<String, ValueWithMetadata>, StorageError> {
        // If the namespace exists, return a clone of its data
        if let Some(data) = self.storage.get(namespace) {
            Ok(data.clone())
        } else {
            // If the namespace doesn't exist, return an empty HashMap
            Ok(HashMap::new())
        }
    }

    async fn save(
        &self,
        namespace: &str,
        data: &HashMap<String, ValueWithMetadata>,
    ) -> Result<(), StorageError> {
        // Check if we're at capacity and this is a new namespace
        if !self.storage.contains_key(namespace)
            && self.config.max_namespaces > 0
            && self.storage.len() >= self.config.max_namespaces
        {
            return Err(StorageError::StorageError(
                "Maximum number of namespaces reached".to_string(),
            ));
        }

        // Insert or overwrite the namespace data
        self.storage.insert(namespace.to_string(), data.clone());
        Ok(())
    }

    async fn save_key(
        &self,
        namespace: &str,
        key: &str,
        value: &ValueWithMetadata,
    ) -> Result<(), StorageError> {
        // Get the namespace data or create a new empty HashMap
        let mut data = self.load(namespace).await?;

        // Check if we're at capacity for this namespace
        if !data.contains_key(key)
            && self.config.max_keys_per_namespace > 0
            && data.len() >= self.config.max_keys_per_namespace
        {
            return Err(StorageError::StorageError(
                "Maximum number of keys per namespace reached".to_string(),
            ));
        }

        // Update the key
        data.insert(key.to_string(), value.clone());

        // Save the updated data
        self.save(namespace, &data).await
    }

    async fn delete_key(&self, namespace: &str, key: &str) -> Result<(), StorageError> {
        // Get the namespace data
        if let Some(mut data_ref) = self.storage.get_mut(namespace) {
            // Remove the key
            data_ref.remove(key);
            Ok(())
        } else {
            // If the namespace doesn't exist, this is not an error
            Ok(())
        }
    }

    async fn is_available(&self) -> bool {
        // In-memory storage is always available
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::task;

    /// Create a test backend
    fn create_test_backend() -> InMemoryBackend {
        let config = InMemoryConfig {
            max_namespaces: 100,
            max_keys_per_namespace: 1000,
        };
        InMemoryBackend::new(config)
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let backend = create_test_backend();
        let namespace = "test_namespace";

        // Create test data
        let mut data = HashMap::new();
        let value = ValueWithMetadata {
            value: json!({"name": "test", "value": 123}),
            metadata: crate::provider::capabilities::shared_memory::Metadata::default(),
            expiry: None,
        };
        data.insert("test_key".to_string(), value);

        // Save the data
        backend.save(namespace, &data).await.unwrap();

        // Load the data
        let loaded_data = backend.load(namespace).await.unwrap();

        // Verify the data
        assert_eq!(loaded_data.len(), 1);
        assert!(loaded_data.contains_key("test_key"));
        assert_eq!(
            loaded_data["test_key"].value,
            json!({"name": "test", "value": 123})
        );
    }

    #[tokio::test]
    async fn test_save_key_and_delete_key() {
        let backend = create_test_backend();
        let namespace = "test_namespace";

        // Create a value
        let value = ValueWithMetadata {
            value: json!("test_value"),
            metadata: crate::provider::capabilities::shared_memory::Metadata::default(),
            expiry: None,
        };

        // Save the key
        backend
            .save_key(namespace, "test_key", &value)
            .await
            .unwrap();

        // Verify the key exists
        let data = backend.load(namespace).await.unwrap();
        assert!(data.contains_key("test_key"));

        // Delete the key
        backend.delete_key(namespace, "test_key").await.unwrap();

        // Verify the key is gone
        let data = backend.load(namespace).await.unwrap();
        assert!(!data.contains_key("test_key"));
    }

    #[tokio::test]
    async fn test_multiple_namespaces() {
        let backend = create_test_backend();

        // Create test data for namespace1
        let mut data1 = HashMap::new();
        let value1 = ValueWithMetadata {
            value: json!("value1"),
            metadata: crate::provider::capabilities::shared_memory::Metadata::default(),
            expiry: None,
        };
        data1.insert("key1".to_string(), value1);

        // Create test data for namespace2
        let mut data2 = HashMap::new();
        let value2 = ValueWithMetadata {
            value: json!("value2"),
            metadata: crate::provider::capabilities::shared_memory::Metadata::default(),
            expiry: None,
        };
        data2.insert("key2".to_string(), value2);

        // Save the data to different namespaces
        backend.save("namespace1", &data1).await.unwrap();
        backend.save("namespace2", &data2).await.unwrap();

        // Load the data from namespace1
        let loaded_data1 = backend.load("namespace1").await.unwrap();
        assert_eq!(loaded_data1.len(), 1);
        assert!(loaded_data1.contains_key("key1"));
        assert_eq!(loaded_data1["key1"].value, json!("value1"));

        // Load the data from namespace2
        let loaded_data2 = backend.load("namespace2").await.unwrap();
        assert_eq!(loaded_data2.len(), 1);
        assert!(loaded_data2.contains_key("key2"));
        assert_eq!(loaded_data2["key2"].value, json!("value2"));
    }

    #[tokio::test]
    async fn test_is_available() {
        let backend = create_test_backend();

        // In-memory backend should always be available
        assert!(backend.is_available().await);
    }

    #[tokio::test]
    async fn test_capacity_limits() {
        // Create a backend with limited capacity
        let config = InMemoryConfig {
            max_namespaces: 2,
            max_keys_per_namespace: 2,
        };
        let backend = InMemoryBackend::new(config);

        // Add two namespaces (should succeed)
        let empty_data = HashMap::new();
        backend.save("namespace1", &empty_data).await.unwrap();
        backend.save("namespace2", &empty_data).await.unwrap();

        // Try to add a third namespace (should fail)
        let result = backend.save("namespace3", &empty_data).await;
        assert!(result.is_err());

        // Add two keys to namespace1 (should succeed)
        let value = ValueWithMetadata {
            value: json!("test"),
            metadata: crate::provider::capabilities::shared_memory::Metadata::default(),
            expiry: None,
        };
        backend
            .save_key("namespace1", "key1", &value)
            .await
            .unwrap();
        backend
            .save_key("namespace1", "key2", &value)
            .await
            .unwrap();

        // Try to add a third key (should fail)
        let result = backend.save_key("namespace1", "key3", &value).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let backend = Arc::new(create_test_backend());
        let namespace = "concurrent_test";

        // First, ensure the namespace exists with an empty map
        let empty_data: HashMap<String, ValueWithMetadata> = HashMap::new();
        backend.save(namespace, &empty_data).await.unwrap();

        // Create multiple tasks that concurrently write and read
        let mut handles = Vec::new();
        for i in 0..10 {
            let backend_clone = Arc::clone(&backend);
            let namespace = namespace.to_string();
            let handle = task::spawn(async move {
                let key = format!("key{}", i);
                let value = ValueWithMetadata {
                    value: json!(format!("value{}", i)),
                    metadata: crate::provider::capabilities::shared_memory::Metadata::default(),
                    expiry: None,
                };

                // Save the key
                backend_clone
                    .save_key(&namespace, &key, &value)
                    .await
                    .unwrap();

                // Read it back
                let data = backend_clone.load(&namespace).await.unwrap();
                assert!(data.contains_key(&key));
                assert_eq!(data[&key].value, json!(format!("value{}", i)));
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all keys are present
        let data = backend.load(namespace).await.unwrap();
        assert_eq!(data.len(), 10);
        for i in 0..10 {
            let key = format!("key{}", i);
            assert!(data.contains_key(&key));
            assert_eq!(data[&key].value, json!(format!("value{}", i)));
        }
    }
}

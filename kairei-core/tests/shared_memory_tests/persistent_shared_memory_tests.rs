//! Integration tests for PersistentSharedMemoryPlugin with LocalFileSystemBackend
//!
//! These tests verify the integration of PersistentSharedMemoryPlugin with
//! the LocalFileSystemBackend implementation.

use std::collections::HashMap;
use std::time::Duration;

use serde_json::json;
use tempfile::TempDir;

use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
use kairei_core::provider::capabilities::storage::{StorageBackend, ValueWithMetadata};
use kairei_core::provider::config::plugins::{
    BackendSpecificConfig, BackendType, LocalFileSystemConfig, PersistenceConfig,
    PersistentSharedMemoryConfig, SharedMemoryConfig,
};
use kairei_core::provider::plugins::persistent_shared_memory::PersistentSharedMemoryPlugin;
use kairei_core::provider::plugins::storage::local_fs::LocalFileSystemBackend;

/// Helper function to create a test plugin with LocalFileSystemBackend
async fn create_test_plugin(
    namespace: Option<String>,
    temp_dir: Option<TempDir>,
) -> (PersistentSharedMemoryPlugin, TempDir) {
    // Create a temporary directory for testing or use the provided one
    let temp_dir = temp_dir.unwrap_or_else(|| TempDir::new().unwrap());
    let temp_path = temp_dir.path().to_string_lossy().to_string();

    // Create local file system config
    let local_config = LocalFileSystemConfig {
        base_dir: temp_path,
        file_extension: "json".to_string(),
    };

    // Create persistence config
    let persistence_config = PersistenceConfig {
        backend_type: BackendType::LocalFileSystem,
        sync_interval: Duration::from_secs(1),
        auto_load: true,
        auto_save: true,
        backend_config: BackendSpecificConfig::Local(local_config),
    };

    // Create shared memory config with the provided namespace or a unique one
    let namespace = namespace.unwrap_or_else(|| {
        format!(
            "test_namespace_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        )
    });

    let shared_memory_config = SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_secs(3600),
        namespace,
    };

    // Create persistent shared memory config
    let config = PersistentSharedMemoryConfig {
        base: shared_memory_config,
        persistence: persistence_config,
    };

    // Create plugin
    let plugin = PersistentSharedMemoryPlugin::new(config).await;

    (plugin, temp_dir)
}

#[tokio::test]
async fn test_basic_persistence() -> Result<(), Box<dyn std::error::Error>> {
    // Create a unique namespace for this test
    let namespace = format!(
        "test_basic_persistence_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Create plugin
    let (plugin, temp_dir) = create_test_plugin(Some(namespace.clone()), None).await;

    // Store a value
    let value = json!({"name": "test", "value": 123});
    plugin.set("test_key", value.clone()).await?;

    // Sync to storage
    plugin.sync().await?;

    // Create a new plugin with the same config to verify persistence
    let (plugin2, _) = create_test_plugin(Some(namespace), Some(temp_dir)).await;

    // Verify the value was persisted
    let retrieved = plugin2.get("test_key").await?;
    assert_eq!(retrieved, value);

    Ok(())
}

#[tokio::test]
async fn test_auto_save() -> Result<(), Box<dyn std::error::Error>> {
    // Create a unique namespace for this test
    let namespace = format!(
        "test_auto_save_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Create plugin with auto-save enabled
    let (plugin, temp_dir) = create_test_plugin(Some(namespace.clone()), None).await;

    // Store a value
    let value = json!({"auto_save": true});
    plugin.set("auto_save_key", value.clone()).await?;

    // Wait a bit for auto-save to happen
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // Create a new plugin with the same config to verify auto-save
    let (plugin2, _) = create_test_plugin(Some(namespace), Some(temp_dir)).await;

    // Verify the value was auto-saved
    let retrieved = plugin2.get("auto_save_key").await?;
    assert_eq!(retrieved, value);

    Ok(())
}

#[tokio::test]
async fn test_auto_load() -> Result<(), Box<dyn std::error::Error>> {
    // Create a unique namespace for this test
    let namespace = format!(
        "test_auto_load_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Create plugin
    let (plugin, temp_dir) = create_test_plugin(Some(namespace.clone()), None).await;

    // Store and sync a value
    let value = json!({"auto_load": true});
    plugin.set("auto_load_key", value.clone()).await?;
    plugin.sync().await?;

    // Create a new plugin with auto-load enabled
    // The value should be automatically loaded
    let (plugin2, _) = create_test_plugin(Some(namespace), Some(temp_dir)).await;

    // Verify the value was auto-loaded
    let retrieved = plugin2.get("auto_load_key").await?;
    assert_eq!(retrieved, value);

    Ok(())
}

#[tokio::test]
async fn test_delete_persistence() -> Result<(), Box<dyn std::error::Error>> {
    // Create a unique namespace for this test
    let namespace = format!(
        "test_delete_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Create plugin
    let (plugin, temp_dir) = create_test_plugin(Some(namespace.clone()), None).await;

    // Store a value
    let value = json!("delete_test");
    plugin.set("delete_key", value).await?;
    plugin.sync().await?;

    // Delete the key
    plugin.delete("delete_key").await?;
    plugin.sync().await?;

    // Create a new plugin
    let (plugin2, _) = create_test_plugin(Some(namespace), Some(temp_dir)).await;

    // Verify the key is gone
    assert!(!plugin2.exists("delete_key").await?);

    Ok(())
}

#[tokio::test]
async fn test_multiple_keys() -> Result<(), Box<dyn std::error::Error>> {
    // Create a unique namespace for this test
    let namespace = format!(
        "test_multiple_keys_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Create plugin
    let (plugin, temp_dir) = create_test_plugin(Some(namespace.clone()), None).await;

    // Store multiple values
    for i in 0..10 {
        let key = format!("key{}", i);
        let value = json!({"index": i});
        plugin.set(&key, value).await?;
    }

    // Sync to storage
    plugin.sync().await?;

    // Create a new plugin
    let (plugin2, _) = create_test_plugin(Some(namespace), Some(temp_dir)).await;

    // Verify all keys were persisted
    for i in 0..10 {
        let key = format!("key{}", i);
        let retrieved = plugin2.get(&key).await?;
        assert_eq!(retrieved, json!({"index": i}));
    }

    Ok(())
}

#[tokio::test]
async fn test_pattern_matching_persistence() -> Result<(), Box<dyn std::error::Error>> {
    // Create a unique namespace for this test
    let namespace = format!(
        "test_pattern_matching_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Create plugin
    let (plugin, temp_dir) = create_test_plugin(Some(namespace.clone()), None).await;

    // Store values with different prefixes
    plugin.set("user_1", json!({"name": "Alice"})).await?;
    plugin.set("user_2", json!({"name": "Bob"})).await?;
    plugin.set("item_1", json!({"name": "Apple"})).await?;
    plugin.set("item_2", json!({"name": "Banana"})).await?;

    // Sync to storage
    plugin.sync().await?;

    // Create a new plugin
    let (plugin2, _) = create_test_plugin(Some(namespace), Some(temp_dir)).await;

    // List keys with pattern
    let user_keys = plugin2.list_keys("user_*").await?;
    assert_eq!(user_keys.len(), 2);
    assert!(user_keys.contains(&"user_1".to_string()));
    assert!(user_keys.contains(&"user_2".to_string()));

    let item_keys = plugin2.list_keys("item_*").await?;
    assert_eq!(item_keys.len(), 2);
    assert!(item_keys.contains(&"item_1".to_string()));
    assert!(item_keys.contains(&"item_2".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_ttl_expiration_persistence() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_string_lossy().to_string();

    // Create local file system config
    let local_config = LocalFileSystemConfig {
        base_dir: temp_path,
        file_extension: "json".to_string(),
    };

    // Create persistence config
    let persistence_config = PersistenceConfig {
        backend_type: BackendType::LocalFileSystem,
        sync_interval: Duration::from_secs(1),
        auto_load: true,
        auto_save: true,
        backend_config: BackendSpecificConfig::Local(local_config),
    };

    // Create shared memory config with short TTL
    let shared_memory_config = SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_millis(100), // Very short TTL
        namespace: "test_namespace".to_string(),
    };

    // Create persistent shared memory config
    let config = PersistentSharedMemoryConfig {
        base: shared_memory_config,
        persistence: persistence_config,
    };

    // Create plugin
    let plugin = PersistentSharedMemoryPlugin::new(config).await;

    // Store a value
    let value = json!("ttl_test");
    plugin.set("ttl_key", value).await?;
    plugin.sync().await?;

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify the key is expired
    assert!(!plugin.exists("ttl_key").await?);

    // Create a new plugin
    let (plugin2, _) = create_test_plugin(Some("direct_namespace".to_string()), None).await;
    plugin2.load().await?;

    // Verify the key is still in storage but expired in memory
    assert!(!plugin2.exists("ttl_key").await?);

    Ok(())
}

#[tokio::test]
async fn test_metadata_persistence() -> Result<(), Box<dyn std::error::Error>> {
    // Create a unique namespace for this test
    let namespace = format!(
        "test_metadata_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Create plugin
    let (plugin, temp_dir) = create_test_plugin(Some(namespace.clone()), None).await;

    // Store a value
    let value = json!("metadata_test");
    plugin.set("metadata_key", value).await?;

    // Get metadata
    let metadata = plugin.get_metadata("metadata_key").await?;
    assert_eq!(metadata.content_type, "application/json");
    assert!(metadata.size > 0);

    // Sync to storage
    plugin.sync().await?;

    // Create a new plugin
    let (plugin2, _) = create_test_plugin(Some(namespace), Some(temp_dir)).await;

    // Verify metadata was persisted
    let metadata2 = plugin2.get_metadata("metadata_key").await?;
    assert_eq!(metadata2.content_type, metadata.content_type);
    assert_eq!(metadata2.size, metadata.size);
    assert_eq!(metadata2.created_at, metadata.created_at);

    Ok(())
}

#[tokio::test]
async fn test_direct_backend_access() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_string_lossy().to_string();

    // Create local file system config
    let local_config = LocalFileSystemConfig {
        base_dir: temp_path.clone(),
        file_extension: "json".to_string(),
    };

    // Create backend
    let backend = LocalFileSystemBackend::new(local_config.clone());

    // Create test data
    let mut data = HashMap::new();
    let value = ValueWithMetadata {
        value: json!("direct_test"),
        metadata: kairei_core::provider::capabilities::shared_memory::Metadata::default(),
        expiry: None,
    };
    data.insert("direct_key".to_string(), value);

    // Save data directly to backend
    backend.save("direct_namespace", &data).await?;

    // Create plugin with same config
    let persistence_config = PersistenceConfig {
        backend_type: BackendType::LocalFileSystem,
        sync_interval: Duration::from_secs(1),
        auto_load: true,
        auto_save: true,
        backend_config: BackendSpecificConfig::Local(local_config),
    };

    let shared_memory_config = SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_secs(3600),
        namespace: "direct_namespace".to_string(),
    };

    let config = PersistentSharedMemoryConfig {
        base: shared_memory_config,
        persistence: persistence_config,
    };

    let plugin = PersistentSharedMemoryPlugin::new(config).await;

    // Load data from storage
    plugin.load().await?;

    // Verify data was loaded
    let retrieved = plugin.get("direct_key").await?;
    assert_eq!(retrieved, json!("direct_test"));

    Ok(())
}

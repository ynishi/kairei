//! Integration tests for the GCPStorageBackend
//!
//! Note: These tests require valid GCP credentials to be available in the environment.
//! They are skipped by default to avoid requiring GCP access for regular testing.
//! To run these tests, set the environment variable RUN_GCP_TESTS=true.

use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

use chrono::Utc;
use kairei_core::provider::capabilities::shared_memory::{Metadata, SharedMemoryCapability};
use kairei_core::provider::capabilities::storage::{StorageBackend, ValueWithMetadata};
use kairei_core::provider::config::BasePluginConfig;
use kairei_core::provider::config::plugins::{
    BackendSpecificConfig, BackendType, GCPAuthMethod, GCPStorageConfig, PersistenceConfig,
    PersistentSharedMemoryConfig, SharedMemoryConfig,
};
use kairei_core::provider::plugins::persistent_shared_memory::PersistentSharedMemoryPlugin;
use kairei_core::provider::plugins::storage::gcp::GCPStorageBackend;

// Helper function to check if GCP tests should be run
fn should_run_gcp_tests() -> bool {
    match env::var("RUN_GCP_TESTS") {
        Ok(value) => value.eq_ignore_ascii_case("true") || value == "1",
        Err(_) => false,
    }
}

// Helper function to create test config
fn create_test_config() -> PersistentSharedMemoryConfig {
    // If environment variables are set for bucket and project, use those
    // Otherwise, use default test values
    let project_id = env::var("GCP_PROJECT_ID").unwrap_or_else(|_| "test-project".to_string());
    let bucket_name = env::var("GCP_BUCKET_NAME").unwrap_or_else(|_| "test-bucket".to_string());

    PersistentSharedMemoryConfig {
        base: SharedMemoryConfig {
            base: BasePluginConfig::default(),
            max_keys: 1000,
            ttl: Duration::from_secs(3600),
            namespace: "test-gcp-persistence".to_string(),
        },
        persistence: PersistenceConfig {
            backend_type: BackendType::GCPStorage,
            sync_interval: Duration::from_secs(0), // Disable auto-sync for testing
            auto_load: false,                      // Manual load for testing
            auto_save: false,                      // Manual save for testing
            backend_config: BackendSpecificConfig::GCP(GCPStorageConfig {
                project_id,
                bucket_name,
                base_path: "test/shared-memory".to_string(),
                auth_method: GCPAuthMethod::ADC, // Use Application Default Credentials
            }),
        },
    }
}

// Helper function to create a test value with metadata
fn create_test_value(value: serde_json::Value) -> ValueWithMetadata {
    let now = Utc::now();
    ValueWithMetadata {
        value,
        metadata: Metadata {
            created_at: now,
            last_modified: now,
            content_type: "application/json".to_string(),
            size: 0, // Will be updated by the system
            tags: Default::default(),
        },
        expiry: None,
    }
}

#[tokio::test]
async fn test_gcp_backend_availability() {
    if !should_run_gcp_tests() {
        println!("Skipping GCP Storage Backend tests. Set RUN_GCP_TESTS=true to run.");
        return;
    }

    let config = create_test_config();

    if let BackendSpecificConfig::GCP(gcp_config) = &config.persistence.backend_config {
        let backend =
            GCPStorageBackend::new(gcp_config.clone()).expect("Failed to create GCP backend");

        // Check if the backend is available
        let available = backend.is_available().await;
        assert!(available, "GCP Storage Backend should be available");
    } else {
        panic!("Expected GCP config");
    }
}

#[tokio::test]
async fn test_gcp_backend_save_load() {
    if !should_run_gcp_tests() {
        println!("Skipping GCP Storage Backend tests. Set RUN_GCP_TESTS=true to run.");
        return;
    }

    let config = create_test_config();

    if let BackendSpecificConfig::GCP(gcp_config) = &config.persistence.backend_config {
        let backend =
            GCPStorageBackend::new(gcp_config.clone()).expect("Failed to create GCP backend");

        // Create test namespace
        let namespace = format!("test-namespace-{}", Utc::now().timestamp());

        // Create test data
        let mut data = HashMap::new();
        data.insert("key1".to_string(), create_test_value(json!("value1")));
        data.insert(
            "key2".to_string(),
            create_test_value(json!({"name": "test", "value": 42})),
        );

        // Save data
        backend
            .save(&namespace, &data)
            .await
            .expect("Failed to save data");

        // Load data
        let loaded_data = backend.load(&namespace).await.expect("Failed to load data");

        // Verify data
        assert_eq!(loaded_data.len(), 2, "Should have 2 keys");
        assert!(loaded_data.contains_key("key1"), "Should contain key1");
        assert!(loaded_data.contains_key("key2"), "Should contain key2");
        assert_eq!(loaded_data["key1"].value, json!("value1"));
        assert_eq!(
            loaded_data["key2"].value,
            json!({"name": "test", "value": 42})
        );

        // Clean up
        for key in data.keys() {
            backend
                .delete_key(&namespace, key)
                .await
                .expect("Failed to delete key");
        }
    } else {
        panic!("Expected GCP config");
    }
}

#[tokio::test]
async fn test_gcp_backend_save_key() {
    if !should_run_gcp_tests() {
        println!("Skipping GCP Storage Backend tests. Set RUN_GCP_TESTS=true to run.");
        return;
    }

    let config = create_test_config();

    if let BackendSpecificConfig::GCP(gcp_config) = &config.persistence.backend_config {
        let backend =
            GCPStorageBackend::new(gcp_config.clone()).expect("Failed to create GCP backend");

        // Create test namespace
        let namespace = format!("test-namespace-{}", Utc::now().timestamp());

        // Save individual keys
        let key1 = "single_key1";
        let value1 = create_test_value(json!("single_value1"));
        backend
            .save_key(&namespace, key1, &value1)
            .await
            .expect("Failed to save key1");

        let key2 = "single_key2";
        let value2 = create_test_value(json!({"name": "single_test", "value": 99}));
        backend
            .save_key(&namespace, key2, &value2)
            .await
            .expect("Failed to save key2");

        // Load data
        let loaded_data = backend.load(&namespace).await.expect("Failed to load data");

        // Verify data
        assert_eq!(loaded_data.len(), 2, "Should have 2 keys");
        assert!(loaded_data.contains_key(key1), "Should contain key1");
        assert!(loaded_data.contains_key(key2), "Should contain key2");
        assert_eq!(loaded_data[key1].value, json!("single_value1"));
        assert_eq!(
            loaded_data[key2].value,
            json!({"name": "single_test", "value": 99})
        );

        // Clean up
        backend
            .delete_key(&namespace, key1)
            .await
            .expect("Failed to delete key1");
        backend
            .delete_key(&namespace, key2)
            .await
            .expect("Failed to delete key2");
    } else {
        panic!("Expected GCP config");
    }
}

#[tokio::test]
async fn test_gcp_backend_delete_key() {
    if !should_run_gcp_tests() {
        println!("Skipping GCP Storage Backend tests. Set RUN_GCP_TESTS=true to run.");
        return;
    }

    let config = create_test_config();

    if let BackendSpecificConfig::GCP(gcp_config) = &config.persistence.backend_config {
        let backend =
            GCPStorageBackend::new(gcp_config.clone()).expect("Failed to create GCP backend");

        // Create test namespace
        let namespace = format!("test-namespace-{}", Utc::now().timestamp());

        // Create test data
        let mut data = HashMap::new();
        data.insert(
            "del_key1".to_string(),
            create_test_value(json!("del_value1")),
        );
        data.insert(
            "del_key2".to_string(),
            create_test_value(json!("del_value2")),
        );

        // Save data
        backend
            .save(&namespace, &data)
            .await
            .expect("Failed to save data");

        // Delete one key
        backend
            .delete_key(&namespace, "del_key1")
            .await
            .expect("Failed to delete key");

        // Load data
        let loaded_data = backend.load(&namespace).await.expect("Failed to load data");

        // Verify data
        assert_eq!(loaded_data.len(), 1, "Should have 1 key after deletion");
        assert!(
            !loaded_data.contains_key("del_key1"),
            "Should not contain deleted key"
        );
        assert!(
            loaded_data.contains_key("del_key2"),
            "Should contain remaining key"
        );

        // Clean up
        backend
            .delete_key(&namespace, "del_key2")
            .await
            .expect("Failed to delete key");
    } else {
        panic!("Expected GCP config");
    }
}

#[tokio::test]
async fn test_gcp_backend_with_compression() {
    if !should_run_gcp_tests() {
        println!("Skipping GCP Storage Backend tests. Set RUN_GCP_TESTS=true to run.");
        return;
    }

    let config = create_test_config();

    if let BackendSpecificConfig::GCP(gcp_config) = &config.persistence.backend_config {
        // Create backend with compression enabled
        let backend = GCPStorageBackend::new_with_compression(gcp_config.clone(), true)
            .expect("Failed to create GCP backend with compression");

        // Create test namespace
        let namespace = format!("test-namespace-compressed-{}", Utc::now().timestamp());

        // Create test data with larger values to benefit from compression
        let mut data = HashMap::new();
        let large_value = json!({
            "array": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            "nested": {
                "a": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "b": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "c": "cccccccccccccccccccccccccccccccccccccccccccccccccc",
                "d": "dddddddddddddddddddddddddddddddddddddddddddddddddd",
            },
            "repeated": "abcdefghijklmnopqrstuvwxyz".repeat(100)
        });
        data.insert(
            "compressed_key".to_string(),
            create_test_value(large_value.clone()),
        );

        // Save data
        backend
            .save(&namespace, &data)
            .await
            .expect("Failed to save compressed data");

        // Load data
        let loaded_data = backend
            .load(&namespace)
            .await
            .expect("Failed to load compressed data");

        // Verify data
        assert_eq!(loaded_data.len(), 1, "Should have 1 key");
        assert!(
            loaded_data.contains_key("compressed_key"),
            "Should contain the key"
        );
        assert_eq!(loaded_data["compressed_key"].value, large_value);

        // Clean up
        backend
            .delete_key(&namespace, "compressed_key")
            .await
            .expect("Failed to delete key");
    } else {
        panic!("Expected GCP config");
    }
}

#[tokio::test]
async fn test_persistent_shared_memory_with_gcp_backend() {
    if !should_run_gcp_tests() {
        println!("Skipping GCP Storage Backend tests. Set RUN_GCP_TESTS=true to run.");
        return;
    }

    let config = create_test_config();

    // Create plugin with GCP backend
    let plugin = PersistentSharedMemoryPlugin::new(config).await;

    // Set some values
    plugin
        .set("test_key1", json!("test_value1"))
        .await
        .expect("Failed to set key1");
    plugin
        .set("test_key2", json!({"name": "test", "value": 42}))
        .await
        .expect("Failed to set key2");

    // Save to storage
    plugin.save().await.expect("Failed to save data");

    // Clear cache (simulate restart)
    // Can't directly access private field, use a public method instead
    plugin.load().await.expect("Failed to clear cache");

    // Load from storage
    plugin.load().await.expect("Failed to load data");

    // Verify values
    let value1 = plugin.get("test_key1").await.expect("Failed to get key1");
    let value2 = plugin.get("test_key2").await.expect("Failed to get key2");

    assert_eq!(value1, json!("test_value1"));
    assert_eq!(value2, json!({"name": "test", "value": 42}));

    // Clean up
    plugin
        .delete("test_key1")
        .await
        .expect("Failed to delete key1");
    plugin
        .delete("test_key2")
        .await
        .expect("Failed to delete key2");
    plugin.save().await.expect("Failed to save after cleanup");
}

//! Error case tests for SharedMemoryCapability
//!
//! These tests verify the error handling capabilities of the
//! SharedMemoryCapability implementation.

use std::time::Duration;

use serde_json::json;

use crate::provider::capabilities::shared_memory::{MemoryError, SharedMemoryCapability};
use crate::provider::config::plugins::SharedMemoryConfig;
use crate::provider::plugins::shared_memory::InMemorySharedMemoryPlugin;
use crate::provider::types::ProviderResult;

/// Helper function to create a test plugin
fn create_test_plugin() -> InMemorySharedMemoryPlugin {
    InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100,
        ttl: Duration::from_secs(3600),
        namespace: "test".to_string(),
    })
}

#[tokio::test]
async fn test_error_cases() -> ProviderResult<()> {
    let plugin = create_test_plugin();
    
    // Test empty key
    let result = plugin.set("", json!("test")).await;
    assert!(
        matches!(result, Err(MemoryError::InvalidKey(_))),
        "Empty key should return InvalidKey error"
    );
    
    // Test key not found
    let result = plugin.get("nonexistent").await;
    assert!(
        matches!(result, Err(MemoryError::KeyNotFound(_))),
        "Nonexistent key should return KeyNotFound error"
    );
    
    // Test delete nonexistent key
    let result = plugin.delete("nonexistent").await;
    assert!(
        matches!(result, Err(MemoryError::KeyNotFound(_))),
        "Deleting nonexistent key should return KeyNotFound error"
    );
    
    // Test invalid pattern
    let result = plugin.list_keys("[invalid-pattern").await;
    assert!(
        matches!(result, Err(MemoryError::PatternError(_))),
        "Invalid pattern should return PatternError"
    );
    
    Ok(())
}

#[tokio::test]
async fn test_capacity_limits() -> ProviderResult<()> {
    // Create plugin with very limited capacity
    let limited_plugin = InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
        base: Default::default(),
        max_keys: 2,
        ttl: Duration::from_secs(3600),
        namespace: "test_limited".to_string(),
    });
    
    // Fill to capacity
    limited_plugin.set("key1", json!(1)).await.unwrap();
    limited_plugin.set("key2", json!(2)).await.unwrap();
    
    // Attempt to exceed capacity
    let result = limited_plugin.set("key3", json!(3)).await;
    assert!(
        matches!(result, Err(MemoryError::StorageError(_))),
        "Exceeding capacity should return StorageError"
    );
    
    // Delete one key to free up space
    limited_plugin.delete("key1").await.unwrap();
    
    // Now we should be able to add another key
    limited_plugin.set("key3", json!(3)).await.unwrap();
    
    // Verify we have the expected keys
    assert!(limited_plugin.exists("key2").await.unwrap());
    assert!(limited_plugin.exists("key3").await.unwrap());
    assert!(!limited_plugin.exists("key1").await.unwrap());
    
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
    unlimited_plugin.set("unlimited_key", json!("value")).await.unwrap();
    
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
    short_ttl_plugin.set("short_key", json!("value")).await.unwrap();
    
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
        matches!(result, Err(MemoryError::KeyNotFound(_))),
        "Metadata for nonexistent key should return KeyNotFound error"
    );
    
    // Test metadata after setting and updating
    plugin.set("meta_key", json!("initial")).await.unwrap();
    let initial_metadata = plugin.get_metadata("meta_key").await.unwrap();
    
    // Update value
    tokio::time::sleep(Duration::from_millis(10)).await;
    plugin.set("meta_key", json!("updated")).await.unwrap();
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
        matches!(result, Err(MemoryError::PatternError(_))),
        "Empty pattern should return PatternError"
    );
    
    // Test wildcard pattern
    let keys = plugin.list_keys("*").await.unwrap();
    assert!(!keys.is_empty(), "Wildcard pattern should return all keys");
    
    Ok(())
}

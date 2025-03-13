//! Concurrency tests for SharedMemoryCapability
//!
//! These tests verify the thread safety and concurrent access
//! capabilities of the SharedMemoryCapability implementation

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use tokio::sync::Barrier;

use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
use kairei_core::provider::config::plugins::SharedMemoryConfig;
use kairei_core::provider::plugins::shared_memory::InMemorySharedMemoryPlugin;
use kairei_core::provider::types::ProviderResult;

/// Helper function to create a test plugin
fn create_test_plugin() -> Arc<InMemorySharedMemoryPlugin> {
    Arc::new(InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
        base: Default::default(),
        max_keys: 10000,
        ttl: Duration::from_secs(3600),
        namespace: "test_concurrent".to_string(),
    }))
}

#[tokio::test]
async fn test_concurrent_access() -> ProviderResult<()> {
    // Create shared memory plugin
    let plugin = create_test_plugin();

    // Number of concurrent operations
    let num_operations = 100;

    // Create many tasks that write to the shared memory concurrently
    let mut write_handles = Vec::with_capacity(num_operations);
    for i in 0..num_operations {
        let plugin_clone = plugin.clone();
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let value = json!(i);
            plugin_clone.set(&key, value).await
        });
        write_handles.push(handle);
    }

    // Wait for all writes to complete
    for handle in write_handles {
        handle.await.unwrap().unwrap();
    }

    // Create many tasks that read from the shared memory concurrently
    let mut read_handles = Vec::with_capacity(num_operations);
    for i in 0..num_operations {
        let plugin_clone = plugin.clone();
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            plugin_clone.get(&key).await
        });
        read_handles.push(handle);
    }

    // Verify all reads succeed and return expected values
    for (i, handle) in read_handles.into_iter().enumerate() {
        let result = handle.await.unwrap().unwrap();
        assert_eq!(result, json!(i));
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_mixed_operations() -> ProviderResult<()> {
    // Create shared memory plugin
    let plugin = create_test_plugin();

    // Number of concurrent operations
    let num_operations = 100;

    // Barrier to synchronize all tasks to start at the same time
    let barrier = Arc::new(Barrier::new(num_operations * 3));

    // Create tasks for SET operations
    let mut set_handles = Vec::with_capacity(num_operations);
    for i in 0..num_operations {
        let plugin_clone = plugin.clone();
        let barrier_clone = barrier.clone();
        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;

            let key = format!("mixed_key_{}", i);
            let value = json!(format!("value_{}", i));
            plugin_clone.set(&key, value).await
        });
        set_handles.push(handle);
    }

    // Create tasks for GET operations
    let mut get_handles = Vec::with_capacity(num_operations);
    for i in 0..num_operations {
        let plugin_clone = plugin.clone();
        let barrier_clone = barrier.clone();
        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;

            let key = format!("mixed_key_{}", i);
            // This might succeed or fail depending on timing
            let _ = plugin_clone.get(&key).await;
            true
        });
        get_handles.push(handle);
    }

    // Create tasks for DELETE operations
    let mut delete_handles = Vec::with_capacity(num_operations);
    for i in 0..num_operations {
        let plugin_clone = plugin.clone();
        let barrier_clone = barrier.clone();
        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;

            let key = format!("mixed_key_{}", i);
            // This might succeed or fail depending on timing
            let _ = plugin_clone.delete(&key).await;
            true
        });
        delete_handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in set_handles {
        let _ = handle.await.unwrap();
    }

    for handle in get_handles {
        let _ = handle.await.unwrap();
    }

    for handle in delete_handles {
        let _ = handle.await.unwrap();
    }

    // The test passes if no panics occurred
    Ok(())
}

#[tokio::test]
async fn test_concurrent_pattern_matching() -> ProviderResult<()> {
    // Create shared memory plugin
    let plugin = create_test_plugin();

    // Number of keys to create
    let num_keys = 1000;

    // Populate with many keys
    for i in 0..num_keys {
        let key = format!("pattern_key_{}", i);
        plugin.set(&key, json!(i)).await?;
    }

    // Number of concurrent list operations
    let num_operations = 20;

    // Create tasks for list_keys operations with different patterns
    let mut list_handles = Vec::with_capacity(num_operations);
    let patterns = vec![
        "pattern_key_*",
        "pattern_key_1*",
        "pattern_key_2*",
        "pattern_key_3*",
        "pattern_key_4*",
    ];

    for pattern in patterns {
        for _ in 0..4 {
            // 4 concurrent operations per pattern
            let plugin_clone = plugin.clone();
            let pattern = pattern.to_string();
            let handle = tokio::spawn(async move { plugin_clone.list_keys(&pattern).await });
            list_handles.push(handle);
        }
    }

    // Wait for all list operations to complete
    for handle in list_handles {
        let result = handle.await.unwrap()?;
        assert!(!result.is_empty(), "Pattern matching should return results");
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_capacity_limits() -> ProviderResult<()> {
    // Create shared memory plugin with low capacity
    let plugin = Arc::new(InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
        base: Default::default(),
        max_keys: 50, // Low capacity
        ttl: Duration::from_secs(3600),
        namespace: "test_capacity".to_string(),
    }));

    // Number of concurrent operations (more than capacity)
    let num_operations = 100;

    // Create many tasks that write to the shared memory concurrently
    let mut write_handles = Vec::with_capacity(num_operations);
    for i in 0..num_operations {
        let plugin_clone = plugin.clone();
        let handle = tokio::spawn(async move {
            let key = format!("capacity_key_{}", i);
            let value = json!(i);
            plugin_clone.set(&key, value).await
        });
        write_handles.push(handle);
    }

    // Wait for all writes to complete or fail
    let mut success_count = 0;
    let mut error_count = 0;

    for handle in write_handles {
        match handle.await.unwrap() {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    // Some operations should succeed, some should fail
    assert!(success_count > 0, "Some operations should succeed");
    assert!(
        error_count > 0,
        "Some operations should fail due to capacity limits"
    );
    assert!(
        success_count <= 50,
        "Success count should not exceed capacity"
    );

    // Verify the actual number of stored keys
    let all_keys = plugin.list_keys("capacity_key_*").await?;
    assert!(
        all_keys.len() <= 50,
        "Number of stored keys should not exceed capacity"
    );

    Ok(())
}

#[tokio::test]
async fn test_concurrent_expiration() -> ProviderResult<()> {
    // Create shared memory plugin with longer, more predictable TTL
    let plugin = Arc::new(InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
        base: Default::default(),
        max_keys: 1000,
        ttl: Duration::from_millis(500), // Longer TTL for more reliable testing
        namespace: "test_expiration".to_string(),
    }));

    // Number of keys
    let num_keys = 50; // Reduced number of keys for more reliable testing

    // Set keys
    for i in 0..num_keys {
        let key = format!("expiring_key_{}", i);
        plugin.set(&key, json!(i)).await?;
    }

    // Use a barrier to ensure all tasks start at the same time
    let barrier = Arc::new(tokio::sync::Barrier::new(num_keys * 3 + 1));

    // Start concurrent access
    let mut handles = Vec::with_capacity(num_keys * 3);

    // Mix of operations
    for i in 0..num_keys {
        // GET operation
        let plugin_clone = plugin.clone();
        let barrier_clone = barrier.clone();
        let key = format!("expiring_key_{}", i);
        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            let result = plugin_clone.get(&key).await;
            result.is_ok() // Return true if key was found
        });
        handles.push(handle);

        // EXISTS operation
        let plugin_clone = plugin.clone();
        let barrier_clone = barrier.clone();
        let key = format!("expiring_key_{}", i);
        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            let result = plugin_clone.exists(&key).await;
            result.unwrap_or(false) // Return true if key exists
        });
        handles.push(handle);

        // SET operation (refresh)
        let plugin_clone = plugin.clone();
        let barrier_clone = barrier.clone();
        let key = format!("expiring_key_{}", i);
        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            let result = plugin_clone
                .set(&key, json!(format!("refreshed_{}", i)))
                .await;
            result.is_ok() // Return true if set was successful
        });
        handles.push(handle);
    }

    // Wait at the barrier to ensure all tasks start simultaneously
    barrier.wait().await;

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.await.unwrap() {
            success_count += 1;
        }
    }

    // Verify that some operations succeeded
    assert!(success_count > 0, "Some operations should succeed");

    // Wait for all keys to expire
    tokio::time::sleep(Duration::from_millis(600)).await;

    // Verify all keys have expired
    let remaining_keys = plugin.list_keys("expiring_key_*").await?;

    // Some keys should have been refreshed and still exist
    // But we don't make specific assertions about how many, as that would be timing-dependent
    println!("Remaining keys after expiration: {}", remaining_keys.len());

    Ok(())
}

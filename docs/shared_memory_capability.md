# Shared Memory Capability

The SharedMemoryCapability allows different agents and providers to share data and state in a thread-safe, high-performance way. This enables sophisticated multi-agent communication patterns and stateful interactions without requiring direct message passing.

## Key Features

- Thread-safe data sharing across providers and agents
- Fast key-value operations (sub-millisecond performance)
- Support for JSON data structures
- TTL-based automatic expiration
- Pattern-based key listing
- Rich metadata for stored values
- Namespace isolation for multi-tenant applications

## Configuration

To enable shared memory in your application, add the shared memory plugin configuration to your provider configuration.

### Configuration Options

| Option | Description | Default | Example |
|--------|-------------|---------|---------|
| `max_keys` | Maximum number of keys allowed in the store (0 = unlimited) | 10000 | `5000` |
| `ttl` | Time-to-live for entries in milliseconds (0 = never expire) | 3600000 (1 hour) | `7200000` (2 hours) |
| `namespace` | Namespace prefix for isolating keys | "shared" | `"my_application"` |

### Configuration Example

```rust
use kairei_core::provider::config::plugins::SharedMemoryConfig;
use std::time::Duration;

// Create a shared memory configuration
let shared_memory_config = SharedMemoryConfig {
    base: Default::default(),
    max_keys: 5000,                      // Limit to 5000 keys
    ttl: Duration::from_secs(7200),      // 2 hour expiration
    namespace: "my_application".to_string(),
};

// Add to provider configuration
let mut plugin_configs = std::collections::HashMap::new();
plugin_configs.insert(
    "shared_memory".to_string(),
    PluginConfig::SharedMemory(shared_memory_config),
);

let provider_config = ProviderConfig {
    name: "my_provider".to_string(),
    provider_type: ProviderType::OpenAIChat,
    plugin_configs,
    ..Default::default()
};
```

## Basic Usage

### Getting a Shared Memory Plugin

```rust
// From a provider registry
let shared_memory = provider_registry.get_or_create_shared_memory_plugin(&shared_memory_config);

// Or create directly (for testing)
let shared_memory = InMemorySharedMemoryPlugin::new(shared_memory_config);
```

### Key-Value Operations

```rust
// Store a value
shared_memory.set("user_123", json!({"name": "Alice", "role": "admin"})).await?;

// Retrieve a value
let user_data = shared_memory.get("user_123").await?;
println!("User name: {}", user_data["name"]);

// Check if a key exists
if shared_memory.exists("user_123").await? {
    println!("User exists!");
}

// Delete a value
shared_memory.delete("user_123").await?;
```

### Working with Metadata

```rust
// Store a value
shared_memory.set("document_456", json!({"title": "Report", "content": "..."})).await?;

// Get metadata about the value
let metadata = shared_memory.get_metadata("document_456").await?;
println!("Document size: {} bytes", metadata.size);
println!("Created at: {}", metadata.created_at);
println!("Last modified: {}", metadata.last_modified);
```

### Pattern Matching

```rust
// Store multiple values with a common prefix
shared_memory.set("user_123", json!({"name": "Alice"})).await?;
shared_memory.set("user_456", json!({"name": "Bob"})).await?;
shared_memory.set("user_789", json!({"name": "Charlie"})).await?;

// List all user keys
let user_keys = shared_memory.list_keys("user_*").await?;
for key in user_keys {
    let user = shared_memory.get(&key).await?;
    println!("User: {}", user["name"]);
}
```

## Advanced Usage Patterns

### Namespace Isolation

```rust
// Create two shared memory plugins with different namespaces
let config1 = SharedMemoryConfig {
    namespace: "app1".to_string(),
    ..Default::default()
};

let config2 = SharedMemoryConfig {
    namespace: "app2".to_string(),
    ..Default::default()
};

let shared_memory1 = provider_registry.get_or_create_shared_memory_plugin(&config1);
let shared_memory2 = provider_registry.get_or_create_shared_memory_plugin(&config2);

// Store values with the same key in different namespaces
shared_memory1.set("settings", json!({"theme": "dark"})).await?;
shared_memory2.set("settings", json!({"theme": "light"})).await?;

// Values are isolated by namespace
assert_eq!(shared_memory1.get("settings").await?["theme"], "dark");
assert_eq!(shared_memory2.get("settings").await?["theme"], "light");
```

### Caching Pattern

```rust
// Function that uses shared memory as a cache
async fn get_user_data(user_id: &str, shared_memory: &impl SharedMemoryCapability) -> Result<Value, Error> {
    let cache_key = format!("user_{}", user_id);
    
    // Try to get from cache first
    if shared_memory.exists(&cache_key).await? {
        return Ok(shared_memory.get(&cache_key).await?);
    }
    
    // Not in cache, fetch from database
    let user_data = fetch_user_from_database(user_id).await?;
    
    // Store in cache with TTL
    shared_memory.set(&cache_key, user_data.clone()).await?;
    
    Ok(user_data)
}
```

### Session State Pattern

```rust
// Store session data
async fn store_session(session_id: &str, data: Value, shared_memory: &impl SharedMemoryCapability) -> Result<(), Error> {
    let key = format!("session_{}", session_id);
    shared_memory.set(&key, data).await?;
    Ok(())
}

// Retrieve session data
async fn get_session(session_id: &str, shared_memory: &impl SharedMemoryCapability) -> Result<Value, Error> {
    let key = format!("session_{}", session_id);
    
    if !shared_memory.exists(&key).await? {
        return Err(Error::SessionNotFound);
    }
    
    Ok(shared_memory.get(&key).await?)
}
```

### Multi-Provider Communication

```rust
// Provider 1 stores data
let shared_memory = registry.get_or_create_shared_memory_plugin(&shared_memory_config);
shared_memory.set("shared_key", json!({"source": "provider1", "data": "..."})).await?;

// Provider 2 retrieves data
let shared_memory = registry.get_or_create_shared_memory_plugin(&shared_memory_config);
let data = shared_memory.get("shared_key").await?;
```

## Best Practices

### Memory Management

1. **Set appropriate TTL values**: Use shorter TTLs for temporary data and longer TTLs for more persistent data.
2. **Clean up unused keys**: Explicitly delete keys when they're no longer needed.
3. **Use namespaces effectively**: Organize data with meaningful namespace names.
4. **Set reasonable max_keys limits**: Prevent unbounded memory growth.

### Concurrency

1. **Handle errors gracefully**: Operations may fail due to concurrent modifications.
2. **Use exists() before get()**: Check if a key exists before attempting to retrieve it.
3. **Implement retry logic**: For critical operations that might fail due to timing issues.

### Data Organization

1. **Use consistent key naming conventions**: Establish patterns like `{type}_{id}` (e.g., `user_123`).
2. **Leverage pattern matching**: Design keys to work well with pattern-based queries.
3. **Store structured data**: Use JSON objects with consistent schemas.

## Error Handling

The SharedMemoryCapability defines several error types:

- `KeyNotFound`: The requested key doesn't exist
- `InvalidKey`: The key format is invalid
- `InvalidValue`: The value couldn't be processed
- `StorageError`: General storage errors (e.g., capacity exceeded)
- `AccessDenied`: Permission issues
- `PatternError`: Invalid pattern for list_keys

Example error handling:

```rust
match shared_memory.get("user_123").await {
    Ok(value) => {
        // Process the value
        println!("User: {}", value["name"]);
    },
    Err(SharedMemoryError::KeyNotFound(_)) => {
        // Handle missing key
        println!("User not found");
    },
    Err(e) => {
        // Handle other errors
        eprintln!("Error retrieving user: {}", e);
    }
}
```

## Troubleshooting

### Common Issues

1. **Key not found**: Ensure the key exists and hasn't expired.
   ```rust
   // Check if key exists before attempting to get it
   if shared_memory.exists("my_key").await? {
       let value = shared_memory.get("my_key").await?;
   }
   ```

2. **Capacity exceeded**: Check the `max_keys` configuration.
   ```rust
   // Increase max_keys in configuration
   let config = SharedMemoryConfig {
       max_keys: 20000,  // Increased from default 10000
       ..Default::default()
   };
   ```

3. **Invalid pattern**: Verify the pattern syntax for list_keys.
   ```rust
   // Valid pattern examples
   shared_memory.list_keys("user_*").await?;     // All keys starting with "user_"
   shared_memory.list_keys("user_?").await?;     // Keys like "user_1", "user_a", etc.
   shared_memory.list_keys("user_[0-9]").await?; // Keys like "user_0" through "user_9"
   ```

4. **Unexpected data**: Check for concurrent modifications.
   ```rust
   // Get metadata to check last modification time
   let metadata = shared_memory.get_metadata("my_key").await?;
   println!("Last modified: {}", metadata.last_modified);
   ```

### Debugging Tips

1. **Check key existence**: Use `exists()` to verify keys.
2. **Inspect metadata**: Use `get_metadata()` to check creation and modification times.
3. **List available keys**: Use `list_keys("*")` to see all keys in a namespace.
4. **Verify namespace**: Ensure you're using the correct namespace configuration.

## Performance Considerations

1. **Key count impact**: Performance may degrade with very large numbers of keys.
   - Consider using multiple namespaces to partition data
   - Implement periodic cleanup of unused keys

2. **Value size impact**: Large values consume more memory and take longer to serialize/deserialize.
   - Store references or IDs instead of large objects when possible
   - Consider compressing large values

3. **Pattern matching overhead**: Complex patterns or large key sets may impact list_keys performance.
   - Use specific patterns rather than broad ones
   - Limit the frequency of pattern matching operations

4. **Namespace isolation**: Using separate namespaces can improve performance by reducing the number of keys in each namespace.
   - Design namespaces around logical boundaries in your application
   - Consider one namespace per agent or component

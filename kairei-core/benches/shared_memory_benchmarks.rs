//! Performance benchmarks for SharedMemoryCapability
//!
//! These benchmarks verify that the SharedMemoryCapability implementation
//! meets the performance requirements.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;
use std::time::Duration;

use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
use kairei_core::provider::config::plugins::SharedMemoryConfig;
use kairei_core::provider::plugins::memory::shared_memory::InMemorySharedMemoryPlugin;

/// Helper function to create a test plugin
fn create_test_plugin() -> InMemorySharedMemoryPlugin {
    InMemorySharedMemoryPlugin::new(SharedMemoryConfig {
        base: Default::default(),
        max_keys: 100000,
        ttl: Duration::from_secs(3600),
        namespace: "benchmark".to_string(),
    })
}

/// Benchmark SET operations
fn bench_set(c: &mut Criterion) {
    let plugin = create_test_plugin();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("shared_memory_set");

    // Benchmark with different payload sizes
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let value = json!(vec!["x"; size]);
            let mut i = 0;

            b.iter(|| {
                let key = format!("bench_set_key_{}", i);
                i += 1;
                rt.block_on(async {
                    plugin.set(&key, black_box(value.clone())).await.unwrap();
                });
            });
        });
    }

    group.finish();
}

/// Benchmark GET operations
fn bench_get(c: &mut Criterion) {
    let plugin = create_test_plugin();
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: Create keys with different payload sizes
    for size in [10, 100, 1000, 10000].iter() {
        let value = json!(vec!["x"; *size]);
        for i in 0..1000 {
            let key = format!("bench_get_key_{}_{}", size, i);
            rt.block_on(async {
                plugin.set(&key, value.clone()).await.unwrap();
            });
        }
    }

    let mut group = c.benchmark_group("shared_memory_get");

    // Benchmark with different payload sizes
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut i = 0;

            b.iter(|| {
                let key = format!("bench_get_key_{}_{}", size, i % 1000);
                i += 1;
                rt.block_on(async {
                    black_box(plugin.get(&key).await.unwrap());
                });
            });
        });
    }

    group.finish();
}

/// Benchmark pattern matching operations
fn bench_pattern_matching(c: &mut Criterion) {
    let plugin = create_test_plugin();
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: Create keys with different prefixes
    for prefix in ["user", "admin", "system", "temp"].iter() {
        for i in 0..1000 {
            let key = format!("{}_{}", prefix, i);
            rt.block_on(async {
                plugin.set(&key, json!(i)).await.unwrap();
            });
        }
    }

    let mut group = c.benchmark_group("shared_memory_pattern");

    // Benchmark with different patterns
    let patterns = [
        "user_*", "admin_*", "system_*", "temp_*", "*_1*", "*_2*", "*_*",
    ];

    for pattern in patterns.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(pattern),
            pattern,
            |b, &pattern| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(plugin.list_keys(pattern).await.unwrap());
                    });
                });
            },
        );
    }

    group.finish();
}

/// Manual performance test to verify sub-millisecond requirements
#[tokio::test]
async fn test_performance() {
    use std::time::Instant;

    // Create plugin
    let plugin = create_test_plugin();

    // Number of operations
    let num_operations = 10000;

    // SET performance
    let start = Instant::now();
    for i in 0..num_operations {
        let key = format!("perf_key_{}", i);
        plugin.set(&key, json!(i)).await.unwrap();
    }
    let set_duration = start.elapsed();
    let set_avg_ns = set_duration.as_nanos() / num_operations as u128;
    println!(
        "SET: {} operations in {:?} (avg: {}ns per op)",
        num_operations, set_duration, set_avg_ns
    );

    // GET performance
    let start = Instant::now();
    for i in 0..num_operations {
        let key = format!("perf_key_{}", i);
        let _ = plugin.get(&key).await.unwrap();
    }
    let get_duration = start.elapsed();
    let get_avg_ns = get_duration.as_nanos() / num_operations as u128;
    println!(
        "GET: {} operations in {:?} (avg: {}ns per op)",
        num_operations, get_duration, get_avg_ns
    );

    // Performance requirement: Operations should take less than 1ms (1,000,000ns) each
    assert!(
        set_avg_ns < 1_000_000,
        "SET performance too slow: {}ns",
        set_avg_ns
    );
    assert!(
        get_avg_ns < 1_000_000,
        "GET performance too slow: {}ns",
        get_avg_ns
    );
}

criterion_group!(benches, bench_set, bench_get, bench_pattern_matching);
criterion_main!(benches);

//! Benchmarks for resource management components

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use std::collections::HashMap;
use bluefelt_core::lobby::{
    connection_manager::{ConnectionManager, ConnectionInfo},
    memory_manager::{MemoryManager, MemoryConfig},
    lock_optimization::{LockOptimizer, LockConfig, BatchedUpdater},
};

#[tokio::test]
async fn benchmark_connection_manager() {
    println!("\n=== Connection Manager Benchmark ===");
    
    let (manager, mut cleanup_rx) = ConnectionManager::new();
    let manager = Arc::new(manager);
    
    // Start cleanup task
    let connections = manager.connections.clone();
    let lobbies = manager.lobbies.clone();
    tokio::spawn(ConnectionManager::start_cleanup_task(
        connections,
        lobbies,
        cleanup_rx,
    ));
    
    // Benchmark connection registration
    let start = Instant::now();
    let iterations = 10000;
    
    for i in 0..iterations {
        manager.register_connection(
            format!("user-{}", i),
            format!("lobby-{}", i % 100),
            Arc::new(Default::default()), // Mock lobby
            i % 2 == 0, // Half are members
        );
    }
    
    let elapsed = start.elapsed();
    println!("Registered {} connections in {:.2}ms ({:.0} ops/sec)",
        iterations,
        elapsed.as_millis(),
        iterations as f64 / elapsed.as_secs_f64()
    );
    
    // Benchmark activity updates
    let start = Instant::now();
    for i in 0..iterations {
        manager.update_activity(&format!("user-{}", i));
    }
    let elapsed = start.elapsed();
    println!("Updated {} activities in {:.2}ms ({:.0} ops/sec)",
        iterations,
        elapsed.as_millis(),
        iterations as f64 / elapsed.as_secs_f64()
    );
    
    // Benchmark stats retrieval
    let start = Instant::now();
    for _ in 0..1000 {
        let _stats = manager.get_stats();
    }
    let elapsed = start.elapsed();
    println!("Retrieved stats 1000 times in {:.2}ms ({:.0} ops/sec)",
        elapsed.as_millis(),
        1000.0 / elapsed.as_secs_f64()
    );
    
    // Benchmark disconnections
    let start = Instant::now();
    for i in 0..iterations {
        manager.disconnect(&format!("user-{}", i), &format!("lobby-{}", i % 100));
    }
    let elapsed = start.elapsed();
    println!("Disconnected {} users in {:.2}ms ({:.0} ops/sec)",
        iterations,
        elapsed.as_millis(),
        iterations as f64 / elapsed.as_secs_f64()
    );
}

#[tokio::test]
async fn benchmark_memory_manager() {
    println!("\n=== Memory Manager Benchmark ===");
    
    let config = MemoryConfig {
        max_memory_mb: 1024,
        chat_history_limit: 1000,
        action_log_limit: 10000,
        snapshot_interval: Duration::from_secs(60),
        old_lobby_threshold: Duration::from_hours(2),
    };
    
    let manager = Arc::new(MemoryManager::new(config));
    
    // Benchmark bounded vec operations
    let start = Instant::now();
    let iterations = 100000;
    
    let bounded_vec = manager.create_bounded_vec::<String>(1000);
    for i in 0..iterations {
        bounded_vec.push(format!("item-{}", i));
    }
    
    let elapsed = start.elapsed();
    println!("Pushed {} items to bounded vec in {:.2}ms ({:.0} ops/sec)",
        iterations,
        elapsed.as_millis(),
        iterations as f64 / elapsed.as_secs_f64()
    );
    
    // Benchmark bounded hashmap operations
    let bounded_map = manager.create_bounded_hashmap::<String, String>(1000);
    let start = Instant::now();
    
    for i in 0..iterations {
        bounded_map.insert(format!("key-{}", i), format!("value-{}", i));
    }
    
    let elapsed = start.elapsed();
    println!("Inserted {} items to bounded map in {:.2}ms ({:.0} ops/sec)",
        iterations,
        elapsed.as_millis(),
        iterations as f64 / elapsed.as_secs_f64()
    );
    
    // Benchmark memory checks
    let start = Instant::now();
    for _ in 0..1000 {
        let _should_gc = manager.should_garbage_collect();
    }
    let elapsed = start.elapsed();
    println!("Performed 1000 GC checks in {:.2}ms ({:.0} ops/sec)",
        elapsed.as_millis(),
        1000.0 / elapsed.as_secs_f64()
    );
}

#[tokio::test]
async fn benchmark_lock_optimizer() {
    println!("\n=== Lock Optimizer Benchmark ===");
    
    let config = LockConfig {
        max_read_time: Duration::from_millis(10),
        max_write_time: Duration::from_millis(50),
        batch_size: 100,
        batch_timeout: Duration::from_millis(10),
        cache_ttl: Duration::from_secs(5),
    };
    
    let optimizer = Arc::new(LockOptimizer::new(config));
    
    // Test data
    let test_data = Arc::new(RwLock::new(HashMap::<String, String>::new()));
    for i in 0..1000 {
        test_data.write().insert(format!("key-{}", i), format!("value-{}", i));
    }
    
    // Benchmark optimized reads
    let start = Instant::now();
    let iterations = 10000;
    
    for i in 0..iterations {
        if let Ok(guard) = optimizer.try_read_with_timeout(&test_data, Duration::from_millis(5)) {
            let _ = guard.get(&format!("key-{}", i % 1000));
        }
    }
    
    let elapsed = start.elapsed();
    println!("Performed {} optimized reads in {:.2}ms ({:.0} ops/sec)",
        iterations,
        elapsed.as_millis(),
        iterations as f64 / elapsed.as_secs_f64()
    );
    
    // Benchmark batched updates
    let updater = BatchedUpdater::new(100, Duration::from_millis(10));
    let start = Instant::now();
    
    for i in 0..iterations {
        updater.add_update(Box::new(move |data: &mut HashMap<String, String>| {
            data.insert(format!("batch-{}", i), format!("value-{}", i));
        }));
    }
    
    // Apply batched updates
    updater.apply_to(&test_data);
    
    let elapsed = start.elapsed();
    println!("Batched {} updates in {:.2}ms ({:.0} ops/sec)",
        iterations,
        elapsed.as_millis(),
        iterations as f64 / elapsed.as_secs_f64()
    );
    
    // Benchmark cached reads
    let cache = optimizer.create_read_cache();
    let start = Instant::now();
    
    // Warm up cache
    for i in 0..100 {
        cache.get_or_compute(
            format!("cached-{}", i),
            || test_data.read().get(&format!("key-{}", i)).cloned().unwrap_or_default()
        );
    }
    
    // Benchmark cache hits
    for _ in 0..iterations {
        for i in 0..100 {
            cache.get_or_compute(
                format!("cached-{}", i),
                || panic!("Should hit cache")
            );
        }
    }
    
    let elapsed = start.elapsed();
    let total_reads = iterations * 100;
    println!("Performed {} cached reads in {:.2}ms ({:.0} ops/sec)",
        total_reads,
        elapsed.as_millis(),
        total_reads as f64 / elapsed.as_secs_f64()
    );
}

#[tokio::test]
async fn stress_test_connection_pool() {
    println!("\n=== Connection Pool Stress Test ===");
    
    let (manager, _) = ConnectionManager::new();
    let pool = &manager.pool;
    
    // Test per-user limits
    let mut guards = Vec::new();
    for i in 0..5 {
        match pool.try_acquire("test-user") {
            Ok(guard) => {
                guards.push(guard);
                println!("Connection {} for test-user: OK", i + 1);
            }
            Err(e) => {
                println!("Connection {} for test-user: {}", i + 1, e);
            }
        }
    }
    
    // Drop connections
    guards.clear();
    
    // Test global limits
    let start = Instant::now();
    let mut successful = 0;
    let mut failed = 0;
    
    for i in 0..120 {
        match pool.try_acquire(&format!("user-{}", i)) {
            Ok(_) => successful += 1,
            Err(_) => failed += 1,
        }
    }
    
    let elapsed = start.elapsed();
    println!("\nGlobal limit test completed in {:.2}ms", elapsed.as_millis());
    println!("Successful: {}, Failed: {} (limit is 100)", successful, failed);
}

#[tokio::test]
async fn benchmark_concurrent_access() {
    println!("\n=== Concurrent Access Benchmark ===");
    
    let (manager, _) = ConnectionManager::new();
    let manager = Arc::new(manager);
    
    // Spawn multiple tasks accessing the connection manager
    let start = Instant::now();
    let tasks = 100;
    let ops_per_task = 1000;
    
    let mut handles = Vec::new();
    for task_id in 0..tasks {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            for i in 0..ops_per_task {
                mgr.update_activity(&format!("user-{}-{}", task_id, i));
                if i % 10 == 0 {
                    let _stats = mgr.get_stats();
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    
    let elapsed = start.elapsed();
    let total_ops = tasks * ops_per_task;
    println!("Performed {} concurrent operations in {:.2}ms ({:.0} ops/sec)",
        total_ops,
        elapsed.as_millis(),
        total_ops as f64 / elapsed.as_secs_f64()
    );
}
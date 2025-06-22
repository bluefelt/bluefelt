//! Simple tests for resource management components

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use parking_lot::RwLock;
use tokio::time::sleep;
use bluefelt_core::lobby::{
    connection_manager::{ConnectionManager, HealthMonitor},
    memory_manager::{MemoryManager, MemoryConfig},
    lock_optimization::{LockOptimizer, LockConfig, BatchedUpdater},
};

#[tokio::test]
async fn test_connection_manager_basic() {
    let (manager, _cleanup_rx) = ConnectionManager::new();
    
    // Test connection pool
    let pool = &manager.pool;
    
    // Should allow 3 connections per user
    let _g1 = pool.try_acquire("user1").expect("First connection");
    let _g2 = pool.try_acquire("user1").expect("Second connection");
    let _g3 = pool.try_acquire("user1").expect("Third connection");
    
    // Fourth should fail
    assert!(pool.try_acquire("user1").is_err());
    
    // Different user should work
    assert!(pool.try_acquire("user2").is_ok());
}

#[test]
fn test_memory_manager_bounded_collections() {
    let config = MemoryConfig::default();
    let manager = MemoryManager::new(config);
    
    // Create bounded vec
    let mut vec = bluefelt_core::lobby::memory_manager::BoundedVec::new(5);
    vec.push("item1".to_string());
    vec.push("item2".to_string());
    vec.push("item3".to_string());
    vec.push("item4".to_string());
    vec.push("item5".to_string());
    vec.push("item6".to_string()); // Should evict first
    
    let items: Vec<_> = vec.iter().cloned().collect();
    assert_eq!(items.len(), 5);
    assert_eq!(items[0], "item2");
    assert_eq!(items[4], "item6");
}

#[test]
fn test_lock_optimizer_timeout() {
    let config = LockConfig {
        max_wait_time: Duration::from_millis(10),
        prefer_try_lock: true,
        backoff_ms: 1,
    };
    let optimizer = LockOptimizer::new(config);
    
    let data = RwLock::new(42);
    
    // Hold write lock in another scope
    let _guard = data.write();
    
    // Try read should timeout
    let start = Instant::now();
    let result = optimizer.try_read_with_timeout(&data);
    let elapsed = start.elapsed();
    
    assert!(result.is_none());
    assert!(elapsed >= Duration::from_millis(10));
    assert!(elapsed < Duration::from_millis(50)); // Should not wait too long
}

#[test]
fn test_health_monitor() {
    let monitor = HealthMonitor::new();
    
    // Initially healthy
    match monitor.check_health() {
        bluefelt_core::lobby::connection_manager::HealthStatus::Healthy => {}
        _ => panic!("Should be healthy initially"),
    }
    
    // The health monitor calculates rate per minute. If we record failures
    // immediately, the rate will be very high. Let's just test that it
    // detects issues when there are many failures.
    
    // Record many failures quickly (should trigger alert)
    for _ in 0..200 {
        monitor.record_failed_send();
    }
    
    // Should now be critical
    match monitor.check_health() {
        bluefelt_core::lobby::connection_manager::HealthStatus::Critical(_) => {}
        status => panic!("Should be critical with many failures, got: {:?}", status),
    }
}

#[tokio::test]
async fn test_batched_updater() {
    let data = Arc::new(RwLock::new(Vec::<String>::new()));
    let config = bluefelt_core::lobby::lock_optimization::BatchConfig {
        max_batch_size: 3,
        max_batch_age: Duration::from_millis(100),
    };
    
    let updater = Arc::new(BatchedUpdater::new(data.clone(), config));
    
    // Start flush task
    BatchedUpdater::start_flush_task(updater.clone());
    
    // Add updates
    updater.update(|vec| vec.push("item1".to_string()));
    updater.update(|vec| vec.push("item2".to_string()));
    updater.update(|vec| vec.push("item3".to_string()));
    
    // Should flush immediately at batch size
    sleep(Duration::from_millis(50)).await;
    assert_eq!(data.read().len(), 3);
    
    // Add more
    updater.update(|vec| vec.push("item4".to_string()));
    updater.update(|vec| vec.push("item5".to_string()));
    
    // Wait for time-based flush
    sleep(Duration::from_millis(150)).await;
    assert_eq!(data.read().len(), 5);
}

#[test]
fn test_memory_config_defaults() {
    let config = MemoryConfig::default();
    assert_eq!(config.max_tables_per_lobby, 100);
    assert_eq!(config.max_history_entries, 1000);
    assert_eq!(config.max_chat_messages, 500);
    assert_eq!(config.table_idle_timeout, Duration::from_secs(3600));
}

#[test]
fn test_lock_config_defaults() {
    let config = LockConfig::default();
    assert_eq!(config.max_wait_time, Duration::from_millis(100));
    assert!(config.prefer_try_lock);
    assert_eq!(config.backoff_ms, 1);
}

#[tokio::test]
async fn test_connection_pool_cleanup() {
    let (manager, _) = ConnectionManager::new();
    let pool = &manager.pool;
    
    // Acquire and release
    {
        let _g1 = pool.try_acquire("temp_user").expect("Should acquire");
        let _g2 = pool.try_acquire("temp_user").expect("Should acquire");
    } // Guards dropped here
    
    // Should be able to acquire again
    let _g3 = pool.try_acquire("temp_user").expect("Should acquire after cleanup");
    let _g4 = pool.try_acquire("temp_user").expect("Should acquire after cleanup");
}

#[test]
fn test_lock_optimizer_metrics() {
    let config = LockConfig::default();
    let optimizer = LockOptimizer::new(config);
    
    let data = RwLock::new(HashMap::<String, i32>::new());
    
    // Successful acquisitions
    for i in 0..10 {
        if let Some(mut guard) = optimizer.try_write_with_timeout::<HashMap<String, i32>>(&data) {
            guard.insert(format!("key{}", i), i);
        }
    }
    
    // Check metrics
    let metrics = optimizer.get_metrics();
    assert_eq!(metrics.total_acquisitions, 10);
    assert_eq!(metrics.failed_acquisitions, 0);
}
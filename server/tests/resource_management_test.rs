//! Tests for server resource management components

use bluefelt_core::lobby::{
    connection_manager::{ConnectionManager, ConnectionPool, HealthMonitor},
    memory_manager::{MemoryManager, MemoryConfig, BoundedVec, BoundedHashMap},
    lock_optimization::{LockOptimizer, LockConfig, BatchedUpdater, ReadCache},
};
use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;

#[tokio::test]
async fn test_connection_pool() {
    let pool = ConnectionPool::new(3, 10);
    
    // Test per-user limit
    let user = "test_user";
    let _guard1 = pool.try_acquire(user).expect("First connection should succeed");
    let _guard2 = pool.try_acquire(user).expect("Second connection should succeed");
    let _guard3 = pool.try_acquire(user).expect("Third connection should succeed");
    
    // Fourth connection should fail
    assert!(pool.try_acquire(user).is_err());
    
    // Different user should succeed
    let _guard4 = pool.try_acquire("other_user").expect("Different user should succeed");
    
    // Drop a guard and try again
    drop(_guard1);
    let _guard5 = pool.try_acquire(user).expect("Should succeed after dropping guard");
}

#[tokio::test]
async fn test_connection_manager() {
    use bluefelt_core::bundle::{Bundle, Manifest};
    use bluefelt_core::lobby::lobby_impl::Lobby;
    use bluefelt_core::lobby::lobby_state::LobbyState;
    use serde_json::json;
    
    // Create a minimal bundle for testing
    let manifest = Manifest {
        game_id: "test-game".to_string(),
        version: "1.0".to_string(),
        spec_version: 1,
        metadata: serde_json::from_value(json!({
            "name": "Test Game",
            "players": {"min": 2, "max": 2}
        })).unwrap(),
    };
    
    let bundle = Arc::new(Bundle {
        id: "test-game".to_string(),
        version: "1.0".to_string(),
        manifest,
        entities: json!({}),
        zones: json!({}),
        actions: json!([]),
        phases: json!({}),
    });
    
    // Create lobby state
    let lobby_state = Arc::new(LobbyState::new(
        "test-lobby".to_string(),
        Some("Test Lobby".to_string()),
        None,
    ));
    
    // Create minimal lobby
    let lobby = Arc::new(Lobby {
        state: lobby_state,
        bundles: Arc::new(bluefelt_core::bundle::BundleMap::new()),
        clients: RwLock::new(std::collections::HashMap::new()),
        rng: Arc::new(parking_lot::Mutex::new(rand::thread_rng())),
        lobby_map: Arc::new(bluefelt_core::lobby::LobbyMap::new()),
    });
    
    let (manager, mut cleanup_rx) = ConnectionManager::new();
    
    // Start cleanup task
    let connections = Arc::clone(&manager.connections);
    let lobbies = Arc::clone(&manager.lobbies);
    tokio::spawn(ConnectionManager::start_cleanup_task(connections, lobbies, cleanup_rx));
    
    // Register connections
    manager.register_connection("user1".to_string(), "lobby1".to_string(), Arc::clone(&lobby), true);
    manager.register_connection("user2".to_string(), "lobby1".to_string(), Arc::clone(&lobby), false);
    
    // Check stats
    let stats = manager.get_stats();
    assert_eq!(stats.total_connections, 2);
    assert_eq!(stats.member_connections, 1);
    assert_eq!(stats.observer_connections, 1);
    
    // Disconnect a user
    manager.disconnect("user1", "lobby1");
    
    // Give cleanup task time to process
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check stats after disconnect
    let stats = manager.get_stats();
    assert_eq!(stats.total_connections, 1);
}

#[test]
fn test_bounded_vec() {
    let mut vec = BoundedVec::new(3);
    
    vec.push(1);
    vec.push(2);
    vec.push(3);
    assert_eq!(vec.len(), 3);
    
    // Adding fourth item should evict first
    vec.push(4);
    assert_eq!(vec.len(), 3);
    
    let items: Vec<_> = vec.iter().copied().collect();
    assert_eq!(items, vec![2, 3, 4]);
}

#[test]
fn test_bounded_hashmap() {
    let mut map = BoundedHashMap::new(3);
    
    map.insert("a", 1);
    map.insert("b", 2);
    map.insert("c", 3);
    assert_eq!(map.len(), 3);
    
    // Adding fourth item should evict oldest
    map.insert("d", 4);
    assert_eq!(map.len(), 3);
    assert!(map.get(&"a").is_none());
    assert_eq!(map.get(&"d"), Some(&4));
}

#[test]
fn test_memory_manager() {
    use bluefelt_core::lobby::lobby_state::LobbyState;
    
    let config = MemoryConfig {
        max_tables_per_lobby: 5,
        max_chat_messages: 10,
        ..Default::default()
    };
    
    let manager = MemoryManager::new(config);
    let lobby = LobbyState::new("test".to_string(), None, None);
    
    // Test table limit check
    assert!(!manager.should_limit_tables(&lobby));
    
    // Add some tables
    for i in 0..5 {
        let table = create_test_table(&format!("table{}", i));
        lobby.tables.lock().insert(format!("table{}", i), table);
    }
    
    assert!(manager.should_limit_tables(&lobby));
}

#[test]
fn test_lock_optimizer() {
    let config = LockConfig {
        max_wait_time: Duration::from_millis(10),
        prefer_try_lock: true,
        backoff_ms: 1,
    };
    
    let optimizer = LockOptimizer::new(config);
    let data = RwLock::new(42);
    
    // Test successful acquisition
    let guard = optimizer.try_read_with_timeout(&data);
    assert!(guard.is_some());
    assert_eq!(*guard.unwrap(), 42);
    
    // Test write acquisition
    let mut guard = optimizer.try_write_with_timeout(&data);
    assert!(guard.is_some());
    *guard.as_mut().unwrap() = 100;
    drop(guard);
    
    // Verify write
    let guard = optimizer.try_read_with_timeout(&data);
    assert_eq!(*guard.unwrap(), 100);
    
    // Check metrics
    let metrics = optimizer.get_metrics();
    assert_eq!(metrics.total_acquisitions, 3);
    assert_eq!(metrics.failed_acquisitions, 0);
}

#[tokio::test]
async fn test_batched_updater() {
    let data = Arc::new(RwLock::new(Vec::<i32>::new()));
    let config = bluefelt_core::lobby::lock_optimization::BatchConfig {
        max_batch_size: 3,
        max_batch_age: Duration::from_millis(100),
    };
    
    let updater = Arc::new(BatchedUpdater::new(Arc::clone(&data), config));
    
    // Start flush task
    BatchedUpdater::start_flush_task(Arc::clone(&updater));
    
    // Add updates
    for i in 0..5 {
        let i = i;
        updater.update(move |vec| vec.push(i));
    }
    
    // First 3 should trigger immediate flush
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(data.read().len(), 3);
    
    // Wait for periodic flush
    tokio::time::sleep(Duration::from_millis(110)).await;
    assert_eq!(data.read().len(), 5);
}

#[test]
fn test_read_cache() {
    let config = bluefelt_core::lobby::lock_optimization::CacheConfig {
        max_age: Duration::from_millis(100),
        max_size: 3,
    };
    
    let cache = ReadCache::new(config);
    let mut compute_count = 0;
    
    // First call should compute
    let value = cache.get_or_compute(&"key1", || {
        compute_count += 1;
        "value1"
    });
    assert_eq!(value, "value1");
    assert_eq!(compute_count, 1);
    
    // Second call should use cache
    let value = cache.get_or_compute(&"key1", || {
        compute_count += 1;
        "value1"
    });
    assert_eq!(value, "value1");
    assert_eq!(compute_count, 1);
    
    // Invalidate and recompute
    cache.invalidate(&"key1");
    let value = cache.get_or_compute(&"key1", || {
        compute_count += 1;
        "value1_new"
    });
    assert_eq!(value, "value1_new");
    assert_eq!(compute_count, 2);
}

#[test]
fn test_health_monitor() {
    let monitor = HealthMonitor::new();
    
    // Initial state should be healthy
    match monitor.check_health() {
        bluefelt_core::lobby::connection_manager::HealthStatus::Healthy => {},
        _ => panic!("Should be healthy initially"),
    }
    
    // Record some failures
    for _ in 0..50 {
        monitor.record_failed_send();
    }
    
    // Should still be healthy (under threshold for 1 minute)
    match monitor.check_health() {
        bluefelt_core::lobby::connection_manager::HealthStatus::Healthy => {},
        _ => panic!("Should still be healthy"),
    }
    
    // Record many failures
    for _ in 0..100 {
        monitor.record_failed_send();
    }
    
    // Should now be critical
    match monitor.check_health() {
        bluefelt_core::lobby::connection_manager::HealthStatus::Critical(_) => {},
        _ => panic!("Should be critical after many failures"),
    }
}

// Helper function to create test table
fn create_test_table(id: &str) -> Arc<bluefelt_core::lobby::table_instance::TableInstance> {
    use bluefelt_core::bundle::{Bundle, Manifest};
    use bluefelt_core::lobby::table_instance::TableInstance;
    use serde_json::json;
    
    let manifest = Manifest {
        game_id: "test-game".to_string(),
        version: "1.0".to_string(),
        spec_version: 1,
        metadata: serde_json::from_value(json!({
            "name": "Test Game",
            "players": {"min": 2, "max": 2}
        })).unwrap(),
    };
    
    let bundle = Arc::new(Bundle {
        id: "test-game".to_string(),
        version: "1.0".to_string(),
        manifest,
        entities: json!({}),
        zones: json!({}),
        actions: json!([]),
        phases: json!({}),
    });
    
    Arc::new(TableInstance::new(
        id.to_string(),
        "test-game".to_string(),
        bundle,
        "owner".to_string(),
        None,
        None,
        None,
    ))
}
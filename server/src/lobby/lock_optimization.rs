//! Lock optimization strategies to reduce contention

use parking_lot::{RwLock, Mutex};
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde_json::Value;

/// Provides optimized lock access patterns
pub struct LockOptimizer {
    /// Configuration for lock timeouts
    config: LockConfig,
    /// Metrics for lock contention
    metrics: Arc<RwLock<LockMetrics>>,
}

pub struct LockConfig {
    /// Maximum time to wait for a lock
    pub max_wait_time: Duration,
    /// Whether to use try_lock patterns
    pub prefer_try_lock: bool,
    /// Backoff strategy for retries
    pub backoff_ms: u64,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            max_wait_time: Duration::from_millis(100),
            prefer_try_lock: true,
            backoff_ms: 1,
        }
    }
}

#[derive(Default)]
struct LockMetrics {
    total_acquisitions: u64,
    failed_acquisitions: u64,
    total_wait_time_us: u64,
    max_wait_time_us: u64,
}

impl LockOptimizer {
    pub fn new(config: LockConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(LockMetrics::default())),
        }
    }
    
    /// Try to acquire a read lock with timeout
    pub fn try_read_with_timeout<'a, T>(&self, lock: &'a RwLock<T>) -> Option<parking_lot::RwLockReadGuard<'a, T>> {
        let start = Instant::now();
        let deadline = start + self.config.max_wait_time;
        
        loop {
            if let Some(guard) = lock.try_read() {
                self.record_acquisition(start.elapsed());
                return Some(guard);
            }
            
            if Instant::now() >= deadline {
                self.record_failed_acquisition();
                return None;
            }
            
            // Backoff
            std::thread::sleep(Duration::from_millis(self.config.backoff_ms));
        }
    }
    
    /// Try to acquire a write lock with timeout
    pub fn try_write_with_timeout<'a, T>(&self, lock: &'a RwLock<T>) -> Option<parking_lot::RwLockWriteGuard<'a, T>> {
        let start = Instant::now();
        let deadline = start + self.config.max_wait_time;
        
        loop {
            if let Some(guard) = lock.try_write() {
                self.record_acquisition(start.elapsed());
                return Some(guard);
            }
            
            if Instant::now() >= deadline {
                self.record_failed_acquisition();
                return None;
            }
            
            // Backoff
            std::thread::sleep(Duration::from_millis(self.config.backoff_ms));
        }
    }
    
    fn record_acquisition(&self, wait_time: Duration) {
        let mut metrics = self.metrics.write();
        metrics.total_acquisitions += 1;
        let wait_us = wait_time.as_micros() as u64;
        metrics.total_wait_time_us += wait_us;
        metrics.max_wait_time_us = metrics.max_wait_time_us.max(wait_us);
    }
    
    fn record_failed_acquisition(&self) {
        self.metrics.write().failed_acquisitions += 1;
    }
    
    pub fn get_metrics(&self) -> LockMetricsSnapshot {
        let metrics = self.metrics.read();
        LockMetricsSnapshot {
            total_acquisitions: metrics.total_acquisitions,
            failed_acquisitions: metrics.failed_acquisitions,
            avg_wait_time_us: if metrics.total_acquisitions > 0 {
                metrics.total_wait_time_us / metrics.total_acquisitions
            } else {
                0
            },
            max_wait_time_us: metrics.max_wait_time_us,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LockMetricsSnapshot {
    pub total_acquisitions: u64,
    pub failed_acquisitions: u64,
    pub avg_wait_time_us: u64,
    pub max_wait_time_us: u64,
}

/// Batched updates to reduce lock frequency
pub struct BatchedUpdater<T> {
    /// Pending updates
    pending: Arc<Mutex<Vec<Box<dyn FnOnce(&mut T) + Send>>>>,
    /// Target for updates
    target: Arc<RwLock<T>>,
    /// Batch configuration
    config: BatchConfig,
}

pub struct BatchConfig {
    /// Maximum updates before flush
    pub max_batch_size: usize,
    /// Maximum time before flush
    pub max_batch_age: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_batch_age: Duration::from_millis(50),
        }
    }
}

impl<T: Send + Sync + 'static> BatchedUpdater<T> {
    pub fn new(target: Arc<RwLock<T>>, config: BatchConfig) -> Self {
        Self {
            pending: Arc::new(Mutex::new(Vec::new())),
            target,
            config,
        }
    }
    
    /// Add an update to the batch
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T) + Send + 'static,
    {
        let mut pending = self.pending.lock();
        pending.push(Box::new(f));
        
        // Check if we should flush
        if pending.len() >= self.config.max_batch_size {
            drop(pending);
            self.flush();
        }
    }
    
    /// Flush all pending updates
    pub fn flush(&self) {
        let updates: Vec<_> = {
            let mut pending = self.pending.lock();
            std::mem::take(&mut *pending)
        };
        
        if updates.is_empty() {
            return;
        }
        
        // Apply all updates in a single write lock
        let mut target = self.target.write();
        for update in updates {
            update(&mut *target);
        }
    }
    
    /// Start a background task to periodically flush
    pub fn start_flush_task(updater: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(updater.config.max_batch_age);
            
            loop {
                interval.tick().await;
                updater.flush();
            }
        });
    }
}

/// Read-through cache to reduce lock contention
pub struct ReadCache<K, V> {
    /// Cached values
    cache: Arc<RwLock<std::collections::HashMap<K, CachedValue<V>>>>,
    /// Cache configuration
    config: CacheConfig,
}

struct CachedValue<V> {
    value: V,
    cached_at: Instant,
}

pub struct CacheConfig {
    /// Maximum cache age
    pub max_age: Duration,
    /// Maximum cache size
    pub max_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_age: Duration::from_secs(5),
            max_size: 1000,
        }
    }
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> ReadCache<K, V> {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            config,
        }
    }
    
    /// Get a value from cache or compute it
    pub fn get_or_compute<F>(&self, key: &K, compute: F) -> V
    where
        F: FnOnce() -> V,
    {
        // Try to get from cache
        {
            let cache = self.cache.read();
            if let Some(cached) = cache.get(key) {
                if cached.cached_at.elapsed() < self.config.max_age {
                    return cached.value.clone();
                }
            }
        }
        
        // Compute value
        let value = compute();
        
        // Update cache
        {
            let mut cache = self.cache.write();
            
            // Evict old entries if at capacity
            if cache.len() >= self.config.max_size {
                let now = Instant::now();
                cache.retain(|_, v| now.duration_since(v.cached_at) < self.config.max_age);
                
                // If still at capacity, remove oldest
                if cache.len() >= self.config.max_size {
                    if let Some(oldest_key) = cache.iter()
                        .min_by_key(|(_, v)| v.cached_at)
                        .map(|(k, _)| k.clone()) {
                        cache.remove(&oldest_key);
                    }
                }
            }
            
            cache.insert(key.clone(), CachedValue {
                value: value.clone(),
                cached_at: Instant::now(),
            });
        }
        
        value
    }
    
    /// Invalidate a cache entry
    pub fn invalidate(&self, key: &K) {
        self.cache.write().remove(key);
    }
    
    /// Clear all cache entries
    pub fn clear(&self) {
        self.cache.write().clear();
    }
}

/// Lock-free state snapshots for read-heavy operations
pub struct SnapshotProvider<T: Clone> {
    /// Current snapshot
    snapshot: Arc<RwLock<Arc<T>>>,
    /// Update interval
    update_interval: Duration,
}

impl<T: Clone + Send + Sync + 'static> SnapshotProvider<T> {
    pub fn new(initial: T, update_interval: Duration) -> Self {
        Self {
            snapshot: Arc::new(RwLock::new(Arc::new(initial))),
            update_interval,
        }
    }
    
    /// Get current snapshot (very fast, no contention)
    pub fn get(&self) -> Arc<T> {
        self.snapshot.read().clone()
    }
    
    /// Update the snapshot
    pub fn update(&self, new_value: T) {
        let new_snapshot = Arc::new(new_value);
        *self.snapshot.write() = new_snapshot;
    }
    
    /// Start background update task
    pub fn start_update_task<F>(
        provider: Arc<Self>,
        source: Arc<RwLock<T>>,
        update_fn: F,
    ) where
        F: Fn(&T) -> T + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(provider.update_interval);
            
            loop {
                interval.tick().await;
                
                // Read source and create new snapshot
                let source_value = source.read().clone();
                let new_snapshot = update_fn(&source_value);
                provider.update(new_snapshot);
            }
        });
    }
}
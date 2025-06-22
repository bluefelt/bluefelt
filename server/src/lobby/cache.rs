use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde_json::Value;
use crate::lobby::LobbyMap;

/// Cached lobby list with automatic invalidation
pub struct LobbyListCache {
    cache: Arc<RwLock<CacheEntry>>,
    lobbies: Arc<LobbyMap>,
    ttl: Duration,
}

struct CacheEntry {
    data: Option<Value>,
    last_updated: Instant,
    generation: u64,
}

impl LobbyListCache {
    pub fn new(lobbies: Arc<LobbyMap>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(CacheEntry {
                data: None,
                last_updated: Instant::now(),
                generation: 0,
            })),
            lobbies,
            ttl: Duration::from_millis(500), // Cache for 500ms
        }
    }

    /// Get the cached lobby list or compute it if stale
    pub fn get_or_compute<F>(&self, compute_fn: F) -> Value 
    where
        F: FnOnce(&LobbyMap) -> Value
    {
        // Try to get from cache with read lock
        {
            let cache_read = self.cache.read();
            if let Some(ref cached_data) = cache_read.data {
                if cache_read.last_updated.elapsed() < self.ttl {
                    println!("[CACHE] Returning cached lobby list (age: {:?})", 
                             cache_read.last_updated.elapsed());
                    return cached_data.clone();
                }
            }
        }

        // Cache miss or stale - compute with write lock
        let mut cache_write = self.cache.write();
        
        // Double-check in case another thread updated while we waited
        if let Some(ref cached_data) = cache_write.data {
            if cache_write.last_updated.elapsed() < self.ttl {
                return cached_data.clone();
            }
        }

        // Compute new value
        println!("[CACHE] Computing fresh lobby list");
        let start = Instant::now();
        let fresh_data = compute_fn(&self.lobbies);
        let compute_time = start.elapsed();
        
        // Update cache
        cache_write.data = Some(fresh_data.clone());
        cache_write.last_updated = Instant::now();
        cache_write.generation += 1;
        
        println!("[CACHE] Lobby list computed in {:?}, cached for next {:?}", 
                 compute_time, self.ttl);
        
        fresh_data
    }

    /// Invalidate the cache
    pub fn invalidate(&self) {
        let mut cache_write = self.cache.write();
        cache_write.data = None;
        println!("[CACHE] Lobby list cache invalidated");
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache_read = self.cache.read();
        CacheStats {
            has_data: cache_read.data.is_some(),
            age: cache_read.last_updated.elapsed(),
            generation: cache_read.generation,
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub has_data: bool,
    pub age: Duration,
    pub generation: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dashmap::DashMap;
    use serde_json::json;

    #[test]
    fn test_cache_hit_miss() {
        let lobbies = Arc::new(DashMap::new());
        let cache = LobbyListCache::new(lobbies.clone());
        
        let mut compute_count = 0;
        
        // First call should compute
        let result1 = cache.get_or_compute(|_| {
            compute_count += 1;
            json!({"computed": 1})
        });
        assert_eq!(compute_count, 1);
        assert_eq!(result1, json!({"computed": 1}));
        
        // Second call should use cache
        let result2 = cache.get_or_compute(|_| {
            compute_count += 1;
            json!({"computed": 2})
        });
        assert_eq!(compute_count, 1); // Not computed again
        assert_eq!(result2, json!({"computed": 1})); // Same cached value
        
        // After invalidation, should recompute
        cache.invalidate();
        let result3 = cache.get_or_compute(|_| {
            compute_count += 1;
            json!({"computed": 3})
        });
        assert_eq!(compute_count, 2);
        assert_eq!(result3, json!({"computed": 3}));
    }
}
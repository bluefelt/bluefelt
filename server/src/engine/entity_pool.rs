//! Memory-efficient entity storage with object pooling

use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;

/// Pool for reusing entity objects to reduce allocation overhead
pub struct EntityPool {
    /// Pools organized by entity type
    pools: Arc<RwLock<HashMap<String, VecDeque<Value>>>>,
    /// Maximum entities per pool
    max_pool_size: usize,
    /// Statistics
    stats: Arc<RwLock<PoolStats>>,
}

#[derive(Default, Debug, Clone)]
pub struct PoolStats {
    pub allocations: u64,
    pub reuses: u64,
    pub returns: u64,
    pub evictions: u64,
}

impl EntityPool {
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            max_pool_size,
            stats: Arc::new(RwLock::new(PoolStats::default())),
        }
    }

    /// Get an entity from the pool or create a new one
    pub fn acquire(&self, entity_type: &str, template: &Value) -> Value {
        let mut pools = self.pools.write();
        let pool = pools.entry(entity_type.to_string()).or_insert_with(VecDeque::new);
        
        if let Some(mut entity) = pool.pop_front() {
            // Reuse existing entity
            self.stats.write().reuses += 1;
            
            // Reset entity to template values
            if let (Value::Object(entity_map), Value::Object(template_map)) = 
                (&mut entity, template) {
                // Keep the same object but update values
                entity_map.clear();
                for (k, v) in template_map {
                    entity_map.insert(k.clone(), v.clone());
                }
            }
            
            entity
        } else {
            // Allocate new entity
            self.stats.write().allocations += 1;
            template.clone()
        }
    }

    /// Return an entity to the pool for reuse
    pub fn release(&self, entity_type: &str, entity: Value) {
        let mut pools = self.pools.write();
        let pool = pools.entry(entity_type.to_string()).or_insert_with(VecDeque::new);
        
        if pool.len() < self.max_pool_size {
            self.stats.write().returns += 1;
            pool.push_back(entity);
        } else {
            // Pool is full, evict oldest
            self.stats.write().evictions += 1;
        }
    }

    /// Clear all pools
    pub fn clear(&self) {
        self.pools.write().clear();
    }

    /// Get current statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.read().clone()
    }
}

/// Efficient storage for large numbers of entities
pub struct EntityStorage {
    /// Entity data indexed by ID
    entities: HashMap<String, Arc<Value>>,
    /// Entity locations (zone_id -> entity_ids)
    locations: HashMap<String, Vec<String>>,
    /// Entity pool for reuse
    pool: EntityPool,
}

impl EntityStorage {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            locations: HashMap::new(),
            pool: EntityPool::new(100),
        }
    }

    /// Add an entity to storage
    pub fn add_entity(&mut self, id: String, entity_type: &str, data: Value, zone_id: &str) {
        // Store entity data
        self.entities.insert(id.clone(), Arc::new(data));
        
        // Track location
        self.locations
            .entry(zone_id.to_string())
            .or_insert_with(Vec::new)
            .push(id);
    }

    /// Move an entity between zones
    pub fn move_entity(&mut self, entity_id: &str, from_zone: &str, to_zone: &str) -> Result<(), String> {
        // Remove from old location
        if let Some(entities) = self.locations.get_mut(from_zone) {
            entities.retain(|id| id != entity_id);
        } else {
            return Err(format!("Entity {} not found in zone {}", entity_id, from_zone));
        }
        
        // Add to new location
        self.locations
            .entry(to_zone.to_string())
            .or_insert_with(Vec::new)
            .push(entity_id.to_string());
        
        Ok(())
    }

    /// Remove an entity from storage
    pub fn remove_entity(&mut self, entity_id: &str, zone_id: &str) -> Option<Arc<Value>> {
        // Remove from location tracking
        if let Some(entities) = self.locations.get_mut(zone_id) {
            entities.retain(|id| id != entity_id);
        }
        
        // Remove and return entity data
        self.entities.remove(entity_id)
    }

    /// Get all entities in a zone
    pub fn get_zone_entities(&self, zone_id: &str) -> Vec<(String, Arc<Value>)> {
        self.locations
            .get(zone_id)
            .map(|entity_ids| {
                entity_ids
                    .iter()
                    .filter_map(|id| {
                        self.entities.get(id)
                            .map(|data| (id.clone(), data.clone()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get entity count in a zone
    pub fn zone_entity_count(&self, zone_id: &str) -> usize {
        self.locations
            .get(zone_id)
            .map(|entities| entities.len())
            .unwrap_or(0)
    }

    /// Total entity count
    pub fn total_entities(&self) -> usize {
        self.entities.len()
    }

    /// Memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        // Estimate based on stored values
        self.entities.values()
            .map(|v| estimate_value_size(v))
            .sum::<usize>()
            + self.locations.iter()
                .map(|(k, v)| k.len() + v.len() * 20) // Rough estimate for string storage
                .sum::<usize>()
    }
}

/// Estimate memory size of a JSON value
fn estimate_value_size(value: &Value) -> usize {
    match value {
        Value::Null => 8,
        Value::Bool(_) => 8,
        Value::Number(_) => 16,
        Value::String(s) => 24 + s.len(),
        Value::Array(arr) => {
            24 + arr.iter().map(estimate_value_size).sum::<usize>()
        }
        Value::Object(obj) => {
            24 + obj.iter()
                .map(|(k, v)| k.len() + estimate_value_size(v))
                .sum::<usize>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_entity_pool_reuse() {
        let pool = EntityPool::new(10);
        let template = json!({"type": "card", "rank": "A"});
        
        // Acquire and release
        let entity1 = pool.acquire("card", &template);
        pool.release("card", entity1);
        
        // Should reuse
        let entity2 = pool.acquire("card", &template);
        
        let stats = pool.stats();
        assert_eq!(stats.allocations, 1);
        assert_eq!(stats.reuses, 1);
        assert_eq!(stats.returns, 1);
    }

    #[test]
    fn test_entity_storage() {
        let mut storage = EntityStorage::new();
        
        // Add entities
        storage.add_entity(
            "card1".to_string(),
            "card",
            json!({"rank": "A", "suit": "hearts"}),
            "deck"
        );
        
        storage.add_entity(
            "card2".to_string(),
            "card", 
            json!({"rank": "K", "suit": "spades"}),
            "deck"
        );
        
        // Check zone contents
        assert_eq!(storage.zone_entity_count("deck"), 2);
        assert_eq!(storage.total_entities(), 2);
        
        // Move entity
        storage.move_entity("card1", "deck", "hand").unwrap();
        assert_eq!(storage.zone_entity_count("deck"), 1);
        assert_eq!(storage.zone_entity_count("hand"), 1);
        
        // Remove entity
        let removed = storage.remove_entity("card2", "deck");
        assert!(removed.is_some());
        assert_eq!(storage.total_entities(), 1);
    }

    #[test]
    fn test_memory_usage_tracking() {
        let mut storage = EntityStorage::new();
        
        // Add some entities
        for i in 0..10 {
            storage.add_entity(
                format!("entity{}", i),
                "test",
                json!({"index": i, "data": "x".repeat(100)}),
                "zone1"
            );
        }
        
        let usage = storage.memory_usage();
        assert!(usage > 1000); // Should be at least 1KB with the data
    }
}
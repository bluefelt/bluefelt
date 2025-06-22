//! Memory management and leak prevention for the lobby system

use super::lobby_state::LobbyState;
use super::table_instance::TableInstance;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

/// Monitors memory usage and prevents unbounded growth
pub struct MemoryManager {
    /// Configuration for memory limits
    config: MemoryConfig,
    /// Tracks resource usage
    usage: Arc<RwLock<ResourceUsage>>,
}

pub struct MemoryConfig {
    /// Maximum tables per lobby
    pub max_tables_per_lobby: usize,
    /// Maximum game history entries
    pub max_history_entries: usize,
    /// Maximum chat messages per table
    pub max_chat_messages: usize,
    /// Maximum game log entries
    pub max_game_log_entries: usize,
    /// Maximum patch history size
    pub max_patch_history: usize,
    /// Table idle timeout
    pub table_idle_timeout: Duration,
    /// History retention period
    pub history_retention: Duration,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_tables_per_lobby: 100,
            max_history_entries: 1000,
            max_chat_messages: 500,
            max_game_log_entries: 1000,
            max_patch_history: 100,
            table_idle_timeout: Duration::from_secs(3600), // 1 hour
            history_retention: Duration::from_secs(86400),  // 24 hours
        }
    }
}

struct ResourceUsage {
    tables_created: usize,
    tables_cleaned: usize,
    messages_trimmed: usize,
    history_purged: usize,
    last_cleanup: Instant,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            tables_created: 0,
            tables_cleaned: 0,
            messages_trimmed: 0,
            history_purged: 0,
            last_cleanup: Instant::now(),
        }
    }
}

impl MemoryManager {
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            config,
            usage: Arc::new(RwLock::new(ResourceUsage {
                last_cleanup: Instant::now(),
                ..Default::default()
            })),
        }
    }
    
    /// Clean up idle tables in a lobby
    pub fn cleanup_idle_tables(&self, lobby: &LobbyState) -> usize {
        let now = SystemTime::now();
        let mut cleaned = 0;
        
        // Get tables to remove
        let tables_to_remove: Vec<String> = {
            let tables = lobby.tables.lock();
            tables.iter()
                .filter_map(|(id, table)| {
                    let status = table.status.read();
                    match &*status {
                        crate::lobby::table_instance::TableStatus::Finished |
                        crate::lobby::table_instance::TableStatus::Abandoned => {
                            // Check if table has been idle too long
                            if let Some(finished_at) = table.finished_at.read().as_ref() {
                                if let Ok(duration) = now.duration_since(*finished_at) {
                                    if duration > self.config.table_idle_timeout {
                                        return Some(id.clone());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    None
                })
                .collect()
        };
        
        // Remove idle tables
        if !tables_to_remove.is_empty() {
            let mut tables = lobby.tables.lock();
            for id in tables_to_remove {
                if tables.remove(&id).is_some() {
                    cleaned += 1;
                    println!("[MemoryManager] Removed idle table: {}", id);
                }
            }
        }
        
        self.usage.write().tables_cleaned += cleaned;
        cleaned
    }
    
    /// Trim chat messages to prevent unbounded growth
    pub fn trim_chat_messages(&self, lobby: &LobbyState) -> usize {
        let mut trimmed = 0;
        
        // Trim lobby chat
        let lobby_trimmed = lobby.chat.trim_old_messages(self.config.max_chat_messages);
        trimmed += lobby_trimmed;
        
        // Trim table chats
        let tables = lobby.tables.lock();
        for table in tables.values() {
            // Table chat is a Vec<ChatMessage>, not a ChatSystem
            let mut chat = table.chat.write();
            let len = chat.len();
            if len > self.config.max_chat_messages {
                let to_remove = len - self.config.max_chat_messages;
                chat.drain(0..to_remove);
                trimmed += to_remove;
            }
        }
        
        self.usage.write().messages_trimmed += trimmed;
        trimmed
    }
    
    /// Clean up game logs to prevent memory bloat
    pub fn cleanup_game_logs(&self, table: &Arc<TableInstance>) -> usize {
        let mut state = table.game_state.write();
        let mut cleaned = 0;
        
        // Trim game log if it exists
        if let Some(log) = state.get_mut("gameLog").and_then(|l| l.as_array_mut()) {
            let original_len = log.len();
            if original_len > self.config.max_game_log_entries {
                // Keep only the most recent entries
                let to_remove = original_len - self.config.max_game_log_entries;
                log.drain(0..to_remove);
                cleaned = to_remove;
            }
        }
        
        cleaned
    }
    
    /// Limit patch history size
    pub fn limit_patch_history(patches: &mut Vec<serde_json::Value>, max_size: usize) {
        if patches.len() > max_size {
            let to_remove = patches.len() - max_size;
            patches.drain(0..to_remove);
        }
    }
    
    /// Check if lobby has too many tables
    pub fn should_limit_tables(&self, lobby: &LobbyState) -> bool {
        let tables = lobby.tables.lock();
        tables.len() >= self.config.max_tables_per_lobby
    }
    
    /// Get memory usage statistics
    pub fn get_stats(&self) -> MemoryStats {
        let usage = self.usage.read();
        MemoryStats {
            tables_created: usage.tables_created,
            tables_cleaned: usage.tables_cleaned,
            messages_trimmed: usage.messages_trimmed,
            history_purged: usage.history_purged,
            last_cleanup: usage.last_cleanup,
        }
    }
    
    /// Run periodic cleanup
    pub async fn run_cleanup_task(manager: Arc<MemoryManager>, lobby_map: Arc<crate::lobby::LobbyMap>) {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
        
        loop {
            interval.tick().await;
            
            println!("[MemoryManager] Running periodic cleanup");
            let start = Instant::now();
            let mut total_cleaned = 0;
            
            // Clean up each lobby
            for entry in lobby_map.iter() {
                let lobby = entry.value();
                
                // Clean idle tables
                total_cleaned += manager.cleanup_idle_tables(&lobby.state);
                
                // Trim chat messages
                manager.trim_chat_messages(&lobby.state);
                
                // Clean game logs in active tables
                let tables = lobby.state.tables.lock();
                for table in tables.values() {
                    manager.cleanup_game_logs(table);
                }
            }
            
            let duration = start.elapsed();
            println!("[MemoryManager] Cleanup completed in {:?}, cleaned {} tables", duration, total_cleaned);
            
            // Update last cleanup time
            manager.usage.write().last_cleanup = Instant::now();
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub tables_created: usize,
    pub tables_cleaned: usize,
    pub messages_trimmed: usize,
    pub history_purged: usize,
    pub last_cleanup: Instant,
}

/// Bounded collection wrapper to prevent unbounded growth
pub struct BoundedVec<T> {
    items: Vec<T>,
    max_size: usize,
}

impl<T> BoundedVec<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: Vec::with_capacity(max_size.min(1000)),
            max_size,
        }
    }
    
    pub fn push(&mut self, item: T) {
        if self.items.len() >= self.max_size {
            // Remove oldest item
            self.items.remove(0);
        }
        self.items.push(item);
    }
    
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.items.iter()
    }
    
    pub fn clear(&mut self) {
        self.items.clear();
    }
    
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.items.retain(f);
    }
}

/// Bounded hash map to prevent unbounded growth
pub struct BoundedHashMap<K, V> {
    items: HashMap<K, V>,
    max_size: usize,
    eviction_order: Vec<K>,
}

impl<K: Clone + Eq + std::hash::Hash, V> BoundedHashMap<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: HashMap::with_capacity(max_size.min(1000)),
            max_size,
            eviction_order: Vec::with_capacity(max_size.min(1000)),
        }
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // If at capacity and key doesn't exist, evict oldest
        if self.items.len() >= self.max_size && !self.items.contains_key(&key) {
            if let Some(oldest_key) = self.eviction_order.first().cloned() {
                self.items.remove(&oldest_key);
                self.eviction_order.remove(0);
            }
        }
        
        // Update eviction order
        if let Some(pos) = self.eviction_order.iter().position(|k| k == &key) {
            self.eviction_order.remove(pos);
        }
        self.eviction_order.push(key.clone());
        
        self.items.insert(key, value)
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        self.items.get(key)
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(pos) = self.eviction_order.iter().position(|k| k == key) {
            self.eviction_order.remove(pos);
        }
        self.items.remove(key)
    }
    
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.items.clear();
        self.eviction_order.clear();
    }
}

/// Memory pressure detector
pub struct MemoryPressureDetector {
    threshold_mb: usize,
    check_interval: Duration,
}

impl MemoryPressureDetector {
    pub fn new(threshold_mb: usize) -> Self {
        Self {
            threshold_mb,
            check_interval: Duration::from_secs(60),
        }
    }
    
    /// Check if memory usage is above threshold
    pub fn is_under_pressure(&self) -> bool {
        // This is a placeholder - in production you'd use actual memory metrics
        // For now, we'll use a simple heuristic based on process statistics
        false
    }
    
    /// Get estimated memory usage in MB
    pub fn get_memory_usage_mb(&self) -> usize {
        // Placeholder implementation
        0
    }
}
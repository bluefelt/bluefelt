use std::collections::VecDeque;
use parking_lot::Mutex;
use serde_json::Value;

/// Maximum number of history entries to keep
const MAX_HISTORY_SIZE: usize = 1000;

/// Maximum size of a single history entry in bytes
const MAX_ENTRY_SIZE: usize = 50_000;

/// A bounded history buffer that automatically removes old entries
pub struct BoundedHistory {
    entries: Mutex<VecDeque<Value>>,
    max_size: usize,
}

impl BoundedHistory {
    pub fn new() -> Self {
        Self::with_capacity(MAX_HISTORY_SIZE)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            entries: Mutex::new(VecDeque::with_capacity(max_size)),
            max_size,
        }
    }

    /// Push a new entry, removing the oldest if at capacity
    pub fn push(&self, entry: Value) {
        // Check entry size to prevent memory bloat
        let entry_size = entry.to_string().len();
        if entry_size > MAX_ENTRY_SIZE {
            eprintln!("[WARN] History entry too large ({} bytes), skipping", entry_size);
            return;
        }

        let mut entries = self.entries.lock();
        
        // Remove oldest entries if at capacity
        while entries.len() >= self.max_size {
            entries.pop_front();
        }
        
        entries.push_back(entry);
    }

    /// Get all entries as a Vec (for compatibility)
    pub fn to_vec(&self) -> Vec<Value> {
        self.entries.lock().iter().cloned().collect()
    }

    /// Get recent entries (last N)
    pub fn recent(&self, count: usize) -> Vec<Value> {
        let entries = self.entries.lock();
        entries.iter()
            .rev()
            .take(count)
            .rev()
            .cloned()
            .collect()
    }

    /// Clear all history
    pub fn clear(&self) {
        self.entries.lock().clear();
    }

    /// Get the current number of entries
    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.lock().is_empty()
    }
}

impl Default for BoundedHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_bounded_history() {
        let history = BoundedHistory::with_capacity(3);
        
        // Add 4 entries
        history.push(json!({"id": 1}));
        history.push(json!({"id": 2}));
        history.push(json!({"id": 3}));
        history.push(json!({"id": 4}));
        
        // Should only have 3 entries (oldest removed)
        assert_eq!(history.len(), 3);
        
        let entries = history.to_vec();
        assert_eq!(entries[0], json!({"id": 2}));
        assert_eq!(entries[1], json!({"id": 3}));
        assert_eq!(entries[2], json!({"id": 4}));
    }

    #[test]
    fn test_large_entry_rejection() {
        let history = BoundedHistory::new();
        
        // Create a large entry
        let large_string = "x".repeat(MAX_ENTRY_SIZE + 1);
        let large_entry = json!({"data": large_string});
        
        history.push(large_entry);
        
        // Should not be added
        assert_eq!(history.len(), 0);
    }
}
//! Helper methods for safe lock management in the lobby system

use super::{table_instance::TableInstance, lobby_state::LobbyState};
use parking_lot::{RwLock, Mutex};
use std::sync::Arc;
use serde_json::Value;

/// Extension trait for Lobby to provide safe lock access patterns
pub trait LobbyLockHelpers {
    /// Execute a function with access to a table instance without exposing locks
    fn with_table<F, R>(&self, table_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Arc<TableInstance>) -> R;
    
    /// Execute a function with mutable access to table game state
    fn with_table_state_mut<F, R>(&self, table_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;
    
    /// Execute a function with read-only access to table game state
    fn with_table_state<F, R>(&self, table_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;
    
    /// Get a snapshot of all table IDs without holding locks
    fn get_table_ids(&self) -> Vec<String>;
    
    /// Get a snapshot of all member names without holding locks
    fn get_member_names(&self) -> Vec<String>;
    
    /// Legacy method for backward compatibility
    fn get_game_ids(&self) -> Vec<String>;
}

/// Implementation for LobbyState
impl LobbyLockHelpers for LobbyState {
    fn with_table<F, R>(&self, table_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Arc<TableInstance>) -> R,
    {
        let tables = self.tables.lock();
        tables.get(table_id).map(|table| f(table))
    }
    
    fn with_table_state_mut<F, R>(&self, table_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R,
    {
        let tables = self.tables.lock();
        tables.get(table_id).map(|table| {
            let mut state = table.game_state.write();
            f(&mut *state)
        })
    }
    
    fn with_table_state<F, R>(&self, table_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R,
    {
        let tables = self.tables.lock();
        tables.get(table_id).map(|table| {
            let state = table.game_state.read();
            f(&*state)
        })
    }
    
    fn get_table_ids(&self) -> Vec<String> {
        self.tables.lock().keys().cloned().collect()
    }
    
    fn get_member_names(&self) -> Vec<String> {
        self.members.read().iter().map(|m| m.id.clone()).collect()
    }
    
    /// Legacy method for backward compatibility
    fn get_game_ids(&self) -> Vec<String> {
        self.get_table_ids()
    }
}

/// Extension trait for TableInstance to provide safe lock access
pub trait TableLockHelpers {
    /// Execute a function with read access to game state
    fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Value) -> R;
    
    /// Execute a function with write access to game state
    fn with_state_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Value) -> R;
    
    /// Get current tick without holding lock
    fn get_tick(&self) -> u64;
    
    /// Increment tick and return new value
    fn increment_tick(&self) -> u64;
}

/// Implementation for TableInstance
impl TableLockHelpers for TableInstance {
    fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Value) -> R,
    {
        let state = self.game_state.read();
        f(&*state)
    }
    
    fn with_state_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Value) -> R,
    {
        let mut state = self.game_state.write();
        f(&mut *state)
    }
    
    fn get_tick(&self) -> u64 {
        *self.tick.lock()
    }
    
    fn increment_tick(&self) -> u64 {
        let mut tick = self.tick.lock();
        *tick += 1;
        *tick
    }
}

/// Helper to avoid nested locks when accessing multiple table properties
pub struct TableSnapshot {
    pub state: Value,
    pub tick: u64,
    pub players: std::collections::HashMap<String, String>,
}

impl TableSnapshot {
    /// Create a snapshot of table data to avoid holding locks
    pub fn from_table(table: &Arc<TableInstance>) -> Self {
        // Acquire locks one at a time and copy data
        let state = table.game_state.read().clone();
        let tick = *table.tick.lock();
        
        // Build player mapping from seats
        let mut players = std::collections::HashMap::new();
        let seats = table.seats.read();
        for (i, seat) in seats.iter().enumerate() {
            if let Some(super::table_instance::SeatOccupant::Player(player_id)) = seat {
                players.insert(format!("p{}", i + 1), player_id.clone());
            }
        }
        
        Self {
            state,
            tick,
            players,
        }
    }
}

/// Legacy type alias for backward compatibility
pub type GameSnapshot = TableSnapshot;

/// Lock ordering hierarchy to prevent deadlocks:
/// 1. lobby_state.tables (outermost)
/// 2. lobby_state.members
/// 3. table_instance.seats/ready_states
/// 4. table_instance.game_state (innermost)
/// 
/// Always acquire locks in this order and release in reverse order.
pub const LOCK_ORDERING_DOCUMENTATION: &str = r#"
Lock Ordering Rules:
1. Always acquire lobby.tables before lobby.members
2. Always acquire table locks after lobby locks
3. Within a table: acquire seats before game_state
4. Never hold locks across async operations
5. Prefer cloning data over holding locks
"#;
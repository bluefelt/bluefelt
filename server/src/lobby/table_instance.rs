//! Table instance management within a lobby
//! 
//! Tables are game instances with enhanced social features like seat claiming,
//! ready states, and countdown timers.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::SystemTime;
use parking_lot::{Mutex, RwLock};
use crate::bundle::Bundle;
use nanoid::nanoid;

/// Represents a single table (game instance) within a lobby
#[derive(Clone)]
pub struct TableInstance {
    /// Unique ID for this table instance
    pub id: String,
    
    /// The game bundle ID (e.g., "tic-tac-toe@1.0.0")
    pub bundle_id: String,
    
    /// Reference to the game bundle
    pub bundle: Arc<Bundle>,
    
    /// Minimum players required to start
    pub min_players: u32,
    
    /// Maximum players allowed
    pub max_players: u32,
    
    /// The player who created the table
    pub owner: String,
    
    /// Current table status
    pub status: Arc<RwLock<TableStatus>>,
    
    /// Seats at the table (None = empty, Some = occupied)
    pub seats: Arc<RwLock<Vec<Option<SeatOccupant>>>>,
    
    /// Ready state for each seat
    pub ready_states: Arc<RwLock<Vec<bool>>>,
    
    /// List of spectators watching the game
    pub spectators: Arc<RwLock<Vec<String>>>,
    
    /// Table-specific chat messages
    pub chat: Arc<RwLock<Vec<ChatMessage>>>,
    
    /// When the countdown ends (if in countdown state)
    pub countdown_ends_at: Arc<RwLock<Option<SystemTime>>>,
    
    /// The actual game state (only populated when playing)
    pub game_state: Arc<RwLock<Value>>,
    
    /// Current tick/version number
    pub tick: Arc<Mutex<u64>>,
    
    /// When the table was created
    pub created_at: SystemTime,
    
    /// When the game ended (if applicable)
    pub ended_at: Option<SystemTime>,
    
    /// When the table finished (game over or abandoned)
    pub finished_at: Arc<RwLock<Option<SystemTime>>>,
}

/// Current status of a table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TableStatus {
    /// Table is open for players to join
    Open,
    /// Countdown timer is running
    Countdown,
    /// Game is in progress
    Playing,
    /// Game has finished
    Finished,
    /// Table was abandoned
    Abandoned,
}

/// Represents who is occupying a seat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "player")]
pub enum SeatOccupant {
    /// Seat is occupied by a player
    Player(String),
    /// Seat is reserved (for future use)
    Reserved,
}

// Use ChatMessage from the chat system
pub use super::chat::ChatMessage;

impl TableInstance {
    /// Create a new table instance (backward compatibility constructor for tests)
    pub fn new(
        id: String,
        bundle_id: String,
        bundle: Arc<Bundle>,
        owner: String,
        _name: Option<String>,
        _min_players: Option<u32>,
        _max_players: Option<u32>,
    ) -> Self {
        let min_players = _min_players.unwrap_or(bundle.manifest.metadata.players.min);
        let max_players = _max_players.unwrap_or(bundle.manifest.metadata.players.max);
        
        // Initialize empty seats and ready states
        let seats = vec![None; max_players as usize];
        let ready_states = vec![false; max_players as usize];
        
        Self {
            id,
            bundle_id,
            bundle,
            min_players,
            max_players,
            owner,
            status: Arc::new(RwLock::new(TableStatus::Open)),
            seats: Arc::new(RwLock::new(seats)),
            ready_states: Arc::new(RwLock::new(ready_states)),
            spectators: Arc::new(RwLock::new(Vec::new())),
            chat: Arc::new(RwLock::new(Vec::new())),
            countdown_ends_at: Arc::new(RwLock::new(None)),
            game_state: Arc::new(RwLock::new(Value::Null)),
            tick: Arc::new(Mutex::new(0)),
            created_at: SystemTime::now(),
            ended_at: None,
            finished_at: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Create a new table instance
    pub fn new_simple(bundle_id: String, bundle: Arc<Bundle>, owner: String) -> Self {
        let min_players = bundle.manifest.metadata.players.min;
        let max_players = bundle.manifest.metadata.players.max;
        
        // Initialize seats with owner in first seat
        let mut seats = vec![None; max_players as usize];
        seats[0] = Some(SeatOccupant::Player(owner.clone()));
        
        let ready_states = vec![false; max_players as usize];
        
        Self {
            id: nanoid!(10),
            bundle_id,
            bundle,
            min_players,
            max_players,
            owner,
            status: Arc::new(RwLock::new(TableStatus::Open)),
            seats: Arc::new(RwLock::new(seats)),
            ready_states: Arc::new(RwLock::new(ready_states)),
            spectators: Arc::new(RwLock::new(Vec::new())),
            chat: Arc::new(RwLock::new(Vec::new())),
            countdown_ends_at: Arc::new(RwLock::new(None)),
            game_state: Arc::new(RwLock::new(Value::Null)),
            tick: Arc::new(Mutex::new(0)),
            created_at: SystemTime::now(),
            ended_at: None,
            finished_at: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Get the number of seated players
    pub fn seated_count(&self) -> usize {
        self.seats.read()
            .iter()
            .filter(|s| s.is_some())
            .count()
    }
    
    /// Check if a specific player is seated
    pub fn is_player_seated(&self, player_id: &str) -> bool {
        self.seats.read()
            .iter()
            .any(|s| matches!(s, Some(SeatOccupant::Player(id)) if id == player_id))
    }
    
    /// Get the seat index for a player (if seated)
    pub fn get_player_seat(&self, player_id: &str) -> Option<usize> {
        self.seats.read()
            .iter()
            .position(|s| matches!(s, Some(SeatOccupant::Player(id)) if id == player_id))
    }
    
    /// Check if the table is full
    pub fn is_full(&self) -> bool {
        self.seated_count() >= self.max_players as usize
    }
    
    /// Check if the table can start (minimum players seated)
    pub fn can_start(&self) -> bool {
        let status = self.status.read();
        matches!(*status, TableStatus::Open | TableStatus::Countdown) 
            && self.seated_count() >= self.min_players as usize
    }
    
    /// Check if all seated players are ready
    pub fn all_ready(&self) -> bool {
        let seats = self.seats.read();
        let ready = self.ready_states.read();
        
        // Check each occupied seat has a corresponding ready state
        seats.iter()
            .enumerate()
            .filter(|(_, seat)| seat.is_some())
            .all(|(i, _)| ready.get(i).copied().unwrap_or(false))
    }
    
    /// Get player IDs for all seated players
    pub fn get_seated_players(&self) -> Vec<String> {
        self.seats.read()
            .iter()
            .filter_map(|s| match s {
                Some(SeatOccupant::Player(id)) => Some(id.clone()),
                _ => None,
            })
            .collect()
    }
    
    /// Claim a seat at the table
    pub fn claim_seat(&self, seat_index: usize, player_id: String, username: String) -> Result<(), String> {
        // Validate seat index
        {
            let seats = self.seats.read();
            if seat_index >= seats.len() {
                return Err(format!("Invalid seat index: {}", seat_index));
            }
        }
        
        // Check table status
        {
            let status = self.status.read();
            match &*status {
                TableStatus::Open => {},
                TableStatus::Countdown => return Err("Cannot claim seat during countdown".to_string()),
                TableStatus::Playing => return Err("Game already in progress".to_string()),
                TableStatus::Finished => return Err("Game is finished".to_string()),
                TableStatus::Abandoned => return Err("Table has been abandoned".to_string()),
            }
        }
        
        // Check if player already has a seat
        {
            let seats = self.seats.read();
            if seats.iter().any(|s| matches!(s, Some(SeatOccupant::Player(id)) if id == &player_id)) {
                return Err("Player already has a seat at this table".to_string());
            }
        }
        
        // Claim the seat
        {
            let mut seats = self.seats.write();
            match &seats[seat_index] {
                None => {
                    seats[seat_index] = Some(SeatOccupant::Player(player_id));
                    Ok(())
                }
                Some(_) => Err("Seat is already taken".to_string()),
            }
        }
    }
    
    /// Release a seat at the table
    pub fn release_seat(&self, seat_index: usize) -> Result<(), String> {
        // Validate seat index
        {
            let seats = self.seats.read();
            if seat_index >= seats.len() {
                return Err(format!("Invalid seat index: {}", seat_index));
            }
        }
        
        // Check table status
        {
            let status = self.status.read();
            match &*status {
                TableStatus::Open | TableStatus::Countdown => {},
                TableStatus::Playing => return Err("Cannot release seat while game is in progress".to_string()),
                TableStatus::Finished => return Err("Game is already finished".to_string()),
                TableStatus::Abandoned => return Err("Table has been abandoned".to_string()),
            }
        }
        
        // Release the seat
        {
            let mut seats = self.seats.write();
            seats[seat_index] = None;
        }
        
        // Reset ready state
        {
            let mut ready = self.ready_states.write();
            ready[seat_index] = false;
        }
        
        // Check if we need to cancel countdown
        self.check_and_cancel_countdown();
        
        Ok(())
    }
    
    /// Set ready state for a player
    pub fn set_ready_state(&self, player_id: &str, ready: bool) -> Result<bool, String> {
        // Find player's seat
        let seat_index = match self.get_player_seat(player_id) {
            Some(idx) => idx,
            None => return Err("Player is not seated at this table".to_string()),
        };
        
        // Update ready state
        {
            let mut ready_states = self.ready_states.write();
            ready_states[seat_index] = ready;
        }
        
        // Check if we should start countdown
        if ready && self.can_start() && self.all_ready() {
            // Start countdown
            Ok(true)
        } else {
            // Check if we should cancel countdown
            if !ready {
                self.check_and_cancel_countdown();
            }
            Ok(false)
        }
    }
    
    /// Check and cancel countdown if conditions no longer met
    pub fn check_and_cancel_countdown(&self) {
        let status = self.status.read();
        if matches!(*status, TableStatus::Countdown) {
            drop(status);
            
            if !self.all_ready() || !self.can_start() {
                let mut status = self.status.write();
                *status = TableStatus::Open;
                let mut countdown = self.countdown_ends_at.write();
                *countdown = None;
            }
        }
    }
    
    /// Add a spectator
    pub fn add_spectator(&self, player_id: String) -> Result<(), String> {
        let mut spectators = self.spectators.write();
        if spectators.contains(&player_id) {
            return Err("Already spectating".to_string());
        }
        spectators.push(player_id);
        Ok(())
    }
    
    /// Remove a spectator
    pub fn remove_spectator(&self, player_id: &str) {
        let mut spectators = self.spectators.write();
        spectators.retain(|id| id != player_id);
    }
    
    /// Add a chat message
    pub fn add_chat_message(&self, from: String, message: String) {
        let mut chat = self.chat.write();
        chat.push(ChatMessage {
            id: nanoid!(10),
            from: from.clone(),
            from_name: from, // Use same as from for now
            message,
            timestamp: SystemTime::now(),
            scope: super::chat::ChatScope::Table(self.id.clone()),
        });
        
        // Keep only last 100 messages
        let len = chat.len();
        if len > 100 {
            chat.drain(0..len - 100);
        }
    }
    
    /// Convert table to a summary for client display
    pub fn to_summary(&self) -> Value {
        let seats = self.seats.read();
        let ready = self.ready_states.read();
        let status = self.status.read();
        let spectators = self.spectators.read();
        let countdown = self.countdown_ends_at.read();
        
        serde_json::json!({
            "id": self.id,
            "bundleId": self.bundle_id,
            "min": self.min_players,
            "max": self.max_players,
            "owner": self.owner,
            "status": *status,
            "seats": seats.iter().map(|s| match s {
                Some(SeatOccupant::Player(id)) => serde_json::json!(id),
                Some(SeatOccupant::Reserved) => serde_json::json!("reserved"),
                None => serde_json::json!(null),
            }).collect::<Vec<_>>(),
            "ready": ready.clone(),
            "spectators": spectators.clone(),
            "countdownEndsAt": countdown.map(|t| 
                t.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
            "playerCount": self.seated_count(),
            "createdAt": self.created_at
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_table_creation() {
        let bundle = Arc::new(crate::bundle::Bundle {
            game_id: "test-game".to_string(),
            manifest: crate::bundle::Manifest {
                game_id: "test-game".to_string(),
                version: "1.0.0".to_string(),
                spec_version: "1.0".to_string(),
                metadata: crate::bundle::ManifestMetadata {
                    name: "Test Game".to_string(),
                    author: "Test Author".to_string(),
                    description: "A test game".to_string(),
                    players: crate::bundle::PlayersRange { min: 2, max: 4 },
                },
                phases: None,
                setup: None,
                zone_groups: None,
            },
            zones: Value::Null,
            entities: Value::Null,
            actions: Value::Null,
            phases: Value::Null,
        });
        
        let table = TableInstance::new(
            "test-table".to_string(),
            "test-game@1.0.0".to_string(),
            bundle,
            "player1".to_string(),
            None,
            None,
            None,
        );
        
        assert_eq!(table.min_players, 2);
        assert_eq!(table.max_players, 4);
        assert_eq!(table.owner, "player1");
        assert_eq!(table.seated_count(), 0);
        assert!(!table.can_start());
    }
    
    #[test]
    fn test_seat_operations() {
        let bundle = Arc::new(crate::bundle::Bundle {
            game_id: "test-game".to_string(),
            manifest: crate::bundle::Manifest {
                game_id: "test-game".to_string(),
                version: "1.0.0".to_string(),
                spec_version: "1.0".to_string(),
                metadata: crate::bundle::ManifestMetadata {
                    name: "Test Game".to_string(),
                    author: "Test Author".to_string(),
                    description: "A test game".to_string(),
                    players: crate::bundle::PlayersRange { min: 2, max: 2 },
                },
                phases: None,
                setup: None,
                zone_groups: None,
            },
            zones: Value::Null,
            entities: Value::Null,
            actions: Value::Null,
            phases: Value::Null,
        });
        
        let table = TableInstance::new(
            "test-game@1.0.0".to_string(),
            "player1".to_string(),
            bundle,
            "test-table".to_string(),
            None,
            None,
            None,
        );
        
        // Add players to seats
        {
            let mut seats = table.seats.write();
            seats[0] = Some(SeatOccupant::Player("player1".to_string()));
            seats[1] = Some(SeatOccupant::Player("player2".to_string()));
        }
        
        assert_eq!(table.seated_count(), 2);
        assert!(table.is_full());
        assert!(table.can_start());
        assert!(table.is_player_seated("player1"));
        assert_eq!(table.get_player_seat("player1"), Some(0));
        assert_eq!(table.get_player_seat("player2"), Some(1));
    }
}
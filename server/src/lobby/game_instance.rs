//! Game instance management within a lobby

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use crate::bundle::Bundle;
use nanoid::nanoid;

/// Represents a single game instance within a lobby
#[derive(Clone)]
pub struct GameInstance {
    /// Unique ID for this game instance
    pub id: String,
    
    /// The game type (e.g., "tic-tac-toe", "go-fish")
    pub game_id: String,
    
    /// Current tick/version number
    pub tick: Arc<Mutex<u64>>,
    
    /// The actual game state
    pub state: Arc<Mutex<Value>>,
    
    /// Players in this game (maps player slot to lobby member ID)
    pub players: Arc<Mutex<GamePlayers>>,
    
    /// Game bundle reference
    pub bundle: Arc<Bundle>,
    
    /// When the game was created
    pub created_at: std::time::SystemTime,
    
    /// When the game ended (if applicable)
    pub ended_at: Option<std::time::SystemTime>,
}

/// Player mapping for a game instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayers {
    /// Maps player slots (p1, p2, etc.) to lobby member IDs
    pub slots: std::collections::HashMap<String, String>,
    
    /// Reverse mapping for quick lookup
    pub members: std::collections::HashMap<String, String>,
}

impl GamePlayers {
    pub fn new() -> Self {
        Self {
            slots: std::collections::HashMap::new(),
            members: std::collections::HashMap::new(),
        }
    }
    
    /// Add a player to the game
    pub fn add_player(&mut self, slot: String, member_id: String) {
        self.slots.insert(slot.clone(), member_id.clone());
        self.members.insert(member_id, slot);
    }
    
    /// Get the slot for a member
    pub fn get_slot(&self, member_id: &str) -> Option<&String> {
        self.members.get(member_id)
    }
    
    /// Get the member in a slot
    pub fn get_member(&self, slot: &str) -> Option<&String> {
        self.slots.get(slot)
    }
    
    /// Check if a member is playing
    pub fn is_playing(&self, member_id: &str) -> bool {
        self.members.contains_key(member_id)
    }
    
    /// Get all playing members
    pub fn get_all_members(&self) -> Vec<String> {
        self.members.keys().cloned().collect()
    }
}

impl GameInstance {
    /// Create a new game instance
    pub fn new(game_id: String, bundle: Arc<Bundle>) -> Self {
        Self {
            id: nanoid!(10),
            game_id,
            tick: Arc::new(Mutex::new(0)),
            state: Arc::new(Mutex::new(Value::Null)),
            players: Arc::new(Mutex::new(GamePlayers::new())),
            bundle,
            created_at: std::time::SystemTime::now(),
            ended_at: None,
        }
    }
    
    /// Check if the game has ended
    pub fn is_ended(&self) -> bool {
        if self.ended_at.is_some() {
            return true;
        }
        
        // Check game state for ended status
        let state = self.state.lock().unwrap();
        state.get("gameStatus")
            .and_then(|status| status.as_str())
            .map(|s| s == "ended" || s.starts_with("won:") || s == "tie" || s == "abandoned")
            .unwrap_or(false)
    }
    
    /// Mark the game as ended
    pub fn mark_ended(&mut self) {
        if self.ended_at.is_none() {
            self.ended_at = Some(std::time::SystemTime::now());
        }
    }
    
    /// Get the player count
    pub fn player_count(&self) -> usize {
        self.players.lock().unwrap().slots.len()
    }
    
    /// Check if game is full
    pub fn is_full(&self) -> bool {
        let player_count = self.player_count();
        let max_players = self.bundle.manifest.metadata.players.max as usize;
        player_count >= max_players
    }
    
    /// Check if game can start
    pub fn can_start(&self) -> bool {
        let player_count = self.player_count();
        let min_players = self.bundle.manifest.metadata.players.min as usize;
        player_count >= min_players && !self.is_ended()
    }
}
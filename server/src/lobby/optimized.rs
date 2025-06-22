use parking_lot::RwLock;
use serde_json::Value;
use std::sync::Arc;
use std::time::SystemTime;

/// Optimized lobby state with reduced lock contention
/// Groups related data under single locks to reduce contention
pub struct OptimizedLobbyState {
    /// Core game state (read-heavy, write on actions)
    pub game: Arc<RwLock<GameState>>,
    /// Player data (moderate read/write)
    pub players: Arc<RwLock<PlayerState>>,
    /// Metadata (rarely changes after creation)
    pub metadata: LobbyMetadata,
}

#[derive(Clone)]
pub struct GameState {
    pub state: Value,
    pub tick: usize,
    pub started: bool,
    pub current_player: Option<String>,
    pub game_status: Option<Value>,
}

#[derive(Clone)]
pub struct PlayerState {
    pub players: Vec<String>,
    pub preferences: std::collections::HashMap<String, super::PlayerPreferences>,
}

pub struct LobbyMetadata {
    pub id: String,
    pub game_id: String,
    pub game_name: String,
    pub created_at: SystemTime,
}

impl OptimizedLobbyState {
    /// Get a snapshot for lobby list without multiple lock acquisitions
    pub fn get_lobby_list_data(&self) -> LobbyListData {
        // Single read lock for game state
        let game = self.game.read();
        let started = game.started;
        let current_player = game.current_player.clone();
        let game_status = game.game_status.clone();
        drop(game);
        
        // Single read lock for players
        let players = self.players.read();
        let player_list = players.players.clone();
        drop(players);
        
        LobbyListData {
            id: self.metadata.id.clone(),
            game_id: self.metadata.game_id.clone(),
            game_name: self.metadata.game_name.clone(),
            players: player_list,
            started,
            current_player,
            game_status,
            created_at: self.metadata.created_at,
        }
    }
}

/// Data needed for lobby list display
#[derive(Clone)]
pub struct LobbyListData {
    pub id: String,
    pub game_id: String,
    pub game_name: String,
    pub players: Vec<String>,
    pub started: bool,
    pub current_player: Option<String>,
    pub game_status: Option<Value>,
    pub created_at: SystemTime,
}

impl LobbyListData {
    /// Convert to JSON for API response
    pub fn to_json(&self) -> Value {
        let mut json = serde_json::json!({
            "id": self.id,
            "game_id": self.game_id,
            "game_name": self.game_name,
            "name": format!("{} - {}", self.game_name, self.id),
            "players": self.players,
            "started": self.started,
            "created_at": self.created_at
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });
        
        if self.started {
            if let Some(ref status) = self.game_status {
                json["gameStatus"] = status.clone();
            }
            
            if let Some(ref current) = self.current_player {
                // Map actor ID to player name
                if current == "p1" && self.players.len() > 0 {
                    json["currentTurn"] = serde_json::json!(self.players[0]);
                } else if current == "p2" && self.players.len() > 1 {
                    json["currentTurn"] = serde_json::json!(self.players[1]);
                }
            }
        }
        
        json
    }
}
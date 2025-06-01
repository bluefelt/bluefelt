//! lobby.rs – minimal in-memory lobby with broadcast fan-out
//! Supports: welcome snapshot → JSON action → diff broadcast

use crate::{bundle::Bundle, engine};
use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex as TokioMutex};

pub type LobbyMap = DashMap<String, Arc<Lobby>>;

/* --------------------------------------------------------------------------
   constructor helper
   ----------------------------------------------------------------------- */
pub fn new_lobby(
    id: String,
    bundle: Bundle,
    lobbies: Arc<LobbyMap>,
    lobby_updates: broadcast::Sender<Message>,
) -> Arc<Lobby> {
    Arc::new(Lobby::new(id, bundle, lobbies, lobby_updates))
}

pub fn current_lobbies_json(lobbies: &LobbyMap) -> serde_json::Value {
    let list = lobbies
        .iter()
        .map(|l| {
            let lobby = l.value();
            let state = lobby.state.lock();
            
            println!("[DEBUG current_lobbies_json] Lobby {} started: {}", l.key(), lobby.is_started());
            
            // Extract game status and current turn
            let mut lobby_json = serde_json::json!({
                "id": l.key(),
                "game_id": lobby.bundle.game_id,
                "name": format!("{} - {}", lobby.bundle.game_id, l.key()),
                "players": lobby.player_list(),
                "started": lobby.is_started()
            });
            
            // Add current turn if game is in progress
            if lobby.is_started() {
                // Check for game status (ended state) FIRST
                let mut game_ended = false;
                
                // First check inside state.meta
                if let Some(meta) = state.get("meta") {
                    println!("[DEBUG current_lobbies_json] Lobby {} has meta", l.key());
                    if let Some(game_status) = meta.get("gameStatus") {
                        println!("[DEBUG current_lobbies_json] Lobby {} has gameStatus in meta: {:?}", l.key(), game_status);
                        lobby_json["gameStatus"] = game_status.clone();
                        if game_status["state"].as_str() == Some("ended") {
                            game_ended = true;
                        }
                    }
                } else {
                    println!("[DEBUG current_lobbies_json] Lobby {} has NO meta", l.key());
                }
                
                // Also check at top level (in case patches were applied there)
                if lobby_json.get("gameStatus").is_none() {
                    if let Some(game_status) = state.get("gameStatus") {
                        println!("[DEBUG current_lobbies_json] Lobby {} has gameStatus at top level: {:?}", l.key(), game_status);
                        lobby_json["gameStatus"] = game_status.clone();
                        if game_status["state"].as_str() == Some("ended") {
                            game_ended = true;
                        }
                    }
                }
                
                // Only add currentTurn if game has NOT ended
                if !game_ended {
                    if let Some(turn) = state.get("turn").and_then(|t| t.as_str()) {
                        // Map actor ID to player name
                        let players = lobby.player_list();
                        if turn == "p1" && players.len() > 0 {
                            lobby_json["currentTurn"] = json!(players[0]);
                        } else if turn == "p2" && players.len() > 1 {
                            lobby_json["currentTurn"] = json!(players[1]);
                        }
                    }
                }
            }
            
            lobby_json
        })
        .collect::<Vec<_>>();
    serde_json::Value::Array(list)
}

/* --------------------------------------------------------------------------
   Lobby struct
   ----------------------------------------------------------------------- */
pub struct Lobby {
    pub id: String,
    pub bundle: Bundle,

    /// authoritative mutable state (JSON object)
    state: Mutex<serde_json::Value>,

    /// broadcast channel for diff events
    tx: broadcast::Sender<Message>,
    
    /// Track connected players
    players: Mutex<Vec<String>>,
    
    /// Game has started flag
    game_started: Mutex<bool>,

    /// Incrementing tick for diff frames
    tick: Mutex<u64>,

    /// Stored diff history
    history: Mutex<Vec<serde_json::Value>>,

    /// Sender for lobby list updates
    lobby_updates: broadcast::Sender<Message>,

    /// Reference to lobby map for updates
    lobbies: Arc<LobbyMap>,
}

impl Lobby {
    pub fn new(
        id: String,
        bundle: Bundle,
        lobbies: Arc<LobbyMap>,
        lobby_updates: broadcast::Sender<Message>,
    ) -> Self {
        let initial = engine::load_initial_state(&bundle);
        let (tx, _) = broadcast::channel(64);
        Self {
            id,
            bundle,
            state: Mutex::new(initial),
            tx,
            players: Mutex::new(Vec::new()),
            game_started: Mutex::new(false),
            tick: Mutex::new(0),
            history: Mutex::new(Vec::new()),
            lobby_updates,
            lobbies,
        }
    }

    fn broadcast_lobby_list(&self) {
        let list = current_lobbies_json(&self.lobbies);
        let _ = self.lobby_updates.send(Message::Text(list.to_string()));
    }

    pub fn players(&self) -> usize {
        // Return the actual player count instead of subscribers
        let players = self.players.lock();
        players.len()
    }
    
    pub fn player_list(&self) -> Vec<String> {
        // Return a copy of the player list
        let players = self.players.lock();
        players.clone()
    }

    /// Map a player's username to their actor ID ("p1" or "p2")
    fn actor_for_player(&self, username: &str) -> Option<String> {
        let players = self.players.lock();
        players
            .iter()
            .position(|p| p == username)
            .map(|idx| format!("p{}", idx + 1))
    }

    pub fn add_player(&self, player_id: String) -> bool {
        let mut players = self.players.lock();
        
        // If this is the same player reconnecting, allow it
        if players.contains(&player_id) {
            println!("[Socket] Player {} is reconnecting to the lobby", player_id);
            return true;
        }
        
        // Check if we already have 2 players (max for tic-tac-toe)
        if players.len() < 2 {
            println!("[Socket] Adding new player {} to the lobby", player_id);
            players.push(player_id);
            drop(players);
            self.broadcast_lobby_list();
            return true;
        }
        
        println!("Could not add player {} - lobby is full", player_id);
        false
    }

    /// Optional method to remove a player - normally not needed as disconnections are handled implicitly
    pub fn remove_player(&self, player_id: &str) -> bool {
        let mut players = self.players.lock();
        let before_len = players.len();
        players.retain(|id| id != player_id);

        if players.len() < before_len {
            println!("[Socket] Player {} removed from lobby", player_id);
            drop(players);
            self.broadcast_lobby_list();
            return true;
        }
        
        println!("[Socket] ERROR: Player {} was not in the lobby and could not be removed", player_id);
        false
    }

    /// Compute action map for each player based on current state
    /// Returns a map from location (e.g., "/zones/board/0/1") to available actions
    fn compute_action_map(state: &serde_json::Value, bundle: &Bundle) -> serde_json::Map<String, serde_json::Value> {
        let mut player_action_maps = serde_json::Map::new();

        println!("[DEBUG action_map] Checking game state - meta exists: {}, meta.gameStatus exists: {}", 
            state.get("meta").is_some(),
            state.get("meta").and_then(|m| m.get("gameStatus")).is_some()
        );

        // Check if game has ended - check both possible locations
        let game_ended = 
            // Check in meta first
            state.get("meta")
                .and_then(|m| m.get("gameStatus"))
                .and_then(|gs| gs.get("state"))
                .and_then(|s| s.as_str())
                .map(|s| s == "ended")
                .unwrap_or(false) ||
            // Also check at top level (in case patches were applied there)
            state.get("gameStatus")
                .and_then(|gs| gs.get("state"))
                .and_then(|s| s.as_str())
                .map(|s| s == "ended")
                .unwrap_or(false);
                
        if game_ended {
            println!("[DEBUG] Game has ended, returning empty action maps for all players");
            // Game has ended, no moves possible
            if let Some(players) = state["players"].as_array() {
                for player in players {
                    if let Some(id) = player["id"].as_str() {
                        player_action_maps.insert(id.to_string(), json!({}));
                    }
                }
            }
            return player_action_maps;
        }

        let turn_player = state["turn"].as_str().unwrap_or("");
        println!("[DEBUG action_map] Current turn player: {}", turn_player);

        if let Some(players) = state["players"].as_array() {
            println!("[DEBUG action_map] Players in game: {:?}", players);
            for player in players {
                if let Some(id) = player["id"].as_str() {
                    println!("[DEBUG action_map] Checking actions for player: {}", id);
                    let mut action_map = serde_json::Map::new();
                    
                    if id == turn_player {
                        println!("[DEBUG action_map] Player {} is the current turn player", id);
                        let current_phase = state.get("meta")
                            .and_then(|m| m.get("currentPhase"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("play");
                        if let Some(actionlist) = bundle.actions.as_array() {
                            println!("[DEBUG action_map] Found {} actions in bundle, current phase: {}", actionlist.len(), current_phase);
                            for a in actionlist {
                                // Skip actions not in current phase
                                if let Some(action_phase) = a["phase"].as_str() {
                                    if action_phase != current_phase {
                                        continue;
                                    }
                                }
                                // Support both 'uses' (new) and 'builtin' (old)
                                let action_impl = a["uses"].as_str()
                                    .or_else(|| a["builtin"].as_str());
                                println!("[DEBUG action_map] Action {} has implementation: {:?}", a["id"].as_str().unwrap_or("unknown"), action_impl);
                                if action_impl == Some("grid.move") || action_impl == Some("moveEntity") {
                                    println!("[DEBUG] Found grid.move action: {:?}", a["id"]);
                                    if let Some(action_id) = a["id"].as_str() {
                                        // Support both 'with' (new) and 'params' (old)
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(target_zone) = params["target"]["zone"].as_str() {
                                                println!("[DEBUG] Target zone: {}", target_zone);
                                            let zone_state = &state["zones"][target_zone];
                                            println!("[DEBUG] Zone state for {}: {:?}", target_zone, zone_state);
                                            if zone_state.is_array() {
                                                let mut valid_options = Vec::new();
                                                let gravity = params["target"]["gravity"].as_bool().unwrap_or(false);
                                                
                                                if gravity {
                                                    // For gravity mode, only check top row (row 0) and exclude full columns
                                                    if let Some(grid) = zone_state.as_array() {
                                                        let cols = grid.get(0).and_then(|r| r.as_array()).map(|r| r.len()).unwrap_or(0);
                                                        for c in 0..cols {
                                                            // Check if column has any empty cell
                                                            let mut has_empty = false;
                                                            for row in grid {
                                                                if let Some(cells) = row.as_array() {
                                                                    if cells.get(c).map(|cell| cell.is_null()).unwrap_or(false) {
                                                                        has_empty = true;
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                            if has_empty {
                                                                valid_options.push(serde_json::json!({
                                                                    "zone": target_zone,
                                                                    "row": 0, // Always top row for gravity
                                                                    "col": c
                                                                }));
                                                            }
                                                        }
                                                    }
                                                } else if action_impl == Some("grid.move") || action_impl == Some("moveEntity") {
                                                    println!("[DEBUG] Checking for flip effects");
                                                    // Check if this has flip effect (Reversi-style)
                                                    let has_flip_effect = params.get("effects").and_then(|e| e.as_array())
                                                        .map(|effects| effects.iter().any(|e| e["type"].as_str() == Some("flip")))
                                                        .unwrap_or(false);
                                                    
                                                    if has_flip_effect {
                                                        // Only show moves that would flip at least one piece
                                                        let source_template = params["source"].as_str().unwrap_or("");
                                                        let source_id = source_template.replace("{actor}", id);
                                                        let player_piece = if let Some(z) = state["zones"].get(&source_id) {
                                                            z["infinite"].as_str().unwrap_or("")
                                                        } else {
                                                            ""
                                                        };
                                                        
                                                        if !player_piece.is_empty() {
                                                            for (r, row) in zone_state.as_array().unwrap().iter().enumerate() {
                                                                for (c, cell) in row.as_array().unwrap().iter().enumerate() {
                                                                    if cell.is_null() && would_flip_any(zone_state, r, c, player_piece) {
                                                                        valid_options.push(serde_json::json!({
                                                                            "zone": target_zone,
                                                                            "row": r,
                                                                            "col": c
                                                                        }));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        // Normal moveEntity without special effects
                                                        println!("[DEBUG] Normal grid.move without flip effects");
                                                        for (r, row) in zone_state.as_array().unwrap().iter().enumerate() {
                                                            for (c, cell) in row.as_array().unwrap().iter().enumerate() {
                                                                if cell.is_null() {
                                                                    println!("[DEBUG] Found empty cell at row {} col {}", r, c);
                                                                    valid_options.push(serde_json::json!({
                                                                        "zone": target_zone,
                                                                        "row": r,
                                                                        "col": c
                                                                    }));
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    // Normal mode - check all empty cells
                                                    for (r, row) in zone_state.as_array().unwrap().iter().enumerate() {
                                                        for (c, cell) in row.as_array().unwrap().iter().enumerate() {
                                                            if cell.is_null() {
                                                                valid_options.push(serde_json::json!({
                                                                    "zone": target_zone,
                                                                    "row": r,
                                                                    "col": c
                                                                }));
                                                            }
                                                        }
                                                    }
                                                }
                                                
                                                // Add each valid option to the action map
                                                let direction = a["ui"]["direction"].as_str().unwrap_or("Make a move");
                                                for option in valid_options {
                                                    if let (Some(zone), Some(row), Some(col)) = 
                                                        (option["zone"].as_str(), option["row"].as_u64(), option["col"].as_u64()) {
                                                        let location = format!("/zones/{}/{}/{}", zone, row, col);
                                                        action_map.insert(location, json!({
                                                            "action": action_id,
                                                            "direction": direction
                                                        }));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    }
                                } else if action_impl == Some("grid.select") || action_impl == Some("selectEntity") {
                                    // Handle piece selection for checkers
                                    if let Some(action_id) = a["id"].as_str() {
                                        // Support both 'with' (new) and 'params' (old)
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(zone_name) = params["zone"].as_str() {
                                            let zone_state = &state["zones"][zone_name];
                                            if zone_state.is_array() {
                                                let mut valid_options = Vec::new();
                                                
                                                // Find all of this player's pieces
                                                for (r, row) in zone_state.as_array().unwrap().iter().enumerate() {
                                                    for (c, cell) in row.as_array().unwrap().iter().enumerate() {
                                                        if let Some(entity) = cell.as_str() {
                                                            if entity.contains(&format!("_{}", id)) {
                                                                // This is the player's piece, check if it has valid moves
                                                                let is_king = entity.contains("king_");
                                                                let is_player1 = entity.contains("_p1");
                                                                let mut has_valid_move = false;
                                                                
                                                                // Check all possible moves for this piece
                                                                for dr in [-2i32, -1, 1, 2].iter() {
                                                                    for dc in [-2i32, -1, 1, 2].iter() {
                                                                        if dr.abs() != dc.abs() {
                                                                            continue; // Only diagonal moves
                                                                        }
                                                                        
                                                                        // Regular pieces can only move forward
                                                                        if !is_king {
                                                                            if (is_player1 && *dr > 0) || (!is_player1 && *dr < 0) {
                                                                                continue;
                                                                            }
                                                                        }
                                                                        
                                                                        let new_row = r as i32 + dr;
                                                                        let new_col = c as i32 + dc;
                                                                        
                                                                        if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                                                                            if let Some(target_cell) = zone_state.as_array()
                                                                                .and_then(|z| z.get(new_row as usize))
                                                                                .and_then(|row| row.as_array())
                                                                                .and_then(|row| row.get(new_col as usize)) {
                                                                                
                                                                                if target_cell.is_null() {
                                                                                    if dr.abs() == 2 {
                                                                                        // Check for capture
                                                                                        let mid_row = ((r as i32 + new_row) / 2) as usize;
                                                                                        let mid_col = ((c as i32 + new_col) / 2) as usize;
                                                                                        if let Some(mid_cell) = zone_state.as_array()
                                                                                            .and_then(|z| z.get(mid_row))
                                                                                            .and_then(|row| row.as_array())
                                                                                            .and_then(|row| row.get(mid_col))
                                                                                            .and_then(|cell| cell.as_str()) {
                                                                                            if !mid_cell.contains(&format!("_{}", id)) && !mid_cell.is_empty() {
                                                                                                has_valid_move = true;
                                                                                                break;
                                                                                            }
                                                                                        }
                                                                                    } else {
                                                                                        has_valid_move = true;
                                                                                        break;
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    if has_valid_move {
                                                                        break;
                                                                    }
                                                                }
                                                                
                                                                if has_valid_move {
                                                                    valid_options.push(serde_json::json!({
                                                                        "zone": zone_name,
                                                                        "row": r,
                                                                        "col": c
                                                                    }));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                
                                                // Add each valid option to the action map
                                                let direction = a["ui"]["direction"].as_str().unwrap_or("Select a piece");
                                                for option in valid_options {
                                                    if let (Some(zone), Some(row), Some(col)) = 
                                                        (option["zone"].as_str(), option["row"].as_u64(), option["col"].as_u64()) {
                                                        let location = format!("/zones/{}/{}/{}", zone, row, col);
                                                        action_map.insert(location, json!({
                                                            "action": action_id,
                                                            "direction": direction
                                                        }));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    }
                                } else if action_impl == Some("grid.moveSelected") || action_impl == Some("moveSelectedEntity") {
                                    // Handle moving a selected piece
                                    if let Some(action_id) = a["id"].as_str() {
                                        // Check if there's a selection for this player
                                        if let Some(selection) = state.get("meta").and_then(|m| m.get("selection")) {
                                            if selection["actor"].as_str() == Some(id) {
                                                let source_row = selection["row"].as_u64().unwrap_or(0) as usize;
                                                let source_col = selection["col"].as_u64().unwrap_or(0) as usize;
                                                
                                                // Support both 'with' (new) and 'params' (old)
                                                let params = a.get("with").or_else(|| a.get("params"));
                                                if let Some(params) = params {
                                                    if let Some(target_zone) = params["target"]["zone"].as_str() {
                                                    let zone_state = &state["zones"][target_zone];
                                                    if zone_state.is_array() {
                                                        let mut valid_options = Vec::new();
                                                        
                                                        // Get the selected piece to check if it's a king
                                                        let (is_king, is_player1) = {
                                                            if let Some(selected_piece) = zone_state.as_array()
                                                                .and_then(|z| z.get(source_row))
                                                                .and_then(|r| r.as_array())
                                                                .and_then(|r| r.get(source_col))
                                                                .and_then(|c| c.as_str()) {
                                                                (selected_piece.contains("king_"), selected_piece.contains("_p1"))
                                                            } else {
                                                                (false, false)
                                                            }
                                                        };
                                                        
                                                        let must_capture_only = is_king;
                                                        
                                                        let mut capture_moves = Vec::new();
                                                        let mut regular_moves = Vec::new();
                                                        
                                                        // Calculate valid moves (diagonals only for checkers)
                                                        for dr in [-2i32, -1, 1, 2].iter() {
                                                            for dc in [-2i32, -1, 1, 2].iter() {
                                                                if dr.abs() != dc.abs() {
                                                                    continue; // Only diagonal moves
                                                                }
                                                                
                                                                // Regular pieces can only move forward
                                                                if !is_king {
                                                                    // Player 1 pieces move up (negative dr), Player 2 pieces move down (positive dr)
                                                                    if (is_player1 && *dr > 0) || (!is_player1 && *dr < 0) {
                                                                        continue; // Skip backward moves for regular pieces
                                                                    }
                                                                }
                                                                
                                                                let new_row = source_row as i32 + dr;
                                                                let new_col = source_col as i32 + dc;
                                                                
                                                                if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                                                                    let target_row = new_row as usize;
                                                                    let target_col = new_col as usize;
                                                                    
                                                                    // Check if target is empty
                                                                    if let Some(target_cell) = zone_state.as_array()
                                                                        .and_then(|z| z.get(target_row))
                                                                        .and_then(|r| r.as_array())
                                                                        .and_then(|r| r.get(target_col)) {
                                                                        
                                                                        if target_cell.is_null() {
                                                                            // For captures, check if there's an opponent piece to jump
                                                                            if dr.abs() == 2 {
                                                                                let mid_row = ((source_row as i32 + new_row) / 2) as usize;
                                                                                let mid_col = ((source_col as i32 + new_col) / 2) as usize;
                                                                                
                                                                                if let Some(mid_cell) = zone_state.as_array()
                                                                                    .and_then(|z| z.get(mid_row))
                                                                                    .and_then(|r| r.as_array())
                                                                                    .and_then(|r| r.get(mid_col))
                                                                                    .and_then(|c| c.as_str()) {
                                                                                    
                                                                                    // Check if it's opponent's piece
                                                                                    if !mid_cell.contains(&format!("_{}", id)) && !mid_cell.is_empty() {
                                                                                        capture_moves.push(serde_json::json!({
                                                                                            "zone": target_zone,
                                                                                            "row": target_row,
                                                                                            "col": target_col
                                                                                        }));
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                // Regular move
                                                                                regular_moves.push(serde_json::json!({
                                                                                    "zone": target_zone,
                                                                                    "row": target_row,
                                                                                    "col": target_col
                                                                                }));
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        
                                                        // If this is a king that can capture, only show capture moves
                                                        if must_capture_only && !capture_moves.is_empty() {
                                                            valid_options = capture_moves;
                                                        } else if !capture_moves.is_empty() || !regular_moves.is_empty() {
                                                            valid_options = capture_moves;
                                                            valid_options.extend(regular_moves);
                                                        }
                                                        
                                                        // Add each valid option to the action map
                                                        let direction = a["ui"]["direction"].as_str().unwrap_or("Move piece");
                                                        for option in valid_options {
                                                            if let (Some(zone), Some(row), Some(col)) = 
                                                                (option["zone"].as_str(), option["row"].as_u64(), option["col"].as_u64()) {
                                                                let location = format!("/zones/{}/{}/{}", zone, row, col);
                                                                action_map.insert(location, json!({
                                                                    "action": action_id,
                                                                    "direction": direction
                                                                }));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    }
                                } else if action_impl == Some("deck.draw") || action_impl == Some("drawCard") {
                                    if let Some(action_id) = a["id"].as_str() {
                                        // For drawCard, check if source deck has cards
                                        // Support both 'with' (new) and 'params' (old)
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(source) = params["source"].as_str() {
                                            if let Some(deck) = state["zones"][source].get("items").and_then(|i| i.as_array()) {
                                                if !deck.is_empty() {
                                                    let direction = a["ui"]["direction"].as_str().unwrap_or("Draw a card");
                                                    // For non-grid zones, use the zone itself as the location
                                                    let location = format!("/zones/{}", source);
                                                    action_map.insert(location, json!({
                                                        "action": action_id,
                                                        "direction": direction
                                                    }));
                                                }
                                            }
                                        }
                                    }
                                    }
                                } else if action_impl == Some("entity.move") {
                                    if let Some(action_id) = a["id"].as_str() {
                                        println!("[DEBUG action_map] Processing entity.move action: {}", action_id);
                                        // Handle entity.move actions for card games
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(source) = params["source"].as_str() {
                                                let source_zone = source.replace("{actor}", id);
                                                println!("[DEBUG action_map] Source zone: {} (from {})", source_zone, source);
                                                
                                                // Check if this is a zone-level action (drawing from deck/discard)
                                                if source_zone == "drawPile" || source_zone == "discardPile" {
                                                    if let Some(zone_data) = state["zones"][&source_zone].as_object() {
                                                        if let Some(items) = zone_data.get("items").and_then(|i| i.as_array()) {
                                                            println!("[DEBUG action_map] Zone {} has {} items", source_zone, items.len());
                                                            if !items.is_empty() {
                                                                // Check conditions before adding action
                                                                let mut conditions_met = true;
                                                                if let Some(conditions) = a.get("conditions").and_then(|c| c.as_array()) {
                                                                    for condition in conditions {
                                                                        if let Some(cond_type) = condition.get("type").and_then(|t| t.as_str()) {
                                                                            if cond_type == "zone.count" {
                                                                                if let Some(with) = condition.get("with") {
                                                                                    let zone_id = with.get("zone").and_then(|z| z.as_str()).unwrap_or("").replace("{actor}", id);
                                                                                    // Get the zone we're checking
                                                                                    if let Some(check_zone) = state["zones"].get(&zone_id) {
                                                                                        if let Some(check_items) = check_zone.get("items").and_then(|i| i.as_array()) {
                                                                                            let count = check_items.len();
                                                                                            if let Some(exact) = with.get("exact").and_then(|e| e.as_u64()) {
                                                                                                println!("[DEBUG conditions] Checking zone.count exact: {} == {}", count, exact);
                                                                                                if count != exact as usize {
                                                                                                    conditions_met = false;
                                                                                                    break;
                                                                                                }
                                                                                            }
                                                                                            if let Some(min) = with.get("min").and_then(|m| m.as_u64()) {
                                                                                                if count < min as usize {
                                                                                                    conditions_met = false;
                                                                                                    break;
                                                                                                }
                                                                                            }
                                                                                            if let Some(max) = with.get("max").and_then(|m| m.as_u64()) {
                                                                                                if count > max as usize {
                                                                                                    conditions_met = false;
                                                                                                    break;
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                
                                                                if conditions_met {
                                                                    let direction = a["ui"]["direction"].as_str().unwrap_or("Select");
                                                                    let location = format!("/zones/{}", source_zone);
                                                                    println!("[DEBUG action_map] Adding zone action at {} -> {}", location, action_id);
                                                                    action_map.insert(location, json!({
                                                                        "action": action_id,
                                                                        "direction": direction
                                                                    }));
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else if source_zone.starts_with("hand_") {
                                                    // This is for discarding cards from hand
                                                    if let Some(zone_data) = state["zones"][&source_zone].as_object() {
                                                        if let Some(items) = zone_data.get("items").and_then(|i| i.as_array()) {
                                                            println!("[DEBUG action_map] Hand {} has {} items", source_zone, items.len());
                                                            
                                                            // Check conditions before adding action
                                                            let mut conditions_met = true;
                                                            if let Some(conditions) = a.get("conditions").and_then(|c| c.as_array()) {
                                                                for condition in conditions {
                                                                    if let Some(cond_type) = condition.get("type").and_then(|t| t.as_str()) {
                                                                        if cond_type == "zone.count" {
                                                                            if let Some(with) = condition.get("with") {
                                                                                let zone_id = with.get("zone").and_then(|z| z.as_str()).unwrap_or("").replace("{actor}", id);
                                                                                if zone_id == source_zone {
                                                                                    if let Some(exact) = with.get("exact").and_then(|e| e.as_u64()) {
                                                                                        if items.len() != exact as usize {
                                                                                            conditions_met = false;
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                    if let Some(min) = with.get("min").and_then(|m| m.as_u64()) {
                                                                                        if items.len() < min as usize {
                                                                                            conditions_met = false;
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                    if let Some(max) = with.get("max").and_then(|m| m.as_u64()) {
                                                                                        if items.len() > max as usize {
                                                                                            conditions_met = false;
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            
                                                            if conditions_met {
                                                                let direction = a["ui"]["direction"].as_str().unwrap_or("Select card");
                                                                // Add action for each card in hand
                                                                for (index, _card) in items.iter().enumerate() {
                                                                    let location = format!("/zones/{}/{}", source_zone, index);
                                                                    action_map.insert(location, json!({
                                                                        "action": action_id,
                                                                        "direction": direction
                                                                    }));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Insert the action map for this player
                    player_action_maps.insert(id.to_string(), json!(action_map));
                }
            }
        }

        player_action_maps
    }

    /// Check if the game has started
    pub fn is_started(&self) -> bool {
        *self.game_started.lock()
    }
    
    /// Build a welcome message with consistent meta handling
    fn build_welcome_message(&self, player_id: &str, include_state: bool) -> serde_json::Value {
        let actor = self.actor_for_player(player_id)
            .unwrap_or_else(|| "spectator".to_string());
        
        if include_state {
            let snapshot = { let g = self.state.lock(); g.clone() };
            let action_map = Lobby::compute_action_map(&snapshot, &self.bundle);
            
            // Build meta object preserving existing meta data
            let mut meta = if let Some(existing_meta) = snapshot.get("meta") {
                existing_meta.clone()
            } else {
                json!({})
            };
            
            // Update with current data
            if let Some(meta_obj) = meta.as_object_mut() {
                meta_obj.insert("actionMap".to_string(), json!(action_map));
                meta_obj.insert("players".to_string(), json!(self.player_list()));
                meta_obj.insert("entities".to_string(), self.bundle.entities.clone());
                meta_obj.insert("zones".to_string(), self.bundle.zones.clone());
                // Ensure gameLog exists
                if !meta_obj.contains_key("gameLog") {
                    meta_obj.insert("gameLog".to_string(), json!([]));
                }
            }
            
            json!({
                "type": "welcome",
                "you": actor,
                "started": true,
                "state": snapshot,
                "meta": meta,
                "tick": *self.tick.lock()
            })
        } else {
            json!({
                "type": "welcome",
                "you": actor,
                "started": false,
                "meta": {
                    "players": self.player_list(),
                    "entities": self.bundle.entities.clone()
                }
            })
        }
    }

pub fn start_game(&self) {
    *self.game_started.lock() = true;
    
    // Get current game state
    let mut snapshot = { let g = self.state.lock(); g.clone() };
    
    // Check if we need to trigger initial phase
    let mut all_patches = Vec::new();
    if let Some(current_phase) = snapshot.get("meta")
        .and_then(|m| m.get("currentPhase"))
        .and_then(|p| p.as_str()) {
        // Look for phase definition in manifest
        if let Some(phases) = self.bundle.manifest.phases.as_ref() {
            if let Some(phases_array) = phases.as_array() {
                for phase_def in phases_array {
                    if phase_def["id"].as_str() == Some(current_phase) {
                        // Execute phase actions (then)
                        if let Some(then_actions) = phase_def.get("then").and_then(|t| t.as_array()) {
                            for then_action in then_actions {
                                if let Some(action_id) = then_action.get("action").and_then(|a| a.as_str()) {
                                    // Execute the triggered action
                                    let trigger_action = json!({
                                        "action": action_id,
                                        "actor": "system"
                                    });
                                    let patches = engine::apply_action(&self.bundle, &mut snapshot, "system", &trigger_action);
                                    // Accumulate patches to send to clients
                                    if let Some(patch_array) = patches.as_array() {
                                        all_patches.extend_from_slice(patch_array);
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
    
    // Update state with any changes from phase triggers
    *self.state.lock() = snapshot.clone();
    
    // Get action map after phase triggers
    let action_map = Lobby::compute_action_map(&snapshot, &self.bundle);
    
    // Build meta object preserving existing meta data
    let mut meta = if let Some(existing_meta) = snapshot.get("meta") {
        existing_meta.clone()
    } else {
        json!({})
    };
    
    // Update with current data
    if let Some(meta_obj) = meta.as_object_mut() {
        meta_obj.insert("actionMap".to_string(), json!(action_map));
        meta_obj.insert("players".to_string(), json!(self.player_list()));
        meta_obj.insert("entities".to_string(), self.bundle.entities.clone());
        meta_obj.insert("zones".to_string(), self.bundle.zones.clone());
        meta_obj.insert("gameLog".to_string(), json!([
            {
                "message": "The game has started",
                "timestamp": chrono::Local::now().format("%H:%M").to_string()
            }
        ])); // Initialize game log with start message
    }
    
    // Send full game state to all connected clients
    let game_started_msg = serde_json::json!({
        "type": "gameStarted",
        "state": snapshot,
        "meta": meta
    });
    let _ = self.tx.send(Message::Text(game_started_msg.to_string()));
    
    // Send patches from phase triggers if any
    if !all_patches.is_empty() {
        println!("[DEBUG] Sending {} patches from phase triggers", all_patches.len());
        let patch_msg = serde_json::json!({
            "type": "patch",
            "patches": all_patches
        });
        let _ = self.tx.send(Message::Text(patch_msg.to_string()));
    }
    
    self.broadcast_lobby_list();
}

/// Accept a new WebSocket client, drive send/recv loops.
    pub async fn accept_client(self: Arc<Self>, socket: WebSocket, player_id: String, join: bool, since: u64) {
        println!("[Lobby] Client {} connecting to lobby {} (join={}, since={})", player_id, self.id, join, since);
        let (ws_tx, mut rx) = socket.split();
        let tx = Arc::new(TokioMutex::new(ws_tx));

        if join {
            if self.add_player(player_id.clone()) {
                // Broadcast player update to all connected clients
                let player_update = serde_json::json!({
                    "type": "playerUpdate",
                    "players": self.player_list()
                });
                let _ = self.tx.send(Message::Text(player_update.to_string()));
            }
        }

        // Note: actor will be determined dynamically as it can change when players join/leave

        // Send initial welcome message and get current tick
        let welcome = self.build_welcome_message(&player_id, self.is_started());
        let current_tick = *self.tick.lock();
        let _ = tx.lock().await.send(Message::Text(welcome.to_string())).await;

        // replay diff history since the provided tick
        // Only replay diffs that are newer than both 'since' and the current tick
        // This prevents duplicating data that's already in the welcome message
        let replay_after = since.max(current_tick);
        let frames = {
            let history = self.history.lock();
            history.clone()
        };
        for frame in frames.into_iter() {
            if frame["tick"].as_u64().unwrap_or(0) > replay_after {
                let _ = tx
                    .lock()
                    .await
                    .send(Message::Text(frame.to_string()))
                    .await;
            }
        }

        // forward broadcast messages
        let mut bcast = self.tx.subscribe();
        println!("[Lobby] Client {} subscribed to broadcasts, receiver count: {}", player_id, self.tx.receiver_count());
        let tx_forward = tx.clone();
        let self_ref = self.clone();
        let player_id_clone = player_id.clone();
        let forward = tokio::spawn(async move {
            while let Ok(msg) = bcast.recv().await {
                // Handle gameStarted message specially to include correct "you" field
                if let Message::Text(text) = &msg {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                        if json["type"] == "gameStarted" {
                            let actor = self_ref
                                .actor_for_player(&player_id_clone)
                                .unwrap_or_else(|| "spectator".to_string());
                            let mut personalized = json.clone();
                            personalized["you"] = serde_json::Value::String(actor);
                            let personalized_msg = Message::Text(personalized.to_string());
                            if tx_forward.lock().await.send(personalized_msg).await.is_err() {
                                break;
                            }
                            continue;
                        }
                    }
                }
                
                if tx_forward.lock().await.send(msg.clone()).await.is_err() {
                    break;
                }
            }
        });

        // handle incoming messages
        while let Some(Ok(message)) = rx.next().await {
            if let Message::Text(text) = message {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json["action"] == "join" {
                        if self.add_player(player_id.clone()) {
                            self.broadcast_lobby_list();
                            
                            // Send updated welcome message to the joining player
                            let welcome = self.build_welcome_message(&player_id, self.is_started());
                            let _ = tx.lock().await.send(Message::Text(welcome.to_string())).await;
                            
                            // Broadcast player update to all OTHER connected clients
                            let player_update = serde_json::json!({
                                "type": "playerUpdate",
                                "players": self.player_list()
                            });
                            let _ = self.tx.send(Message::Text(player_update.to_string()));
                        }
                    } else if json["action"] == "leave" {
                        if self.remove_player(&player_id) {
                            self.broadcast_lobby_list();
                            // Broadcast player update to all connected clients
                            let player_update = serde_json::json!({
                                "type": "playerUpdate",
                                "players": self.player_list()
                            });
                            let _ = self.tx.send(Message::Text(player_update.to_string()));
                        }
                    } else if json["action"] == "start_game" {
                        if !self.is_started() && self.players() >= 2 {
                            self.start_game();
                        }
                    } else if json.get("action").is_some() && self.is_started() {
                        // Get current actor assignment
                        let current_actor = self
                            .actor_for_player(&player_id)
                            .unwrap_or_else(|| "spectator".to_string());
                        
                        println!("[Socket] Processing action from {} (actor: {}): {:?}", player_id, current_actor, json);
                        let current_turn = self.state.lock()["turn"].as_str().unwrap_or("").to_string();
                        println!("[Socket] Current turn: {}, Actor attempting move: {}", current_turn, current_actor);
                        
                        
                        // Store action info for game log generation
                        let action_id = json["action"].as_str().unwrap_or("");
                        let args = json.get("args").cloned();
                        
                        let diff =
                            engine::apply_action(&self.bundle, &mut self.state.lock(), &current_actor, &json);
                        println!("[Socket] Action result diff: {:?}", diff);
                        
                        let tick = {
                            let mut t = self.tick.lock();
                            *t += 1;
                            *t
                        };
                        let mut patch_ops = Vec::new();
                        if let Some(arr) = diff.as_array() {
                            for op in arr {
                                println!("[DEBUG] Original patch: {:?}", op);
                                if let Some(path) = op.get("path").and_then(|p| p.as_str()) {
                                    let mut op_obj = op.clone();
                                    // Only prefix with /state if the path doesn't start with /meta
                                    if !path.starts_with("/meta") {
                                        op_obj["path"] =
                                            serde_json::Value::String(format!("/state{}", path));
                                    }
                                    println!("[DEBUG] Transformed patch: {:?}", op_obj);
                                    patch_ops.push(op_obj);
                                } else {
                                    patch_ops.push(op.clone());
                                }
                            }
                        }
                        
                        // Check if game just ended in the patches
                        let mut game_ended = false;
                        let mut game_end_info = None;
                        for op in &patch_ops {
                            let path = op.get("path").and_then(|p| p.as_str()).unwrap_or("");
                            let is_game_status = path == "/meta/gameStatus" || path == "/state/meta/gameStatus";
                            if is_game_status {
                                println!("[DEBUG] Found gameStatus patch at path: {}, value: {:?}", path, op.get("value"));
                                if let Some(value) = op.get("value") {
                                    if value.get("state").and_then(|s| s.as_str()) == Some("ended") {
                                        game_ended = true;
                                        game_end_info = Some(value.clone());
                                        break;
                                    }
                                }
                            }
                        }
                        
                        println!("[DEBUG] Game ended check: {}", game_ended);
                        
                        // Only compute possible actions if game hasn't ended
                        if game_ended {
                            println!("[DEBUG] Game has ended! Setting empty possibleActions for all players");
                            // Game just ended, set empty possibleActions for all players
                            if let Some(players) = self.state.lock()["players"].as_array() {
                                for player in players {
                                    if let Some(id) = player["id"].as_str() {
                                        println!("[DEBUG] Setting empty possibleActions for player: {}", id);
                                        patch_ops.push(serde_json::json!({
                                            "op": "replace",
                                            "path": format!("/meta/possibleActions/{}", id),
                                            "value": []
                                        }));
                                    }
                                }
                            }
                        } else {
                            // Game still active, compute action map normally
                            let current_state = { let g = self.state.lock(); g.clone() };
                            let action_map = Lobby::compute_action_map(&current_state, &self.bundle);
                            for (pid, actions) in action_map {
                                patch_ops.push(serde_json::json!({
                                    "op": "replace",
                                    "path": format!("/meta/actionMap/{}", pid),
                                    "value": actions
                                }));
                            }
                        }
                        
                        // Generate game log entry if action was successful and has a log template
                        if !diff.as_array().map(|a| a.is_empty()).unwrap_or(true) {
                            if let Some(action_spec) = self.bundle.actions.as_array()
                                .and_then(|actions| actions.iter().find(|a| a["id"].as_str() == Some(action_id))) {
                                if let Some(log_template) = action_spec["ui"]["logTemplate"].as_str() {
                                    // Build the log entry
                                    let mut log_text = log_template.to_string();
                                    
                                    // Replace {player} with the player name
                                    log_text = log_text.replace("{player}", &player_id);
                                    
                                    // Replace args like {row} and {col}
                                    if let Some(args_obj) = args {
                                        if let Some(row) = args_obj["row"].as_i64() {
                                            log_text = log_text.replace("{row}", &(row + 1).to_string()); // 1-indexed for display
                                        }
                                        if let Some(col) = args_obj["col"].as_i64() {
                                            log_text = log_text.replace("{col}", &(col + 1).to_string()); // 1-indexed for display
                                        }
                                    }
                                    
                                    // Create timestamp
                                    let now = chrono::Local::now();
                                    let timestamp = now.format("%H:%M").to_string();
                                    
                                    // Add game log entry to patches
                                    patch_ops.push(serde_json::json!({
                                        "op": "add",
                                        "path": "/meta/gameLog/-",
                                        "value": {
                                            "player": player_id,
                                            "actor": current_actor,
                                            "message": log_text,
                                            "timestamp": timestamp
                                        }
                                    }));
                                }
                            }
                        }
                        
                        // Add game end log entry if game just ended
                        if game_ended {
                            if let Some(end_info) = game_end_info {
                                let log_message = if let Some(winner) = end_info.get("winner").and_then(|w| w.as_str()) {
                                    // Map actor ID to player name
                                    let winner_name = if winner == "p1" && self.player_list().len() > 0 {
                                        self.player_list()[0].clone()
                                    } else if winner == "p2" && self.player_list().len() > 1 {
                                        self.player_list()[1].clone()
                                    } else {
                                        winner.to_string()
                                    };
                                    format!("{} wins!", winner_name)
                                } else {
                                    "Game ended in a tie!".to_string()
                                };
                                
                                patch_ops.push(serde_json::json!({
                                    "op": "add",
                                    "path": "/meta/gameLog/-",
                                    "value": {
                                        "message": log_message,
                                        "timestamp": chrono::Local::now().format("%H:%M").to_string()
                                    }
                                }));
                            }
                        }
                        
                        
                        let frame = serde_json::json!({"type": "diff", "tick": tick, "patch": patch_ops});
                        self.history.lock().push(frame.clone());
                        println!("[Lobby] Broadcasting diff with tick {} to {} receivers", tick, self.tx.receiver_count());
                        let _ = self.tx.send(Message::Text(frame.to_string()));
                        
                        // If game ended, broadcast updated lobby list
                        if game_ended {
                            println!("[Lobby] Game ended, broadcasting updated lobby list");
                            self.broadcast_lobby_list();
                        }
                    }
                }
            }
        }

        forward.abort();
    }
}

// Helper function to check if placing a piece at (row, col) would flip any opponent pieces
fn would_flip_any(board: &serde_json::Value, row: usize, col: usize, player_piece: &str) -> bool {
    let directions = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1)
    ];
    
    let board_array = match board.as_array() {
        Some(arr) => arr,
        None => return false,
    };
    
    let board_size = board_array.len();
    let opponent_piece = if player_piece.contains("_p1") {
        player_piece.replace("_p1", "_p2")
    } else {
        player_piece.replace("_p2", "_p1")
    };
    
    for (dr, dc) in directions.iter() {
        let mut r = row as i32 + dr;
        let mut c = col as i32 + dc;
        let mut found_opponent = false;
        
        while r >= 0 && r < board_size as i32 && c >= 0 && c < board_size as i32 {
            let row_idx = r as usize;
            let col_idx = c as usize;
            
            if let Some(row_array) = board_array[row_idx].as_array() {
                if col_idx < row_array.len() {
                    match row_array[col_idx].as_str() {
                        Some(piece) if piece == opponent_piece => {
                            found_opponent = true;
                        }
                        Some(piece) if piece == player_piece => {
                            if found_opponent {
                                return true; // Would flip at least one piece
                            }
                            break;
                        }
                        _ => break, // Empty cell or edge
                    }
                }
            }
            
            r += dr;
            c += dc;
        }
    }
    
    false
}

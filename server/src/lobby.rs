//! lobby.rs – minimal in-memory lobby with broadcast fan-out
//! Supports: welcome snapshot → JSON action → diff broadcast

use crate::{bundle::Bundle, engine, message_format::{MessageFormat, UpdateFormat, format_welcome_message, patch_to_full_state}};
use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex as TokioMutex};
use std::collections::HashMap;

pub type LobbyMap = DashMap<String, Arc<Lobby>>;

/// Client connection preferences
#[derive(Clone)]
struct ClientInfo {
    message_format: MessageFormat,
    update_format: UpdateFormat,
}

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
                
                // Check for game status at top level
                if let Some(game_status) = state.get("gameStatus") {
                    println!("[DEBUG current_lobbies_json] Lobby {} has gameStatus: {:?}", l.key(), game_status);
                    lobby_json["gameStatus"] = game_status.clone();
                    if game_status["state"].as_str() == Some("ended") {
                        game_ended = true;
                    }
                }
                
                // Only add currentTurn if game has NOT ended
                if !game_ended {
                    if let Some(current_player) = state.get("currentPlayer").and_then(|t| t.as_str()) {
                        // Map actor ID to player name
                        let players = lobby.player_list();
                        if current_player == "p1" && players.len() > 0 {
                            lobby_json["currentTurn"] = json!(players[0]);
                        } else if current_player == "p2" && players.len() > 1 {
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
    
    /// Client format preferences
    client_formats: Mutex<HashMap<String, ClientInfo>>,

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
            client_formats: Mutex::new(HashMap::new()),
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

    /// Expand zone groups with player templates to include actual player IDs
    fn expand_zone_metadata(zones: &serde_json::Value, players: &[String]) -> serde_json::Value {
        if let Some(zones_array) = zones.as_array() {
            let mut expanded_zones = Vec::new();
            
            for zone in zones_array {
                if let Some(zone_obj) = zone.as_object() {
                    let zone_id = zone_obj.get("id").and_then(|id| id.as_str()).unwrap_or("");
                    
                    if zone_id.contains("{player}") {
                        // Expand for each player
                        for (idx, _player) in players.iter().enumerate() {
                            let player_id = format!("p{}", idx + 1);
                            let mut player_zone = zone_obj.clone();
                            
                            // Replace {player} in id
                            if let Some(id_val) = player_zone.get_mut("id") {
                                if let Some(id_str) = id_val.as_str() {
                                    *id_val = json!(id_str.replace("{player}", &player_id));
                                }
                            }
                            
                            // Replace {player} in name
                            if let Some(name_val) = player_zone.get_mut("name") {
                                if let Some(name_str) = name_val.as_str() {
                                    *name_val = json!(name_str.replace("{player}", &format!("Player {}", idx + 1)));
                                }
                            }
                            
                            expanded_zones.push(json!(player_zone));
                        }
                    } else {
                        // Keep zone as-is
                        expanded_zones.push(zone.clone());
                    }
                }
            }
            
            json!(expanded_zones)
        } else {
            zones.clone()
        }
    }
    
    fn expand_zone_groups(zone_groups: &serde_json::Value, players: &[String]) -> serde_json::Value {
        if let Some(groups_array) = zone_groups.as_array() {
            let mut expanded_groups = Vec::new();
            
            for group in groups_array {
                if let Some(group_obj) = group.as_object() {
                    // Check if this group has {player} template in title or zones
                    let title = group_obj.get("title").and_then(|t| t.as_str()).unwrap_or("");
                    let has_player_template = title.contains("{player}") || 
                        group_obj.get("zones")
                            .and_then(|z| z.as_array())
                            .map(|zones| zones.iter().any(|z| z.as_str().map(|s| s.contains("{player}")).unwrap_or(false)))
                            .unwrap_or(false);
                    
                    if has_player_template {
                        // Create a separate group for each player
                        for (idx, player) in players.iter().enumerate() {
                            let player_id = format!("p{}", idx + 1);
                            let mut player_group = group_obj.clone();
                            
                            // Replace {player} in title
                            if let Some(title_val) = player_group.get_mut("title") {
                                if let Some(title_str) = title_val.as_str() {
                                    *title_val = json!(title_str.replace("{player}", player));
                                }
                            }
                            
                            // Replace {player} in id to make it unique
                            if let Some(id_val) = player_group.get_mut("id") {
                                if let Some(id_str) = id_val.as_str() {
                                    *id_val = json!(format!("{}_{}", id_str, player_id));
                                }
                            }
                            
                            // Replace {player} in zones
                            if let Some(zones_val) = player_group.get_mut("zones") {
                                if let Some(zones) = zones_val.as_array_mut() {
                                    for zone in zones.iter_mut() {
                                        if let Some(zone_str) = zone.as_str() {
                                            *zone = json!(zone_str.replace("{player}", &player_id));
                                        }
                                    }
                                }
                            }
                            
                            expanded_groups.push(json!(player_group));
                        }
                    } else {
                        // Keep group as-is
                        expanded_groups.push(group.clone());
                    }
                }
            }
            
            json!(expanded_groups)
        } else {
            zone_groups.clone()
        }
    }

    /// Get the prompt for the current phase if it's a playerAction phase
    fn get_current_phase_prompt(state: &serde_json::Value, bundle: &Bundle) -> Option<String> {
        // Get current phase states
        let phase_states = state.get("phases")?;
        
        // Look through all active phases to find player action phases with prompts
        if let Some(states_obj) = phase_states.as_object() {
            for (_track, phase_id) in states_obj {
                if let Some(current_phase_id) = phase_id.as_str() {
                    // Find the phase definition by searching all phase sets
                    if let Some(phase_sets_array) = bundle.phases.as_array() {
                        for phase_set in phase_sets_array {
                            if let Some(phases_array) = phase_set["phases"].as_array() {
                                if let Some(phase_def) = phases_array.iter().find(|p| p["id"].as_str() == Some(current_phase_id)) {
                                    // Check for different prompt sources
                                    
                                    // 1. Legacy type: "playerAction" with direct prompt field
                                    if phase_def["type"].as_str() == Some("playerAction") {
                                        if let Some(prompt) = phase_def["prompt"].as_str() {
                                            return Some(prompt.to_string());
                                        }
                                    }
                                    
                                    // 2. Modern style: phases with possibleActions and ui.prompt
                                    if phase_def.get("possibleActions").is_some() {
                                        if let Some(ui_prompt) = phase_def["ui"]["prompt"].as_str() {
                                            // Replace {actor} with current turn player
                                            let mut prompt = ui_prompt.to_string();
                                            if let Some(turn_player) = state.get("currentPlayer").and_then(|t| t.as_str()) {
                                                prompt = prompt.replace("{actor}", turn_player);
                                            }
                                            return Some(prompt);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
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

        let turn_player = state.get("currentPlayer")
            .and_then(|cp| cp.as_str())
            .unwrap_or("");
        println!("[DEBUG action_map] Current turn player: {}", turn_player);

        let players = state.get("players")
            .and_then(|p| p.as_array())
            .unwrap_or(&Vec::new())
            .clone();

        if !players.is_empty() {
            println!("[DEBUG action_map] Players in game: {:?}", players);
            for (idx, _player) in players.iter().enumerate() {
                let id = format!("p{}", idx + 1);
                println!("[DEBUG action_map] Checking actions for player: {}", id);
                let mut action_map = serde_json::Map::new();
                
                if id == turn_player {
                    println!("[DEBUG action_map] Player {} is the current turn player", id);
                    
                    // Get current phases - check phases at top level (our format)
                    let mut current_phases = Vec::new();
                    if let Some(phases) = state.get("phases").and_then(|p| p.as_object()) {
                        println!("[DEBUG action_map] Found phases: {:?}", phases);
                        for (_phase_set, phase_id) in phases {
                            if let Some(phase_str) = phase_id.as_str() {
                                // Extract just the phase ID (e.g., "play" from "game.play")
                                if let Some(phase_part) = phase_str.split('.').last() {
                                    current_phases.push(phase_part.to_string());
                                } else {
                                    current_phases.push(phase_str.to_string());
                                }
                            }
                        }
                    }
                    
                    println!("[DEBUG action_map] Current phases: {:?}", current_phases);
                    if let Some(actionlist) = bundle.actions.as_array() {
                        println!("[DEBUG action_map] Found {} actions in bundle, current phases: {:?}", actionlist.len(), current_phases);
                        
                        // Find which actions are allowed in current phases
                        if let Some(phase_sets) = bundle.phases.as_array() {
                            let mut allowed_actions = Vec::new();
                            
                            // For each current phase, we need to find it within the phase sets
                            for current_phase in &current_phases {
                                    // Phase sets contain nested phases
                                    for phase_set in phase_sets {
                                        if let Some(phases) = phase_set["phases"].as_array() {
                                            if let Some(phase_def) = phases.iter().find(|p| p["id"].as_str() == Some(current_phase)) {
                                                println!("[DEBUG action_map] Found phase {} with definition: {:?}", current_phase, phase_def);
                                                // Check both 'possibleActions' (new) and 'actions' (old) fields
                                                let actions = phase_def["possibleActions"].as_array()
                                                    .or_else(|| phase_def["actions"].as_array());
                                                
                                                if let Some(actions) = actions {
                                                    println!("[DEBUG action_map] Phase {} has {} possible actions", current_phase, actions.len());
                                                    for action in actions {
                                                        if let Some(action_id) = action.as_str() {
                                                            allowed_actions.push(action_id);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                println!("[DEBUG action_map] Allowed actions in current phases: {:?}", allowed_actions);
                                
                                for a in actionlist {
                                    let action_id = a["id"].as_str().unwrap_or("");
                                    
                                    // Skip actions not allowed in current phases
                                    if !allowed_actions.contains(&action_id) {
                                        println!("[DEBUG action_map] Skipping action {} - not in allowed actions", action_id);
                                        continue;
                                    }
                                // Support both 'uses' (new) and 'builtin' (old)
                                let action_impl = a["uses"].as_str()
                                    .or_else(|| a["builtin"].as_str());
                                println!("[DEBUG action_map] Action {} has implementation: {:?}", a["id"].as_str().unwrap_or("unknown"), action_impl);
                                if action_impl == Some("place") {
                                    // Handle place action for tic-tac-toe and similar games
                                    println!("[DEBUG] Found place action: {:?}", a["id"]);
                                    if let Some(action_id) = a["id"].as_str() {
                                        // For place actions, we need to find all empty cells on the board
                                        if let Some(zones) = state.get("zones").and_then(|z| z.as_object()) {
                                            for (zone_id, zone_data) in zones {
                                                println!("[DEBUG] Checking zone {} for place action", zone_id);
                                                
                                                // Handle new format where zone has type and cells
                                                if let Some(zone_obj) = zone_data.as_object() {
                                                    if zone_obj.get("type").and_then(|t| t.as_str()) == Some("grid") {
                                                        if let Some(cells) = zone_obj.get("cells").and_then(|c| c.as_array()) {
                                                            println!("[DEBUG] Found grid zone with cells");
                                                            for (r, row) in cells.iter().enumerate() {
                                                                if let Some(row_array) = row.as_array() {
                                                                    for (c, cell) in row_array.iter().enumerate() {
                                                                        if cell.is_null() {
                                                                            let location = format!("/zones/{}/cells/{}/{}", zone_id, r, c);
                                                                            println!("[DEBUG] Empty cell at {}", location);
                                                                            
                                                                            // Get UI direction from action
                                                                            let direction = a.get("ui")
                                                                                .and_then(|ui| ui.get("direction"))
                                                                                .and_then(|d| d.as_str())
                                                                                .unwrap_or("Select this location");
                                                                            
                                                                            action_map.insert(location, serde_json::json!({
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
                                } else if action_impl == Some("placeWithGravity") {
                                    // Handle placeWithGravity action for Connect 4 and similar games
                                    println!("[DEBUG] Found placeWithGravity action: {:?}", a["id"]);
                                    if let Some(action_id) = a["id"].as_str() {
                                        // Get the zone parameter from the action
                                        if let Some(zone_path) = a.get("with")
                                            .and_then(|w| w.get("zone"))
                                            .and_then(|z| z.as_str()) {
                                            
                                            // Extract zone name from path like "/zones/board"
                                            let zone_name = zone_path.strip_prefix("/zones/").unwrap_or(zone_path);
                                            
                                            if let Some(zones) = state.get("zones").and_then(|z| z.as_object()) {
                                                if let Some(zone_data) = zones.get(zone_name) {
                                                    if let Some(zone_obj) = zone_data.as_object() {
                                                        if zone_obj.get("type").and_then(|t| t.as_str()) == Some("grid") {
                                                            if let Some(cells) = zone_obj.get("cells").and_then(|c| c.as_array()) {
                                                                println!("[DEBUG] Found grid zone for gravity action");
                                                                
                                                                // Get number of columns from first row
                                                                if let Some(first_row) = cells.get(0).and_then(|r| r.as_array()) {
                                                                    for col in 0..first_row.len() {
                                                                        // Check if column has space (any null cell)
                                                                        let mut has_space = false;
                                                                        for row in cells {
                                                                            if let Some(row_array) = row.as_array() {
                                                                                if row_array.get(col).map(|c| c.is_null()).unwrap_or(false) {
                                                                                    has_space = true;
                                                                                    break;
                                                                                }
                                                                            }
                                                                        }
                                                                        
                                                                        if has_space {
                                                                            let location = format!("/zones/{}/columns/{}", zone_name, col);
                                                                            println!("[DEBUG] Available column at {}", location);
                                                                            
                                                                            // Get UI direction from action
                                                                            let direction = a.get("ui")
                                                                                .and_then(|ui| ui.get("direction"))
                                                                                .and_then(|d| d.as_str())
                                                                                .unwrap_or("Click this column");
                                                                            
                                                                            action_map.insert(location, serde_json::json!({
                                                                                "action": action_id,
                                                                                "direction": direction,
                                                                                "targetColumn": col
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
                                } else if action_impl == Some("grid.move") || action_impl == Some("presets.grid.move") || action_impl == Some("moveEntity") {
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
                                                        let source_id = source_template.replace("{actor}", &id);
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
                                        if let Some(selection) = state.get("selection") {
                                            if selection["actor"].as_str() == Some(&id) {
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
                                } else if action_impl == Some("entity.move") || action_impl == Some("presets.entity.move") {
                                    if let Some(action_id) = a["id"].as_str() {
                                        println!("[DEBUG action_map] Processing entity.move action: {}", action_id);
                                        // Handle entity.move actions for card games
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(source) = params["source"].as_str() {
                                                let source_zone = source.replace("{actor}", &id);
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
                                                                                    let zone_id = with.get("zone").and_then(|z| z.as_str()).unwrap_or("").replace("{actor}", &id);
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
                                                            println!("[DEBUG action_map] Processing action {} for hand", action_id);
                                                            
                                                            // Check conditions before adding action
                                                            let mut conditions_met = true;
                                                            if let Some(conditions) = a.get("conditions").and_then(|c| c.as_array()) {
                                                                for condition in conditions {
                                                                    if let Some(cond_type) = condition.get("type").and_then(|t| t.as_str()) {
                                                                        if cond_type == "zone.count" {
                                                                            if let Some(with) = condition.get("with") {
                                                                                let zone_id = with.get("zone").and_then(|z| z.as_str()).unwrap_or("").replace("{actor}", &id);
                                                                                if zone_id == source_zone {
                                                                                    if let Some(exact) = with.get("exact").and_then(|e| e.as_u64()) {
                                                                                        println!("[DEBUG conditions] Checking exact count for {}: {} == {}", zone_id, items.len(), exact);
                                                                                        if items.len() != exact as usize {
                                                                                            println!("[DEBUG conditions] Condition failed: {} != {}", items.len(), exact);
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
                            } // Close phases_array check
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
    fn build_welcome_message(&self, player_id: &str, include_state: bool, format: &MessageFormat) -> serde_json::Value {
        let actor = self.actor_for_player(player_id)
            .unwrap_or_else(|| "spectator".to_string());
        
        if include_state {
            let snapshot = { let g = self.state.lock(); g.clone() };
            let action_map = Lobby::compute_action_map(&snapshot, &self.bundle);
            
            // Build clean client meta object (no game state duplication)
            let mut meta = json!({});
            
            // Client-specific fields will be created fresh each time
            
            // Update with current data
            if let Some(meta_obj) = meta.as_object_mut() {
                meta_obj.insert("actionMap".to_string(), json!(action_map));
                meta_obj.insert("players".to_string(), json!(self.player_list()));
                meta_obj.insert("entities".to_string(), self.bundle.entities.clone());
                // Expand zone metadata for player-specific zones
                let expanded_zones = Lobby::expand_zone_metadata(&self.bundle.zones, &self.player_list());
                meta_obj.insert("zones".to_string(), expanded_zones);
                // Include zoneGroups from manifest if present, expanded with player IDs
                if let Some(zone_groups) = &self.bundle.manifest.zone_groups {
                    let expanded_groups = Lobby::expand_zone_groups(&json!(zone_groups), &self.player_list());
                    meta_obj.insert("zoneGroups".to_string(), expanded_groups);
                }
                // Ensure gameLog exists
                if !meta_obj.contains_key("gameLog") {
                    meta_obj.insert("gameLog".to_string(), json!([]));
                }
                // Ensure phaseDisplayMessages exists
                if !meta_obj.contains_key("phaseDisplayMessages") {
                    meta_obj.insert("phaseDisplayMessages".to_string(), json!([]));
                }
                
                // Add current phase prompt for turn player
                if let Some(turn_player) = snapshot.get("currentPlayer").and_then(|t| t.as_str()) {
                    if actor == turn_player {
                        // Get current phase prompt
                        let phase_prompt = Lobby::get_current_phase_prompt(&snapshot, &self.bundle);
                        if let Some(prompt) = phase_prompt {
                            meta_obj.insert("currentPhasePrompt".to_string(), json!(prompt));
                        }
                    }
                }
            }
            
            let base_message = json!({
                "type": "welcome",
                "you": actor,
                "started": true,
                "game": snapshot,
                "ui": meta,
                "tick": *self.tick.lock()
            });
            
            // Apply formatting based on client preferences
            format_welcome_message(base_message, format.clone())
        } else {
            let mut meta = serde_json::Map::new();
            meta.insert("players".to_string(), json!(self.player_list()));
            meta.insert("entities".to_string(), self.bundle.entities.clone());
            // Include zone metadata
            let expanded_zones = Lobby::expand_zone_metadata(&self.bundle.zones, &self.player_list());
            meta.insert("zones".to_string(), expanded_zones);
            
            // Include zoneGroups from manifest if present, expanded with player IDs
            if let Some(zone_groups) = &self.bundle.manifest.zone_groups {
                let expanded_groups = Lobby::expand_zone_groups(&json!(zone_groups), &self.player_list());
                meta.insert("zoneGroups".to_string(), expanded_groups);
            }
            
            let base_message = json!({
                "type": "welcome",
                "you": actor,
                "started": false,
                "ui": meta
            });
            
            // Apply formatting based on client preferences
            format_welcome_message(base_message, format.clone())
        }
    }

pub fn start_game(self: Arc<Self>) {
    *self.game_started.lock() = true;
    
    // Get current game state
    let snapshot = { let g = self.state.lock(); g.clone() };
    
    // Don't process phases here - we'll do it after sending the initial game state
    println!("[DEBUG start_game] Will process phases after initial state is sent");
    
    // Update state with any changes from phase processing
    *self.state.lock() = snapshot.clone();
    
    // Get action map after phase triggers
    let action_map = Lobby::compute_action_map(&snapshot, &self.bundle);
    
    // Build UI meta object
    let mut meta = json!({});
    
    // Update with current data
    if let Some(meta_obj) = meta.as_object_mut() {
        meta_obj.insert("actionMap".to_string(), json!(action_map));
        meta_obj.insert("players".to_string(), json!(self.player_list()));
        meta_obj.insert("entities".to_string(), self.bundle.entities.clone());
        // Expand zone metadata for player-specific zones
        let expanded_zones = Lobby::expand_zone_metadata(&self.bundle.zones, &self.player_list());
        meta_obj.insert("zones".to_string(), expanded_zones);
        // Include zoneGroups from manifest if present, expanded with player IDs
        if let Some(zone_groups) = &self.bundle.manifest.zone_groups {
            let expanded_groups = Lobby::expand_zone_groups(&json!(zone_groups), &self.player_list());
            meta_obj.insert("zoneGroups".to_string(), expanded_groups);
        }
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
        "game": snapshot,
        "ui": meta
    });
    let _ = self.tx.send(Message::Text(game_started_msg.to_string()));
    
    self.broadcast_lobby_list();
    
    // Now process phases after initial state is sent
    let self_clone = self.clone();
    let bundle = self.bundle.clone();
    tokio::spawn(async move {
        // Small delay to ensure clients receive the initial state
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let mut max_iterations = 10;
        loop {
            println!("[DEBUG] Phase processing iteration {}", 11 - max_iterations);
            
            // Get current state
            let mut snapshot = { let g = self_clone.state.lock(); g.clone() };
            println!("[DEBUG] Current state phases: {:?}", snapshot.get("phases"));
            
            // Process phases
            let phase_patches = engine::process_phases(&bundle, &mut snapshot);
            
            if let Ok(phase_arr) = phase_patches {
                if phase_arr.is_empty() {
                    println!("[DEBUG] No more phase patches, stopping phase processing");
                    break;
                }
                
                println!("[DEBUG] Got {} patches from phase processing", phase_arr.len());
                
                // Apply patches to our state
                for patch in &phase_arr {
                    engine::apply_patch_to_state(&mut snapshot, patch);
                }
                
                // Update the lobby state
                *self_clone.state.lock() = snapshot.clone();
                
                // After phase transition, recompute action map
                let action_map = Lobby::compute_action_map(&snapshot, &bundle);
                println!("[DEBUG] Recomputed action map after phase transition: {:?}", action_map);
                
                // Increment tick
                let tick = {
                    let mut t = self_clone.tick.lock();
                    *t += 1;
                    *t
                };
                
                // Prepare patches for client - need to prefix paths
                let mut client_patches = Vec::new();
                for patch in &phase_arr {
                    if let Some(path) = patch.get("path").and_then(|p| p.as_str()) {
                        let mut patch_obj = patch.clone();
                        // Prefix with /game for game state patches
                        if !path.starts_with("/ui") {
                            let prefixed_path = if path.starts_with("/game") {
                                path.to_string()
                            } else {
                                format!("/game{}", path)
                            };
                            patch_obj["path"] = json!(prefixed_path);
                        }
                        client_patches.push(patch_obj);
                    } else {
                        client_patches.push(patch.clone());
                    }
                }
                
                // Add action map update patch
                client_patches.push(json!({
                    "op": "replace",
                    "path": "/ui/actionMap",
                    "value": action_map
                }));
                
                // Add current phase prompt for turn player
                let current_state = { let g = self_clone.state.lock(); g.clone() };
                if let Some(_turn_player) = current_state.get("currentPlayer").and_then(|t| t.as_str()) {
                    // Get current phase prompt
                    let phase_prompt = Lobby::get_current_phase_prompt(&current_state, &bundle);
                    if let Some(prompt) = phase_prompt {
                        // Always use "replace" for phase prompts
                        let op = "replace";
                        client_patches.push(json!({
                            "op": op,
                            "path": "/ui/currentPhasePrompt",
                            "value": prompt
                        }));
                    } else {
                        // Remove prompt if no playerAction phase is active
                        client_patches.push(json!({
                            "op": "remove",
                            "path": "/ui/currentPhasePrompt"
                        }));
                    }
                }
                
                // Send patches to clients
                let frame = json!({
                    "type": "diff",
                    "tick": tick,
                    "patch": client_patches
                });
                self_clone.history.lock().push(frame.clone());
                let _ = self_clone.tx.send(Message::Text(frame.to_string()));
                
                // Delay between phase actions for visual effect
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
            } else {
                break;
            }
            
            max_iterations -= 1;
            if max_iterations == 0 {
                println!("[DEBUG] Max iterations reached, stopping phase processing");
                break;
            }
        }
        
        println!("[DEBUG] Phase processing completed");
    });
}

/// Accept a new WebSocket client, drive send/recv loops.
    pub async fn accept_client(self: Arc<Self>, socket: WebSocket, player_id: String, join: bool, since: u64, format: String, updates: String) {
        println!("[Lobby] Client {} connecting to lobby {} (join={}, since={}, format={}, updates={})", player_id, self.id, join, since, format, updates);
        let (ws_tx, mut rx) = socket.split();
        let tx = Arc::new(TokioMutex::new(ws_tx));
        
        // Parse client format preferences
        let message_format = MessageFormat::from(format.as_str());
        let update_format = UpdateFormat::from(updates.as_str());
        
        // Store client preferences
        {
            let mut clients = self.client_formats.lock();
            clients.insert(player_id.clone(), ClientInfo {
                message_format: message_format.clone(),
                update_format: update_format.clone(),
            });
        }

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
        let welcome = self.build_welcome_message(&player_id, self.is_started(), &message_format);
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
        // Get client preferences for replay
        let client_prefs = self.client_formats.lock().get(&player_id).cloned();
        let client_update_format = client_prefs.as_ref().map(|cp| &cp.update_format).unwrap_or(&UpdateFormat::Patch);
        let client_message_format = client_prefs.as_ref().map(|cp| &cp.message_format).unwrap_or(&MessageFormat::Standard);
        for frame in frames.into_iter() {
            if frame["tick"].as_u64().unwrap_or(0) > replay_after {
                let mut frame_to_send = frame.clone();
                
                // Convert diff to full state if client prefers full updates
                if frame["type"] == "diff" && matches!(client_update_format, UpdateFormat::Full) {
                    let current_state = self.state.lock().clone();
                    let tick = frame["tick"].as_u64().unwrap_or(0);
                    frame_to_send = patch_to_full_state(&current_state, &frame["patch"], tick);
                }
                
                // Apply message format
                frame_to_send = format_welcome_message(frame_to_send, client_message_format.clone());
                
                let _ = tx
                    .lock()
                    .await
                    .send(Message::Text(frame_to_send.to_string()))
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
                // Get client preferences
                let client_info = self_ref.client_formats.lock().get(&player_id_clone).cloned();
                let update_format = client_info.as_ref().map(|ci| &ci.update_format).unwrap_or(&UpdateFormat::Patch);
                let message_format = client_info.as_ref().map(|ci| &ci.message_format).unwrap_or(&MessageFormat::Standard);
                
                // Handle different message types
                if let Message::Text(text) = &msg {
                    if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(text) {
                        // Handle gameStarted message specially to include correct "you" field
                        if json["type"] == "gameStarted" {
                            let actor = self_ref
                                .actor_for_player(&player_id_clone)
                                .unwrap_or_else(|| "spectator".to_string());
                            json["you"] = serde_json::Value::String(actor);
                            
                            // Apply message format
                            json = format_welcome_message(json, message_format.clone());
                            
                            let personalized_msg = Message::Text(json.to_string());
                            if tx_forward.lock().await.send(personalized_msg).await.is_err() {
                                break;
                            }
                            continue;
                        }
                        
                        // Handle diff messages based on update format preference
                        if json["type"] == "diff" && matches!(update_format, UpdateFormat::Full) {
                            // Convert patch to full state update
                            let current_state = self_ref.state.lock().clone();
                            let tick = json["tick"].as_u64().unwrap_or(0);
                            let full_state = patch_to_full_state(&current_state, &json["patch"], tick);
                            
                            // Apply message format to the full state
                            let formatted = format_welcome_message(full_state, message_format.clone());
                            
                            let full_msg = Message::Text(formatted.to_string());
                            if tx_forward.lock().await.send(full_msg).await.is_err() {
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
                            let client_format = self.client_formats.lock()
                                .get(&player_id)
                                .map(|c| c.message_format.clone())
                                .unwrap_or(MessageFormat::Standard);
                            let welcome = self.build_welcome_message(&player_id, self.is_started(), &client_format);
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
                            self.clone().start_game();
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
                        
                        // Find the action definition to get the verb and validate conditions
                        let mut verb = None;
                        let mut action_valid = true;
                        if let Some(actions) = self.bundle.actions.as_array() {
                            for action in actions {
                                if action["id"].as_str() == Some(action_id) {
                                    // Validate action conditions before executing
                                    if let Some(when_conditions) = action.get("when").and_then(|w| w.as_array()) {
                                        for condition in when_conditions {
                                            if let Some(condition_type) = condition.get("condition").and_then(|c| c.as_str()) {
                                                if condition_type == "zone.isEmpty" {
                                                    if let Some(with_obj) = condition.get("with").and_then(|w| w.as_object()) {
                                                        if let Some(zone_template) = with_obj.get("zone").and_then(|z| z.as_str()) {
                                                            // Replace {target} with the actual location from args
                                                            if let Some(args_obj) = args.as_ref().and_then(|a| a.as_object()) {
                                                                if let Some(location) = args_obj.get("location").and_then(|l| l.as_str()) {
                                                                    let zone_path = zone_template.replace("{target}", location);
                                                                    
                                                                    // Check if the zone/cell is actually empty
                                                                    let state = self.state.lock();
                                                                    let is_empty = if zone_path.contains("/cells/") {
                                                                        // For grid cells, check if the cell is null
                                                                        get_value_at_path(&state, &zone_path).map(|v| v.is_null()).unwrap_or(false)
                                                                    } else {
                                                                        // For other zones, you might have different empty logic
                                                                        false
                                                                    };
                                                                    
                                                                    if !is_empty {
                                                                        println!("[Socket] Action validation failed: zone {} is not empty", zone_path);
                                                                        action_valid = false;
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else if condition_type == "player.isActor" {
                                                    // Check if the current player is the one making the move
                                                    let state = self.state.lock();
                                                    let current_player = state.get("currentPlayer")
                                                        .and_then(|cp| cp.as_str())
                                                        .unwrap_or("");
                                                    
                                                    if current_player != current_actor {
                                                        println!("[Socket] Action validation failed: player {} is not the current turn player ({})", current_actor, current_player);
                                                        action_valid = false;
                                                        break;
                                                    }
                                                }
                                                // Add other condition types here as needed
                                            }
                                        }
                                    }
                                    
                                    if action_valid {
                                        let verb_raw = action["uses"].as_str()
                                            .or_else(|| action["builtin"].as_str());
                                        
                                        // Map presets to actual verbs
                                        verb = verb_raw.map(|v| match v {
                                            "presets.turn.advance" => "nextTurn",
                                            "presets.entity.move" => "moveEntity",
                                            "presets.cards.draw" => "draw",
                                            "presets.phase.set" => "setPhase",
                                            other => other
                                        });
                                    }
                                    break;
                                }
                            }
                        }
                        
                        if verb.is_none() {
                            println!("[Socket] Error: Action {} not found or has no verb", action_id);
                            continue;
                        }
                        
                        if !action_valid {
                            println!("[Socket] Error: Action {} validation failed", action_id);
                            continue;
                        }
                        
                        // Create the action in the format expected by apply_action
                        let engine_action = json!({
                            "verb": verb,
                            "args": args
                        });
                        
                        let diff =
                            engine::apply_action(&self.bundle, &mut self.state.lock(), &current_actor, &engine_action);
                        println!("[Socket] Action result diff: {:?}", diff);
                        
                        let tick = {
                            let mut t = self.tick.lock();
                            *t += 1;
                            *t
                        };
                        let mut patch_ops = Vec::new();
                        if let Ok(ref arr) = diff {
                            for op in arr {
                                println!("[DEBUG] Original patch: {:?}", op);
                                if let Some(path) = op.get("path").and_then(|p| p.as_str()) {
                                    let mut op_obj = op.clone();
                                    // Add /game prefix only if path doesn't already start with /game
                                    let prefixed_path = if path.starts_with("/game") {
                                        path.to_string()
                                    } else {
                                        format!("/game{}", path)
                                    };
                                    op_obj["path"] = serde_json::Value::String(prefixed_path);
                                    println!("[DEBUG] Transformed patch: {:?}", op_obj);
                                    patch_ops.push(op_obj);
                                } else {
                                    patch_ops.push(op.clone());
                                }
                            }
                        }
                        
                        // Process "then" actions if the initial action succeeded
                        if diff.as_ref().map(|a| !a.is_empty()).unwrap_or(false) {
                            println!("[Socket] Action succeeded, checking for 'then' actions");
                            
                            // Find the action definition to check for "then" clause
                            if let Some(actions) = self.bundle.actions.as_array() {
                                if let Some(action_def) = actions.iter().find(|a| a["id"].as_str() == Some(action_id)) {
                                    if let Some(then_actions) = action_def.get("then").and_then(|t| t.as_array()) {
                                        println!("[Socket] Found {} 'then' actions", then_actions.len());
                                        
                                        for then_action in then_actions {
                                            if let Some(then_action_id) = then_action["action"].as_str() {
                                                println!("[Socket] Processing 'then' action: {}", then_action_id);
                                                
                                                // Find the then action definition
                                                if let Some(then_action_def) = actions.iter().find(|a| a["id"].as_str() == Some(then_action_id)) {
                                                    let then_verb_raw = then_action_def["uses"].as_str()
                                                        .or_else(|| then_action_def["builtin"].as_str());
                                                    
                                                    if let Some(verb_raw) = then_verb_raw {
                                                        // Map presets to actual verbs
                                                        let verb = match verb_raw {
                                                            "presets.turn.advance" => "nextTurn",
                                                            "presets.entity.move" => "moveEntity",
                                                            "presets.cards.draw" => "draw",
                                                            "presets.phase.set" => "setPhase",
                                                            v => v
                                                        };
                                                        
                                                        println!("[Socket] 'then' action verb: {} (mapped from {})", verb, verb_raw);
                                                        
                                                        // Create the then action
                                                        let args = then_action_def.get("with").cloned().unwrap_or(json!({}));
                                                        let then_engine_action = json!({
                                                            "verb": verb,
                                                            "args": args
                                                        });
                                                        
                                                        // Apply the then action
                                                        let then_diff = engine::apply_action(
                                                            &self.bundle, 
                                                            &mut self.state.lock(), 
                                                            &current_actor, 
                                                            &then_engine_action
                                                        );
                                                        
                                                        println!("[Socket] 'then' action result: {:?}", then_diff);
                                                        
                                                        // Add the patches from the then action
                                                        if let Ok(then_arr) = then_diff {
                                                            for op in then_arr {
                                                                if let Some(path) = op.get("path").and_then(|p| p.as_str()) {
                                                                    let mut op_obj = op.clone();
                                                                    // Don't add /game prefix to /ui patches, they should stay at top level
                                                                    if path.starts_with("/ui/") {
                                                                        // Keep /ui patches at top level
                                                                        patch_ops.push(op_obj);
                                                                    } else {
                                                                        // Add /game prefix to all other patches modifying game state
                                                                        let prefixed_path = if path.starts_with("/game") {
                                                                            path.to_string()
                                                                        } else {
                                                                            format!("/game{}", path)
                                                                        };
                                                                        op_obj["path"] = json!(prefixed_path);
                                                                        patch_ops.push(op_obj);
                                                                    }
                                                                } else {
                                                                    patch_ops.push(op.clone());
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
                        
                        // Check if game just ended in the patches
                        let mut game_ended = false;
                        let mut game_end_info = None;
                        for op in &patch_ops {
                            let path = op.get("path").and_then(|p| p.as_str()).unwrap_or("");
                            let is_game_status = path == "/ui/gameStatus" || path == "/game/gameStatus";
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
                            // Game still active, process phases first
                            let phase_patches = engine::process_phases(&self.bundle, &mut self.state.lock());
                            if let Ok(phase_arr) = phase_patches {
                                patch_ops.extend(phase_arr);
                            }
                            
                            // Then compute action map normally
                            let current_state = { let g = self.state.lock(); g.clone() };
                            let action_map = Lobby::compute_action_map(&current_state, &self.bundle);
                            for (pid, actions) in action_map {
                                patch_ops.push(serde_json::json!({
                                    "op": "replace",
                                    "path": format!("/ui/actionMap/{}", pid),
                                    "value": actions
                                }));
                            }
                            
                            // Add current phase prompt for turn player
                            if let Some(_turn_player) = current_state.get("currentPlayer").and_then(|t| t.as_str()) {
                                // Get current phase prompt
                                let phase_prompt = Lobby::get_current_phase_prompt(&current_state, &self.bundle);
                                if let Some(prompt) = phase_prompt {
                                    // Always use "replace" for phase prompts
                                    patch_ops.push(serde_json::json!({
                                        "op": "replace",
                                        "path": "/ui/currentPhasePrompt",
                                        "value": prompt
                                    }));
                                } else {
                                    // Remove prompt if no playerAction phase is active
                                    patch_ops.push(serde_json::json!({
                                        "op": "remove",
                                        "path": "/ui/currentPhasePrompt"
                                    }));
                                }
                            }
                        }
                        
                        // Generate game log entry if action was successful and has a log template
                        if diff.as_ref().map(|a| !a.is_empty()).unwrap_or(false) {
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
                                        if let Some(column) = args_obj["column"].as_i64() {
                                            log_text = log_text.replace("{column}", &(column + 1).to_string()); // 1-indexed for display
                                        }
                                    }
                                    
                                    // Create timestamp
                                    let now = chrono::Local::now();
                                    let timestamp = now.format("%H:%M").to_string();
                                    
                                    // Add game log entry to patches
                                    patch_ops.push(serde_json::json!({
                                        "op": "add",
                                        "path": "/ui/gameLog/-",
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
                                    "path": "/ui/gameLog/-",
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
                        
                        // If game ended, broadcast updated lobby list (but do it AFTER the diff to avoid race conditions)
                        if game_ended {
                            println!("[Lobby] Game ended, will broadcast updated lobby list after small delay");
                            let self_clone = self.clone();
                            tokio::spawn(async move {
                                // Small delay to ensure the diff message is processed first
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                self_clone.broadcast_lobby_list();
                            });
                        }
                    }
                }
            }
        }

        forward.abort();
        
        // Clean up client format preferences on disconnect
        {
            let mut clients = self.client_formats.lock();
            clients.remove(&player_id);
        }
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

// Helper function to get a value at a specific path in the JSON state
fn get_value_at_path<'a>(state: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in path_parts {
        if let Ok(index) = part.parse::<usize>() {
            // This is an array index
            if let Some(array) = current.as_array() {
                if index < array.len() {
                    current = &array[index];
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            // This is an object key
            current = current.get(part)?;
        }
    }
    
    Some(current)
}

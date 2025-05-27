//! lobby.rs – minimal in-memory lobby with broadcast fan-out
//! Supports: welcome snapshot → JSON verb → diff broadcast

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
                "name": format!("{} - Lobby {}", lobby.bundle.game_id, &l.key()[0..6]),
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

    /// Compute possible verbs for each player based on current state
    fn possible_verbs(state: &serde_json::Value, bundle: &Bundle) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();

        println!("[DEBUG possible_verbs] Checking game state - meta exists: {}, meta.gameStatus exists: {}", 
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
            println!("[DEBUG] Game has ended, returning empty possibleVerbs for all players");
            // Game has ended, no moves possible
            if let Some(players) = state["players"].as_array() {
                for player in players {
                    if let Some(id) = player["id"].as_str() {
                        map.insert(id.to_string(), serde_json::Value::Array(vec![]));
                    }
                }
            }
            return map;
        }

        let turn_player = state["turn"].as_str().unwrap_or("");

        if let Some(players) = state["players"].as_array() {
            for player in players {
                if let Some(id) = player["id"].as_str() {
                    let mut verbs_by_id: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
                    
                    if id == turn_player {
                        if let Some(verblist) = bundle.verbs.as_array() {
                            for v in verblist {
                                if v["builtin"].as_str() == Some("moveEntity") {
                                    if let Some(verb_id) = v["id"].as_str() {
                                        if let Some(target_zone) = v["params"]["target"]["zone"].as_str() {
                                            let zone_state = &state["zones"][target_zone];
                                            if zone_state.is_array() {
                                                let mut valid_options = Vec::new();
                                                let gravity = v["params"]["target"]["gravity"].as_bool().unwrap_or(false);
                                                
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
                                                
                                                if !valid_options.is_empty() {
                                                    let direction = v["ui"]["direction"].as_str().unwrap_or("Make a move");
                                                    verbs_by_id.insert(verb_id.to_string(), serde_json::json!({
                                                        "verb": verb_id,
                                                        "direction": direction,
                                                        "validOptions": valid_options
                                                    }));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Convert HashMap to Vec
                    let verbs: Vec<serde_json::Value> = verbs_by_id.into_values().collect();
                    map.insert(id.to_string(), serde_json::Value::Array(verbs));
                }
            }
        }

        map
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
            let possible = Lobby::possible_verbs(&snapshot, &self.bundle);
            
            // Build meta object preserving existing meta data
            let mut meta = if let Some(existing_meta) = snapshot.get("meta") {
                existing_meta.clone()
            } else {
                json!({})
            };
            
            // Update with current data
            if let Some(meta_obj) = meta.as_object_mut() {
                meta_obj.insert("possibleVerbs".to_string(), json!(possible));
                meta_obj.insert("players".to_string(), json!(self.player_list()));
                meta_obj.insert("entities".to_string(), self.bundle.entities.clone());
                meta_obj.insert("zones".to_string(), self.bundle.zones.clone());
            }
            
            json!({
                "type": "welcome",
                "you": actor,
                "started": true,
                "state": snapshot,
                "meta": meta
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
    
    // Get current game state and possible verbs
    let snapshot = { let g = self.state.lock(); g.clone() };
    let possible = Lobby::possible_verbs(&snapshot, &self.bundle);
    
    // Build meta object preserving existing meta data
    let mut meta = if let Some(existing_meta) = snapshot.get("meta") {
        existing_meta.clone()
    } else {
        json!({})
    };
    
    // Update with current data
    if let Some(meta_obj) = meta.as_object_mut() {
        meta_obj.insert("possibleVerbs".to_string(), json!(possible));
        meta_obj.insert("players".to_string(), json!(self.player_list()));
        meta_obj.insert("entities".to_string(), self.bundle.entities.clone());
        meta_obj.insert("zones".to_string(), self.bundle.zones.clone());
    }
    
    // Send full game state to all connected clients
    let game_started_msg = serde_json::json!({
        "type": "gameStarted",
        "state": snapshot,
        "meta": meta
    });
    let _ = self.tx.send(Message::Text(game_started_msg.to_string()));
    
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

        // Send initial welcome message
        let welcome = self.build_welcome_message(&player_id, self.is_started());
        let _ = tx.lock().await.send(Message::Text(welcome.to_string())).await;

        // replay diff history since the provided tick
        let frames = {
            let history = self.history.lock();
            history.clone()
        };
        for frame in frames.into_iter() {
            if frame["tick"].as_u64().unwrap_or(0) > since {
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
                    } else if json.get("verb").is_some() && self.is_started() {
                        // Get current actor assignment
                        let current_actor = self
                            .actor_for_player(&player_id)
                            .unwrap_or_else(|| "spectator".to_string());
                        
                        println!("[Socket] Processing verb from {} (actor: {}): {:?}", player_id, current_actor, json);
                        let current_turn = self.state.lock()["turn"].as_str().unwrap_or("").to_string();
                        println!("[Socket] Current turn: {}, Actor attempting move: {}", current_turn, current_actor);
                        
                        let diff =
                            engine::apply_verb(&self.bundle, &mut self.state.lock(), &current_actor, &json);
                        println!("[Socket] Verb result diff: {:?}", diff);
                        
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
                        let game_ended = patch_ops.iter().any(|op| {
                            let path = op.get("path").and_then(|p| p.as_str()).unwrap_or("");
                            let is_game_status = path == "/meta/gameStatus" || path == "/state/meta/gameStatus";
                            if is_game_status {
                                println!("[DEBUG] Found gameStatus patch at path: {}, value: {:?}", path, op.get("value"));
                            }
                            is_game_status && 
                            op.get("value").and_then(|v| v.get("state")).and_then(|s| s.as_str()) == Some("ended")
                        });
                        
                        println!("[DEBUG] Game ended check: {}", game_ended);
                        
                        // Only compute possible verbs if game hasn't ended
                        if game_ended {
                            println!("[DEBUG] Game has ended! Setting empty possibleVerbs for all players");
                            // Game just ended, set empty possibleVerbs for all players
                            if let Some(players) = self.state.lock()["players"].as_array() {
                                for player in players {
                                    if let Some(id) = player["id"].as_str() {
                                        println!("[DEBUG] Setting empty possibleVerbs for player: {}", id);
                                        patch_ops.push(serde_json::json!({
                                            "op": "replace",
                                            "path": format!("/meta/possibleVerbs/{}", id),
                                            "value": []
                                        }));
                                    }
                                }
                            }
                        } else {
                            // Game still active, compute possible verbs normally
                            let current_state = { let g = self.state.lock(); g.clone() };
                            let possible = Lobby::possible_verbs(&current_state, &self.bundle);
                            for (pid, verbs) in possible {
                                patch_ops.push(serde_json::json!({
                                    "op": "replace",
                                    "path": format!("/meta/possibleVerbs/{}", pid),
                                    "value": verbs
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

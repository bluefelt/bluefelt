//! Refactored lobby implementation with separated lobby/game state

use super::lobby_state::*;
use crate::lobby::action_map::compute_action_map;
use crate::{bundle::BundleMap, message_format::{MessageFormat, UpdateFormat}};
use axum::extract::ws::Message;
use parking_lot::{Mutex, RwLock};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::broadcast;
use std::collections::HashMap;

/// Replace {player} template with actual player ID
fn replace_player_template(value: &Value, player_id: &str) -> Value {
    match value {
        Value::String(s) => {
            Value::String(s.replace("{player}", player_id))
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| replace_player_template(v, player_id)).collect())
        }
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), replace_player_template(v, player_id));
            }
            Value::Object(new_obj)
        }
        _ => value.clone()
    }
}

/// Client connection info
#[derive(Clone)]
pub struct ClientInfo {
    pub message_format: MessageFormat,
    pub update_format: UpdateFormat,
    pub username: String,
}

/// The Lobby structure - manages lobby state and multiple games
pub struct Lobby {
    /// The persistent lobby state
    pub state: Arc<LobbyState>,
    
    /// Bundle map for creating games
    pub bundles: Arc<BundleMap>,
    
    /// Client connection preferences (read-heavy during broadcasts)
    pub clients: RwLock<HashMap<String, ClientInfo>>,
    
    /// RNG for game randomization
    pub rng: Arc<super::rng::GameRng>,
    
    /// Reference to global lobby map (for cleanup)
    pub lobby_map: Arc<super::LobbyMap>,
}

impl Lobby {
    /// Create a new lobby
    pub fn new(
        name: String,
        bundles: Arc<BundleMap>,
        lobby_map: Arc<super::LobbyMap>,
        seed: Option<[u8; 32]>,
    ) -> Arc<Self> {
        let state = Arc::new(LobbyState::new(name));
        let rng = Arc::new(super::rng::GameRng::new(seed));
        
        Arc::new(Self {
            state,
            bundles,
            clients: RwLock::new(HashMap::new()),
            rng,
            lobby_map,
        })
    }
    
    /// Handle a member joining the lobby
    pub fn join_lobby(&self, username: String, client_info: ClientInfo) -> Result<(), String> {
        // Check if lobby is archived
        if self.state.is_archived() {
            return Err("Cannot join archived lobby".to_string());
        }
        
        // Check if this will be the first member
        let is_first_member = self.state.members.read().is_empty();
        
        // Add to lobby members
        self.state.add_member(username.clone())?;
        
        // If first member, set as owner
        if is_first_member {
            let mut owner = self.state.owner.lock();
            *owner = Some(username.clone());
        }
        
        // Store client info
        self.clients.write().insert(username.clone(), client_info);
        
        // Send lobby joined message
        self.broadcast_lobby_state();
        
        Ok(())
    }
    
    /// Handle a member disconnecting (but not necessarily leaving)
    pub fn disconnect_member(&self, username: &str) {
        // Mark as disconnected but don't remove
        self.state.set_member_connected(username, false);
        self.broadcast_lobby_state();
    }
    
    /// Rename the lobby (owner only)
    pub fn rename_lobby(&self, username: &str, new_name: String) -> Result<(), String> {
        // Check if user is the owner
        let owner = self.state.owner.lock();
        match &*owner {
            Some(owner_name) if owner_name == username => {
                drop(owner); // Release the lock before modifying
                self.state.rename(new_name);
                self.broadcast_lobby_state();
                Ok(())
            }
            _ => Err("Only the lobby owner can rename the lobby".to_string())
        }
    }
    
    /// Handle a member leaving the lobby
    pub fn leave_lobby(&self, username: &str) {
        // First, collect table IDs where the user is playing (don't hold locks)
        let table_ids_to_leave: Vec<String> = {
            let tables = self.state.tables.lock();
            tables.iter()
                .filter_map(|(table_id, table)| {
                    let seats = table.seats.read();
                    if seats.iter().any(|seat| {
                        matches!(seat, Some(crate::lobby::table_instance::SeatOccupant::Player(id)) if id == username)
                    }) {
                        Some(table_id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };
        
        // Handle table leaving
        for table_id in table_ids_to_leave {
            // TODO: Implement proper table leaving logic
            // This should release the seat and potentially clean up the table
            if let Some(table) = self.state.get_table(&table_id) {
                // For now, just release the seat
                let mut seats = table.seats.write();
                for seat in seats.iter_mut() {
                    if matches!(seat, Some(crate::lobby::table_instance::SeatOccupant::Player(id)) if id == username) {
                        *seat = None;
                    }
                }
            }
        }
        
        // Check if the leaving member is the owner
        let is_owner = {
            let owner = self.state.owner.lock();
            owner.as_ref() == Some(&username.to_string())
        };
        
        // Remove from lobby
        self.state.remove_member(username);
        self.clients.write().remove(username);
        
        // Handle ownership transfer if needed
        if is_owner {
            let new_owner = self.state.transfer_ownership();
            if let Some(new_owner_id) = new_owner {
                println!("[Lobby] Ownership transferred to {}", new_owner_id);
            } else {
                // No members left, archive the lobby
                println!("[Lobby] No members remaining, archiving lobby {}", self.state.id);
                self.state.archive();
                // Remove from global map
                self.lobby_map.remove(&self.state.id);
                return;
            }
        }
        
        // Check if lobby should close
        if self.state.should_auto_close() {
            // Archive and remove from global map
            self.state.archive();
            self.lobby_map.remove(&self.state.id);
        } else {
            self.broadcast_lobby_state();
        }
    }
    
    /// Create a new game in this lobby
    /// DEPRECATED: Use create_table instead
    #[allow(dead_code)]
    pub fn create_game(&self, game_type: &str, creator: &str) -> Result<String, String> {
        // Legacy method - games are now managed through tables
        // Redirect to table creation for backward compatibility
        let bundle = self.bundles.get_latest(game_type)
            .ok_or_else(|| format!("Unknown game type: {}", game_type))?;
        
        let table_id = self.state.create_table(game_type.to_string(), Arc::new(bundle), creator.to_string())?;
        
        self.broadcast_lobby_state();
        Ok(table_id)
    }
    
    /// Join a game as a player
    /// DEPRECATED: Use table seat claiming instead
    #[allow(dead_code)]
    pub fn join_game(&self, game_id: &str, username: &str) -> Result<(), String> {
        use super::seat_manager::SeatManager;
        
        // Get the table
        let table = self.state.get_table(game_id)
            .ok_or_else(|| "Table not found".to_string())?;
        
        // Use atomic seat claiming to prevent race conditions
        let seat_index = SeatManager::atomic_claim_any_seat(
            &table,
            username.to_string(),
            username.to_string(),
        )?;
        
        // Send game joined message to player
        self.send_game_joined(username, game_id);
        
        // Broadcast updated state
        self.broadcast_lobby_state();
        
        Ok(())
    }
    
    /// Legacy join_game method - now uses table seat claiming
    /// DEPRECATED: Use table seat claiming instead
    #[allow(dead_code)]
    fn join_game_old(&self, game_id: &str, username: &str) -> Result<(), String> {
        let table = self.state.get_table(game_id)
            .ok_or_else(|| "Table not found".to_string())?;
        
        // Find next available seat
        let seats = table.seats.read();
        let mut available_seat = None;
        for (i, seat) in seats.iter().enumerate() {
            if seat.is_none() {
                available_seat = Some(i);
                break;
            }
        }
        drop(seats);
        
        let seat_index = available_seat.ok_or_else(|| "No available seats".to_string())?;
        
        // Claim the seat
        table.claim_seat(seat_index, username.to_string(), username.to_string())?;
        
        // Send game joined message to player
        self.send_game_joined(username, game_id);
        
        // Broadcast updated state
        self.broadcast_lobby_state();
        
        Ok(())
    }
    
    /// Start a game
    /// DEPRECATED: Use countdown system instead
    #[allow(dead_code)]
    pub fn start_game(&self, game_id: &str) -> Result<(), String> {
        println!("[lobby] Starting game: {}", game_id);
        let table = self.state.get_table(game_id)
            .ok_or_else(|| "Table not found".to_string())?;
        
        // Check if table can start (has minimum players and all are ready)
        let seated_count = table.seated_count();
        if seated_count < table.min_players as usize {
            return Err("Table cannot start - insufficient players".to_string());
        }
        
        // For legacy compatibility, automatically set all seated players as ready
        {
            let mut ready_states = table.ready_states.write();
            for i in 0..seated_count {
                ready_states[i] = true;
            }
        }
        
        // Use the countdown manager to start the game
        use super::countdown_manager::CountdownManager;
        let lobby_arc = std::sync::Arc::new(Self {
            state: self.state.clone(),
            bundles: self.bundles.clone(),
            clients: parking_lot::RwLock::new(self.clients.read().clone()),
            rng: self.rng.clone(),
            lobby_map: self.lobby_map.clone(),
        });
        CountdownManager::start_countdown(&table, &lobby_arc, 0); // Start immediately
        
        Ok(())
    }
    
    /// Process game action
    /// DEPRECATED: Use table-based action processing instead
    #[allow(dead_code)]
    pub fn process_game_action(
        &self,
        game_id: &str,
        player_id: &str,
        action: Value,
    ) -> Result<Vec<Value>, String> {
        let table = self.state.get_table(game_id)
            .ok_or_else(|| "Table not found".to_string())?;
        
        // Check if player is seated at the table
        let seats = table.seats.read();
        let mut slot = None;
        for (i, seat) in seats.iter().enumerate() {
            if let Some(crate::lobby::table_instance::SeatOccupant::Player(id)) = seat {
                if id == player_id {
                    slot = Some(format!("p{}", i + 1));
                    break;
                }
            }
        }
        drop(seats);
        
        let slot = slot.ok_or_else(|| "Player not seated at table".to_string())?;
        
        // Handle both old format (direct verb/args) and new format (action name with data)
        println!("[process_game_action] Received action: {:?}", action);
        
        // First, determine if we need to resolve the action and get action_def if needed
        let (resolved_action, action_def_opt) = if action.get("verb").is_some() {
            // Already in engine format
            println!("[process_game_action] Action already has verb");
            (action.clone(), None)
        } else if let Some(action_name) = action.get("action").and_then(|a| a.as_str()) {
            println!("[process_game_action] Resolving action: {}", action_name);
            // Need to resolve action name to verb/args format
            // Find the action definition
            let action_def = if let Some(actions) = table.bundle.actions.as_array() {
                actions.iter()
                    .find(|a| a.get("id").and_then(|id| id.as_str()) == Some(action_name))
                    .ok_or_else(|| format!("Unknown action: {}", action_name))?
            } else {
                return Err("No actions defined".to_string());
            };
            
            // Extract verb and build args
            println!("[process_game_action] Found action definition: {:?}", action_def);
            let verb = action_def.get("uses")
                .or_else(|| action_def.get("verb"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Action missing verb/uses".to_string())?;
            println!("[process_game_action] Verb: {}", verb);
            
            // Merge action args with any additional data
            let mut args = action_def.get("with").cloned().unwrap_or(json!({}));
            if let Some(args_obj) = args.as_object_mut() {
                // Add any additional arguments from the action data
                if let Some(data) = action.as_object() {
                    for (key, value) in data {
                        if key != "action" {
                            args_obj.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            
            let resolved = json!({
                "verb": verb,
                "args": args
            });
            
            (resolved, Some((action_def.clone(), action_name.to_string())))
        } else {
            return Err("Invalid action format - missing verb or action name".to_string());
        };
        
        // Apply action to game state
        let mut state = table.game_state.write();
        let mut patches = crate::engine::apply_action(&table.bundle, &mut state, &slot, &resolved_action)
            .map_err(|e| format!("Action failed: {:?}", e))?;
        
        // Process "then" actions if the main action succeeded and we have action_def
        if !patches.is_empty() {
            if let Some((action_def, action_name)) = action_def_opt {
                if let Some(then_actions) = action_def.get("then").and_then(|t| t.as_array()) {
                    println!("[process_game_action] Found {} 'then' actions for {}", then_actions.len(), action_name);
                    
                    for then_action in then_actions {
                        if let Some(then_action_id) = then_action["action"].as_str() {
                            println!("[process_game_action] Processing 'then' action: {}", then_action_id);
                            
                            // Find the then action definition
                            if let Some(actions) = table.bundle.actions.as_array() {
                                if let Some(then_action_def) = actions.iter().find(|a| a["id"].as_str() == Some(then_action_id)) {
                                    // Create resolved action for the then action
                                    let then_verb = then_action_def.get("uses")
                                        .or_else(|| then_action_def.get("verb"))
                                        .and_then(|v| v.as_str())
                                        .ok_or_else(|| format!("Then action {} missing verb/uses", then_action_id))?;
                                    
                                    // Get the raw args and replace template variables
                                    let mut then_args = then_action_def.get("with").cloned().unwrap_or(json!({}));
                                    
                                    // Replace {player} with the current actor (slot)
                                    then_args = replace_player_template(&then_args, &slot);
                                    
                                    // Also replace standard template vars
                                    then_args = crate::engine::patches::replace_template_vars(&then_args, &state);
                                    then_args = crate::engine::patches::replace_actor_template(&then_args, &slot);
                                    
                                    let then_resolved = json!({
                                        "verb": then_verb,
                                        "args": then_args
                                    });
                                    
                                    // Apply the then action
                                    let then_patches = crate::engine::apply_action(&table.bundle, &mut state, &slot, &then_resolved)
                                        .map_err(|e| format!("Then action {} failed: {:?}", then_action_id, e))?;
                                    patches.extend(then_patches);
                                } else {
                                    println!("[process_game_action] WARNING: 'then' action {} not found in bundle", then_action_id);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        drop(state);
        
        // Increment tick
        *table.tick.lock() += 1;
        
        // Process any phase changes after all actions
        let mut phase_state = table.game_state.write();
        let phase_patches = crate::engine::process_phases(&table.bundle, &mut phase_state)
            .unwrap_or_else(|e| {
                println!("[process_game_action] Phase processing error: {}", e);
                vec![]
            });
        drop(phase_state);
        patches.extend(phase_patches);
        
        Ok(patches)
    }
    
    /// Process phase changes for a game
    /// DEPRECATED: Phase processing moved to table system
    #[allow(dead_code)]
    fn process_game_phases(&self, game_id: &str) -> Result<(), String> {
        let table = self.state.get_table(game_id)
            .ok_or_else(|| "Table not found".to_string())?;
        
        let mut all_patches = Vec::new();
        let max_iterations = 10;
        
        for _ in 0..max_iterations {
            let mut state = table.game_state.write();
            let phase_patches = crate::engine::process_phases(&table.bundle, &mut state);
            drop(state);
            
            if let Ok(patches) = phase_patches {
                if patches.is_empty() {
                    break;
                }
                all_patches.extend(patches);
            } else {
                break;
            }
        }
        
        // Check if game ended
        let state = table.game_state.read();
        if let Some(status) = state.get("gameStatus").and_then(|s| s.as_str()) {
            if status == "ended" || status.starts_with("won:") || status == "tie" {
                drop(state);
                // Update table status to Finished
                let mut table_status = table.status.write();
                *table_status = crate::lobby::table_instance::TableStatus::Finished;
            }
        }
        
        Ok(())
    }
    
    /// Broadcast lobby state to all connected clients
    pub fn broadcast_lobby_state(&self) {
        let name = self.state.name.lock().clone();
        let owner = self.state.owner.lock().clone();
        let archived = self.state.is_archived();
        
        let msg = json!({
            "type": "lobbyState",
            "lobby": {
                "id": self.state.id,
                "name": name,
                "owner": owner,
                "archived": archived,
                "members": self.state.members.read().clone(),
                "tables": self.get_tables_summary(),
                "games": self.get_games_summary(), // Keep for backward compatibility
                "completedGames": [], // TODO: Track completed tables/games
            }
        });
        
        let _ = self.state.tx.send(msg.to_string());
    }
    
    /// Get summary of active tables
    pub fn get_tables_summary(&self) -> Vec<Value> {
        let tables = self.state.tables.lock();
        tables.values().map(|table| table.to_summary()).collect()
    }

    /// Get summary of active games (for backward compatibility)
    #[allow(dead_code)]
    pub fn get_games_summary(&self) -> Vec<Value> {
        // For backward compatibility, convert tables to legacy game format
        let tables = self.state.tables.lock();
        tables.values().map(|table| {
            let status = table.status.read();
            let game_status = match &*status {
                crate::lobby::table_instance::TableStatus::Open => "preparing",
                crate::lobby::table_instance::TableStatus::Countdown => "preparing",
                crate::lobby::table_instance::TableStatus::Playing => "playing",
                crate::lobby::table_instance::TableStatus::Finished => "ended",
                crate::lobby::table_instance::TableStatus::Abandoned => "ended",
            };
            
            // Convert seats to legacy players format
            let seats = table.seats.read();
            let mut players = serde_json::Map::new();
            for (i, seat) in seats.iter().enumerate() {
                if let Some(crate::lobby::table_instance::SeatOccupant::Player(player_id)) = seat {
                    players.insert(format!("p{}", i + 1), json!(player_id));
                }
            }
            
            json!({
                "id": table.id,
                "type": table.bundle_id,
                "status": game_status,
                "players": players,
            })
        }).collect()
    }
    
    /// Send game joined message to a player
    /// DEPRECATED: Use table seat claim notifications instead
    fn send_game_joined(&self, username: &str, game_id: &str) {
        let table = match self.state.get_table(game_id) {
            Some(t) => t,
            None => return,
        };
        
        // Find player's seat at the table
        let seats = table.seats.read();
        let mut slot = None;
        for (i, seat) in seats.iter().enumerate() {
            if let Some(crate::lobby::table_instance::SeatOccupant::Player(id)) = seat {
                if id == username {
                    slot = Some(format!("p{}", i + 1));
                    break;
                }
            }
        }
        drop(seats);
        
        let slot = match slot {
            Some(s) => s,
            None => return,
        };
        
        let msg = json!({
            "type": "gameJoined",
            "gameInstanceId": game_id,
            "gameId": table.bundle_id,
            "you": slot,
            "players": table.get_seated_players(),
            "to": username,  // Target specific user
        });
        
        // Send to specific player
        let _ = self.state.tx.send(msg.to_string());
    }
    
    /// Broadcast game started to all players and observers
    /// DEPRECATED: Use countdown manager notifications instead
    pub fn broadcast_game_started(&self, game_id: &str) {
        let table = match self.state.get_table(game_id) {
            Some(t) => t,
            None => {
                println!("[lobby] ERROR: Table {} not found for broadcast_game_started", game_id);
                return;
            }
        };
        
        // Get table state and player mapping
        let state = table.game_state.read().clone();
        let tick = *table.tick.lock();
        let seated_players = table.get_seated_players();
        let game_type = table.bundle_id.clone();
        
        // Create player mapping from seats
        let mut player_mapping = std::collections::HashMap::new();
        let seats = table.seats.read();
        for (i, seat) in seats.iter().enumerate() {
            if let Some(crate::lobby::table_instance::SeatOccupant::Player(player_id)) = seat {
                player_mapping.insert(format!("p{}", i + 1), player_id.clone());
            }
        }
        drop(seats);
        
        // Compute UI data for each player
        println!("[lobby] Sending gameStarted to {} players", seated_players.len());
        for (slot, member_id) in &player_mapping {
            println!("[lobby] Sending gameStarted to player {} (slot {})", member_id, slot);
            let ui_data = self.compute_table_ui(&table, slot, &state);
            
            let msg = json!({
                "type": "gameStarted",
                "gameInstanceId": game_id,
                "tableId": game_id,  // The game_id here is actually the table ID
                "gameId": game_type,
                "tick": tick,
                "you": slot,
                "players": player_mapping.clone(),
                "state": state.clone(),
                "ui": ui_data,
                "to": member_id,  // Target specific user
            });
            
            // Send to specific player
            println!("[lobby] Sending message of size: {} bytes", msg.to_string().len());
            if let Err(e) = self.state.tx.send(msg.to_string()) {
                println!("[lobby] ERROR: Failed to send gameStarted message: {:?}", e);
            } else {
                println!("[lobby] Successfully sent gameStarted to {}", member_id);
            }
        }
        
        // Also send to spectators
        let spectators = table.spectators.read();
        for observer in spectators.iter() {
            let msg = json!({
                "type": "gameStarted", 
                "gameInstanceId": game_id,
                "tableId": game_id,  // The game_id here is actually the table ID
                "gameId": game_type,
                "tick": tick,
                "you": null,  // Observer
                "players": player_mapping.clone(),
                "state": state.clone(),
                "ui": self.compute_table_ui(&table, "", &state),
                "to": observer,  // Target specific observer
            });
            
            let _ = self.state.tx.send(msg.to_string());
        }
    }
    
    /// Compute UI data for a table/game
    /// DEPRECATED: Use table-specific UI computation instead
    #[allow(dead_code)]
    fn compute_table_ui(&self, table: &Arc<crate::lobby::table_instance::TableInstance>, player_slot: &str, state: &Value) -> Value {
        println!("[lobby] compute_table_ui: Starting for player slot {}", player_slot);
        
        // Use simplified action map computation
        let action_map = compute_action_map(state, &table.bundle);
        
        // Extract zone information from bundle
        let zones = if let Some(zones_def) = table.bundle.zones.as_array() {
            zones_def.iter().filter_map(|zone| {
                if let Some(zone_obj) = zone.as_object() {
                    if let Some(id) = zone_obj.get("id").and_then(|v| v.as_str()) {
                        let mut zone_ui = json!({
                            "id": id,
                        });
                        
                        // Add tier - convert string tiers to numbers
                        if let Some(tier_value) = zone_obj.get("tier") {
                            let tier_num = match tier_value.as_str() {
                                Some("tactical") => 0,
                                Some("strategic") => 1, 
                                Some("meta") => 2,
                                _ => tier_value.as_u64().unwrap_or(0) as i32
                            };
                            zone_ui["tier"] = json!(tier_num);
                        } else {
                            zone_ui["tier"] = json!(0);
                        }
                        
                        // Add zone type if available  
                        if let Some(zone_type) = zone_obj.get("type").and_then(|v| v.as_str()) {
                            zone_ui["type"] = json!(zone_type);
                        }
                        
                        // Add shape information
                        if let Some(shape) = zone_obj.get("shape") {
                            zone_ui["shape"] = shape.clone();
                        }
                        
                        // Add shape metadata
                        if let Some(shape_meta) = zone_obj.get("shapeMeta") {
                            zone_ui["shapeMeta"] = shape_meta.clone();
                        }
                        
                        return Some(zone_ui);
                    }
                }
                None
            }).collect::<Vec<_>>()
        } else {
            vec![]
        };
        
        json!({
            "actionMap": action_map,
            "gameLog": [],
            "zones": zones,
            "entities": table.bundle.entities.clone(),
            "gameMetadata": &table.bundle.manifest,
        })
    }
    
    /// Legacy compute_game_ui method - redirects to compute_table_ui
    #[allow(dead_code)]
    fn compute_game_ui(&self, _game: &Arc<super::game_instance::GameInstance>, player_slot: &str, _state: &Value) -> Value {
        // This method is deprecated but kept for compatibility
        // Return minimal UI data
        json!({
            "actionMap": {},
            "gameLog": [],
            "zones": [],
            "entities": {},
            "gameMetadata": {},
        })
    }
    
    /// Process game action using the new ActionExecutor pipeline
    pub fn process_game_action_v2(
        &self,
        table_id: &str,
        slot: &str,
        action: &Value,
    ) -> Result<Vec<Value>, String> {
        println!("[process_game_action_v2] Starting with action: {:?}", action);
        
        let table = self.state.get_table(table_id)
            .ok_or_else(|| "Table not found".to_string())?;
        
        // Lock the game state for the entire action processing
        let mut state = table.game_state.write();
        
        // Create and configure the action executor
        let mut executor = crate::engine::action_executor::ActionExecutor::new(table.bundle.clone());
        
        // Register specialized verb executors
        executor.register_verb("conditionalAction", Box::new(crate::engine::action_executor::ConditionalActionExecutor));
        
        // Extract action ID and args
        let action_id = if let Some(action_name) = action.get("action").and_then(|a| a.as_str()) {
            action_name.to_string()
        } else if action.get("verb").is_some() {
            // Direct verb execution - create a synthetic action ID
            return Err("Direct verb execution not supported in v2 pipeline".to_string());
        } else {
            return Err("Invalid action format - missing action name".to_string());
        };
        
        // Create initial context
        let initial_args = action.get("args").cloned()
            .or_else(|| {
                // For backward compatibility, collect all non-action fields as args
                if let Some(obj) = action.as_object() {
                    let mut args = serde_json::Map::new();
                    for (k, v) in obj {
                        if k != "action" {
                            args.insert(k.clone(), v.clone());
                        }
                    }
                    Some(Value::Object(args))
                } else {
                    None
                }
            })
            .unwrap_or(json!({}));
        
        let context = crate::engine::action_executor::ActionContext::new(slot.to_string(), initial_args);
        
        // Initialize the execution queue with the user action
        let mut pending_actions = vec![
            crate::engine::action_executor::TriggeredAction {
                action_id,
                context,
                source: crate::engine::action_executor::ActionSource::UserAction,
                priority: 1000, // Highest priority for user actions
            }
        ];
        
        // Collect all patches and logs
        let mut all_patches = Vec::new();
        let mut total_actions_executed = 0;
        let start_time = std::time::Instant::now();
        
        // Process all actions in the queue
        while let Some(triggered_action) = pending_actions.pop() {
            // Sort remaining actions by priority (stable sort preserves insertion order)
            pending_actions.sort_by_key(|a| -a.priority);
            
            println!("[process_game_action_v2] Executing action '{}' from {:?} at depth {}",
                triggered_action.action_id, triggered_action.source, triggered_action.context.depth);
            
            // Execute the action
            match executor.execute_action(
                &mut state,
                &triggered_action.action_id,
                triggered_action.context,
            ) {
                Ok(result) => {
                    println!("[process_game_action_v2] Action produced {} patches and {} triggered actions",
                        result.patches.len(), result.triggered_actions.len());
                    
                    // Collect patches
                    all_patches.extend(result.patches);
                    
                    // Queue triggered actions
                    pending_actions.extend(result.triggered_actions);
                    
                    // Update metrics
                    total_actions_executed += result.metrics.total_actions;
                    
                    // Log execution details in debug mode
                    if false { // TODO: Re-enable when log crate is available
                        for log in &result.logs {
                            println!("[ACTION_LOG] {} - {} - {:?}", 
                                log.action_id, log.verb, log.result);
                        }
                    }
                }
                Err(e) => {
                    println!("[process_game_action_v2] Action execution failed: {}", e);
                    return Err(format!("Action execution failed: {}", e));
                }
            }
        }
        
        // Drop state before processing phases
        drop(state);
        
        // Increment tick
        *table.tick.lock() += 1;
        
        // Process phase changes after all actions
        let mut phase_state = table.game_state.write();
        match crate::engine::process_phases(&table.bundle, &mut phase_state) {
            Ok(phase_patches) => {
                println!("[process_game_action_v2] Phase processing produced {} patches", 
                    phase_patches.len());
                all_patches.extend(phase_patches);
            }
            Err(e) => {
                println!("[process_game_action_v2] Phase processing error: {}", e);
            }
        }
        drop(phase_state);
        
        // Log final metrics
        let elapsed = start_time.elapsed();
        println!("[process_game_action_v2] Completed: {} actions executed, {} patches generated, {:?} elapsed",
            total_actions_executed, all_patches.len(), elapsed);
        
        Ok(all_patches)
    }
    
    /// Send current game state to a specific player
    /// This is used when a player reconnects and needs to restore their game view
    pub fn send_game_state_to_player(&self, table_id: &str, username: &str) {
        println!("[send_game_state_to_player] Sending game state for table {} to {}", table_id, username);
        
        let table = match self.state.get_table(table_id) {
            Some(t) => t,
            None => {
                println!("[send_game_state_to_player] ERROR: Table {} not found", table_id);
                return;
            }
        };
        
        // Ensure table is in Playing state
        {
            let status = table.status.read();
            if *status != crate::lobby::table_instance::TableStatus::Playing {
                println!("[send_game_state_to_player] Table {} is not in Playing state", table_id);
                return;
            }
        }
        
        // Get table state and metadata
        let state = table.game_state.read().clone();
        let tick = *table.tick.lock();
        let game_type = table.bundle_id.clone();
        
        // Create player mapping from seats
        let mut player_mapping = std::collections::HashMap::new();
        let seats = table.seats.read();
        let mut user_slot = None;
        
        for (i, seat) in seats.iter().enumerate() {
            if let Some(crate::lobby::table_instance::SeatOccupant::Player(player_id)) = seat {
                let slot = format!("p{}", i + 1);
                player_mapping.insert(slot.clone(), player_id.clone());
                
                // Find this user's slot
                if player_id == username {
                    user_slot = Some(slot);
                }
            }
        }
        drop(seats);
        
        // Determine the player's slot or if they're a spectator
        let (slot, is_spectator) = if let Some(s) = user_slot {
            (s, false)
        } else if table.spectators.read().contains(&username.to_string()) {
            ("".to_string(), true)
        } else {
            // User is neither seated nor spectating
            println!("[send_game_state_to_player] User {} is not part of table {}", username, table_id);
            return;
        };
        
        // Compute UI data for the player
        let ui_data = self.compute_table_ui(&table, &slot, &state);
        
        // Create and send the gameStarted message
        let msg = json!({
            "type": "gameStarted",
            "gameInstanceId": table_id,
            "tableId": table_id,  // Include tableId for client navigation
            "gameId": game_type,
            "tick": tick,
            "you": if is_spectator { Value::Null } else { Value::String(slot) },
            "players": player_mapping.clone(),
            "state": state.clone(),
            "ui": ui_data,
            "to": username,  // Target specific user
        });
        
        // Send to specific player
        println!("[send_game_state_to_player] Sending game state message to {}", username);
        if let Err(e) = self.state.tx.send(msg.to_string()) {
            println!("[send_game_state_to_player] ERROR: Failed to send game state: {:?}", e);
        } else {
            println!("[send_game_state_to_player] Successfully sent game state to {}", username);
        }
    }
}
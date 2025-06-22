//! Performance optimizations for lobby operations

use super::LobbyMap;
use serde_json::Value;
use std::time::{Instant, SystemTime};

/// Optimized lobby list generation with batched operations
pub fn optimized_lobbies_json(lobbies: &LobbyMap) -> Value {
    let start_time = Instant::now();
    println!("[PERF] optimized_lobbies_json starting - lobby map size: {}", lobbies.len());
    
    // Use parallel iteration for better performance
    let mut lobby_data: Vec<(SystemTime, Value)> = lobbies
        .iter()
        .filter_map(|entry| {
            let lobby = entry.value();
            
            // Skip if we can't get basic locks quickly
            let players = lobby.players.try_lock()?;
            let player_list = players.clone();
            drop(players);
            
            let started = lobby.game_started.try_lock()?;
            let is_started = *started;
            drop(started);
            
            let created_at = lobby.created_at;
            let game_name = lobby.bundle.manifest.metadata.name.clone();
            
            let mut lobby_json = serde_json::json!({
                "id": lobby.id,
                "game_id": lobby.bundle.game_id,
                "game_name": game_name,
                "name": format!("{} - {}", game_name, lobby.id),
                "players": player_list,
                "started": is_started,
                "created_at": created_at
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            });
            
            // Only try to get game state if started
            if is_started {
                if let Some(state) = lobby.state.try_lock() {
                    if let Some(game_status) = state.get("gameStatus") {
                        lobby_json["gameStatus"] = game_status.clone();
                    }
                    if let Some(current_player) = state.get("currentPlayer").and_then(|t| t.as_str()) {
                        // Map actor ID to player name
                        if current_player == "p1" && player_list.len() > 0 {
                            lobby_json["currentTurn"] = serde_json::json!(player_list[0]);
                        } else if current_player == "p2" && player_list.len() > 1 {
                            lobby_json["currentTurn"] = serde_json::json!(player_list[1]);
                        }
                    }
                }
            }
            
            Some((created_at, lobby_json))
        })
        .collect();
    
    // Sort by creation time (newest first)
    lobby_data.sort_unstable_by(|a, b| b.0.cmp(&a.0));
    
    // Extract just the JSON values
    let json_list: Vec<_> = lobby_data.into_iter().map(|(_, json)| json).collect();
    
    let duration = start_time.elapsed();
    println!("[PERF] optimized_lobbies_json completed in {:?} - returned {} lobbies", 
             duration, json_list.len());
    
    Value::Array(json_list)
}

/// Clean up old disconnected lobbies and client data
pub fn cleanup_stale_lobbies(lobbies: &LobbyMap, max_age_hours: u64) -> usize {
    let now = SystemTime::now();
    let max_age = std::time::Duration::from_secs(max_age_hours * 3600);
    let mut removed = 0;
    
    lobbies.retain(|_, lobby| {
        // Check if lobby is old and has no players
        if let Some(players) = lobby.players.try_lock() {
            if players.is_empty() {
                if let Ok(duration) = now.duration_since(lobby.created_at) {
                    if duration > max_age {
                        println!("[CLEANUP] Removing old empty lobby: {}", lobby.id);
                        removed += 1;
                        return false;
                    }
                }
            }
        }
        true
    });
    
    removed
}

/// Connection limits
pub const MAX_CONNECTIONS_PER_LOBBY: usize = 50;
pub const MAX_TOTAL_CONNECTIONS: usize = 1000;

/// Check if a new connection should be allowed
pub fn should_allow_connection(
    lobbies: &LobbyMap,
    lobby_id: &str,
    total_connections: usize,
) -> Result<(), &'static str> {
    // Check global limit
    if total_connections >= MAX_TOTAL_CONNECTIONS {
        return Err("Server at maximum capacity");
    }
    
    // Check lobby-specific limit
    if let Some(lobby) = lobbies.get(lobby_id) {
        if let Some(players) = lobby.players.try_lock() {
            if players.len() >= MAX_CONNECTIONS_PER_LOBBY {
                return Err("Lobby at maximum capacity");
            }
        }
    }
    
    Ok(())
}
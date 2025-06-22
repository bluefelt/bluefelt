//! Player management functionality for lobbies

use super::{Lobby, PlayerPreferences};
use serde_json::json;

impl Lobby {
    /// Get the number of players in the lobby
    pub fn players(&self) -> usize {
        let players = self.players.lock();
        players.len()
    }
    
    /// Get a copy of the player list
    pub fn player_list(&self) -> Vec<String> {
        let players = self.players.lock();
        players.clone()
    }

    /// Map a player's username to their actor ID ("p1" or "p2")
    pub fn actor_for_player(&self, username: &str) -> Option<String> {
        let players = self.players.lock();
        players
            .iter()
            .position(|p| p == username)
            .map(|idx| format!("p{}", idx + 1))
    }

    /// Add a player to the lobby
    pub fn add_player(&self, player_id: String) -> bool {
        let mut players = self.players.lock();
        
        // If this is the same player reconnecting, allow it
        if players.contains(&player_id) {
            println!("[Socket] Player {} is reconnecting to the lobby", player_id);
            return true;
        }
        
        // Check if we already have max players for this game
        if players.len() < self.bundle.manifest.metadata.players.max as usize {
            println!("[Socket] Adding new player {} to the lobby", player_id);
            players.push(player_id);
            drop(players);
            self.request_lobby_list_broadcast();
            return true;
        }
        
        println!("Could not add player {} - lobby is full", player_id);
        false
    }

    /// Remove a player from the lobby
    pub fn remove_player(&self, player_id: &str) -> bool {
        let mut players = self.players.lock();
        let before_len = players.len();
        players.retain(|id| id != player_id);

        if players.len() < before_len {
            println!("[Socket] Player {} removed from lobby", player_id);
            drop(players);
            self.request_lobby_list_broadcast();
            return true;
        }
        
        println!("[Socket] ERROR: Player {} was not in the lobby and could not be removed", player_id);
        false
    }

    /// Update player preferences and broadcast to other players
    pub fn update_player_preferences(&self, player_id: &str, preferences: PlayerPreferences) {
        {
            let mut prefs = self.player_preferences.lock();
            prefs.insert(player_id.to_string(), preferences);
        }
        
        // Broadcast updated player preferences to all clients
        self.broadcast_player_preferences();
    }
    
    /// Get player preferences for a specific player
    pub fn get_player_preferences(&self, player_id: &str) -> Option<PlayerPreferences> {
        let prefs = self.player_preferences.lock();
        prefs.get(player_id).cloned()
    }
    
    /// Get all player preferences as JSON for broadcasting
    pub fn get_all_player_preferences(&self) -> serde_json::Value {
        let prefs = self.player_preferences.lock();
        let mut result = serde_json::Map::new();
        
        for (player_id, pref) in prefs.iter() {
            // Map username to actor ID for consistency
            if let Some(actor_id) = self.actor_for_player(&pref.username) {
                result.insert(actor_id, json!({
                    "username": pref.username,
                    "tokenId": pref.token_id,
                    "colorSchemeId": pref.color_scheme_id,
                    "playerColor": pref.player_color,
                    "cardStyleId": pref.card_style_id
                }));
            }
        }
        
        json!(result)
    }
    
    /// Broadcast player preferences to all connected clients
    pub fn broadcast_player_preferences(&self) {
        let preferences = self.get_all_player_preferences();
        let message = json!({
            "type": "playerPreferencesUpdate",
            "preferences": preferences
        });
        let _ = self.tx.send(axum::extract::ws::Message::Text(message.to_string()));
    }
}
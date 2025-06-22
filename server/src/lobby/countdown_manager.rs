//! Countdown management for game start
//!
//! This module handles the countdown timer when all players are ready,
//! including automatic game start and cancellation logic.

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use super::table_instance::{TableInstance, TableStatus, SeatOccupant};
use super::lobby_impl::Lobby;
use rand::RngCore;

pub struct CountdownManager;

impl CountdownManager {
    /// Start a countdown for the table
    /// Returns true if countdown was started, false if already counting down
    pub fn start_countdown(
        table: &Arc<TableInstance>,
        lobby: &Arc<Lobby>,
        duration_secs: u64,
    ) -> bool {
        // Check if already counting down
        {
            let status = table.status.read();
            if matches!(*status, TableStatus::Countdown) {
                return false;
            }
        }
        
        // Set countdown end time
        let ends_at = SystemTime::now() + Duration::from_secs(duration_secs);
        {
            let mut countdown = table.countdown_ends_at.write();
            *countdown = Some(ends_at);
        }
        
        // Update status
        {
            let mut status = table.status.write();
            *status = TableStatus::Countdown;
        }
        
        // Spawn countdown task
        let table_clone = Arc::clone(table);
        let lobby_clone = Arc::clone(lobby);
        let table_id = table.id.clone();
        
        tokio::spawn(async move {
            // Wait for countdown duration
            sleep(Duration::from_secs(duration_secs)).await;
            
            // Check if countdown is still active (might have been cancelled)
            let should_start = {
                let countdown = table_clone.countdown_ends_at.read();
                let status = table_clone.status.read();
                countdown.is_some() && matches!(*status, TableStatus::Countdown)
            };
            
            if should_start {
                // Start the game
                if let Err(e) = Self::start_game_from_countdown(&table_clone, &lobby_clone) {
                    eprintln!("Failed to start game from countdown: {}", e);
                    // Reset to waiting state
                    let mut status = table_clone.status.write();
                    *status = TableStatus::Open;
                    let mut countdown = table_clone.countdown_ends_at.write();
                    *countdown = None;
                }
            }
        });
        
        true
    }
    
    /// Cancel an active countdown
    pub fn cancel_countdown(table: &Arc<TableInstance>) -> bool {
        let mut countdown = table.countdown_ends_at.write();
        let mut status = table.status.write();
        
        if countdown.is_some() && matches!(*status, TableStatus::Countdown) {
            *countdown = None;
            *status = TableStatus::Open;
            true
        } else {
            false
        }
    }
    
    /// Check if countdown should be cancelled (not enough ready players)
    pub fn check_countdown_conditions(table: &Arc<TableInstance>) -> bool {
        let seats = table.seats.read();
        let ready_states = table.ready_states.read();
        
        // Count seated and ready players
        let mut ready_count = 0;
        for (i, seat) in seats.iter().enumerate() {
            if seat.is_some() && ready_states[i] {
                ready_count += 1;
            }
        }
        
        // Need all seated players to be ready AND minimum players
        let seated_count = seats.iter().filter(|s| s.is_some()).count();
        ready_count == seated_count && ready_count >= table.min_players as usize
    }
    
    /// Start the game from countdown completion
    fn start_game_from_countdown(
        table: &Arc<TableInstance>,
        lobby: &Arc<Lobby>,
    ) -> Result<(), String> {
        // Clear countdown
        {
            let mut countdown = table.countdown_ends_at.write();
            *countdown = None;
        }
        
        // Create game instance ID
        let game_instance_id = nanoid::nanoid!(10);
        
        // Get seated players for game initialization
        let player_mapping = {
            let seats = table.seats.read();
            let mut mapping = std::collections::HashMap::new();
            for (i, seat) in seats.iter().enumerate() {
                if let Some(SeatOccupant::Player(player_id)) = seat {
                    // Map seat index to player slot (p1, p2, etc.)
                    let slot = format!("p{}", i + 1);
                    mapping.insert(slot, player_id.clone());
                }
            }
            mapping
        };
        
        // Initialize game state using the bundle
        let player_names: Vec<String> = (0..table.max_players)
            .map(|i| player_mapping.get(&format!("p{}", i + 1)).cloned().unwrap_or_else(|| format!("Player{}", i + 1)))
            .collect();
        
        let mut rng = rand::rng();
        let initial_state = crate::engine::state::load_initial_state_with_player_names(
            &table.bundle,
            &player_names,
            &mut rng,
        );
        
        // Update table with game state and process phases
        {
            let mut game_state = table.game_state.write();
            *game_state = initial_state;
            
            // Process initial phase transitions
            println!("[countdown] Processing initial phases for game {}", table.id);
            println!("[countdown] Current phase state: {:?}", game_state.get("phases"));
            
            match crate::engine::process_phases(&table.bundle, &mut game_state) {
                Ok(patches) => {
                    println!("[countdown] Phase processing successful, generated {} patches", patches.len());
                    for (i, patch) in patches.iter().enumerate() {
                        println!("[countdown] Patch {}: {:?}", i, patch);
                    }
                    
                    // Apply the patches to the game state
                    if !patches.is_empty() {
                        println!("[countdown] Applying {} patches to game state", patches.len());
                        for patch in &patches {
                            crate::engine::apply_patch_to_state(&mut game_state, patch);
                        }
                        println!("[countdown] After patches, phase state: {:?}", game_state.get("phases"));
                    }
                }
                Err(e) => {
                    println!("[countdown] ERROR: Failed to process initial phases: {}", e);
                }
            }
        }
        
        // Update status
        {
            let mut status = table.status.write();
            *status = TableStatus::Playing;
        }
        
        // Broadcast game started event
        Self::broadcast_game_started(lobby, &table.id, &game_instance_id);
        
        Ok(())
    }
    
    /// Broadcast that a game has started from a table
    fn broadcast_game_started(lobby: &Arc<Lobby>, table_id: &str, game_instance_id: &str) {
        // Use the legacy broadcast_game_started for compatibility
        lobby.broadcast_game_started(table_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::Bundle;
    
    fn create_test_table() -> Arc<TableInstance> {
        let bundle = Arc::new(Bundle {
            game_id: "test-game".to_string(),
            manifest: crate::bundle::Manifest {
                game_id: "test-game".to_string(),
                version: "1.0".to_string(),
                spec_version: "1.0".to_string(),
                metadata: crate::bundle::ManifestMetadata {
                    name: "Test Game".to_string(),
                    author: "Test".to_string(),
                    players: crate::bundle::PlayersRange {
                        min: 2,
                        max: 4,
                    },
                    description: "Test game".to_string(),
                },
                phases: None,
                setup: None,
                zone_groups: None,
            },
            entities: serde_json::Value::Null,
            zones: serde_json::Value::Null,
            actions: serde_json::Value::Null,
            phases: serde_json::Value::Null,
        });
        
        Arc::new(TableInstance::new_simple(
            "test-game".to_string(),
            bundle,
            "owner123".to_string(),
        ))
    }
    
    #[test]
    fn test_countdown_conditions() {
        let table = create_test_table();
        
        // No players ready - should not countdown
        assert!(!CountdownManager::check_countdown_conditions(&table));
        
        // Add players and set ready
        {
            let mut seats = table.seats.write();
            seats[0] = Some(SeatOccupant::Player("p1".to_string()));
            seats[1] = Some(SeatOccupant::Player("p2".to_string()));
        }
        
        {
            let mut ready = table.ready_states.write();
            ready[0] = true;
            ready[1] = true;
        }
        
        // Now should be ready for countdown
        assert!(CountdownManager::check_countdown_conditions(&table));
    }
}
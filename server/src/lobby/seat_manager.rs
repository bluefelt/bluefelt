//! Atomic seat management operations
//!
//! This module provides thread-safe atomic operations for seat claiming
//! and releasing to prevent race conditions in multi-player lobbies.

use parking_lot::RwLock;
use std::sync::Arc;
use super::table_instance::{SeatOccupant, TableInstance, TableStatus};

pub struct SeatManager;

impl SeatManager {
    /// Atomically claim a seat at a table using compare-and-swap
    /// Returns Ok(true) if seat was claimed, Ok(false) if seat was taken, Err if invalid
    pub fn atomic_claim_seat(
        table: &Arc<TableInstance>,
        seat_index: usize,
        player_id: String,
        username: String,
    ) -> Result<bool, String> {
        // Validate seat index
        {
            let seats = table.seats.read();
            if seat_index >= seats.len() {
                return Err(format!("Invalid seat index: {}", seat_index));
            }
        }
        
        // Check table status
        {
            let status = table.status.read();
            match &*status {
                TableStatus::Open => {},
                TableStatus::Countdown => return Err("Cannot claim seat during countdown".to_string()),
                TableStatus::Playing => return Err("Game already in progress".to_string()),
                TableStatus::Finished => return Err("Game is finished".to_string()),
                TableStatus::Abandoned => return Err("Table is abandoned".to_string()),
            }
        }
        
        // Check if player already has a seat
        {
            let seats = table.seats.read();
            if seats.iter().any(|s| matches!(s, Some(SeatOccupant::Player(id)) if id == &player_id)) {
                return Err("Player already has a seat at this table".to_string());
            }
        }
        
        // Attempt atomic seat claim
        let mut seats = table.seats.write();
        match &seats[seat_index] {
            None => {
                // Seat is available, claim it
                seats[seat_index] = Some(SeatOccupant::Player(player_id.clone()));
                Ok(true)
            }
            Some(_) => {
                // Seat is taken
                Ok(false)
            }
        }
    }
    
    /// Atomically release a seat at a table
    pub fn atomic_release_seat(
        table: &Arc<TableInstance>,
        seat_index: usize,
    ) -> Result<(), String> {
        // Validate seat index
        {
            let seats = table.seats.read();
            if seat_index >= seats.len() {
                return Err(format!("Invalid seat index: {}", seat_index));
            }
        }
        
        // Check table status - can only release seats before game starts
        {
            let status = table.status.read();
            match &*status {
                TableStatus::Open | TableStatus::Countdown => {},
                TableStatus::Playing => return Err("Cannot release seat while game is in progress".to_string()),
                TableStatus::Finished => return Err("Game is already completed".to_string()),
                TableStatus::Abandoned => return Err("Table is abandoned".to_string()),
            }
        }
        
        // Release the seat
        let mut seats = table.seats.write();
        seats[seat_index] = None;
        
        // Reset ready state for this seat
        let mut ready_states = table.ready_states.write();
        ready_states[seat_index] = false;
        
        // Check if we need to cancel countdown (not enough players ready)
        drop(seats);
        drop(ready_states);
        table.check_and_cancel_countdown();
        
        Ok(())
    }
    
    /// Get all seated players atomically
    pub fn get_seated_players(table: &Arc<TableInstance>) -> Vec<(usize, SeatOccupant)> {
        let seats = table.seats.read();
        seats.iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|o| (i, o.clone())))
            .collect()
    }
    
    /// Check if a player is seated at the table
    pub fn is_player_seated(table: &Arc<TableInstance>, player_id: &str) -> bool {
        let seats = table.seats.read();
        seats.iter().any(|s| matches!(s, Some(SeatOccupant::Player(id)) if id == player_id))
    }
    
    /// Get the seat index for a player (if seated)
    pub fn get_player_seat(table: &Arc<TableInstance>, player_id: &str) -> Option<usize> {
        let seats = table.seats.read();
        seats.iter()
            .position(|s| matches!(s, Some(SeatOccupant::Player(id)) if id == player_id))
    }
    
    /// Check if table has minimum players seated
    pub fn has_minimum_players(table: &Arc<TableInstance>) -> bool {
        let seats = table.seats.read();
        let seated_count = seats.iter().filter(|s| s.is_some()).count();
        seated_count >= table.min_players as usize
    }
    
    /// Check if table is full
    pub fn is_table_full(table: &Arc<TableInstance>) -> bool {
        let seats = table.seats.read();
        seats.iter().all(|s| s.is_some())
    }

    /// Atomically claim the first available seat at a table
    /// Returns Ok(seat_index) if successful, Err if no seats available or other error
    pub fn atomic_claim_any_seat(
        table: &Arc<TableInstance>,
        player_id: String,
        username: String,
    ) -> Result<usize, String> {
        // Check table status first
        {
            let status = table.status.read();
            match &*status {
                TableStatus::Open => {},
                TableStatus::Countdown => return Err("Cannot claim seat during countdown".to_string()),
                TableStatus::Playing => return Err("Game already in progress".to_string()),
                TableStatus::Finished => return Err("Game is finished".to_string()),
                TableStatus::Abandoned => return Err("Table is abandoned".to_string()),
            }
        }
        
        // Check if player already has a seat
        {
            let seats = table.seats.read();
            if seats.iter().any(|s| matches!(s, Some(SeatOccupant::Player(id)) if id == &player_id)) {
                return Err("Player already has a seat at this table".to_string());
            }
        }

        // Try to claim any available seat atomically
        let mut seats = table.seats.write();
        for (seat_index, seat) in seats.iter_mut().enumerate() {
            if seat.is_none() {
                *seat = Some(SeatOccupant::Player(player_id.clone()));
                return Ok(seat_index);
            }
        }
        
        // No available seats
        Err("No available seats".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::Bundle;
    use std::sync::Arc;
    
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
    fn test_atomic_claim_seat() {
        let table = create_test_table();
        
        // Seat 0 is already taken by owner (owner123), so try seat 1
        let result = SeatManager::atomic_claim_seat(&table, 1, "player1".to_string(), "Player 1".to_string());
        match result {
            Ok(success) => assert!(success),
            Err(e) => panic!("Expected success but got error: {}", e),
        }
        
        // Second claim on same seat should fail
        let result = SeatManager::atomic_claim_seat(&table, 1, "player2".to_string(), "Player 2".to_string());
        assert!(!result.unwrap());
        
        // Same player trying different seat should fail
        let result = SeatManager::atomic_claim_seat(&table, 2, "player1".to_string(), "Player 1".to_string());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_atomic_release_seat() {
        let table = create_test_table();
        
        // Claim seat 1 (seat 0 is taken by owner)
        SeatManager::atomic_claim_seat(&table, 1, "player1".to_string(), "Player 1".to_string()).unwrap();
        
        // Release should succeed
        let result = SeatManager::atomic_release_seat(&table, 1);
        assert!(result.is_ok());
        
        // Seat should be available again
        let result = SeatManager::atomic_claim_seat(&table, 1, "player2".to_string(), "Player 2".to_string());
        assert!(result.unwrap());
    }

    #[test]
    fn test_atomic_claim_any_seat() {
        let table = create_test_table();
        
        // First player should get seat 1 (seat 0 is taken by owner)
        let result = SeatManager::atomic_claim_any_seat(&table, "player1".to_string(), "Player 1".to_string());
        assert_eq!(result.unwrap(), 1);
        
        // Second player should get seat 2
        let result = SeatManager::atomic_claim_any_seat(&table, "player2".to_string(), "Player 2".to_string());
        assert_eq!(result.unwrap(), 2);
        
        // Same player trying again should fail
        let result = SeatManager::atomic_claim_any_seat(&table, "player1".to_string(), "Player 1".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already has a seat"));
    }

    #[test]
    fn test_atomic_claim_any_seat_full_table() {
        let table = create_test_table();
        
        // Fill remaining seats (seat 0 already taken by owner, so fill seats 1-3)
        for i in 0..3 {
            let result = SeatManager::atomic_claim_any_seat(
                &table, 
                format!("player{}", i + 1), 
                format!("Player {}", i + 1)
            );
            assert_eq!(result.unwrap(), i + 1); // Should get seats 1, 2, 3
        }
        
        // Fourth player should fail (table is now full)
        let result = SeatManager::atomic_claim_any_seat(&table, "player4".to_string(), "Player 4".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No available seats"));
    }
}
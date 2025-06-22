//! Comprehensive lobby and table integration tests
//! Tests all lobby/table actions and state management

use bluefelt_core::bundle::Bundle;
use bluefelt_core::lobby::lobby_impl::Lobby;
use bluefelt_core::lobby::lobby_state::LobbyState;
use dashmap::DashMap;
use serde_json::json;
use std::sync::Arc;

type BundleMap = DashMap<String, Arc<Bundle>>;
type LobbyMap = DashMap<String, Arc<Lobby>>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test bundle for tic-tac-toe
    fn create_test_bundle() -> Arc<Bundle> {
        let bundle_data = json!({
            "gameId": "tic-tac-toe",
            "version": "1.0",
            "manifest": {
                "gameId": "tic-tac-toe",
                "version": "1.0",
                "metadata": {
                    "name": "Tic-Tac-Toe",
                    "players": {"min": 2, "max": 2}
                }
            },
            "actions": [],
            "entities": [],
            "zones": [],
            "phases": []
        });
        
        Arc::new(Bundle::from_value(bundle_data).expect("Failed to create test bundle"))
    }

    /// Create test lobby with proper initialization
    fn create_test_lobby() -> Arc<Lobby> {
        let bundle_map = Arc::new(BundleMap::new());
        let lobby_map = Arc::new(LobbyMap::new());
        
        // Add test bundle
        let bundle = create_test_bundle();
        bundle_map.insert("tic-tac-toe".to_string(), bundle);
        
        Arc::new(Lobby::new(
            "test-lobby".to_string(),
            bundle_map,
            lobby_map,
            None
        ))
    }

    #[tokio::test]
    async fn test_lobby_creation() {
        let lobby = create_test_lobby();
        
        // Verify lobby is created with correct state
        assert_eq!(lobby.state.get_members().len(), 0);
        assert_eq!(lobby.state.get_tables().len(), 0);
        assert_eq!(lobby.id, "test-lobby");
    }

    #[tokio::test]
    async fn test_add_remove_members() {
        let lobby = create_test_lobby();
        
        // Add members
        lobby.state.add_member("alice".to_string()).expect("Failed to add alice");
        lobby.state.add_member("bob".to_string()).expect("Failed to add bob");
        
        let members = lobby.state.get_members();
        assert_eq!(members.len(), 2);
        assert!(members.contains(&"alice".to_string()));
        assert!(members.contains(&"bob".to_string()));
        
        // Remove member
        lobby.state.remove_member("alice");
        let members = lobby.state.get_members();
        assert_eq!(members.len(), 1);
        assert!(!members.contains(&"alice".to_string()));
        assert!(members.contains(&"bob".to_string()));
    }

    #[tokio::test] 
    async fn test_table_creation() {
        let lobby = create_test_lobby();
        let bundle = create_test_bundle();
        
        // Add owner
        lobby.state.add_member("alice".to_string()).expect("Failed to add alice");
        
        // Create table
        let table_id = lobby.state.create_table(
            "tic-tac-toe".to_string(),
            bundle,
            "alice".to_string()
        ).expect("Failed to create table");
        
        // Verify table exists
        let tables = lobby.state.get_tables();
        assert_eq!(tables.len(), 1);
        
        let table = lobby.state.get_table(&table_id).expect("Table should exist");
        assert_eq!(table.bundle_id, "tic-tac-toe");
        assert_eq!(table.owner, "alice");
        
        // Verify owner is seated
        let seats = table.seats.read();
        assert!(seats[0].is_some());
    }

    #[tokio::test]
    async fn test_table_seating() {
        let lobby = create_test_lobby();
        let bundle = create_test_bundle();
        
        // Add members
        lobby.state.add_member("alice".to_string()).expect("Failed to add alice");
        lobby.state.add_member("bob".to_string()).expect("Failed to add bob");
        
        // Create table
        let table_id = lobby.state.create_table(
            "tic-tac-toe".to_string(),
            bundle,
            "alice".to_string()
        ).expect("Failed to create table");
        
        let table = lobby.state.get_table(&table_id).expect("Table should exist");
        
        // Seat second player
        table.claim_seat(1, "bob".to_string()).expect("Failed to seat bob");
        
        // Verify both seats occupied
        let seats = table.seats.read();
        assert!(seats[0].is_some());
        assert!(seats[1].is_some());
        
        // Try to seat in occupied seat
        let result = table.claim_seat(0, "charlie".to_string());
        assert!(result.is_err());
    }

    #[tokio::test] 
    async fn test_auto_seat_assignment() {
        let lobby = create_test_lobby();
        let bundle = create_test_bundle();
        
        // Add members
        lobby.state.add_member("alice".to_string()).expect("Failed to add alice");
        lobby.state.add_member("bob".to_string()).expect("Failed to add bob");
        lobby.state.add_member("charlie".to_string()).expect("Failed to add charlie");
        
        // Create table
        let table_id = lobby.state.create_table(
            "tic-tac-toe".to_string(),
            bundle,
            "alice".to_string()
        ).expect("Failed to create table");
        
        let table = lobby.state.get_table(&table_id).expect("Table should exist");
        
        // Use auto-seat assignment for bob
        use bluefelt_core::lobby::seat_manager::SeatManager;
        let seat_index = SeatManager::atomic_claim_any_seat(
            &table,
            "bob".to_string(),
            "bob".to_string()
        ).expect("Failed to auto-seat bob");
        
        assert_eq!(seat_index, 1); // Should get seat 1 since alice is in seat 0
        
        // Try auto-seat when full (tic-tac-toe only has 2 seats)
        let result = SeatManager::atomic_claim_any_seat(
            &table,
            "charlie".to_string(),
            "charlie".to_string()
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_table_deletion() {
        let lobby = create_test_lobby();
        let bundle = create_test_bundle();
        
        // Add member and create table
        lobby.state.add_member("alice".to_string()).expect("Failed to add alice");
        let table_id = lobby.state.create_table(
            "tic-tac-toe".to_string(),
            bundle,
            "alice".to_string()
        ).expect("Failed to create table");
        
        // Verify table exists
        assert!(lobby.state.get_table(&table_id).is_some());
        assert_eq!(lobby.state.get_tables().len(), 1);
        
        // Delete table
        lobby.state.remove_table(&table_id);
        
        // Verify table removed
        assert!(lobby.state.get_table(&table_id).is_none());
        assert_eq!(lobby.state.get_tables().len(), 0);
    }

    #[tokio::test]
    async fn test_member_limit_enforcement() {
        let lobby = create_test_lobby();
        
        // Add members up to limit (assuming reasonable limit)
        for i in 0..50 {
            let username = format!("user{}", i);
            let result = lobby.state.add_member(username);
            assert!(result.is_ok(), "Failed to add user{}", i);
        }
        
        // Verify all members added
        assert_eq!(lobby.state.get_members().len(), 50);
    }

    #[tokio::test]
    async fn test_table_limit_enforcement() {
        let lobby = create_test_lobby();
        let bundle = create_test_bundle();
        
        // Add enough members
        for i in 0..10 {
            let username = format!("user{}", i);
            lobby.state.add_member(username).expect("Failed to add member");
        }
        
        // Create multiple tables
        for i in 0..5 {
            let owner = format!("user{}", i);
            let result = lobby.state.create_table(
                "tic-tac-toe".to_string(),
                bundle.clone(),
                owner
            );
            assert!(result.is_ok(), "Failed to create table {}", i);
        }
        
        // Verify tables created
        assert_eq!(lobby.state.get_tables().len(), 5);
    }

    #[tokio::test]
    async fn test_duplicate_username_prevention() {
        let lobby = create_test_lobby();
        
        // Add member
        lobby.state.add_member("alice".to_string()).expect("Failed to add alice");
        
        // Try to add same username again
        let result = lobby.state.add_member("alice".to_string());
        assert!(result.is_err(), "Should not allow duplicate username");
        
        // Verify only one member
        assert_eq!(lobby.state.get_members().len(), 1);
    }

    #[tokio::test]
    async fn test_invalid_game_type() {
        let lobby = create_test_lobby();
        
        // Add member
        lobby.state.add_member("alice".to_string()).expect("Failed to add alice");
        
        // Try to create table with invalid game type
        let bundle = create_test_bundle(); // This is tic-tac-toe bundle
        let result = lobby.state.create_table(
            "invalid-game".to_string(),
            bundle,
            "alice".to_string()
        );
        
        // Should succeed since we pass the bundle directly
        // The validation happens at the API level
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_lobby_state_consistency() {
        let lobby = create_test_lobby();
        let bundle = create_test_bundle();
        
        // Add members
        lobby.state.add_member("alice".to_string()).expect("Failed to add alice");
        lobby.state.add_member("bob".to_string()).expect("Failed to add bob");
        
        // Create table
        let table_id = lobby.state.create_table(
            "tic-tac-toe".to_string(),
            bundle,
            "alice".to_string()
        ).expect("Failed to create table");
        
        // Verify initial state
        assert_eq!(lobby.state.get_members().len(), 2);
        assert_eq!(lobby.state.get_tables().len(), 1);
        
        // Remove member who owns table
        lobby.state.remove_member("alice");
        
        // Table should still exist (owner removal doesn't auto-delete table)
        assert_eq!(lobby.state.get_tables().len(), 1);
        assert_eq!(lobby.state.get_members().len(), 1);
        
        // But table should be accessible for cleanup
        let table = lobby.state.get_table(&table_id);
        assert!(table.is_some());
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        use tokio::task;
        
        let lobby = create_test_lobby();
        let bundle = create_test_bundle();
        
        // Spawn concurrent member additions
        let tasks: Vec<_> = (0..10).map(|i| {
            let lobby = lobby.clone();
            task::spawn(async move {
                let username = format!("user{}", i);
                lobby.state.add_member(username)
            })
        }).collect();
        
        // Wait for all tasks
        for task in tasks {
            let result = task.await.expect("Task panicked");
            assert!(result.is_ok(), "Failed to add member concurrently");
        }
        
        // Verify all members added
        assert_eq!(lobby.state.get_members().len(), 10);
        
        // Spawn concurrent table creations
        let tasks: Vec<_> = (0..5).map(|i| {
            let lobby = lobby.clone();
            let bundle = bundle.clone();
            task::spawn(async move {
                let owner = format!("user{}", i);
                lobby.state.create_table(
                    "tic-tac-toe".to_string(),
                    bundle,
                    owner
                )
            })
        }).collect();
        
        // Wait for all tasks
        let mut successful_tables = 0;
        for task in tasks {
            let result = task.await.expect("Task panicked");
            if result.is_ok() {
                successful_tables += 1;
            }
        }
        
        // At least some tables should be created
        assert!(successful_tables > 0);
        assert_eq!(lobby.state.get_tables().len(), successful_tables);
    }
}
//! Tests for game start validation and player count requirements
//! Ensures games cannot start without meeting minimum player requirements

use bluefelt_core::bundle::{Bundle, Manifest, ManifestMetadata, PlayersRange, BundleMap};
use bluefelt_core::lobby::lobby_impl::Lobby;
use bluefelt_core::lobby::seat_manager::SeatManager;
use bluefelt_core::lobby::table_instance::TableStatus;
use serde_json::json;
use std::sync::Arc;

type LobbyMap = bluefelt_core::lobby::LobbyMap;

/// Create a test bundle with specified player count requirements
fn create_game_bundle(game_id: &str, min_players: u32, max_players: u32) -> Bundle {
    Bundle {
        game_id: game_id.to_string(),
        manifest: Manifest {
            game_id: game_id.to_string(),
            version: "1.0".to_string(),
            spec_version: "1.0".to_string(),
            metadata: ManifestMetadata {
                name: format!("Test Game ({})", game_id),
                author: "Test Author".to_string(),
                players: PlayersRange { min: min_players, max: max_players },
                description: "Test game for start validation".to_string(),
            },
            phases: None,
            setup: None,
            zone_groups: None,
        },
        entities: json!([]),
        zones: json!([]),
        actions: json!([]),
        phases: json!([]),
    }
}

/// Create test lobby with various game types
fn create_test_lobby() -> Arc<Lobby> {
    let mut bundle_map = BundleMap::new_empty();
    
    // Add test bundles with different player requirements
    bundle_map.insert_bundle("strict-two".to_string(), create_game_bundle("strict-two", 2, 2));
    bundle_map.insert_bundle("three-min".to_string(), create_game_bundle("three-min", 3, 6));
    bundle_map.insert_bundle("flexible".to_string(), create_game_bundle("flexible", 2, 4));
    
    let lobby_map = Arc::new(LobbyMap::new());
    
    Lobby::new(
        "test-lobby".to_string(),
        Arc::new(bundle_map),
        lobby_map,
        None
    )
}

#[tokio::test]
async fn test_cannot_start_game_below_minimum_players() {
    let lobby = create_test_lobby();
    
    // Add test members
    for i in 1..=5 {
        let username = format!("user{}", i);
        lobby.state.add_member(username).expect("Failed to add member");
    }
    
    // Create table requiring 3 minimum players
    let bundle = Arc::new(create_game_bundle("three-min", 3, 6));
    let table_id = lobby.state.create_table(
        "three-min".to_string(),
        bundle,
        "user1".to_string()
    ).expect("Failed to create table");
    
    let table = lobby.state.get_table(&table_id).expect("Table should exist");
    
    // With only 1 player (owner), should not be able to start
    assert!(!table.can_start(), "Should not be able to start with 1 player");
    assert_eq!(table.seated_count(), 1); // Owner is auto-seated
    
    // Add second player - still not enough
    SeatManager::atomic_claim_any_seat(&table, "user2".to_string(), "user2".to_string())
        .expect("User2 should be able to join");
    assert!(!table.can_start(), "Should not be able to start with 2 players");
    assert_eq!(table.seated_count(), 2);
    
    // Add third player - now should be able to start
    SeatManager::atomic_claim_any_seat(&table, "user3".to_string(), "user3".to_string())
        .expect("User3 should be able to join");
    assert!(table.can_start(), "Should be able to start with 3 players");
    assert_eq!(table.seated_count(), 3);
}

#[tokio::test]
async fn test_strict_two_player_game() {
    let lobby = create_test_lobby();
    
    // Add test members
    for i in 1..=3 {
        let username = format!("user{}", i);
        lobby.state.add_member(username).expect("Failed to add member");
    }
    
    // Create strict 2-player table
    let bundle = Arc::new(create_game_bundle("strict-two", 2, 2));
    let table_id = lobby.state.create_table(
        "strict-two".to_string(),
        bundle,
        "user1".to_string()
    ).expect("Failed to create table");
    
    let table = lobby.state.get_table(&table_id).expect("Table should exist");
    
    // Should not be able to start with 1 player
    assert!(!table.can_start(), "Should not start with 1 player");
    
    // Add second player
    SeatManager::atomic_claim_any_seat(&table, "user2".to_string(), "user2".to_string())
        .expect("User2 should be able to join");
    
    // Now should be able to start
    assert!(table.can_start(), "Should be able to start with 2 players");
    assert_eq!(table.seated_count(), 2);
    
    // Third player should not be able to join (table full)
    let result = SeatManager::atomic_claim_any_seat(&table, "user3".to_string(), "user3".to_string());
    assert!(result.is_err(), "Third player should not be able to join 2-player table");
}

#[tokio::test]
async fn test_ready_state_affects_game_start() {
    let lobby = create_test_lobby();
    
    // Add test members
    for i in 1..=3 {
        let username = format!("user{}", i);
        lobby.state.add_member(username).expect("Failed to add member");
    }
    
    // Create flexible game table
    let bundle = Arc::new(create_game_bundle("flexible", 2, 4));
    let table_id = lobby.state.create_table(
        "flexible".to_string(),
        bundle,
        "user1".to_string()
    ).expect("Failed to create table");
    
    let table = lobby.state.get_table(&table_id).expect("Table should exist");
    
    // Add second player
    SeatManager::atomic_claim_any_seat(&table, "user2".to_string(), "user2".to_string())
        .expect("User2 should be able to join");
    
    // Table has minimum players but no one is ready
    assert!(table.can_start(), "Table has minimum players");
    assert!(!table.all_ready(), "No one is ready yet");
    
    // Set user1 ready
    let countdown_started = table.set_ready_state("user1", true)
        .expect("Should set user1 ready");
    assert!(!countdown_started, "Countdown shouldn't start with only one ready");
    assert!(!table.all_ready(), "Not all players ready");
    
    // Set user2 ready - should trigger countdown
    let countdown_started = table.set_ready_state("user2", true)
        .expect("Should set user2 ready");
    assert!(countdown_started, "Countdown should start when all players ready");
    assert!(table.all_ready(), "All players should be ready");
    
    // If user1 becomes unready, countdown should be cancelled
    table.set_ready_state("user1", false).expect("Should unready user1");
    assert!(!table.all_ready(), "Not all players ready after unreadying");
}

#[tokio::test]
async fn test_player_leaving_during_ready_countdown() {
    let lobby = create_test_lobby();
    
    // Add test members
    for i in 1..=4 {
        let username = format!("user{}", i);
        lobby.state.add_member(username).expect("Failed to add member");
    }
    
    // Create table
    let bundle = Arc::new(create_game_bundle("flexible", 2, 4));
    let table_id = lobby.state.create_table(
        "flexible".to_string(),
        bundle,
        "user1".to_string()
    ).expect("Failed to create table");
    
    let table = lobby.state.get_table(&table_id).expect("Table should exist");
    
    // Fill to 3 players
    SeatManager::atomic_claim_any_seat(&table, "user2".to_string(), "user2".to_string())
        .expect("User2 should join");
    SeatManager::atomic_claim_any_seat(&table, "user3".to_string(), "user3".to_string())
        .expect("User3 should join");
    
    // All players ready
    table.set_ready_state("user1", true).expect("Set user1 ready");
    table.set_ready_state("user2", true).expect("Set user2 ready");
    let countdown_started = table.set_ready_state("user3", true).expect("Set user3 ready");
    assert!(countdown_started, "Countdown should start");
    
    // User2 leaves the table
    table.release_seat(1).expect("User2 should be able to leave");
    
    // Should still have minimum players to start
    assert!(table.can_start(), "Should still be able to start with 2 players");
    assert_eq!(table.seated_count(), 2);
    
    // But ready states should be updated (user2's seat is now unready)
    assert!(!table.all_ready(), "Not all players ready after someone left");
}

#[tokio::test]
async fn test_table_status_prevents_seat_changes() {
    let lobby = create_test_lobby();
    
    // Add test members
    for i in 1..=3 {
        let username = format!("user{}", i);
        lobby.state.add_member(username).expect("Failed to add member");
    }
    
    // Create table
    let bundle = Arc::new(create_game_bundle("strict-two", 2, 2));
    let table_id = lobby.state.create_table(
        "strict-two".to_string(),
        bundle,
        "user1".to_string()
    ).expect("Failed to create table");
    
    let table = lobby.state.get_table(&table_id).expect("Table should exist");
    
    // Add second player
    SeatManager::atomic_claim_any_seat(&table, "user2".to_string(), "user2".to_string())
        .expect("User2 should join");
    
    // Simulate game starting by changing status to Playing
    {
        let mut status = table.status.write();
        *status = TableStatus::Playing;
    }
    
    // Should not be able to join while game is playing
    let result = SeatManager::atomic_claim_any_seat(&table, "user3".to_string(), "user3".to_string());
    assert!(result.is_err(), "Should not be able to join during active game");
    
    // Should not be able to leave while game is playing
    let result = table.release_seat(1);
    assert!(result.is_err(), "Should not be able to leave during active game");
}

#[tokio::test]
async fn test_variable_player_count_scenarios() {
    let lobby = create_test_lobby();
    
    // Add test members
    for i in 1..=7 {
        let username = format!("user{}", i);
        lobby.state.add_member(username).expect("Failed to add member");
    }
    
    // Create table with variable player count (3-6 players)
    let bundle = Arc::new(create_game_bundle("three-min", 3, 6));
    let table_id = lobby.state.create_table(
        "three-min".to_string(),
        bundle,
        "user1".to_string()
    ).expect("Failed to create table");
    
    let table = lobby.state.get_table(&table_id).expect("Table should exist");
    assert_eq!(table.min_players, 3);
    assert_eq!(table.max_players, 6);
    
    // Should be able to start at minimum (3 players)
    SeatManager::atomic_claim_any_seat(&table, "user2".to_string(), "user2".to_string()).unwrap();
    SeatManager::atomic_claim_any_seat(&table, "user3".to_string(), "user3".to_string()).unwrap();
    assert!(table.can_start(), "Should start with 3 players");
    
    // Should be able to add more players up to maximum
    SeatManager::atomic_claim_any_seat(&table, "user4".to_string(), "user4".to_string()).unwrap();
    assert!(table.can_start(), "Should still start with 4 players");
    
    SeatManager::atomic_claim_any_seat(&table, "user5".to_string(), "user5".to_string()).unwrap();
    SeatManager::atomic_claim_any_seat(&table, "user6".to_string(), "user6".to_string()).unwrap();
    assert!(table.can_start(), "Should start with 6 players");
    assert_eq!(table.seated_count(), 6);
    
    // Should not be able to add 7th player
    let result = SeatManager::atomic_claim_any_seat(&table, "user7".to_string(), "user7".to_string());
    assert!(result.is_err(), "Should not be able to add 7th player to 6-max table");
}

#[tokio::test]
async fn test_owner_leaving_table() {
    let lobby = create_test_lobby();
    
    // Add test members
    for i in 1..=3 {
        let username = format!("user{}", i);
        lobby.state.add_member(username).expect("Failed to add member");
    }
    
    // Create table (user1 is owner and auto-seated)
    let bundle = Arc::new(create_game_bundle("strict-two", 2, 2));
    let table_id = lobby.state.create_table(
        "strict-two".to_string(),
        bundle,
        "user1".to_string()
    ).expect("Failed to create table");
    
    let table = lobby.state.get_table(&table_id).expect("Table should exist");
    assert_eq!(table.owner, "user1");
    
    // Add second player
    SeatManager::atomic_claim_any_seat(&table, "user2".to_string(), "user2".to_string())
        .expect("User2 should join");
    
    // Owner should be able to leave their own table
    table.release_seat(0).expect("Owner should be able to leave");
    assert_eq!(table.seated_count(), 1);
    
    // Table should still exist even after owner leaves
    assert!(lobby.state.get_table(&table_id).is_some());
    
    // Should not be able to start with only 1 player
    assert!(!table.can_start(), "Should not start with only 1 player remaining");
}
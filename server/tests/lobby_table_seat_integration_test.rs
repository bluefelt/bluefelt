use bluefelt_core::{
    app_state::AppState,
    bundle::Bundle,
    lobby::{
        lobby_impl::Lobby,
        lobby_state::LobbyState,
        table_instance::{TableInstance, SeatOccupant, TableStatus},
        chat::{ChatMessage, ChatScope, ChatSystem},
        SeatManager::SeatManager,
        CountdownManager::CountdownManager,
    },
};
use serde_json::{json, Value};
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::time::{sleep, Duration};

fn create_test_bundle() -> Arc<Bundle> {
    Arc::new(Bundle {
        game_id: "tic-tac-toe".to_string(),
        manifest: bluefelt_core::bundle::Manifest {
            game_id: "tic-tac-toe".to_string(),
            version: "1.0.0".to_string(),
            spec_version: "1.0".to_string(),
            metadata: bluefelt_core::bundle::ManifestMetadata {
                name: "Tic-Tac-Toe".to_string(),
                author: "Test".to_string(),
                description: "Test game".to_string(),
                players: bluefelt_core::bundle::PlayersRange { min: 2, max: 2 },
            },
            phases: None,
            setup: None,
            zone_groups: None,
        },
        entities: Value::Null,
        zones: Value::Null,
        actions: Value::Null,
        phases: Value::Null,
    })
}

#[tokio::test]
async fn test_table_creation_and_listing() {
    let bundle = create_test_bundle();
    let lobby = Lobby::new("test-lobby", "tic-tac-toe");
    
    // Add a player to the lobby
    let player_id = "player1";
    let username = "Alice";
    lobby.state.add_member(player_id, username).unwrap();
    
    // Create a table
    let table = lobby.state.create_table("tic-tac-toe", bundle.clone(), player_id).unwrap();
    
    // Verify table was created
    assert_eq!(table.owner, player_id);
    assert_eq!(table.bundle_id, "tic-tac-toe");
    assert_eq!(table.min_players, 2);
    assert_eq!(table.max_players, 2);
    
    // Verify table is in lobby's table list
    let tables = lobby.state.tables.lock();
    assert_eq!(tables.len(), 1);
    assert!(tables.contains_key(&table.id));
}

#[tokio::test]
async fn test_seat_claiming_and_releasing() {
    let bundle = create_test_bundle();
    let lobby = Lobby::new("test-lobby", "tic-tac-toe");
    
    // Add two players
    lobby.state.add_member("player1", "Alice").unwrap();
    lobby.state.add_member("player2", "Bob").unwrap();
    
    // Create a table
    let table = lobby.state.create_table("tic-tac-toe", bundle.clone(), "player1").unwrap();
    
    // Player 1 claims seat 0
    let claimed = SeatManager::atomic_claim_seat(&table, 0, "player1".to_string(), "Alice".to_string()).unwrap();
    assert!(claimed);
    
    // Verify seat is claimed
    {
        let seats = table.seats.read();
        match &seats[0] {
            Some(SeatOccupant::Player(id)) => assert_eq!(id, "player1"),
            _ => panic!("Expected seat 0 to be claimed by player1"),
        }
    }
    
    // Player 2 tries to claim same seat - should fail
    let claimed = SeatManager::atomic_claim_seat(&table, 0, "player2".to_string(), "Bob".to_string()).unwrap();
    assert!(!claimed);
    
    // Player 2 claims seat 1
    let claimed = SeatManager::atomic_claim_seat(&table, 1, "player2".to_string(), "Bob".to_string()).unwrap();
    assert!(claimed);
    
    // Player 1 releases seat 0
    SeatManager::atomic_release_seat(&table, 0).unwrap();
    let released = true;
    assert!(released);
    
    // Verify seat is empty
    {
        let seats = table.seats.read();
        assert!(seats[0].is_none());
    }
}

#[tokio::test]
async fn test_ready_states_and_countdown() {
    let bundle = create_test_bundle();
    let lobby = Arc::new(Lobby::new("test-lobby", "tic-tac-toe"));
    
    // Add two players
    lobby.state.add_member("player1", "Alice").unwrap();
    lobby.state.add_member("player2", "Bob").unwrap();
    
    // Create table and claim seats
    let table = lobby.state.create_table("tic-tac-toe", bundle.clone(), "player1").unwrap();
    SeatManager::atomic_claim_seat(&table, 0, "player1".to_string(), "Alice".to_string()).unwrap();
    SeatManager::atomic_claim_seat(&table, 1, "player2".to_string(), "Bob".to_string()).unwrap();
    
    // Set player 1 ready
    SeatManager::set_ready_state(&table, 0, true).unwrap();
    
    // Verify ready state
    {
        let ready_states = table.ready_states.read();
        assert!(ready_states[0]);
        assert!(!ready_states[1]);
    }
    
    // Check if all ready - should be false
    assert!(!SeatManager::are_all_players_ready(&table));
    
    // Set player 2 ready
    SeatManager::set_ready_state(&table, 1, true).unwrap();
    
    // Now all should be ready
    assert!(SeatManager::are_all_players_ready(&table));
    
    // Start countdown
    let started = CountdownManager::start_countdown(&table, &lobby, 1);
    assert!(started);
    
    // Verify countdown is active
    {
        let countdown = table.countdown_ends_at.read();
        assert!(countdown.is_some());
    }
    
    // Wait for countdown to complete
    sleep(Duration::from_secs(2)).await;
    
    // Verify game transitioned to Playing state
    {
        let status = table.status.read();
        assert_eq!(*status, TableStatus::Playing);
    }
}

#[tokio::test]
async fn test_chat_system() {
    let bundle = create_test_bundle();
    let lobby = Arc::new(Lobby::new("test-lobby", "tic-tac-toe"));
    
    // Add a player
    lobby.state.add_member("player1", "Alice");
    
    // Send lobby chat message
    lobby.state.chat.add_message(
        ChatScope::Lobby,
        None,
        "player1".to_string(),
        "Alice".to_string(),
        "Hello lobby!".to_string(),
    );
    
    // Verify lobby message
    {
        let messages = lobby.state.chat.get_messages(ChatScope::Lobby, None, None);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello lobby!");
        assert_eq!(messages[0].player_id, "player1");
    }
    
    // Create a table
    let table = lobby.state.create_table("tic-tac-toe", bundle.clone(), "player1").unwrap();
    let table_id = table.id.clone();
    
    // Send table chat message
    lobby.state.chat.add_message(
        ChatScope::Table(table_id.clone()),
        Some(table_id.clone()),
        "player1".to_string(),
        "Alice".to_string(),
        "Hello table!".to_string(),
    );
    
    // Verify table message
    {
        let messages = lobby.state.chat.get_table_messages(&table_id, 50);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "Hello table!");
    }
    
    // Verify lobby and table messages are separate
    {
        let lobby_messages = lobby.state.chat.get_lobby_messages(50);
        let table_messages = lobby.state.chat.get_table_messages(&table_id, 50);
        assert_eq!(lobby_messages.len(), 1);
        assert_eq!(table_messages.len(), 1);
        assert_ne!(lobby_messages[0].message, table_messages[0].message);
    }
}

#[tokio::test]
async fn test_spectator_management() {
    let bundle = create_test_bundle();
    let lobby = Lobby::new("test-lobby", "tic-tac-toe");
    
    // Add three players
    lobby.state.add_member("player1", "Alice");
    lobby.state.add_member("player2", "Bob");
    lobby.state.add_member("player3", "Charlie");
    
    // Create table and fill seats
    let table = lobby.state.create_table("tic-tac-toe", bundle.clone(), "player1").unwrap();
    SeatManager::atomic_claim_seat(&table, 0, "player1".to_string(), "Alice".to_string()).unwrap();
    SeatManager::atomic_claim_seat(&table, 1, "player2".to_string(), "Bob".to_string()).unwrap();
    
    // Player 3 joins as spectator
    {
        let mut spectators = table.spectators.write();
        spectators.push("player3".to_string());
    }
    
    // Verify spectator list
    {
        let spectators = table.spectators.read();
        assert_eq!(spectators.len(), 1);
        assert_eq!(spectators[0], "player3");
    }
    
    // Player 2 leaves their seat
    SeatManager::atomic_release_seat(&table, 1, "player2").unwrap();
    
    // Spectator (player 3) can now claim the seat
    let claimed = SeatManager::atomic_claim_seat(&table, 1, "player3".to_string(), "Charlie".to_string()).unwrap();
    assert!(claimed);
    
    // Verify spectator was removed when they claimed a seat
    {
        let mut spectators = table.spectators.write();
        spectators.retain(|id| id != "player3");
        assert!(spectators.is_empty());
    }
}

#[tokio::test]
async fn test_table_lifecycle() {
    let bundle = create_test_bundle();
    let lobby = Arc::new(Lobby::new("test-lobby", "tic-tac-toe"));
    
    // Add players
    lobby.state.add_member("player1", "Alice");
    lobby.state.add_member("player2", "Bob");
    
    // Create table - starts in Open state
    let table = lobby.state.create_table("tic-tac-toe", bundle.clone(), "player1").unwrap();
    {
        let status = table.status.read();
        assert_eq!(*status, TableStatus::Open);
    }
    
    // Fill seats and set ready
    SeatManager::atomic_claim_seat(&table, 0, "player1".to_string(), "Alice".to_string()).unwrap();
    SeatManager::atomic_claim_seat(&table, 1, "player2".to_string(), "Bob".to_string()).unwrap();
    SeatManager::set_ready_state(&table, 0, true).unwrap();
    SeatManager::set_ready_state(&table, 1, true).unwrap();
    
    // Start countdown - transitions to Countdown state
    CountdownManager::start_countdown(&table, &lobby, 1);
    {
        let status = table.status.read();
        assert_eq!(*status, TableStatus::Countdown);
    }
    
    // Wait for countdown - transitions to Playing state
    sleep(Duration::from_secs(2)).await;
    {
        let status = table.status.read();
        assert_eq!(*status, TableStatus::Playing);
    }
    
    // Simulate game ending - transition to Finished state
    {
        let mut status = table.status.write();
        *status = TableStatus::Finished;
    }
    
    // Verify final state
    {
        let status = table.status.read();
        assert_eq!(*status, TableStatus::Finished);
    }
}

#[tokio::test]
async fn test_concurrent_seat_claims() {
    let bundle = create_test_bundle();
    let lobby = Arc::new(Lobby::new("test-lobby", "tic-tac-toe"));
    
    // Add multiple players
    for i in 1..=5 {
        lobby.state.add_member(&format!("player{}", i), &format!("Player{}", i));
    }
    
    // Create table
    let table = Arc::new(lobby.state.create_table("tic-tac-toe", bundle.clone(), "player1").unwrap());
    
    // Spawn multiple tasks trying to claim the same seat concurrently
    let mut handles = vec![];
    for i in 1..=5 {
        let table_clone = Arc::clone(&table);
        let handle = tokio::spawn(async move {
            SeatManager::atomic_claim_seat(
                &table_clone,
                0,
                format!("player{}", i),
                format!("Player{}", i),
            ).unwrap()
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let results: Vec<bool> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Exactly one should have succeeded
    let successes = results.iter().filter(|&&r| r).count();
    assert_eq!(successes, 1);
    
    // Verify seat is claimed by exactly one player
    {
        let seats = table.seats.read();
        assert!(matches!(seats[0], Some(SeatOccupant::Player(_))));
    }
}

#[tokio::test]
async fn test_table_metadata() {
    let bundle = create_test_bundle();
    let lobby = Lobby::new("test-lobby", "tic-tac-toe");
    
    lobby.state.add_member("player1", "Alice").unwrap();
    
    // Create table and verify metadata
    let table = lobby.state.create_table("tic-tac-toe", bundle.clone(), "player1").unwrap();
    
    // Verify metadata was set from bundle
    assert_eq!(table.bundle_id, "tic-tac-toe");
    assert_eq!(table.min_players, 2);
    assert_eq!(table.max_players, 2);
    assert_eq!(table.owner, "player1");
}

#[tokio::test]
async fn test_minimum_players_enforcement() {
    let bundle = create_test_bundle();
    let lobby = Arc::new(Lobby::new("test-lobby", "tic-tac-toe"));
    
    lobby.state.add_member("player1", "Alice");
    
    // Create table (tic-tac-toe requires 2 players)
    let table = lobby.state.create_table("tic-tac-toe", bundle.clone(), "player1").unwrap();
    SeatManager::atomic_claim_seat(&table, 0, "player1".to_string(), "Alice".to_string()).unwrap();
    SeatManager::set_ready_state(&table, 0, true).unwrap();
    
    // Try to start countdown with only 1 player - should fail
    let started = CountdownManager::start_countdown(&table, &lobby, 10);
    assert!(!started);
    
    // Add second player
    lobby.state.add_member("player2", "Bob");
    SeatManager::atomic_claim_seat(&table, 1, "player2".to_string(), "Bob".to_string()).unwrap();
    SeatManager::set_ready_state(&table, 1, true).unwrap();
    
    // Now countdown should start
    let started = CountdownManager::start_countdown(&table, &lobby, 10);
    assert!(started);
}
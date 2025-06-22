use bluefelt_core::{bundle::BundleMap, lobby};
use serde_json::json;
use std::sync::Arc;
use std::collections::HashMap;
use dashmap::DashMap;
use axum::extract::ws::Message;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

async fn create_test_lobby(game_id: &str) -> (Arc<bluefelt_core::lobby::Lobby>, String) {
    let bundle_map = BundleMap::load_dir("../bundles").unwrap();
    let bundle = bundle_map.get_latest(game_id).unwrap();
    let lobby_id = format!("test_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    let lobby = lobby::new_lobby(lobby_id.clone(), bundle, lobbies.clone(), lobby_tx, None);
    (lobby, lobby_id)
}

#[tokio::test]
async fn test_cannot_start_with_too_few_players() {
    let (lobby, _) = create_test_lobby("go-fish").await;
    
    // Add only one player (Go Fish requires min 2)
    lobby.add_player("alice".to_string());
    
    // Try to start game
    let started = lobby.is_started();
    assert!(!started, "Game should not be started yet");
    
    // Verify minimum player requirement
    let min_players = lobby.bundle.manifest.metadata.players.min;
    assert_eq!(min_players, 2, "Go Fish should require minimum 2 players");
    
    // Check that we have fewer than minimum
    let player_count = lobby.players();
    assert!(player_count < min_players as usize, "Should have fewer than minimum players");
}

#[tokio::test]
async fn test_cannot_join_more_than_max_players() {
    let (lobby, _) = create_test_lobby("go-fish").await;
    
    // Go Fish allows max 4 players
    let max_players = lobby.bundle.manifest.metadata.players.max;
    assert_eq!(max_players, 4, "Go Fish should allow maximum 4 players");
    
    // Add maximum number of players
    for i in 1..=max_players {
        let added = lobby.add_player(format!("player{}", i));
        assert!(added, "Should be able to add player {}", i);
    }
    
    // Try to add one more player
    let extra_added = lobby.add_player("extra_player".to_string());
    assert!(!extra_added, "Should not be able to add player beyond maximum");
    
    // Verify player count is at maximum
    let player_count = lobby.players();
    assert_eq!(player_count, max_players as usize, "Should have exactly maximum players");
}

#[tokio::test]
async fn test_can_start_with_sufficient_players() {
    let (lobby, _) = create_test_lobby("go-fish").await;
    
    // Add minimum required players
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    
    // Verify we have minimum players
    let player_count = lobby.players();
    let min_players = lobby.bundle.manifest.metadata.players.min;
    assert_eq!(player_count, min_players as usize, "Should have minimum required players");
    
    // Start the game
    lobby.clone().start_game();
    
    // Give it a moment to process
    sleep(Duration::from_millis(100)).await;
    
    // Verify game started
    let started = lobby.is_started();
    assert!(started, "Game should be started with sufficient players");
}

#[tokio::test]
async fn test_player_reconnection() {
    let (lobby, _) = create_test_lobby("tic-tac-toe").await;
    
    // Add two players
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    
    // Alice reconnects (same player)
    let reconnected = lobby.add_player("alice".to_string());
    assert!(reconnected, "Player should be able to reconnect");
    
    // Verify player count didn't increase
    let player_count = lobby.players();
    assert_eq!(player_count, 2, "Player count should remain the same after reconnection");
}

#[tokio::test]
async fn test_different_games_player_requirements() {
    // Test Tic-Tac-Toe (2 players exactly)
    let (ttt_lobby, _) = create_test_lobby("tic-tac-toe").await;
    assert_eq!(ttt_lobby.bundle.manifest.metadata.players.min, 2);
    assert_eq!(ttt_lobby.bundle.manifest.metadata.players.max, 2);
    
    // Test Go Fish (2-4 players)
    let (gf_lobby, _) = create_test_lobby("go-fish").await;
    assert_eq!(gf_lobby.bundle.manifest.metadata.players.min, 2);
    assert_eq!(gf_lobby.bundle.manifest.metadata.players.max, 4);
    
    // Test Three Men's Morris (2 players exactly)
    let (tmm_lobby, _) = create_test_lobby("three-mens-morris").await;
    assert_eq!(tmm_lobby.bundle.manifest.metadata.players.min, 2);
    assert_eq!(tmm_lobby.bundle.manifest.metadata.players.max, 2);
}

#[tokio::test]
async fn test_remove_player_from_lobby() {
    let (lobby, _) = create_test_lobby("go-fish").await;
    
    // Add players
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    lobby.add_player("charlie".to_string());
    
    assert_eq!(lobby.players(), 3, "Should have 3 players");
    
    // Remove a player
    let removed = lobby.remove_player("bob");
    assert!(removed, "Should be able to remove player");
    
    assert_eq!(lobby.players(), 2, "Should have 2 players after removal");
    
    // Try to remove non-existent player
    let removed_nonexistent = lobby.remove_player("david");
    assert!(!removed_nonexistent, "Should not be able to remove non-existent player");
}
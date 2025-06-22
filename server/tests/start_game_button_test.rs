use bluefelt_core::{bundle::{Bundle, BundleMap}, lobby, message_format::MessageFormat};
use serde_json::json;
use std::sync::Arc;
use dashmap::DashMap;
use axum::extract::ws::Message;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

/// Test that verifies the Start Game button visibility fix
/// This addresses the issue where the button wouldn't appear in Go Fish lobby
/// even with sufficient players due to missing manifest data and "you" field
#[tokio::test]
async fn test_start_game_button_visibility() {
    // Create a Go Fish lobby
    let bundle_map = BundleMap::load_dir("./bundles").unwrap();
    let bundle = bundle_map.get_latest("go-fish").unwrap();
    let lobby_id = format!("test_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    let lobby = lobby::new_lobby(lobby_id.clone(), bundle, lobbies.clone(), lobby_tx, None);
    
    // Add two players (minimum required for Go Fish)
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    
    // Test 1: Welcome message before game starts should include manifest
    let welcome_json = lobby.build_welcome_message("alice", false, &MessageFormat::Standard);
    
    // Verify manifest is present
    assert!(welcome_json["ui"]["manifest"].is_object(), "Manifest should be present in welcome message");
    assert_eq!(
        welcome_json["ui"]["manifest"]["metadata"]["players"]["min"], 
        2, 
        "Minimum players should be 2"
    );
    assert_eq!(
        welcome_json["ui"]["manifest"]["metadata"]["players"]["max"], 
        4, 
        "Maximum players should be 4"
    );
    
    // Verify started field is present and false
    assert_eq!(welcome_json["started"], false, "Started field should be false before game starts");
    
    // Test 2: "you" field should be set for existing players
    assert_eq!(welcome_json["you"], "alice", "You field should be set to player name for existing players");
    
    // Test 3: Player list should be correct
    let players = lobby.player_list();
    assert_eq!(players.len(), 2, "Should have 2 players");
    assert!(players.contains(&"alice".to_string()), "Alice should be in player list");
    assert!(players.contains(&"bob".to_string()), "Bob should be in player list");
    
    // Test 4: New player connecting should get null "you" field
    let welcome_json_new = lobby.build_welcome_message("charlie", false, &MessageFormat::Standard);
    assert_eq!(welcome_json_new["you"], json!(null), "You field should be null for non-joined players");
    
    // Test 5: After game starts, "you" should be player ID (p1, p2, etc)
    lobby.clone().start_game();
    sleep(Duration::from_millis(100)).await;
    
    let welcome_json_started = lobby.build_welcome_message("alice", true, &MessageFormat::Standard);
    assert_eq!(welcome_json_started["started"], true, "Started field should be true after game starts");
    assert_eq!(welcome_json_started["you"], "p1", "You field should be player ID after game starts");
}

/// Test that reconnecting players are properly recognized
#[tokio::test]
async fn test_player_reconnection_recognition() {
    let bundle_map = BundleMap::load_dir("./bundles").unwrap();
    let bundle = bundle_map.get_latest("tic-tac-toe").unwrap();
    let lobby_id = format!("test_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    let lobby = lobby::new_lobby(lobby_id.clone(), bundle, lobbies.clone(), lobby_tx, None);
    
    // Add a player
    lobby.add_player("alice".to_string());
    
    // Simulate reconnection - get welcome message for existing player
    let welcome_json = lobby.build_welcome_message("alice", false, &MessageFormat::Standard);
    
    // Player should be recognized as already joined
    assert_eq!(welcome_json["you"], "alice", "Reconnecting player should be recognized");
    assert_eq!(welcome_json["started"], false, "Game should not be started");
    
    // Verify manifest is still included
    assert!(welcome_json["ui"]["manifest"].is_object(), "Manifest should be present for reconnecting players");
}

/// Test the full flow that enables Start Game button
#[tokio::test]
async fn test_start_game_button_requirements() {
    let bundle_map = BundleMap::load_dir("./bundles").unwrap();
    let bundle = bundle_map.get_latest("go-fish").unwrap();
    let lobby_id = format!("test_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    let lobby = lobby::new_lobby(lobby_id.clone(), bundle, lobbies.clone(), lobby_tx, None);
    
    // Step 1: First player joins
    lobby.add_player("player1".to_string());
    let json1 = lobby.build_welcome_message("player1", false, &MessageFormat::Standard);
    
    // Should have all requirements except sufficient players
    assert_eq!(json1["started"], false, "Game not started");
    assert_eq!(json1["you"], "player1", "Player recognized");
    assert_eq!(json1["ui"]["manifest"]["metadata"]["players"]["min"], 2, "Min players is 2");
    assert_eq!(json1["ui"]["players"].as_array().unwrap().len(), 1, "Only 1 player");
    
    // Step 2: Second player joins
    lobby.add_player("player2".to_string());
    let json2 = lobby.build_welcome_message("player2", false, &MessageFormat::Standard);
    
    // Now all requirements are met for Start Game button
    assert_eq!(json2["started"], false, "Game not started");
    assert_eq!(json2["you"], "player2", "Player recognized");
    assert_eq!(json2["ui"]["players"].as_array().unwrap().len(), 2, "Now have 2 players");
    
    // The client should now show Start Game button because:
    // - started = false (game not started)
    // - you = "player2" (joined = true)
    // - players.length >= min (2 >= 2, canStartGame = true)
}
use bluefelt_core::{
    lobby::{new_lobby, Lobby, LobbyMap},
    bundle::BundleMap,
    engine::verbs::multi_step::MultiStepState,
    message_format::MessageFormat,
};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use serde_json::json;
use tokio::sync::broadcast;
use axum::extract::ws::Message;
use dashmap::DashMap;

#[tokio::test]
async fn test_multi_step_state_reconnection() {
    // Load bundles
    let bundles_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bundles");
    let bundles = BundleMap::load_dir(bundles_dir.to_str().unwrap()).unwrap();
    
    // Create necessary dependencies for new_lobby
    let lobby_map: Arc<LobbyMap> = Arc::new(DashMap::new());
    let (lobby_updates, _) = broadcast::channel::<Message>(100);
    
    // Get the Three Men's Morris bundle
    let bundle = bundles.get_latest("three-mens-morris").expect("Failed to get Three Men's Morris bundle").clone();
    
    // Create a test lobby with Three Men's Morris (which has multi-step actions)
    let lobby = new_lobby("test".to_string(), bundle, lobby_map, lobby_updates, None);
    
    // Add players
    assert!(lobby.add_player("Alice".to_string()));
    assert!(lobby.add_player("Bob".to_string()));
    
    // Start the game
    lobby.clone().start_game();
    
    // Test 1: Create a multi-step action for player 1
    let result = lobby.start_multi_step_action("p1", "movePiece", true);
    assert!(result.is_ok(), "Should be able to start multi-step action");
    
    // Store some data in the multi-step action
    assert!(lobby.store_multi_step_value("p1", "selectedPiece", json!("/zones/board/cells/0/0")).is_ok());
    assert!(lobby.advance_multi_step("p1").is_ok());
    assert!(lobby.store_multi_step_value("p1", "destination", json!("/zones/board/cells/1/1")).is_ok());
    
    // Verify the multi-step state exists
    let state = lobby.get_multi_step_state("p1");
    assert!(state.is_some(), "Multi-step state should exist for p1");
    let state = state.unwrap();
    assert_eq!(state.action_id, "movePiece");
    assert_eq!(state.current_step, 1);
    assert_eq!(state.stored_values.len(), 2);
    
    // Test 2: Simulate player disconnection by removing from client_formats
    // (This simulates what happens when a WebSocket closes)
    {
        let mut clients = lobby.client_formats.lock();
        clients.remove("Alice");
        // Note: Players remain in the lobby for reconnection support
    }
    
    // Verify multi-step state is preserved after disconnection
    let preserved_state = lobby.get_multi_step_state("p1");
    assert!(preserved_state.is_some(), "Multi-step state should be preserved after disconnection");
    
    // Test 3: Simulate reconnection by building welcome message
    // (This simulates what happens when a player reconnects)
    let message_format = MessageFormat::Standard;
    let welcome_message = lobby.build_welcome_message("Alice", lobby.is_started(), &message_format);
    
    // Verify the welcome message includes the restored multi-step state
    assert_eq!(welcome_message["type"], "welcome");
    assert_eq!(welcome_message["started"], true);
    assert_eq!(welcome_message["you"], "p1");
    
    let ui = &welcome_message["ui"];
    assert!(ui.is_object(), "UI should be an object");
    assert!(ui["multiStepState"].is_object(), "Multi-step state should be included in welcome message");
    
    let restored_state = &ui["multiStepState"];
    assert_eq!(restored_state["actionId"], "movePiece");
    assert_eq!(restored_state["currentStepIndex"], 1);
    assert_eq!(restored_state["canCancel"], true);
    assert!(restored_state["storedData"].is_object(), "Stored data should be preserved");
    
    let stored_data = &restored_state["storedData"];
    assert_eq!(stored_data["selectedPiece"], "/zones/board/cells/0/0");
    assert_eq!(stored_data["destination"], "/zones/board/cells/1/1");
    
    println!("✅ Multi-step state reconnection test passed!");
}

#[tokio::test]
async fn test_disconnected_player_cleanup() {
    // Load bundles
    let bundles_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bundles");
    let bundles = BundleMap::load_dir(bundles_dir.to_str().unwrap()).unwrap();
    
    // Create necessary dependencies for new_lobby
    let lobby_map: Arc<LobbyMap> = Arc::new(DashMap::new());
    let (lobby_updates, _) = broadcast::channel::<Message>(100);
    
    // Get the Three Men's Morris bundle
    let bundle = bundles.get_latest("three-mens-morris").expect("Failed to get Three Men's Morris bundle").clone();
    
    // Create a test lobby
    let lobby = new_lobby("test".to_string(), bundle, lobby_map, lobby_updates, None);
    
    // Add players and start game
    assert!(lobby.add_player("Alice".to_string()));
    assert!(lobby.add_player("Bob".to_string()));
    lobby.clone().start_game();
    
    // Start multi-step actions for both players
    assert!(lobby.start_multi_step_action("p1", "movePiece", true).is_ok());
    assert!(lobby.start_multi_step_action("p2", "movePiece", true).is_ok());
    
    // Simulate both players disconnecting
    {
        let mut clients = lobby.client_formats.lock();
        clients.remove("Alice");
        clients.remove("Bob");
    }
    
    // Verify both states exist before cleanup
    assert!(lobby.get_multi_step_state("p1").is_some());
    assert!(lobby.get_multi_step_state("p2").is_some());
    
    // Test cleanup with very short timeout (should clean up both)
    let cleaned_up = lobby.cleanup_disconnected_multi_step_states(Duration::from_millis(0));
    assert_eq!(cleaned_up.len(), 2, "Should clean up both disconnected players' states");
    assert!(cleaned_up.contains(&"p1".to_string()));
    assert!(cleaned_up.contains(&"p2".to_string()));
    
    // Verify states are removed
    assert!(lobby.get_multi_step_state("p1").is_none());
    assert!(lobby.get_multi_step_state("p2").is_none());
    
    // Test that cleanup doesn't affect anything when no states exist
    let cleaned_up_again = lobby.cleanup_disconnected_multi_step_states(Duration::from_millis(0));
    assert_eq!(cleaned_up_again.len(), 0, "Should not clean up anything when no states exist");
    
    println!("✅ Disconnected player cleanup test passed!");
}

#[tokio::test]
async fn test_connected_vs_disconnected_cleanup() {
    // Load bundles
    let bundles_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bundles");
    let bundles = BundleMap::load_dir(bundles_dir.to_str().unwrap()).unwrap();
    
    // Create necessary dependencies for new_lobby
    let lobby_map: Arc<LobbyMap> = Arc::new(DashMap::new());
    let (lobby_updates, _) = broadcast::channel::<Message>(100);
    
    // Get the Three Men's Morris bundle
    let bundle = bundles.get_latest("three-mens-morris").expect("Failed to get Three Men's Morris bundle").clone();
    
    // Create a test lobby
    let lobby = new_lobby("test".to_string(), bundle, lobby_map, lobby_updates, None);
    
    // Add players and start game
    assert!(lobby.add_player("Alice".to_string()));
    assert!(lobby.add_player("Bob".to_string()));
    lobby.clone().start_game();
    
    // Start multi-step actions for both players
    assert!(lobby.start_multi_step_action("p1", "movePiece", true).is_ok());
    assert!(lobby.start_multi_step_action("p2", "movePiece", true).is_ok());
    
    // Add Alice back to client_formats to simulate she's still connected
    {
        let mut clients = lobby.client_formats.lock();
        clients.insert("Alice".to_string(), bluefelt_core::lobby::ClientInfo {
            message_format: MessageFormat::Standard,
            update_format: bluefelt_core::message_format::UpdateFormat::Patch,
        });
        // Bob remains disconnected (not in client_formats)
    }
    
    // Test cleanup - should only clean up Bob's state, not Alice's
    let cleaned_up = lobby.cleanup_disconnected_multi_step_states(Duration::from_millis(0));
    assert_eq!(cleaned_up.len(), 1, "Should only clean up disconnected player's state");
    assert_eq!(cleaned_up[0], "p2", "Should clean up p2 (Bob's) state");
    
    // Verify Alice's state is preserved but Bob's is removed
    assert!(lobby.get_multi_step_state("p1").is_some(), "Connected player's state should be preserved");
    assert!(lobby.get_multi_step_state("p2").is_none(), "Disconnected player's state should be removed");
    
    println!("✅ Connected vs disconnected cleanup test passed!");
}

#[tokio::test]
async fn test_multi_step_state_format_in_welcome_message() {
    // Load bundles
    let bundles_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bundles");
    let bundles = BundleMap::load_dir(bundles_dir.to_str().unwrap()).unwrap();
    
    // Create necessary dependencies for new_lobby
    let lobby_map: Arc<LobbyMap> = Arc::new(DashMap::new());
    let (lobby_updates, _) = broadcast::channel::<Message>(100);
    
    // Get the Three Men's Morris bundle
    let bundle = bundles.get_latest("three-mens-morris").expect("Failed to get Three Men's Morris bundle").clone();
    
    // Create a test lobby
    let lobby = new_lobby("test".to_string(), bundle, lobby_map, lobby_updates, None);
    
    // Add players and start game
    assert!(lobby.add_player("Alice".to_string()));
    assert!(lobby.add_player("Bob".to_string()));
    lobby.clone().start_game();
    
    // Test welcome message without multi-step state
    let message_format = MessageFormat::Standard;
    let welcome_without_multi_step = lobby.build_welcome_message("Alice", lobby.is_started(), &message_format);
    assert!(welcome_without_multi_step["ui"]["multiStepState"].is_null() || 
            !welcome_without_multi_step["ui"].as_object().unwrap().contains_key("multiStepState"),
            "Should not include multi-step state when none exists");
    
    // Start a multi-step action
    assert!(lobby.start_multi_step_action("p1", "movePiece", true).is_ok());
    assert!(lobby.store_multi_step_value("p1", "testKey", json!("testValue")).is_ok());
    
    // Test welcome message with multi-step state
    let welcome_with_multi_step = lobby.build_welcome_message("Alice", lobby.is_started(), &message_format);
    
    let multi_step_state = &welcome_with_multi_step["ui"]["multiStepState"];
    assert!(multi_step_state.is_object(), "Multi-step state should be an object");
    
    // Verify all required fields are present
    assert!(multi_step_state["actionId"].is_string(), "actionId should be present and string");
    assert!(multi_step_state["currentStepIndex"].is_number(), "currentStepIndex should be present and number");
    assert!(multi_step_state["storedData"].is_object(), "storedData should be present and object");
    assert!(multi_step_state["canCancel"].is_boolean(), "canCancel should be present and boolean");
    assert!(multi_step_state["requiresConfirmation"].is_boolean(), "requiresConfirmation should be present and boolean");
    assert!(multi_step_state["stepActionMap"].is_object(), "stepActionMap should be present and object");
    
    // Verify values
    assert_eq!(multi_step_state["actionId"], "movePiece");
    assert_eq!(multi_step_state["currentStepIndex"], 0);
    assert_eq!(multi_step_state["canCancel"], true);
    assert_eq!(multi_step_state["storedData"]["testKey"], "testValue");
    
    println!("✅ Multi-step state format in welcome message test passed!");
}
use bluefelt_core::{
    lobby::{new_lobby, Lobby, LobbyMap},
    bundle::BundleMap,
    engine::verbs::multi_step::MultiStepState,
};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use serde_json::json;
use tokio::sync::broadcast;
use axum::extract::ws::Message;
use dashmap::DashMap;

#[tokio::test]
async fn test_multi_step_timeout_functionality() {
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
    
    // Test 1: Create a multi-step action and verify it's stored with timestamps
    let result = lobby.start_multi_step_action("p1", "movePiece", true);
    assert!(result.is_ok(), "Should be able to start multi-step action");
    
    // Verify the multi-step state was created with timestamps
    let state = lobby.get_multi_step_state("p1");
    assert!(state.is_some(), "Multi-step state should exist for p1");
    
    let state = state.unwrap();
    assert_eq!(state.action_id, "movePiece");
    assert_eq!(state.current_step, 0);
    assert!(state.can_cancel);
    
    // Verify timestamps are set and reasonable
    let now = SystemTime::now();
    assert!(state.created_at <= now, "Created timestamp should be in the past");
    assert!(state.last_activity <= now, "Last activity should be in the past");
    assert!(now.duration_since(state.created_at).unwrap() < Duration::from_secs(1), 
        "Created timestamp should be very recent");
    
    // Test 2: Update the multi-step state and verify last_activity is updated
    let initial_last_activity = state.last_activity;
    
    // Wait a small amount to ensure timestamp difference
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Store a value (which should update last_activity)
    let result = lobby.store_multi_step_value("p1", "testKey", json!("testValue"));
    assert!(result.is_ok(), "Should be able to store multi-step value");
    
    // Verify last_activity was updated
    let updated_state = lobby.get_multi_step_state("p1").unwrap();
    assert!(updated_state.last_activity > initial_last_activity, 
        "Last activity should be updated after storing value");
    
    // Test 3: Timeout functionality with immediate timeout (0 duration)
    let timed_out = lobby.timeout_expired_multi_step_actions(Duration::from_secs(0));
    assert_eq!(timed_out.len(), 1, "Should timeout 1 multi-step action");
    assert_eq!(timed_out[0], "p1", "Should timeout p1's action");
    
    // Verify the state was removed
    let state = lobby.get_multi_step_state("p1");
    assert!(state.is_none(), "Multi-step state should be removed after timeout");
    
    // Test 4: Multiple players with different timeout scenarios
    // Create two multi-step actions
    assert!(lobby.start_multi_step_action("p1", "movePiece", true).is_ok());
    assert!(lobby.start_multi_step_action("p2", "movePiece", true).is_ok());
    
    // Update one of them to have recent activity
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(lobby.store_multi_step_value("p2", "recent", json!("activity")).is_ok());
    
    // Timeout with a very short duration - only p1 should timeout
    let timed_out = lobby.timeout_expired_multi_step_actions(Duration::from_millis(5));
    assert_eq!(timed_out.len(), 1, "Should timeout 1 action");
    assert_eq!(timed_out[0], "p1", "Should timeout p1's older action");
    
    // p2 should still have their action
    assert!(lobby.get_multi_step_state("p2").is_some(), "p2 should still have active action");
    
    // Test 5: No timeouts when actions are within timeout window
    let timed_out = lobby.timeout_expired_multi_step_actions(Duration::from_secs(60));
    assert_eq!(timed_out.len(), 0, "Should not timeout any actions within timeout window");
    
    // p2 should still have their action
    assert!(lobby.get_multi_step_state("p2").is_some(), "p2 should still have active action");
    
    println!("✅ All multi-step timeout tests passed!");
}

#[tokio::test]
async fn test_multi_step_state_timestamp_updates() {
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
    
    // Start multi-step action
    assert!(lobby.start_multi_step_action("p1", "movePiece", true).is_ok());
    
    let initial_state = lobby.get_multi_step_state("p1").unwrap();
    let initial_created = initial_state.created_at;
    let initial_activity = initial_state.last_activity;
    
    // Test that update_multi_step_state updates last_activity
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    let mut updated_state = initial_state.clone();
    updated_state.current_step = 1;
    assert!(lobby.update_multi_step_state("p1", updated_state).is_ok());
    
    let state_after_update = lobby.get_multi_step_state("p1").unwrap();
    assert_eq!(state_after_update.created_at, initial_created, "Created timestamp should not change");
    assert!(state_after_update.last_activity > initial_activity, "Last activity should be updated");
    assert_eq!(state_after_update.current_step, 1, "State changes should be preserved");
    
    // Test that advance_multi_step updates last_activity
    tokio::time::sleep(Duration::from_millis(10)).await;
    let activity_before_advance = state_after_update.last_activity;
    
    assert!(lobby.advance_multi_step("p1").is_ok());
    
    let state_after_advance = lobby.get_multi_step_state("p1").unwrap();
    assert!(state_after_advance.last_activity > activity_before_advance, 
        "Last activity should be updated after advance");
    assert_eq!(state_after_advance.current_step, 2, "Step should be advanced");
    
    println!("✅ All timestamp update tests passed!");
}

#[test]
fn test_multi_step_state_creation_with_timestamps() {
    // Test creating MultiStepState directly
    let now = SystemTime::now();
    let state = MultiStepState {
        action_id: "test".to_string(),
        current_step: 0,
        stored_values: HashMap::new(),
        can_cancel: true,
        deferred_logs: Vec::new(),
        created_at: now,
        last_activity: now,
    };
    
    assert_eq!(state.created_at, now);
    assert_eq!(state.last_activity, now);
    assert_eq!(state.action_id, "test");
    
    // Test cloning preserves timestamps
    let cloned = state.clone();
    assert_eq!(cloned.created_at, state.created_at);
    assert_eq!(cloned.last_activity, state.last_activity);
    
    println!("✅ MultiStepState creation test passed!");
}
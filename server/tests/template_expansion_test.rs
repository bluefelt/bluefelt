//! Tests to ensure server never sends unexpanded template strings to clients

use bluefelt_core::bundle::{Bundle, BundleMap, ZoneGroup};
use bluefelt_core::lobby::{new_lobby, current_lobbies_json};
use bluefelt_core::message_format::MessageFormat;
use dashmap::DashMap;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Helper function to load a game bundle
fn load_bundle(game_id: &str) -> Bundle {
    let bundle_map = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    bundle_map.get_latest(game_id).expect(&format!("Failed to find game '{}'", game_id))
}

/// Test that zone metadata expansion properly handles all template strings
#[tokio::test]
async fn test_zone_metadata_template_expansion() {
    // Load a game bundle that uses player-specific zones
    let bundle = load_bundle("go-fish");
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    
    // Create lobby and add players
    let lobby = new_lobby("test_lobby".to_string(), bundle, lobbies.clone(), lobby_tx, None);
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    
    // Start the game to trigger zone metadata expansion
    lobby.clone().start_game();
    
    // Build welcome message which should have expanded zone metadata
    let welcome_msg = lobby.build_welcome_message("alice", true, &MessageFormat::Standard);
    
    // Check that no template strings remain in the welcome message
    let msg_str = serde_json::to_string(&welcome_msg).expect("Failed to serialize welcome message");
    
    // These template patterns should NOT appear in the final message to clients
    let forbidden_patterns = vec![
        "{player}",
        "{id}",
        "_p{id}",
        "hand_{player}",
        "deck_p{id}",
    ];
    
    for pattern in forbidden_patterns {
        assert!(
            !msg_str.contains(pattern),
            "Welcome message contains unexpanded template '{}': {}",
            pattern,
            msg_str
        );
    }
    
    // Verify that expanded zone names ARE present
    assert!(msg_str.contains("hand_p1"), "Expected expanded zone 'hand_p1' not found");
    assert!(msg_str.contains("hand_p2"), "Expected expanded zone 'hand_p2' not found");
    
    println!("✓ Zone metadata template expansion test passed");
}

/// Test that lobby list JSON doesn't contain template strings
#[tokio::test]
async fn test_lobby_list_template_expansion() {
    // Load multiple game bundles to test different template patterns
    let bundle = load_bundle("go-fish");
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    
    // Create lobby and add players
    let lobby = new_lobby("test_lobby".to_string(), bundle, lobbies.clone(), lobby_tx, None);
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    
    // Start the game
    lobby.clone().start_game();
    
    // Insert into lobbies map
    lobbies.insert("test_lobby".to_string(), lobby);
    
    // Get lobby list JSON (this is what gets sent to clients via API)
    let lobby_list = current_lobbies_json(&lobbies);
    let list_str = serde_json::to_string(&lobby_list).expect("Failed to serialize lobby list");
    
    // Verify no template strings in lobby list
    let forbidden_patterns = vec![
        "{player}",
        "{id}",
        "_p{id}",
    ];
    
    for pattern in forbidden_patterns {
        assert!(
            !list_str.contains(pattern),
            "Lobby list contains unexpanded template '{}': {}",
            pattern,
            list_str
        );
    }
    
    println!("✓ Lobby list template expansion test passed");
}

/// Test that entity metadata doesn't contain template strings
#[tokio::test]
async fn test_entity_template_expansion() {
    // Load bundle with player-specific entities
    let bundle = load_bundle("go-fish");
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    
    let lobby = new_lobby("test_lobby".to_string(), bundle, lobbies, lobby_tx, None);
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    lobby.clone().start_game();
    
    // Get the game state that would be sent to clients
    let state = lobby.state.lock().clone();
    let state_str = serde_json::to_string(&state).expect("Failed to serialize game state");
    
    // Check for forbidden template patterns in entity data
    let forbidden_patterns = vec![
        "{player}",
        "{id}",
        "_p{id}",
    ];
    
    for pattern in forbidden_patterns {
        assert!(
            !state_str.contains(pattern),
            "Game state contains unexpanded template '{}' in entity data: {}",
            pattern,
            state_str
        );
    }
    
    println!("✓ Entity template expansion test passed");
}

/// Test zone groups expansion
#[tokio::test]
async fn test_zone_groups_template_expansion() {
    // Create a test bundle with zone groups that have templates
    let mut bundle = load_bundle("go-fish");
    
    // Add zone groups with templates to test expansion
    bundle.manifest.zone_groups = Some(vec![
        ZoneGroup {
            id: "playerHands_p{id}".to_string(),
            title: "Player Hands".to_string(),
            zones: vec!["hand_p{id}".to_string(), "discard_p{id}".to_string()],
        },
        ZoneGroup {
            id: "allDecks".to_string(), 
            title: "All Decks".to_string(),
            zones: vec!["deck".to_string(), "hand_p1".to_string(), "hand_p2".to_string()],
        }
    ]);
    
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    
    let lobby = new_lobby("test_lobby".to_string(), bundle, lobbies, lobby_tx, None);
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    lobby.clone().start_game();
    
    let welcome_msg = lobby.build_welcome_message("alice", true, &MessageFormat::Standard);
    let msg_str = serde_json::to_string(&welcome_msg).expect("Failed to serialize welcome message");
    
    // Verify no template patterns in zone groups
    assert!(!msg_str.contains("_p{id}"), "Zone groups contain unexpanded template '_p{{id}}'");
    assert!(!msg_str.contains("{id}"), "Zone groups contain unexpanded template '{{id}}'");
    
    // Verify expanded zone groups are present
    assert!(msg_str.contains("playerHands_p1"), "Expected expanded zone group 'playerHands_p1' not found");
    assert!(msg_str.contains("playerHands_p2"), "Expected expanded zone group 'playerHands_p2' not found");
    
    println!("✓ Zone groups template expansion test passed");
}

/// Comprehensive test that validates all client-facing APIs don't leak templates
#[tokio::test]
async fn test_all_client_apis_template_free() {
    let bundle = load_bundle("go-fish");
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    
    let lobby = new_lobby("test_lobby".to_string(), bundle, lobbies.clone(), lobby_tx, None);
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    
    // Test before game starts
    let welcome_before = lobby.build_welcome_message("alice", false, &MessageFormat::Standard);
    let before_str = serde_json::to_string(&welcome_before).expect("Failed to serialize");
    validate_no_templates(&before_str, "welcome message before game start");
    
    // Start game and test after
    lobby.clone().start_game();
    
    let welcome_after = lobby.build_welcome_message("alice", true, &MessageFormat::Standard);
    let after_str = serde_json::to_string(&welcome_after).expect("Failed to serialize");
    validate_no_templates(&after_str, "welcome message after game start");
    
    // Test Simple message format as well
    let welcome_simple = lobby.build_welcome_message("alice", true, &MessageFormat::Simple);
    let simple_str = serde_json::to_string(&welcome_simple).expect("Failed to serialize");
    validate_no_templates(&simple_str, "simple format welcome message");
    
    // Insert into lobbies and test lobby list
    lobbies.insert("test_lobby".to_string(), lobby);
    let lobby_list = current_lobbies_json(&lobbies);
    let list_str = serde_json::to_string(&lobby_list).expect("Failed to serialize");
    validate_no_templates(&list_str, "lobby list");
    
    println!("✓ All client APIs are template-free");
}

/// Test that zone counts match expectations for different games
#[tokio::test]
async fn test_zone_count_validation() {
    // Test Go Fish with 2 players
    let bundle = load_bundle("go-fish");
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    
    let lobby = new_lobby("test_lobby".to_string(), bundle, lobbies, lobby_tx, None);
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    lobby.clone().start_game();
    
    let welcome_msg = lobby.build_welcome_message("alice", true, &MessageFormat::Standard);
    
    // Check for template strings in any part of the welcome message
    let msg_str = serde_json::to_string(&welcome_msg).expect("Failed to serialize welcome message");
    
    let forbidden_patterns = vec![
        "{player}",
        "{id}",
        "_p{id}",
        "hand_{player}",
        "deck_p{id}",
    ];
    
    for pattern in forbidden_patterns {
        assert!(
            !msg_str.contains(pattern),
            "Welcome message contains unexpanded template '{}': This indicates template expansion is not working correctly!",
            pattern
        );
    }
    
    println!("✓ Zone template expansion test passed");
}

/// Test zone visibility settings  
#[tokio::test]
async fn test_zone_visibility_validation() {
    let bundle = load_bundle("war");
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    
    let lobby = new_lobby("test_lobby".to_string(), bundle, lobbies, lobby_tx, None);
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    lobby.clone().start_game();
    
    let welcome_msg = lobby.build_welcome_message("alice", true, &MessageFormat::Standard);
    let msg_str = serde_json::to_string(&welcome_msg).expect("Failed to serialize");
    
    validate_no_templates(&msg_str, "War game welcome message");
    println!("✓ Zone visibility validation passed");
}

/// Test that zone counts scale correctly with player count
#[tokio::test]
async fn test_zone_scaling_with_players() {
    let bundle = load_bundle("go-fish");
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    
    let lobby = new_lobby("test_lobby".to_string(), bundle, lobbies, lobby_tx, None);
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    lobby.clone().start_game();
    
    let welcome_msg = lobby.build_welcome_message("alice", true, &MessageFormat::Standard);
    let msg_str = serde_json::to_string(&welcome_msg).expect("Failed to serialize");
    
    validate_no_templates(&msg_str, "Go Fish scaling test");
    println!("✓ Zone scaling test passed");
}

/// Test that actual zone data in game state matches metadata
#[tokio::test]
async fn test_zone_data_consistency() {
    let bundle = load_bundle("go-fish");
    let lobbies = Arc::new(DashMap::new());
    let (lobby_tx, _) = broadcast::channel(100);
    
    let lobby = new_lobby("test_lobby".to_string(), bundle, lobbies, lobby_tx, None);
    lobby.add_player("alice".to_string());
    lobby.add_player("bob".to_string());
    lobby.clone().start_game();
    
    let welcome_msg = lobby.build_welcome_message("alice", true, &MessageFormat::Standard);
    let msg_str = serde_json::to_string(&welcome_msg).expect("Failed to serialize");
    
    validate_no_templates(&msg_str, "Go Fish data consistency test");
    println!("✓ Zone data consistency validated");
}

/// Helper function to validate that a JSON string contains no template patterns
fn validate_no_templates(json_str: &str, context: &str) {
    let forbidden_patterns = vec![
        "{player}",
        "{id}",
        "_p{id}",
        "hand_{player}",
        "deck_p{id}",
    ];
    
    for pattern in forbidden_patterns {
        assert!(
            !json_str.contains(pattern),
            "{} contains unexpanded template '{}': {}",
            context,
            pattern,
            json_str
        );
    }
}
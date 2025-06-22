use bluefelt_core::{bundle, engine::load_initial_state};
use serde_json::json;

#[test]
fn test_three_player_state_creation() {
    // Test that the state creation works with 3 players
    let manifest = bundle::Manifest {
        game_id: "test-3player".to_string(),
        version: "1.0".to_string(),
        spec_version: "1.0".to_string(),
        metadata: bundle::ManifestMetadata {
            name: "Test 3-Player Game".to_string(),
            author: "Test Author".to_string(),
            players: bundle::PlayersRange { min: 2, max: 3 },
            description: "Test game for 2-3 players".to_string(),
        },
        phases: None,
        setup: None,
        zone_groups: None,
    };

    let bundle = bundle::Bundle {
        game_id: "test-3player".to_string(),
        manifest,
        entities: json!([]),
        zones: json!([]),
        actions: json!([]),
        phases: json!([]),
    };

    let state = load_initial_state(&bundle);
    
    // Verify that 3 players were created
    let players = state.get("players").unwrap().as_array().unwrap();
    assert_eq!(players.len(), 3);
    
    // Verify player IDs are correct
    assert_eq!(players[0]["id"], "p1");
    assert_eq!(players[1]["id"], "p2");
    assert_eq!(players[2]["id"], "p3");
}

#[test]
fn test_four_player_state_creation() {
    // Test that the state creation works with 4 players
    let manifest = bundle::Manifest {
        game_id: "test-4player".to_string(),
        version: "1.0".to_string(),
        spec_version: "1.0".to_string(),
        metadata: bundle::ManifestMetadata {
            name: "Test 4-Player Game".to_string(),
            author: "Test Author".to_string(),
            players: bundle::PlayersRange { min: 2, max: 4 },
            description: "Test game for 2-4 players".to_string(),
        },
        phases: None,
        setup: None,
        zone_groups: None,
    };

    let bundle = bundle::Bundle {
        game_id: "test-4player".to_string(),
        manifest,
        entities: json!([]),
        zones: json!([]),
        actions: json!([]),
        phases: json!([]),
    };

    let state = load_initial_state(&bundle);
    
    // Verify that 4 players were created
    let players = state.get("players").unwrap().as_array().unwrap();
    assert_eq!(players.len(), 4);
    
    // Verify player IDs are correct
    assert_eq!(players[0]["id"], "p1");
    assert_eq!(players[1]["id"], "p2");
    assert_eq!(players[2]["id"], "p3");
    assert_eq!(players[3]["id"], "p4");
}

#[test]
fn test_player_zone_expansion() {
    // Test that player-specific zones are created for all players
    let manifest = bundle::Manifest {
        game_id: "test-zones".to_string(),
        version: "1.0".to_string(),
        spec_version: "1.0".to_string(),
        metadata: bundle::ManifestMetadata {
            name: "Test Zone Game".to_string(),
            author: "Test Author".to_string(),
            players: bundle::PlayersRange { min: 2, max: 3 },
            description: "Test game for zone creation".to_string(),
        },
        phases: None,
        setup: None,
        zone_groups: None,
    };

    let zones = json!([
        {
            "id": "hand_{player}",
            "type": "list"
        }
    ]);

    let bundle = bundle::Bundle {
        game_id: "test-zones".to_string(),
        manifest,
        entities: json!([]),
        zones,
        actions: json!([]),
        phases: json!([]),
    };

    let state = load_initial_state(&bundle);
    
    // Verify that player-specific zones were created
    let zones = state.get("zones").unwrap().as_object().unwrap();
    assert!(zones.contains_key("hand_p1"));
    assert!(zones.contains_key("hand_p2"));
    assert!(zones.contains_key("hand_p3"));
}
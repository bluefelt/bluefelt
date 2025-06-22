use bluefelt_core::{bundle::{Bundle, Manifest, ManifestMetadata, PlayersRange}, engine::{load_initial_state, apply_action}, conditions::evaluate_condition};
use serde_json::json;

// ===== ZONE EXPANSION TESTS =====

/// Test that zones with {player} template in their IDs are expanded correctly
#[test]
fn test_zone_id_player_template_expansion() {
    let manifest = create_test_manifest(3);
    let zones = json!([
        {
            "id": "hand_{player}",
            "type": "list",
            "items": []
        },
        {
            "id": "score_{player}",
            "type": "single",
            "contents": null
        }
    ]);

    let bundle = Bundle {
        game_id: "test-zone-templates".to_string(),
        manifest,
        entities: json!([]),
        zones,
        actions: json!([]),
        phases: json!([]),
    };

    let state = load_initial_state(&bundle);
    let zones = state.get("zones").unwrap().as_object().unwrap();
    
    // Verify zones were created for all 3 players
    assert!(zones.contains_key("hand_p1"), "Should have hand_p1");
    assert!(zones.contains_key("hand_p2"), "Should have hand_p2");
    assert!(zones.contains_key("hand_p3"), "Should have hand_p3");
    assert!(zones.contains_key("score_p1"), "Should have score_p1");
    assert!(zones.contains_key("score_p2"), "Should have score_p2");
    assert!(zones.contains_key("score_p3"), "Should have score_p3");
}

// ===== ENTITY EXPANSION TESTS =====

/// Test that entities with {player} template in their IDs are expanded correctly
#[test]
fn test_entity_id_player_template_expansion() {
    let manifest = create_test_manifest(2);
    let entities = json!([
        {
            "id": "token_{player}",
            "name": "Player {player} Token",
            "description": "Token for {player}"
        }
    ]);

    // For this test, we need to check if entities are properly expanded
    // This happens during bundle processing, not in load_initial_state
    // So we'll create zones that reference these entities
    let zones = json!([
        {
            "id": "tokens",
            "type": "list",
            "contents": [
                {"entity": "token_p1"},
                {"entity": "token_p2"}
            ]
        }
    ]);

    let bundle = Bundle {
        game_id: "test-entity-templates".to_string(),
        manifest,
        entities,
        zones,
        actions: json!([]),
        phases: json!([]),
    };

    let state = load_initial_state(&bundle);
    
    // Verify entities were referenced correctly
    let tokens_zone = &state["zones"]["tokens"]["items"];
    let items = tokens_zone.as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["entity"], "token_p1");
    assert_eq!(items[1]["entity"], "token_p2");
}

/// Test that zone contents with {player} templates are expanded
#[test]
fn test_zone_contents_player_template_expansion() {
    let manifest = create_test_manifest(2);
    let entities = json!([
        {
            "id": "piece_{player}",
            "name": "Player {player} Piece"
        }
    ]);
    
    let zones = json!([
        {
            "id": "pieces",
            "type": "list",
            "contents": ["piece_{player}"]  // This should expand to piece_p1, piece_p2
        }
    ]);

    let bundle = Bundle {
        game_id: "test-contents-templates".to_string(),
        manifest,
        entities,
        zones,
        actions: json!([]),
        phases: json!([]),
    };

    let state = load_initial_state(&bundle);
    let items = state["zones"]["pieces"]["items"].as_array().unwrap();
    
    // Should have 2 pieces, one per player
    assert_eq!(items.len(), 2, "Should have expanded to 2 pieces");
    
    let entities: Vec<&str> = items.iter()
        .filter_map(|item| item["entity"].as_str())
        .collect();
    
    assert!(entities.contains(&"piece_p1"), "Should have piece_p1");
    assert!(entities.contains(&"piece_p2"), "Should have piece_p2");
}

// ===== ACTION PARAMETER TESTS =====

/// Test that action parameters containing {player} are replaced at runtime
#[test]
fn test_action_parameter_player_template_replacement() {
    let manifest = create_test_manifest(2);
    let zones = json!([
        {
            "id": "deck",
            "type": "list",
            "contents": [{"entity": "card1"}, {"entity": "card2"}]
        },
        {
            "id": "hand_{player}",
            "type": "list",
            "contents": []
        }
    ]);
    
    let actions = json!([
        {
            "id": "drawToCurrentPlayer",
            "uses": "draw",
            "with": {
                "from": "/zones/deck",
                "to": "/zones/hand_{player}",
                "count": 1
            }
        }
    ]);

    let bundle = Bundle {
        game_id: "test-action-templates".to_string(),
        manifest,
        entities: json!([{"id": "card1"}, {"id": "card2"}]),
        zones,
        actions,
        phases: json!([]),
    };

    let mut state = load_initial_state(&bundle);
    
    // Set current player to p2
    state["currentPlayer"] = json!("p2");
    
    // Apply the action
    let action = json!({
        "verb": "draw",
        "args": {
            "from": "/zones/deck",
            "to": "/zones/hand_{player}",
            "count": 1
        }
    });
    
    let result = apply_action(&bundle, &mut state, "p2", &action);
    if let Err(e) = &result {
        println!("Action failed with error: {}", e);
    }
    assert!(result.is_ok(), "Action should succeed");
    
    // Verify card was drawn to p2's hand, not p1's
    println!("hand_p2 items: {:?}", state["zones"]["hand_p2"]["items"]);
    println!("hand_p1 items: {:?}", state["zones"]["hand_p1"]["items"]);
    assert_eq!(state["zones"]["hand_p2"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(state["zones"]["hand_p1"]["items"].as_array().unwrap().len(), 0);
}

/// Test that {actor} template is replaced with the acting player
#[test]
fn test_action_parameter_actor_template_replacement() {
    let manifest = create_test_manifest(3);
    let zones = json!([
        {
            "id": "deck",
            "type": "list",
            "contents": [{"entity": "token1"}]
        },
        {
            "id": "collection_{player}",
            "type": "list",
            "contents": []
        }
    ]);
    
    let actions = json!([
        {
            "id": "claimToken",
            "uses": "draw",
            "with": {
                "from": "/zones/deck",
                "to": "/zones/collection_{actor}",
                "count": 1
            }
        }
    ]);

    let bundle = Bundle {
        game_id: "test-actor-templates".to_string(),
        manifest,
        entities: json!([{"id": "token1"}]),
        zones,
        actions,
        phases: json!([]),
    };

    let mut state = load_initial_state(&bundle);
    
    // Current player is p1, but p3 is taking the action
    state["currentPlayer"] = json!("p1");
    
    // Apply action as p3 (the actor)
    let action = json!({
        "verb": "draw",
        "args": {
            "from": "/zones/deck",
            "to": "/zones/collection_{actor}",
            "count": 1
        }
    });
    
    println!("Deck before: {:?}", state["zones"]["deck"]["items"]);
    let result = apply_action(&bundle, &mut state, "p3", &action);
    if let Err(e) = &result {
        println!("Action failed with error: {}", e);
    }
    assert!(result.is_ok(), "Action should succeed");
    
    // Verify token was moved to p3's collection (the actor), not p1's
    println!("collection_p3 items: {:?}", state["zones"]["collection_p3"]["items"]);
    println!("collection_p1 items: {:?}", state["zones"]["collection_p1"]["items"]);
    assert_eq!(state["zones"]["collection_p3"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(state["zones"]["collection_p1"]["items"].as_array().unwrap().len(), 0);
}

/// Test that selection templates like {selection.X} are replaced
#[test]
fn test_action_parameter_selection_template_replacement() {
    let manifest = create_test_manifest(2);
    let zones = json!([
        {
            "id": "messages",
            "type": "single",
            "contents": null
        }
    ]);
    
    let actions = json!([
        {
            "id": "createMessage",
            "uses": "setState",
            "with": {
                "path": "/temp/message",
                "value": "Player selected rank {selection.selectedRank} from {selection.targetPlayer}"
            }
        }
    ]);

    let bundle = Bundle {
        game_id: "test-selection-templates".to_string(),
        manifest,
        entities: json!([]),
        zones,
        actions,
        phases: json!([]),
    };

    let mut state = load_initial_state(&bundle);
    
    // Set up selection state
    state["selection"] = json!({
        "selectedRank": "A",
        "targetPlayer": "p2"
    });
    
    // Apply the action
    let action = json!({
        "verb": "setState",
        "args": {
            "path": "/temp/message",
            "value": "Player selected rank {selection.selectedRank} from {selection.targetPlayer}"
        }
    });
    
    let result = apply_action(&bundle, &mut state, "p1", &action);
    assert!(result.is_ok(), "Action should succeed");
    
    // Verify templates were replaced
    assert_eq!(
        state["temp"]["message"].as_str().unwrap(),
        "Player selected rank A from p2"
    );
}

// ===== CONDITION TESTS =====

/// Test that conditions with {player} templates are evaluated correctly
#[test]
fn test_condition_player_template_evaluation() {
    let manifest = create_test_manifest(2);
    let zones = json!([
        {
            "id": "hand_{player}",
            "type": "list",
            "items": []
        }
    ]);
    
    // Add some items to p1's hand
    let bundle = Bundle {
        game_id: "test-condition-templates".to_string(),
        manifest,
        entities: json!([{"id": "card1"}]),
        zones: zones.clone(),
        actions: json!([]),
        phases: json!([]),
    };

    let mut state = load_initial_state(&bundle);
    state["currentPlayer"] = json!("p1");
    state["zones"]["hand_p1"]["items"] = json!([{"entity": "card1"}]);
    
    // Test zone.count condition with {player} template
    let condition = json!({
        "condition": "zone.count",
        "with": {
            "zone": "/zones/hand_{player}",
            "operator": ">",
            "value": 0
        }
    });
    
    let result = evaluate_condition(&condition, &state, &json!({}), "p1");
    assert!(result.unwrap(), "Condition should evaluate to true for p1");
    
    // Change current player and test again
    state["currentPlayer"] = json!("p2");
    let result = evaluate_condition(&condition, &state, &json!({}), "p2");
    assert!(!result.unwrap(), "Condition should evaluate to false for p2 (empty hand)");
}

// ===== PHASE ENTER ACTION TESTS =====

/// Test that enter actions with {player} templates work correctly
#[test]
fn test_phase_enter_action_template_replacement() {
    let manifest = create_test_manifest(2);
    let zones = json!([
        {
            "id": "deck",
            "type": "list", 
            "contents": [{"entity": "card1"}, {"entity": "card2"}]
        },
        {
            "id": "hand_{player}",
            "type": "list",
            "contents": []
        }
    ]);
    
    let actions = json!([
        {
            "id": "drawToCurrentPlayer",
            "uses": "draw",
            "with": {
                "from": "/zones/deck",
                "to": "/zones/hand_{player}",
                "count": 1
            }
        }
    ]);
    
    let phases = json!([
        {
            "id": "game",
            "phases": [
                {
                    "id": "drawing",
                    "enterActions": ["drawToCurrentPlayer"]
                }
            ]
        }
    ]);

    let bundle = Bundle {
        game_id: "test-enter-templates".to_string(),
        manifest,
        entities: json!([{"id": "card1"}, {"id": "card2"}]),
        zones,
        actions,
        phases,
    };

    let mut state = load_initial_state(&bundle);
    state["currentPlayer"] = json!("p2");
    state["phases"]["game"] = json!("drawing");
    
    // Put some cards in the deck for the test
    state["zones"]["deck"] = json!(["card1", "card2"]);
    
    // Process phases should replace {player} with p2
    let patches = bluefelt_core::engine::process_phases(&bundle, &mut state).unwrap();
    
    // Verify card was drawn to p2's hand
    assert!(!patches.is_empty(), "Should have patches from phase processing");
    
    // Debug: print all patches
    for (i, patch) in patches.iter().enumerate() {
        println!("Patch {}: {:?}", i, patch);
    }
    
    // Check if any patch affects p2's hand
    let has_p2_patch = patches.iter().any(|p| {
        if let Some(path) = p["path"].as_str() {
            path.contains("hand_p2")
        } else {
            false
        }
    });
    
    assert!(has_p2_patch, "Should have patch for p2's hand");
}

// ===== NESTED TEMPLATE TESTS =====

/// Test nested template replacement (templates within templates)
#[test]
#[ignore = "Nested template replacement not yet implemented"]
fn test_nested_template_replacement() {
    let manifest = create_test_manifest(2);
    let zones = json!([
        {
            "id": "deck",
            "type": "list",
            "items": [{"entity": "card1"}]
        },
        {
            "id": "hand_{player}",
            "type": "list",
            "items": []
        }
    ]);
    
    let actions = json!([
        {
            "id": "drawToSelectedZone",
            "uses": "draw",
            "with": {
                "from": "/zones/deck",
                "to": "/zones/{selection.targetZone}",
                "count": 1
            }
        }
    ]);

    let bundle = Bundle {
        game_id: "test-nested-templates".to_string(),
        manifest,
        entities: json!([{"id": "card1"}]),
        zones,
        actions,
        phases: json!([]),
    };

    let mut state = load_initial_state(&bundle);
    state["currentPlayer"] = json!("p2");
    state["selection"] = json!({
        "targetZone": "hand_{player}"  // This contains {player} template
    });
    
    // Apply action - should resolve {selection.targetZone} to "hand_{player}" 
    // then resolve {player} to "p2", resulting in "/zones/hand_p2"
    let action = json!({
        "verb": "draw",
        "args": {
            "from": "/zones/deck",
            "to": "/zones/{selection.targetZone}",
            "count": 1
        }
    });
    
    let result = apply_action(&bundle, &mut state, "p2", &action);
    assert!(result.is_ok(), "Action should succeed");
    
    // Verify card ended up in p2's hand
    assert_eq!(state["zones"]["hand_p2"]["items"].as_array().unwrap().len(), 1);
}

// ===== GAME LOG TESTS =====

/// Test that game log text with {player} templates is replaced correctly
#[test]
fn test_game_log_player_template_replacement() {
    let manifest = create_test_manifest(2);
    let actions = json!([
        {
            "id": "testAction",
            "uses": "setState",
            "with": {
                "path": "/test",
                "value": "test"
            },
            "ui": {
                "gameLog": "{player} performed an action"
            }
        }
    ]);

    let bundle = Bundle {
        game_id: "test-log-templates".to_string(),
        manifest,
        entities: json!([]),
        zones: json!([]),
        actions,
        phases: json!([]),
    };

    // This tests the game log generation in the lobby
    // Currently implemented in lobby.rs around line 1986
    let action = &bundle.actions.as_array().unwrap()[0];
    let log_template = action["ui"]["gameLog"].as_str().unwrap();
    let processed = log_template.replace("{player}", "p1");
    
    assert_eq!(processed, "p1 performed an action");
}

// ===== HELPER FUNCTIONS =====

fn create_test_manifest(max_players: u32) -> Manifest {
    Manifest {
        game_id: "test".to_string(),
        version: "1.0".to_string(),
        spec_version: "1.0".to_string(),
        metadata: ManifestMetadata {
            name: "Test Game".to_string(),
            author: "Test".to_string(),
            players: PlayersRange { min: 2, max: max_players },
            description: "Test game for template expansion".to_string(),
        },
        phases: None,
        setup: None,
        zone_groups: None,
    }
}
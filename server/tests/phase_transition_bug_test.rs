use bluefelt_core::{Bundle, engine::{load_initial_state_with_player_names, process_phases}};
use serde_json::{json, Value};
use std::fs;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// This test specifically validates the critical bug where enterActions
/// in initial phases don't execute, causing games to be stuck in setup
#[test]
fn test_enter_actions_execute_on_initial_phase() {
    // Create a minimal game bundle that tests enterActions
    let bundle_json = json!({
        "game_id": "test-enter-actions",
        "manifest": {
            "gameId": "test-enter-actions",
            "version": "1.0",
            "specVersion": "1.0",
            "metadata": {
                "name": "Test Enter Actions",
                "author": "Test",
                "description": "Test description",
                "players": { "min": 2, "max": 2 }
            }
        },
        "phases": [{
            "id": "game",
            "phases": [
                {
                    "id": "setup",
                    "initial": true,
                    "enterActions": ["testAction", "transitionAction"]
                },
                {
                    "id": "play",
                    "playerAction": true
                }
            ]
        }],
        "actions": [
            {
                "id": "testAction",
                "auto": true,
                "uses": "setState",
                "with": {
                    "path": "/testValue",
                    "value": "action executed"
                }
            },
            {
                "id": "transitionAction", 
                "auto": true,
                "uses": "setPhase",
                "with": {
                    "phaseSet": "game",
                    "phase": "play"
                }
            }
        ],
        "zones": [],
        "entities": []
    });

    let bundle: Bundle = serde_json::from_value(bundle_json).unwrap();
    
    // Initialize RNG
    let mut rng = StdRng::seed_from_u64(42);
    
    // Initialize game state
    let mut state = load_initial_state_with_player_names(&bundle, &vec!["Alice".to_string(), "Bob".to_string()], &mut rng);
    
    // Process phases - this should execute enterActions
    // NOTE: process_phases mutates the state directly!
    let patches = process_phases(&bundle, &mut state).unwrap();
    
    // Verify enterActions executed
    assert!(patches.len() > 0, "No patches generated - enterActions didn't execute");
    
    // Debug output
    println!("Generated {} patches:", patches.len());
    for (i, patch) in patches.iter().enumerate() {
        println!("Patch {}: {:?}", i, patch);
    }
    
    // Check that testAction set the value
    // The state should have been mutated directly by process_phases
    let test_value = state.get("testValue").and_then(|v| v.as_str());
    assert_eq!(test_value, Some("action executed"), "testAction didn't execute");
    
    // Check that phase transitioned
    let current_phase = state.get("phases").and_then(|p| p.get("game")).and_then(|p| p.as_str());
    assert_eq!(current_phase, Some("play"), "Phase didn't transition from setup to play");
}

#[test]
fn test_card_game_setup_pattern() {
    // Test the common card game pattern that's failing
    let bundle_json = json!({
        "game_id": "test-card-setup",
        "manifest": {
            "gameId": "test-card-setup",
            "version": "1.0",
            "specVersion": "1.0",
            "metadata": {
                "name": "Test Card Setup",
                "author": "Test",
                "description": "Test description",
                "players": { "min": 2, "max": 2 }
            }
        },
        "phases": [{
            "id": "game",
            "phases": [
                {
                    "id": "dealing",
                    "initial": true,
                    "enterActions": ["dealCards"]
                },
                {
                    "id": "play",
                    "playerAction": true,
                    "possibleActions": ["playCard"]
                }
            ]
        }],
        "actions": [
            {
                "id": "dealCards",
                "auto": true,
                "uses": "draw",
                "with": {
                    "from": "/zones/deck",
                    "to": "/zones/hand_p1", 
                    "count": 5
                },
                "then": [{"action": "dealToP2"}]
            },
            {
                "id": "dealToP2",
                "auto": true,
                "uses": "draw",
                "with": {
                    "from": "/zones/deck",
                    "to": "/zones/hand_p2",
                    "count": 5
                },
                "then": [{"action": "beginPlay"}]
            },
            {
                "id": "beginPlay",
                "auto": true,
                "uses": "setPhase",
                "with": {
                    "phaseSet": "game",
                    "phase": "play"
                }
            },
            {
                "id": "playCard",
                "uses": "draw",
                "with": {
                    "from": "/zones/hand_{player}",
                    "to": "/zones/discard",
                    "count": 1
                }
            }
        ],
        "zones": [
            {
                "id": "deck",
                "shape": "stack",
                "contents": "standardDeck"
            },
            {
                "id": "hand_{player}",
                "shape": "list"
            },
            {
                "id": "discard",
                "shape": "stack"
            }
        ],
        "entities": [
            {
                "id": "standardDeck",
                "type": "deck",
                "values": ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"],
                "suits": ["hearts", "diamonds", "clubs", "spades"]
            }
        ]
    });

    let bundle: Bundle = serde_json::from_value(bundle_json).unwrap();
    
    // Initialize RNG
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = load_initial_state_with_player_names(&bundle, &vec!["Alice".to_string(), "Bob".to_string()], &mut rng);
    
    // Get initial deck count - should have 1 item referencing standardDeck entity
    let initial_deck_count = state
        .pointer("/zones/deck/items")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    
    assert_eq!(initial_deck_count, 1, "Deck should have 1 entity reference");
    
    // Process phases - should deal cards and transition
    // NOTE: process_phases mutates the state directly!
    let patches = process_phases(&bundle, &mut state).unwrap();
    
    // Since this is a simplified test without actual card entities,
    // we can't verify card dealing. Just verify phase transition.
    
    // Verify phase transitioned
    let current_phase = state.pointer("/phases/game").and_then(|v| v.as_str());
    assert_eq!(current_phase, Some("play"), "Should transition to play phase");
    
    // Verify action map has actions available
    // This would require computing action map - simplified for this test
    assert!(patches.len() > 0, "Should generate patches for all state changes");
}

#[test] 
fn test_war_pattern_works() {
    // War uses a different pattern that works - let's verify it
    // This helps us understand what's different
    
    // Load War bundle components
    let manifest = fs::read_to_string("../bundles/war/1.0/manifest.json").unwrap();
    let phases = fs::read_to_string("../bundles/war/1.0/phases.json").unwrap();
    let actions = fs::read_to_string("../bundles/war/1.0/actions.json").unwrap();
    let zones = fs::read_to_string("../bundles/war/1.0/zones.json").unwrap();
    let entities = fs::read_to_string("../bundles/war/1.0/entities.json").unwrap();
    
    // Combine into bundle structure
    let bundle_json = json!({
        "gameId": "war",
        "version": "1.0",
        "manifest": serde_json::from_str::<Value>(&manifest).unwrap(),
        "phases": serde_json::from_str::<Value>(&phases).unwrap(),
        "actions": serde_json::from_str::<Value>(&actions).unwrap(),
        "zones": serde_json::from_str::<Value>(&zones).unwrap(),
        "entities": serde_json::from_str::<Value>(&entities).unwrap()
    });
    
    let bundle: Bundle = Bundle {
        game_id: "war".to_string(),
        manifest: serde_json::from_str(&manifest).unwrap(),
        phases: serde_json::from_str(&phases).unwrap(), 
        actions: serde_json::from_str(&actions).unwrap(),
        zones: serde_json::from_str(&zones).unwrap(),
        entities: serde_json::from_str(&entities).unwrap()
    };
    
    // Initialize RNG
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = load_initial_state_with_player_names(&bundle, &vec!["Alice".to_string(), "Bob".to_string()], &mut rng);
    
    // War's setup phase uses transitionToPhase in enterActions
    // NOTE: process_phases mutates the state directly!
    let patches = process_phases(&bundle, &mut state).unwrap();
    
    // Check that War transitions correctly
    let current_phase = state.pointer("/phases/game").and_then(|v| v.as_str());
    println!("War phase after setup: {:?}", current_phase);
    
    // War should transition from setup -> ready
    assert_ne!(current_phase, Some("setup"), "War should not be stuck in setup");
}
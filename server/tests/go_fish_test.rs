use bluefelt_core::bundle::BundleMap;
use serde_json::json;

#[test]
fn test_go_fish_game_setup() {
    // Load Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Failed to get Go Fish bundle");
    
    use bluefelt_core::engine::state::load_initial_state;
    let state = load_initial_state(&bundle);
    
    // Verify initial state for Go Fish
    assert_eq!(state["turn"], 0);
    assert_eq!(state["currentPlayer"], "p1");
    
    // Verify zones exist
    assert!(state["zones"]["pool"].is_object());
    assert!(state["zones"]["hand_p1"].is_object());
    assert!(state["zones"]["hand_p2"].is_object());
    assert!(state["zones"]["pairs_p1"].is_object());
    assert!(state["zones"]["pairs_p2"].is_object());
    
    // If 4 players are supported, verify their zones too
    if bundle.manifest.metadata.players.max >= 3 {
        assert!(state["zones"]["hand_p3"].is_object());
        assert!(state["zones"]["pairs_p3"].is_object());
    }
    
    if bundle.manifest.metadata.players.max >= 4 {
        assert!(state["zones"]["hand_p4"].is_object());
        assert!(state["zones"]["pairs_p4"].is_object());
    }
}

#[test]
fn test_go_fish_player_counts() {
    // Load Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Failed to get Go Fish bundle");
    
    // Verify player count constraints
    assert_eq!(bundle.manifest.metadata.players.min, 2);
    assert_eq!(bundle.manifest.metadata.players.max, 4);
    
    // Test state creation with different player counts
    use bluefelt_core::engine::state::load_initial_state;
    let state = load_initial_state(&bundle);
    let players = state["players"].as_array().unwrap();
    assert_eq!(players.len(), 4); // Max players are always created
    
    // Verify player IDs
    assert_eq!(players[0]["id"], "p1");
    assert_eq!(players[1]["id"], "p2");
    assert_eq!(players[2]["id"], "p3");
    assert_eq!(players[3]["id"], "p4");
}

#[test]
fn test_go_fish_standard_deck_generation() {
    // Load Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Failed to get Go Fish bundle");
    
    // Check that standardDeck has been expanded into 52 individual cards
    let entities = bundle.entities.as_array().expect("Entities should be an array");
    
    // Count card entities
    let card_count = entities.iter().filter(|entity| {
        entity.get("id").and_then(|id| id.as_str())
            .map(|id| id.starts_with("card_"))
            .unwrap_or(false)
    }).count();
    
    assert_eq!(card_count, 52, "Go Fish should have 52 card entities from standardDeck expansion");
    
    // Verify some specific cards exist
    let card_ids: Vec<&str> = entities.iter()
        .filter_map(|entity| entity.get("id").and_then(|id| id.as_str()))
        .filter(|id| id.starts_with("card_"))
        .collect();
    
    assert!(card_ids.contains(&"card_hearts_a"), "Should have Ace of Hearts");
    assert!(card_ids.contains(&"card_spades_k"), "Should have King of Spades");
    assert!(card_ids.contains(&"card_clubs_2"), "Should have 2 of Clubs");
    assert!(card_ids.contains(&"card_diamonds_10"), "Should have 10 of Diamonds");
}

#[test]
fn test_go_fish_zones_configuration() {
    // Load Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Failed to get Go Fish bundle");
    
    let zones = bundle.zones.as_array().expect("Zones should be an array");
    
    // Verify essential zones exist
    let zone_ids: Vec<&str> = zones.iter()
        .filter_map(|zone| zone.get("id").and_then(|id| id.as_str()))
        .collect();
    
    assert!(zone_ids.contains(&"pool"), "Should have pool zone");
    assert!(zone_ids.contains(&"hand_{player}"), "Should have hand_{{player}} zone");
    assert!(zone_ids.contains(&"pairs_{player}"), "Should have pairs_{{player}} zone");
    
    // Check for choice zone if it exists
    let has_choice_zone = zone_ids.iter().any(|&id| id.contains("choice"));
    if has_choice_zone {
        println!("Go Fish has choice zones for rank selection");
    }
    
    // Verify zone types and visibility
    for zone in zones {
        if let Some(zone_id) = zone.get("id").and_then(|id| id.as_str()) {
            match zone_id {
                "pool" => {
                    assert_eq!(zone.get("shape").and_then(|t| t.as_str()), Some("stack"));
                    // Pool should be hidden or show count only
                    let visibility = zone.get("visibility").and_then(|v| v.as_str());
                    assert!(visibility == Some("count") || visibility == Some("hidden"));
                }
                id if id.starts_with("hand_") => {
                    assert_eq!(zone.get("shape").and_then(|t| t.as_str()), Some("list"));
                    assert_eq!(zone.get("visibility").and_then(|v| v.as_str()), Some("owner"));
                }
                id if id.starts_with("pairs_") => {
                    assert_eq!(zone.get("shape").and_then(|t| t.as_str()), Some("stack"));
                    assert_eq!(zone.get("visibility").and_then(|v| v.as_str()), Some("public"));
                }
                _ => {} // Other zones are optional
            }
        }
    }
}

#[test]
fn test_go_fish_phases() {
    // Load Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Failed to get Go Fish bundle");
    
    // Check that phases are properly defined
    if let Some(phases) = bundle.phases.as_array() {
        // Look for the main game phase set
        let game_phase = phases.iter().find(|phase| {
            phase.get("id").and_then(|id| id.as_str()) == Some("game")
        });
        
        if let Some(game_phase) = game_phase {
            if let Some(sub_phases) = game_phase.get("phases").and_then(|p| p.as_array()) {
                let phase_names: Vec<&str> = sub_phases.iter()
                    .filter_map(|phase| phase.get("id").and_then(|id| id.as_str()))
                    .collect();
                
                // Verify key phases exist
                assert!(phase_names.contains(&"selectingRank"), "Should have selectingRank phase");
                
                // Check for initial phase
                let has_initial = sub_phases.iter().any(|phase| {
                    phase.get("initial").and_then(|init| init.as_bool()) == Some(true)
                });
                assert!(has_initial, "Should have an initial phase");
            }
        }
    }
}

#[test]
fn test_go_fish_actions_structure() {
    // Load Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Failed to get Go Fish bundle");
    
    let actions = bundle.actions.as_array().expect("Actions should be an array");
    
    // Verify we have some actions defined
    assert!(!actions.is_empty(), "Go Fish should have actions defined");
    
    // Check for key action patterns
    let action_ids: Vec<&str> = actions.iter()
        .filter_map(|action| action.get("id").and_then(|id| id.as_str()))
        .collect();
    
    // Look for essential actions (these might vary based on implementation)
    let has_turn_start = action_ids.iter().any(|&id| id.contains("start") || id.contains("Turn"));
    let has_selection = action_ids.iter().any(|&id| id.contains("select") || id.contains("Rank"));
    
    if has_turn_start {
        println!("Go Fish has turn start actions");
    }
    
    if has_selection {
        println!("Go Fish has selection actions");
    }
    
    // Verify actions have required fields
    for action in actions {
        assert!(action.get("id").is_some(), "All actions should have an id");
        
        // Check for auto actions
        if action.get("auto").and_then(|auto| auto.as_bool()) == Some(true) {
            println!("Found auto action: {}", action.get("id").unwrap().as_str().unwrap_or("unknown"));
        }
    }
}

#[test]
fn test_go_fish_bundle_completeness() {
    // Load Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Failed to get Go Fish bundle");
    
    // Verify all essential components exist
    assert_eq!(bundle.game_id, "go-fish");
    assert_eq!(bundle.manifest.game_id, "go-fish");
    assert_eq!(bundle.manifest.version, "1.0");
    
    // Verify metadata
    assert_eq!(bundle.manifest.metadata.name, "Go Fish");
    assert!(bundle.manifest.metadata.description.contains("card"));
    
    // Verify all required JSON components are valid
    assert!(bundle.entities.is_array());
    assert!(bundle.zones.is_array());
    assert!(bundle.actions.is_array());
    assert!(bundle.phases.is_array());
    
    println!("✓ Go Fish bundle is complete and valid");
}

#[test]
fn test_go_fish_player_selection() {
    use bluefelt_core::engine::state::load_initial_state_with_player_count;
    use bluefelt_core::engine::verbs::apply_verb;
    
    // Load Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Failed to get Go Fish bundle");
    
    // Create initial state with 2 players
    let mut state = load_initial_state_with_player_count(&bundle, Some(2));
    
    // Deck shuffling happens automatically during state initialization
    
    // Deal cards to players (7 cards each for 2 players)
    let deal_p1_args = json!({
        "from": "/zones/pool",
        "to": "/zones/hand_p1",
        "count": 7
    });
    apply_verb(&mut state, "draw", &deal_p1_args, &bundle).expect("Failed to deal to p1");
    
    let deal_p2_args = json!({
        "from": "/zones/pool", 
        "to": "/zones/hand_p2",
        "count": 7
    });
    apply_verb(&mut state, "draw", &deal_p2_args, &bundle).expect("Failed to deal to p2");
    
    // Set the game phase to selectingPlayer where selectPlayer should be active
    let set_phase_args = json!({
        "phaseSet": "game",
        "phase": "selectingPlayer"
    });
    apply_verb(&mut state, "setPhase", &set_phase_args, &bundle).expect("Failed to set phase");
    
    // Now we need to manually compute the action map since we're not using the Lobby
    // This is a limitation of this test approach, but we can check the action definitions
    
    // Check that the selectPlayer action exists in the bundle
    let actions = bundle.actions.as_array().expect("Actions should be an array");
    let select_player_action = actions.iter().find(|a| {
        a.get("id").and_then(|id| id.as_str()) == Some("selectPlayer")
    }).expect("selectPlayer action should exist");
    
    println!("selectPlayer action definition: {}", serde_json::to_string_pretty(&select_player_action).unwrap());
    
    // Verify the action has the right phase requirements
    let when_conditions = select_player_action.get("when")
        .and_then(|w| w.as_array())
        .expect("selectPlayer should have when conditions");
    
    assert!(when_conditions.iter().any(|condition| {
        condition.get("condition") == Some(&json!("phase.is")) &&
        condition.get("with").and_then(|w| w.get("phaseSet")) == Some(&json!("game")) &&
        condition.get("with").and_then(|w| w.get("phase")) == Some(&json!("selectingPlayer"))
    }), "selectPlayer should be active in game.selectingPlayer phase");
    
    // Check the state has the necessary players
    let players = state.get("players").and_then(|p| p.as_array()).expect("Should have players");
    assert_eq!(players.len(), 2, "Should have 2 players");
    assert_eq!(players[0]["id"], "p1");
    assert_eq!(players[1]["id"], "p2");
    
    println!("✓ Go Fish selectPlayer action is properly configured");
}

#[test]
fn test_go_fish_action_map_generation() {
    use bluefelt_core::lobby::Lobby;
    use bluefelt_core::engine::state::load_initial_state_with_player_count;
    use bluefelt_core::engine::verbs::apply_verb;
    
    // Load Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Failed to get Go Fish bundle");
    
    // Create initial state with 2 players
    let mut state = load_initial_state_with_player_count(&bundle, Some(2));
    
    // Deck shuffling happens automatically during state initialization
    
    // Deal cards to players (7 cards each for 2 players)
    let deal_p1_args = json!({
        "from": "/zones/pool",
        "to": "/zones/hand_p1",
        "count": 7
    });
    apply_verb(&mut state, "draw", &deal_p1_args, &bundle).expect("Failed to deal to p1");
    
    let deal_p2_args = json!({
        "from": "/zones/pool", 
        "to": "/zones/hand_p2",
        "count": 7
    });
    apply_verb(&mut state, "draw", &deal_p2_args, &bundle).expect("Failed to deal to p2");
    
    // Set the game phase to selectingPlayer where selectPlayer should be active
    let set_phase_args = json!({
        "phaseSet": "game",
        "phase": "selectingPlayer"
    });
    apply_verb(&mut state, "setPhase", &set_phase_args, &bundle).expect("Failed to set phase");
    
    // Compute the action map
    let action_map = Lobby::compute_action_map(&state, &bundle);
    
    println!("Action map computed: {}", serde_json::to_string_pretty(&action_map).unwrap());
    
    // Check that P1 has selectPlayer actions
    if let Some(p1_actions) = action_map.get("p1") {
        let p1_action_map = p1_actions.as_object().unwrap();
        
        // Count selectPlayer actions for P1
        let select_player_actions: Vec<_> = p1_action_map.iter()
            .filter(|(loc, action)| {
                action.get("action").and_then(|a| a.as_str()) == Some("selectPlayer")
            })
            .collect();
        
        println!("Found {} player selection actions for P1", select_player_actions.len());
        for (loc, action) in &select_player_actions {
            println!("  Location: {} -> {:?}", loc, action);
        }
        
        // P1 should be able to select P2 in choice zone
        let expected_location = "/zones/choice_p1/players/p2";
        assert!(select_player_actions.len() > 0, 
            "P1 should have at least one selectPlayer action, but found none. Available actions: {:?}", 
            p1_action_map.keys().collect::<Vec<_>>());
        
        if p1_action_map.contains_key(expected_location) {
            let action = &p1_action_map[expected_location];
            assert_eq!(action["action"], "selectPlayer");
            assert_eq!(action["targetPlayer"], "p2");
            println!("✓ Found selectPlayer action at expected location");
        } else {
            println!("Expected location {} not found. Available locations: {:?}", 
                expected_location, p1_action_map.keys().collect::<Vec<_>>());
        }
    } else {
        panic!("No action map found for p1");
    }
}
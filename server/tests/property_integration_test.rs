use bluefelt_core::{bundle, engine::verbs::apply_verb, conditions::evaluate_condition};
use serde_json::json;

fn create_go_fish_test_bundle() -> bundle::Bundle {
    bundle::Bundle {
        game_id: "go-fish-test".to_string(),
        manifest: bundle::Manifest {
            game_id: "go-fish-test".to_string(),
            version: "1.0".to_string(),
            spec_version: "1.0".to_string(),
            metadata: bundle::ManifestMetadata {
                name: "Go Fish Test".to_string(),
                author: "Test Author".to_string(),
                players: bundle::PlayersRange { min: 2, max: 4 },
                description: "Test game for Go Fish property operations".to_string(),
            },
            phases: None,
            setup: None,
            zone_groups: None,
        },
        entities: json!([]),
        zones: json!([]),
        actions: json!([]),
        phases: json!([]),
        _hooks: None,
    }
}

#[test]
fn test_complete_go_fish_meld_flow() {
    let bundle = create_go_fish_test_bundle();
    
    // Setup initial state with player having some cards
    let mut state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": [
                    {"entity": "card_2_hearts"},
                    {"entity": "card_2_spades"},
                    {"entity": "card_2_diamonds"},
                    {"entity": "card_2_clubs"},
                    {"entity": "card_7_spades"},
                    {"entity": "card_ace_clubs"}
                ]
            },
            "hand_p2": {
                "type": "list",
                "items": [
                    {"entity": "card_king_hearts"},
                    {"entity": "card_queen_spades"}
                ]
            },
            "pairs_p1": {
                "type": "list",
                "items": []
            },
            "temp": {}
        }
    });

    // Step 1: Query available ranks in player's hand
    let query_args = json!({
        "zone": "/zones/hand_p1",
        "property": "rank",
        "storePath": "/temp/available_ranks"
    });

    let result = apply_verb(&mut state, "queryEntities", &query_args, &bundle);
    assert!(result.is_ok());

    let available_ranks = state["temp"]["available_ranks"].as_array().unwrap();
    assert_eq!(available_ranks.len(), 3); // 2, 7, ace
    assert!(available_ranks.contains(&json!("2")));
    assert!(available_ranks.contains(&json!("7")));
    assert!(available_ranks.contains(&json!("ace")));

    // Step 2: Check if player has 4 cards of rank "2"
    let has_four_condition = json!({
        "condition": "zone.countWhere",
        "with": {
            "zone": "/zones/hand_p1",
            "property": "rank",
            "value": "2",
            "operator": "==",
            "count": 4
        }
    });

    let has_four = evaluate_condition(&has_four_condition, &state, &json!({}), "p1").unwrap();
    assert_eq!(has_four, true);

    // Step 3: Transfer all cards of rank "2" to pairs zone (forming a meld)
    let transfer_args = json!({
        "from": "/zones/hand_p1",
        "to": "/zones/pairs_p1",
        "property": "rank",
        "value": "2"
    });

    let result = apply_verb(&mut state, "transferMatching", &transfer_args, &bundle);
    assert!(result.is_ok());

    // Verify the meld was formed correctly
    let hand_items = state["zones"]["hand_p1"]["items"].as_array().unwrap();
    let pairs_items = state["zones"]["pairs_p1"]["items"].as_array().unwrap();
    
    assert_eq!(hand_items.len(), 2); // Only 7_spades and ace_clubs remain
    assert_eq!(pairs_items.len(), 4); // All four 2s moved to pairs

    // Verify hand contains expected cards
    assert!(hand_items.iter().any(|item| item["entity"] == "card_7_spades"));
    assert!(hand_items.iter().any(|item| item["entity"] == "card_ace_clubs"));

    // Verify pairs contains all 2s
    assert!(pairs_items.iter().any(|item| item["entity"] == "card_2_hearts"));
    assert!(pairs_items.iter().any(|item| item["entity"] == "card_2_spades"));
    assert!(pairs_items.iter().any(|item| item["entity"] == "card_2_diamonds"));
    assert!(pairs_items.iter().any(|item| item["entity"] == "card_2_clubs"));

    // Step 4: Check if player can form another meld (should be false now)
    let can_form_another_condition = json!({
        "condition": "zone.countWhere",
        "with": {
            "zone": "/zones/hand_p1",
            "property": "rank",
            "value": "7",
            "operator": ">=",
            "count": 4
        }
    });

    let can_form_another = evaluate_condition(&can_form_another_condition, &state, &json!({}), "p1").unwrap();
    assert_eq!(can_form_another, false);
}

#[test]
fn test_go_fish_asking_workflow() {
    let bundle = create_go_fish_test_bundle();
    
    // Setup state where p1 asks p2 for cards
    let mut state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": [
                    {"entity": "card_7_hearts"},
                    {"entity": "card_7_spades"}
                ]
            },
            "hand_p2": {
                "type": "list",
                "items": [
                    {"entity": "card_7_diamonds"},
                    {"entity": "card_king_hearts"},
                    {"entity": "card_queen_spades"}
                ]
            }
        }
    });

    // Step 1: Check if p1 has cards of rank "7" to ask for
    let has_rank_condition = json!({
        "condition": "zone.hasMatching",
        "with": {
            "zone": "/zones/hand_p1",
            "property": "rank",
            "value": "7"
        }
    });

    let has_rank = evaluate_condition(&has_rank_condition, &state, &json!({}), "p1").unwrap();
    assert_eq!(has_rank, true);

    // Step 2: Check if p2 has cards of rank "7" to give
    let p2_has_rank_condition = json!({
        "condition": "zone.hasMatching",
        "with": {
            "zone": "/zones/hand_p2",
            "property": "rank",
            "value": "7"
        }
    });

    let p2_has_rank = evaluate_condition(&p2_has_rank_condition, &state, &json!({}), "p2").unwrap();
    assert_eq!(p2_has_rank, true);

    // Step 3: Transfer matching cards from p2 to p1
    let transfer_args = json!({
        "from": "/zones/hand_p2",
        "to": "/zones/hand_p1",
        "property": "rank",
        "value": "7"
    });

    let result = apply_verb(&mut state, "transferMatching", &transfer_args, &bundle);
    assert!(result.is_ok());

    // Verify the transfer worked
    let p1_hand = state["zones"]["hand_p1"]["items"].as_array().unwrap();
    let p2_hand = state["zones"]["hand_p2"]["items"].as_array().unwrap();
    
    assert_eq!(p1_hand.len(), 3); // p1 now has 3 sevens
    assert_eq!(p2_hand.len(), 2); // p2 has 2 cards left (king, queen)

    // Verify p1 has all the sevens
    let seven_count = p1_hand.iter()
        .filter(|item| {
            if let Some(entity_id) = item.get("entity").and_then(|e| e.as_str()) {
                entity_id.starts_with("card_7_")
            } else {
                false
            }
        })
        .count();
    assert_eq!(seven_count, 3);

    // Step 4: Check if p1 can now form a complete set (needs 4, has 3)
    let can_form_set_condition = json!({
        "condition": "zone.countWhere",
        "with": {
            "zone": "/zones/hand_p1",
            "property": "rank",
            "value": "7",
            "operator": "==",
            "count": 4
        }
    });

    let can_form_set = evaluate_condition(&can_form_set_condition, &state, &json!({}), "p1").unwrap();
    assert_eq!(can_form_set, false); // Still needs one more
}

#[test]
fn test_property_operations_error_cases() {
    let bundle = create_go_fish_test_bundle();
    
    // Test queryEntities with invalid zone
    let mut state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": []
            }
        }
    });

    let args = json!({
        "zone": "/zones/nonexistent",
        "property": "rank",
        "storePath": "/temp/ranks"
    });

    let result = apply_verb(&mut state, "queryEntities", &args, &bundle);
    assert!(result.is_err());

    // Test transferMatching with invalid zones
    let args = json!({
        "from": "/zones/nonexistent",
        "to": "/zones/hand_p1",
        "property": "rank",
        "value": "2"
    });

    let result = apply_verb(&mut state, "transferMatching", &args, &bundle);
    assert!(result.is_err());

    // Test condition with invalid zone
    let condition = json!({
        "condition": "zone.hasMatching",
        "with": {
            "zone": "/zones/nonexistent",
            "property": "rank",
            "value": "2"
        }
    });

    let result = evaluate_condition(&condition, &state, &json!({}), "p1");
    assert!(result.is_err());
}

#[test]
fn test_multi_player_property_operations() {
    let bundle = create_go_fish_test_bundle();
    
    // Setup state with 3 players
    let mut state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": [
                    {"entity": "card_ace_hearts"},
                    {"entity": "card_ace_spades"}
                ]
            },
            "hand_p2": {
                "type": "list",
                "items": [
                    {"entity": "card_ace_diamonds"},
                    {"entity": "card_king_hearts"}
                ]
            },
            "hand_p3": {
                "type": "list",
                "items": [
                    {"entity": "card_ace_clubs"},
                    {"entity": "card_queen_spades"}
                ]
            }
        }
    });

    // Test conditions work correctly for different players
    let ace_condition = json!({
        "condition": "zone.hasMatching",
        "with": {
            "zone": "/zones/hand_{player}",
            "property": "rank",
            "value": "ace"
        }
    });

    // All players should have aces
    assert_eq!(evaluate_condition(&ace_condition, &state, &json!({}), "p1").unwrap(), true);
    assert_eq!(evaluate_condition(&ace_condition, &state, &json!({}), "p2").unwrap(), true);
    assert_eq!(evaluate_condition(&ace_condition, &state, &json!({}), "p3").unwrap(), true);

    // Test transferring from p2 to p1
    let transfer_args = json!({
        "from": "/zones/hand_p2",
        "to": "/zones/hand_p1",
        "property": "rank",
        "value": "ace"
    });

    let result = apply_verb(&mut state, "transferMatching", &transfer_args, &bundle);
    assert!(result.is_ok());

    // Verify p1 now has 3 aces, p2 has 1 card left
    let p1_hand = state["zones"]["hand_p1"]["items"].as_array().unwrap();
    let p2_hand = state["zones"]["hand_p2"]["items"].as_array().unwrap();
    
    assert_eq!(p1_hand.len(), 3);
    assert_eq!(p2_hand.len(), 1);
    assert_eq!(p2_hand[0]["entity"], "card_king_hearts");
}
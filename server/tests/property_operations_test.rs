use bluefelt_core::{bundle, engine::verbs::apply_verb, conditions::evaluate_condition};
use serde_json::json;

fn create_test_bundle() -> bundle::Bundle {
    bundle::Bundle {
        game_id: "test-property".to_string(),
        manifest: bundle::Manifest {
            game_id: "test-property".to_string(),
            version: "1.0".to_string(),
            spec_version: "1.0".to_string(),
            metadata: bundle::ManifestMetadata {
                name: "Test Property Game".to_string(),
                author: "Test Author".to_string(),
                players: bundle::PlayersRange { min: 2, max: 4 },
                description: "Test game for property operations".to_string(),
            },
            phases: None,
            setup: None,
            zone_groups: None,
        },
        entities: json!([]),
        zones: json!([]),
        actions: json!([]),
        phases: json!([]),
    }
}

#[test]
fn test_query_entities_by_rank() {
    let bundle = create_test_bundle();
    let mut state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": [
                    {"entity": "card_2_hearts"},
                    {"entity": "card_7_spades"},
                    {"entity": "card_2_diamonds"},
                    {"entity": "card_ace_clubs"}
                ]
            }
        },
        "temp": {}
    });

    let args = json!({
        "zone": "/zones/hand_p1",
        "property": "rank",
        "storePath": "/temp/ranks"
    });

    let result = apply_verb(&mut state, "queryEntities", &args, &bundle);
    assert!(result.is_ok());

    // Check that unique ranks were extracted and sorted
    let ranks = state["temp"]["ranks"].as_array().unwrap();
    assert_eq!(ranks.len(), 3); // 2, 7, ace
    assert_eq!(ranks[0], "2");
    assert_eq!(ranks[1], "7");
    assert_eq!(ranks[2], "ace");
}

#[test]
fn test_query_entities_by_suit() {
    let bundle = create_test_bundle();
    let mut state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": [
                    {"entity": "card_2_hearts"},
                    {"entity": "card_7_spades"},
                    {"entity": "card_king_hearts"},
                    {"entity": "card_ace_clubs"}
                ]
            }
        },
        "temp": {}
    });

    let args = json!({
        "zone": "/zones/hand_p1",
        "property": "suit",
        "storePath": "/temp/suits"
    });

    let result = apply_verb(&mut state, "queryEntities", &args, &bundle);
    assert!(result.is_ok());

    // Check that unique suits were extracted and sorted
    let suits = state["temp"]["suits"].as_array().unwrap();
    assert_eq!(suits.len(), 3); // clubs, hearts, spades (alphabetical)
    assert_eq!(suits[0], "clubs");
    assert_eq!(suits[1], "hearts");
    assert_eq!(suits[2], "spades");
}

#[test]
fn test_transfer_matching_cards() {
    let bundle = create_test_bundle();
    let mut state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": [
                    {"entity": "card_2_hearts"},
                    {"entity": "card_7_spades"},
                    {"entity": "card_2_diamonds"},
                    {"entity": "card_ace_clubs"},
                    {"entity": "card_2_clubs"}
                ]
            },
            "pairs_p1": {
                "type": "list",
                "items": []
            }
        }
    });

    let args = json!({
        "from": "/zones/hand_p1",
        "to": "/zones/pairs_p1",
        "property": "rank",
        "value": "2"
    });

    let result = apply_verb(&mut state, "transferMatching", &args, &bundle);
    assert!(result.is_ok());

    // Check that all 2s were moved
    let hand_items = state["zones"]["hand_p1"]["items"].as_array().unwrap();
    let pairs_items = state["zones"]["pairs_p1"]["items"].as_array().unwrap();
    
    assert_eq!(hand_items.len(), 2); // 7_spades, ace_clubs remaining
    assert_eq!(pairs_items.len(), 3); // All three 2s moved
    
    // Verify remaining cards in hand
    assert!(hand_items.iter().any(|item| item["entity"] == "card_7_spades"));
    assert!(hand_items.iter().any(|item| item["entity"] == "card_ace_clubs"));
    
    // Verify moved cards in pairs
    assert!(pairs_items.iter().any(|item| item["entity"] == "card_2_hearts"));
    assert!(pairs_items.iter().any(|item| item["entity"] == "card_2_diamonds"));
    assert!(pairs_items.iter().any(|item| item["entity"] == "card_2_clubs"));
}

#[test]
fn test_zone_has_matching_condition() {
    let state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": [
                    {"entity": "card_2_hearts"},
                    {"entity": "card_7_spades"},
                    {"entity": "card_ace_clubs"}
                ]
            }
        }
    });

    // Test condition that should be true
    let condition = json!({
        "condition": "zone.hasMatching",
        "with": {
            "zone": "/zones/hand_{player}",
            "property": "rank",
            "value": "2"
        }
    });
    
    assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);

    // Test condition that should be false
    let condition = json!({
        "condition": "zone.hasMatching",
        "with": {
            "zone": "/zones/hand_{player}",
            "property": "rank",
            "value": "king"
        }
    });
    
    assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), false);
}

#[test]
fn test_zone_count_where_condition() {
    let state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": [
                    {"entity": "card_2_hearts"},
                    {"entity": "card_2_spades"},
                    {"entity": "card_2_diamonds"},
                    {"entity": "card_7_clubs"}
                ]
            }
        }
    });

    // Test counting cards with rank "2"
    let condition = json!({
        "condition": "zone.countWhere",
        "with": {
            "zone": "/zones/hand_{player}",
            "property": "rank",
            "value": "2",
            "operator": "==",
            "count": 3
        }
    });
    
    assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);

    // Test with >= operator
    let condition = json!({
        "condition": "zone.countWhere",
        "with": {
            "zone": "/zones/hand_{player}",
            "property": "rank",
            "value": "2",
            "operator": ">=",
            "count": 2
        }
    });
    
    assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);

    // Test condition that should be false
    let condition = json!({
        "condition": "zone.countWhere",
        "with": {
            "zone": "/zones/hand_{player}",
            "property": "rank",
            "value": "2",
            "operator": "==",
            "count": 5
        }
    });
    
    assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), false);
}

#[test]
fn test_transfer_matching_no_matches() {
    let bundle = create_test_bundle();
    let mut state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": [
                    {"entity": "card_7_spades"},
                    {"entity": "card_ace_clubs"}
                ]
            },
            "pairs_p1": {
                "type": "list",
                "items": []
            }
        }
    });

    let args = json!({
        "from": "/zones/hand_p1",
        "to": "/zones/pairs_p1",
        "property": "rank",
        "value": "2"
    });

    let result = apply_verb(&mut state, "transferMatching", &args, &bundle);
    assert!(result.is_ok());

    // Check that no cards were moved
    let hand_items = state["zones"]["hand_p1"]["items"].as_array().unwrap();
    let pairs_items = state["zones"]["pairs_p1"]["items"].as_array().unwrap();
    
    assert_eq!(hand_items.len(), 2); // Original cards remain
    assert_eq!(pairs_items.len(), 0); // No cards moved
}

#[test]
fn test_query_entities_empty_zone() {
    let bundle = create_test_bundle();
    let mut state = json!({
        "zones": {
            "hand_p1": {
                "type": "list",
                "items": []
            }
        },
        "temp": {}
    });

    let args = json!({
        "zone": "/zones/hand_p1",
        "property": "rank",
        "storePath": "/temp/ranks"
    });

    let result = apply_verb(&mut state, "queryEntities", &args, &bundle);
    assert!(result.is_ok());

    // Check that empty array was stored
    let ranks = state["temp"]["ranks"].as_array().unwrap();
    assert_eq!(ranks.len(), 0);
}
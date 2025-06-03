use bluefelt_core::conditions::evaluate_condition;
use serde_json::json;

#[test]
fn test_entity_filter_replacement() {
    // Test state with pieces for both players
    let state = json!({
        "zones": {
            "board": {
                "cells": [
                    [{"entity": "piece_p1"}, {"entity": "piece_p2"}, null],
                    [{"entity": "piece_p1"}, null, null],
                    [null, null, null]
                ]
            }
        }
    });

    // Test counting pieces for p1 using {player} placeholder
    let condition = json!({
        "condition": "zone.count",
        "with": {
            "zone": "/zones/board",
            "entity": "piece_{player}",  // Should become "piece_p1"
            "operator": "==",
            "value": 2
        }
    });

    // Should correctly count 2 pieces for p1
    let result = evaluate_condition(&condition, &state, &json!({}), "p1").unwrap();
    assert!(result, "Should correctly count 2 pieces for p1");

    // Test with p2 (should count 1 piece)
    let condition_p2 = json!({
        "condition": "zone.count", 
        "with": {
            "zone": "/zones/board",
            "entity": "piece_{player}",  // Should become "piece_p2"
            "operator": "==",
            "value": 1
        }
    });

    let result_p2 = evaluate_condition(&condition_p2, &state, &json!({}), "p2").unwrap();
    assert!(result_p2, "Should correctly count 1 piece for p2");
}

#[test]
fn test_placement_limit_condition() {
    // Test the specific condition from Three Men's Morris
    let state = json!({
        "zones": {
            "board": {
                "cells": [
                    [{"entity": "piece_p1"}, {"entity": "piece_p1"}, {"entity": "piece_p1"}],
                    [null, null, null],
                    [null, null, null]
                ]
            }
        }
    });

    // Test the "< 3" condition when player has exactly 3 pieces
    let condition = json!({
        "condition": "zone.count",
        "with": {
            "zone": "/zones/board",
            "entity": "piece_{player}",
            "operator": "<", 
            "value": 3
        }
    });

    // Should return false (3 is not < 3)
    let result = evaluate_condition(&condition, &state, &json!({}), "p1").unwrap();
    assert!(!result, "Player with 3 pieces should not be able to place more (3 < 3 = false)");

    // Test with p2 who has 0 pieces
    let result_p2 = evaluate_condition(&condition, &state, &json!({}), "p2").unwrap();
    assert!(result_p2, "Player with 0 pieces should be able to place (0 < 3 = true)");
}
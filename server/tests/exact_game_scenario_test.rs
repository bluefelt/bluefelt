use bluefelt_core::conditions::evaluate_condition;
use serde_json::json;

#[test]
fn test_exact_game_scenario() {
    // Test the exact scenario from the game where both players have 3 pieces
    // Based on the game logs, let's simulate a realistic board state
    let state = json!({
        "zones": {
            "board": {
                "cells": [
                    [{"entity": "piece_p1"}, {"entity": "piece_p2"}, {"entity": "piece_p1"}],
                    [{"entity": "piece_p1"}, {"entity": "piece_p2"}, null],
                    [null, {"entity": "piece_p2"}, null]
                ]
            }
        }
    });

    // Test the exact conditions from checkPhaseTransition action
    let p1_condition = json!({
        "condition": "zone.count",
        "with": {
            "zone": "/zones/board",
            "entity": "piece_p1",
            "operator": "==",
            "value": 3
        }
    });

    let p2_condition = json!({
        "condition": "zone.count", 
        "with": {
            "zone": "/zones/board",
            "entity": "piece_p2",
            "operator": "==",
            "value": 3
        }
    });

    let p1_result = evaluate_condition(&p1_condition, &state, &json!({}), "p1").unwrap();
    let p2_result = evaluate_condition(&p2_condition, &state, &json!({}), "p2").unwrap();

    println!("P1 condition result: {}", p1_result);
    println!("P2 condition result: {}", p2_result);
    println!("Both conditions met (AND): {}", p1_result && p2_result);

    // Both should be true for phase transition to occur
    assert!(p1_result, "P1 should have exactly 3 pieces");
    assert!(p2_result, "P2 should have exactly 3 pieces");
    assert!(p1_result && p2_result, "Phase transition should occur when both players have 3 pieces");
}
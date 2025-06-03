use bluefelt_core::conditions::evaluate_condition;
use serde_json::json;

#[test]
fn test_phase_transition_conditions() {
    // Test state where p1 has 3 pieces, p2 has 2 pieces
    let state = json!({
        "zones": {
            "board": {
                "cells": [
                    [{"entity": "piece_p1"}, {"entity": "piece_p1"}, {"entity": "piece_p1"}],
                    [{"entity": "piece_p2"}, {"entity": "piece_p2"}, null],
                    [null, null, null]
                ]
            }
        }
    });

    // Test p1 condition (should pass: 3 == 3)
    let p1_condition = json!({
        "condition": "zone.count",
        "with": {
            "zone": "/zones/board",
            "entity": "piece_p1",
            "operator": "==",
            "value": 3
        }
    });

    let p1_result = evaluate_condition(&p1_condition, &state, &json!({}), "p1").unwrap();
    assert!(p1_result, "P1 should have exactly 3 pieces");

    // Test p2 condition (should fail: 2 != 3)
    let p2_condition = json!({
        "condition": "zone.count", 
        "with": {
            "zone": "/zones/board",
            "entity": "piece_p2",
            "operator": "==",
            "value": 3
        }
    });

    let p2_result = evaluate_condition(&p2_condition, &state, &json!({}), "p2").unwrap();
    assert!(!p2_result, "P2 should not have 3 pieces yet (has 2)");

    // Now test when both have 3 pieces
    let state_both_3 = json!({
        "zones": {
            "board": {
                "cells": [
                    [{"entity": "piece_p1"}, {"entity": "piece_p1"}, {"entity": "piece_p1"}],
                    [{"entity": "piece_p2"}, {"entity": "piece_p2"}, {"entity": "piece_p2"}],
                    [null, null, null]
                ]
            }
        }
    });

    let p1_result_final = evaluate_condition(&p1_condition, &state_both_3, &json!({}), "p1").unwrap();
    let p2_result_final = evaluate_condition(&p2_condition, &state_both_3, &json!({}), "p2").unwrap();

    assert!(p1_result_final, "P1 should have exactly 3 pieces");
    assert!(p2_result_final, "P2 should have exactly 3 pieces"); 
}

#[test]
fn test_phase_transition_should_only_occur_when_both_have_3() {
    // This test simulates the AND logic of the checkPhaseTransition action
    
    let scenarios = vec![
        // (p1_count, p2_count, should_transition)
        (0, 0, false),
        (1, 0, false), 
        (1, 1, false),
        (2, 1, false),
        (3, 2, false), // This was the failing case
        (2, 3, false),
        (3, 3, true),  // Only this should transition
    ];

    for (p1_count, p2_count, should_transition) in scenarios {
        // Create state with specified piece counts
        let mut cells = vec![vec![json!(null); 3]; 3];
        
        // Place pieces in a way that doesn't overlap
        let mut placed = 0;
        
        // Place p1 pieces first
        for row in 0..3 {
            for col in 0..3 {
                if placed < p1_count {
                    cells[row][col] = json!({"entity": "piece_p1"});
                    placed += 1;
                }
            }
        }
        
        // Place p2 pieces in remaining empty spots
        placed = 0;
        for row in 0..3 {
            for col in 0..3 {
                if placed < p2_count && cells[row][col] == json!(null) {
                    cells[row][col] = json!({"entity": "piece_p2"});
                    placed += 1;
                }
            }
        }

        let state = json!({
            "zones": {
                "board": {
                    "cells": cells
                }
            }
        });

        // Test both conditions (AND logic)
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
        
        // Phase transition should only occur when BOTH conditions are true
        let actual_transition = p1_result && p2_result;
        
        assert_eq!(actual_transition, should_transition, 
            "Failed for p1={}, p2={}: expected transition={}, but p1_condition={}, p2_condition={}, actual={}", 
            p1_count, p2_count, should_transition, p1_result, p2_result, actual_transition);
    }
}
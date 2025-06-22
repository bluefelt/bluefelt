#[cfg(test)]
mod test_pattern_matching {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_grid_line_of_marks_pattern_matching() {
        // Create a simple 3x3 board with a winning line
        let state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [{"entity": "mark_p1"}, {"entity": "mark_p2"}, null],
                        [null, {"entity": "mark_p1"}, null],
                        [null, null, {"entity": "mark_p1"}]
                    ]
                }
            }
        });

        let condition = json!({
            "condition": "grid.lineOfMarks",
            "with": {
                "zone": "zones/board",
                "entity": "mark_{player}",
                "lineLength": 3,
                "directions": ["diagonal"]
            }
        });

        let args = json!({});
        let current_actor = "p1";

        let result = evaluate_condition(&condition, &state, &args, current_actor);
        assert!(result.is_ok(), "Condition evaluation should succeed");
        assert_eq!(result.unwrap(), true, "Should detect winning diagonal line for p1");
    }

    #[test]
    fn test_matches_pattern_condition_direct() {
        // Test the pattern matching function directly
        assert_eq!(matches_pattern_condition("mark_p1", "mark_p1"), true);
        assert_eq!(matches_pattern_condition("mark_p1", "mark_{player}"), false); // Should be false since template not replaced
        assert_eq!(matches_pattern_condition("mark_p1", "mark_"), true); // Should be true if base pattern works
    }
}
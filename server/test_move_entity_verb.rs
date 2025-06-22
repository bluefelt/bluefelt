#[cfg(test)]
mod test_move_entity {
    use serde_json::json;
    use bluefelt_core::engine::verbs::apply_move_entity;

    #[test]
    fn test_move_entity_generates_patches() {
        let mut state = json!({
            "zones": {
                "board": {
                    "type": "grid",
                    "cells": [
                        [{"entity": "piece_p1"}, null, null],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            }
        });

        let args = json!({
            "from": "/zones/board/cells/0/0",
            "to": "/zones/board/cells/1/0"
        });

        let result = apply_move_entity(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        println!("Generated patches: {:?}", patches);
        
        // Should generate 2 patches
        assert_eq!(patches.len(), 2, "Expected 2 patches (remove and add)");
        
        // First patch should remove from source
        assert_eq!(patches[0]["op"], "replace");
        assert_eq!(patches[0]["path"], "/zones/board/cells/0/0");
        assert!(patches[0]["value"].is_null());
        
        // Second patch should add to destination
        assert_eq!(patches[1]["op"], "replace");
        assert_eq!(patches[1]["path"], "/zones/board/cells/1/0");
        assert_eq!(patches[1]["value"]["entity"], "piece_p1");
        
        // Check state was updated
        assert!(state["zones"]["board"]["cells"][0][0].is_null());
        assert_eq!(state["zones"]["board"]["cells"][1][0]["entity"], "piece_p1");
    }
}
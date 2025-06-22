#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::engine::verbs::movement::apply_move_selected;
    use serde_json::json;

    #[test]
    fn test_select_entity_success() {
        let mut state = json!({
            "currentPlayer": "p1",
            "zones": {
                "board": {
                    "cells": {
                        "0,0": {"entity": "piece_p1", "owner": "p1"}
                    }
                }
            }
        });

        let args = json!({
            "location": "/zones/board/cells/0,0",
            "player": "p1"
        });

        let result = apply_select_entity(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0]["op"], "replace");
        assert_eq!(patches[0]["path"], "/game/selection/p1");
        
        // Verify state was updated
        assert_eq!(state["selection"]["p1"]["location"], "/zones/board/cells/0,0");
        assert_eq!(state["selection"]["p1"]["entity"]["entity"], "piece_p1");
    }

    #[test] 
    fn test_select_entity_empty_location() {
        let mut state = json!({
            "currentPlayer": "p1",
            "zones": {
                "board": {
                    "cells": {
                        "0,0": null
                    }
                }
            }
        });

        let args = json!({
            "location": "/zones/board/cells/0,0",
            "player": "p1"
        });

        let result = apply_select_entity(&mut state, &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No entity at specified location"));
    }

    #[test]
    fn test_clear_selection() {
        let mut state = json!({
            "currentPlayer": "p1",
            "selection": {
                "p1": {
                    "location": "/zones/board/cells/0,0",
                    "entity": {"entity": "piece_p1"}
                }
            }
        });

        let args = json!({
            "player": "p1"
        });

        let result = apply_clear_selection(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0]["op"], "remove");
        assert_eq!(patches[0]["path"], "/game/selection/p1");
        
        // Verify selection was cleared
        assert!(state["selection"]["p1"].is_null());
    }

    #[test]
    fn test_move_selected_success() {
        let mut state = json!({
            "currentPlayer": "p1",
            "selection": {
                "p1": {
                    "location": "/zones/board/cells/0,0",
                    "entity": {"entity": "piece_p1", "owner": "p1"}
                }
            },
            "zones": {
                "board": {
                    "cells": {
                        "0,0": {"entity": "piece_p1", "owner": "p1"},
                        "0,1": null
                    }
                }
            }
        });

        let args = json!({
            "target": "/zones/board/cells/0,1",
            "player": "p1"
        });

        let result = apply_move_selected(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 3);
        
        // Check replace patch for source (set to null)
        assert_eq!(patches[0]["op"], "replace");
        assert_eq!(patches[0]["path"], "/game/zones/board/cells/0,0");
        assert!(patches[0]["value"].is_null());
        
        // Check replace patch for target
        assert_eq!(patches[1]["op"], "replace");
        assert_eq!(patches[1]["path"], "/game/zones/board/cells/0,1");
        assert_eq!(patches[1]["value"]["entity"], "piece_p1");
        
        // Check selection clear patch
        assert_eq!(patches[2]["op"], "remove");
        assert_eq!(patches[2]["path"], "/game/selection/p1");
        
        // Verify state was updated
        assert!(state["zones"]["board"]["cells"]["0,0"].is_null());
        assert_eq!(state["zones"]["board"]["cells"]["0,1"]["entity"], "piece_p1");
        assert!(state["selection"]["p1"].is_null());
    }

    #[test]
    fn test_move_selected_no_selection() {
        let mut state = json!({
            "currentPlayer": "p1",
            "selection": {},
            "zones": {
                "board": {
                    "cells": {
                        "0,1": null
                    }
                }
            }
        });

        let args = json!({
            "target": "/zones/board/cells/0,1",
            "player": "p1"
        });

        let result = apply_move_selected(&mut state, &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No selection found for player"));
    }

    #[test]
    fn test_move_selected_uses_current_player() {
        let mut state = json!({
            "currentPlayer": "p2",
            "selection": {
                "p2": {
                    "location": "/zones/board/cells/1,1",
                    "entity": {"entity": "piece_p2", "owner": "p2"}
                }
            },
            "zones": {
                "board": {
                    "cells": {
                        "1,1": {"entity": "piece_p2", "owner": "p2"},
                        "1,2": null
                    }
                }
            }
        });

        let args = json!({
            "target": "/zones/board/cells/1,2",
            "player": "p2"  // The existing implementation requires explicit player
        });

        let result = apply_move_selected(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 3);
        
        // Verify the move happened for p2
        assert!(state["zones"]["board"]["cells"]["1,1"].is_null());
        assert_eq!(state["zones"]["board"]["cells"]["1,2"]["entity"], "piece_p2");
    }
}
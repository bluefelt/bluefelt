use bluefelt_core::{Bundle, load_initial_state, apply_verb};
use serde_json::{json, Value};
use std::path::Path;

/// Create a minimal tic-tac-toe bundle for testing
fn create_test_ttt_bundle() -> Bundle {
    let manifest = json!({
        "gameId": "tic-tac-toe",
        "version": "1.0",
        "specVersion": "1.0",
        "metadata": {
            "name": "Tic Tac Toe",
            "author": "Test",
            "description": "Test game",
            "players": {
                "min": 2,
                "max": 2
            }
        }
    });

    let entities = json!([
        {
            "id": "x_token",
            "type": "token",
            "props": {
                "player": "p1"
            },
            "ui": {
                "tokenType": "token_p1"
            }
        },
        {
            "id": "o_token", 
            "type": "token",
            "props": {
                "player": "p2"
            },
            "ui": {
                "tokenType": "token_p2"
            }
        }
    ]);

    let zones = json!([
        {
            "id": "board",
            "type": "grid",
            "rows": 3,
            "cols": 3,
            "contents": "empty"
        }
    ]);

    let actions = json!([
        {
            "id": "place",
            "uses": "place",
            "ui": {
                "direction": "Choose a cell"
            }
        }
    ]);

    let phases = json!([]);

    Bundle {
        game_id: "tic-tac-toe".to_string(),
        manifest: serde_json::from_value(manifest).unwrap(),
        entities,
        zones,
        actions,
        phases,
        hooks: None,
    }
}

#[tokio::test]
async fn test_load_initial_state_ttt() {
    let bundle = create_test_ttt_bundle();
    let state = load_initial_state(&bundle);

    // Check game state structure
    assert_eq!(state["tick"], 0);
    assert_eq!(state["turn"], 0);
    assert_eq!(state["currentPlayer"], "p1");
    assert_eq!(state["gameStatus"]["state"], "playing");
    
    // Check players
    let players = state["players"].as_array().unwrap();
    assert_eq!(players.len(), 2);
    assert_eq!(players[0]["id"], "p1");
    assert_eq!(players[1]["id"], "p2");

    // Check board zone
    let board = &state["zones"]["board"];
    assert_eq!(board["type"], "grid");
    let cells = board["cells"].as_array().unwrap();
    assert_eq!(cells.len(), 3);
    
    // All cells should be null initially
    for row in cells {
        let row_array = row.as_array().unwrap();
        assert_eq!(row_array.len(), 3);
        for cell in row_array {
            assert!(cell.is_null());
        }
    }
}

#[tokio::test] 
async fn test_apply_place_verb_ttt() {
    let bundle = create_test_ttt_bundle();
    let mut state = load_initial_state(&bundle);

    // Place X token at (0,0)
    let args = json!({
        "location": "/zones/board/cells/0/0",
        "entity": "x_token"
    });

    let result = apply_verb(&mut state, "place", &args, &bundle);
    assert!(result.is_ok());

    let patches = result.unwrap();
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0]["op"], "replace");
    assert_eq!(patches[0]["path"], "/game/zones/board/cells/0/0");

    // Check that the cell was updated
    let cell = &state["zones"]["board"]["cells"][0][0];
    assert_eq!(cell["entity"], "x_token");
}

#[tokio::test]
async fn test_apply_next_turn_verb_ttt() {
    let bundle = create_test_ttt_bundle();
    let mut state = load_initial_state(&bundle);

    let args = json!({});
    let result = apply_verb(&mut state, "nextTurn", &args, &bundle);
    assert!(result.is_ok());

    let patches = result.unwrap();
    assert_eq!(patches.len(), 3); // game/tick, game/turn, game/currentPlayer

    // Check state was updated
    assert_eq!(state["tick"], 1);
    assert_eq!(state["turn"], 1);
    assert_eq!(state["currentPlayer"], "p2");

    // Test wrapping back to player 1
    let result = apply_verb(&mut state, "nextTurn", &args, &bundle);
    assert!(result.is_ok());
    
    assert_eq!(state["turn"], 0);
    assert_eq!(state["currentPlayer"], "p1");
}

#[tokio::test]
async fn test_complete_ttt_game_sequence() {
    let bundle = create_test_ttt_bundle();
    let mut state = load_initial_state(&bundle);

    // Simulate a complete tic-tac-toe game
    let game_moves = vec![
        ("place", json!({"location": "/zones/board/cells/0/0", "entity": "x_token"})), // X
        ("nextTurn", json!({})),
        ("place", json!({"location": "/zones/board/cells/0/1", "entity": "o_token"})), // O  
        ("nextTurn", json!({})),
        ("place", json!({"location": "/zones/board/cells/1/1", "entity": "x_token"})), // X
        ("nextTurn", json!({})),
        ("place", json!({"location": "/zones/board/cells/0/2", "entity": "o_token"})), // O
        ("nextTurn", json!({})),
        ("place", json!({"location": "/zones/board/cells/2/2", "entity": "x_token"})), // X wins diagonally
    ];

    let mut tick_count = 0;
    for (verb, args) in game_moves {
        let result = apply_verb(&mut state, verb, &args, &bundle);
        assert!(result.is_ok(), "Failed to apply verb: {} with args: {}", verb, args);
        
        if verb == "nextTurn" {
            tick_count += 1;
            assert_eq!(state["tick"], tick_count);
        }
    }

    // Verify final board state
    let board = &state["zones"]["board"]["cells"];
    
    // Row 0: X O O
    assert_eq!(board[0][0]["entity"], "x_token");
    assert_eq!(board[0][1]["entity"], "o_token");
    assert_eq!(board[0][2]["entity"], "o_token");
    
    // Row 1: _ X _
    assert!(board[1][0].is_null());
    assert_eq!(board[1][1]["entity"], "x_token");
    assert!(board[1][2].is_null());
    
    // Row 2: _ _ X
    assert!(board[2][0].is_null());
    assert!(board[2][1].is_null());
    assert_eq!(board[2][2]["entity"], "x_token");

    // X should have won (diagonal: 0,0 -> 1,1 -> 2,2)
}

#[tokio::test]
async fn test_invalid_place_verb() {
    let bundle = create_test_ttt_bundle();
    let mut state = load_initial_state(&bundle);

    // Try to place at invalid location
    let args = json!({
        "location": "/zones/board/cells/5/5", // Out of bounds
        "entity": "x_token"
    });

    let result = apply_verb(&mut state, "place", &args, &bundle);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unknown_verb() {
    let bundle = create_test_ttt_bundle();
    let mut state = load_initial_state(&bundle);

    let args = json!({});
    let result = apply_verb(&mut state, "unknown_verb", &args, &bundle);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown verb"));
}

#[tokio::test]
async fn test_state_consistency_after_multiple_operations() {
    let bundle = create_test_ttt_bundle();
    let mut state = load_initial_state(&bundle);

    // Perform multiple operations and check state consistency
    for i in 0..3 {
        for j in 0..3 {
            let entity = if (i + j) % 2 == 0 { "x_token" } else { "o_token" };
            
            let place_args = json!({
                "location": format!("/zones/board/cells/{}/{}", i, j),
                "entity": entity
            });
            
            let result = apply_verb(&mut state, "place", &place_args, &bundle);
            assert!(result.is_ok());
            
            let next_turn_args = json!({});
            let result = apply_verb(&mut state, "nextTurn", &next_turn_args, &bundle);
            assert!(result.is_ok());
        }
    }

    // Verify all cells are filled
    let board = &state["zones"]["board"]["cells"];
    for i in 0..3 {
        for j in 0..3 {
            assert!(!board[i][j].is_null(), "Cell ({},{}) should not be null", i, j);
        }
    }

    // Verify final tick count
    assert_eq!(state["tick"], 9); // 9 nextTurn calls
}

/// Test bundle loading from actual files if available
#[tokio::test]
async fn test_load_real_ttt_bundle() {
    let bundle_path = Path::new("../bundles/tic-tac-toe/1.0");
    
    if bundle_path.exists() {
        // Try to load the real bundle
        if let Ok(manifest_content) = std::fs::read_to_string(bundle_path.join("manifest.json")) {
            let manifest: Value = serde_json::from_str(&manifest_content).unwrap();
            
            // Basic validation
            assert_eq!(manifest["gameId"], "tic-tac-toe");
            assert_eq!(manifest["version"], "1.0");
            assert!(manifest["metadata"]["name"].as_str().is_some());
        }
    }
}
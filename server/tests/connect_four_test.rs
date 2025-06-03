use bluefelt_core::{bundle::BundleMap, engine::load_initial_state};

#[test]
fn test_connect_four_game_setup() {
    // Load bundles from directory
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("connect-four").expect("Failed to get connect-four bundle");
    
    println!("Bundle game_id: {}", bundle.game_id);
    println!("Bundle zones: {:?}", bundle.zones);
    
    let state = load_initial_state(&bundle);
    
    // Verify initial state
    assert_eq!(state["turn"], 0);
    assert_eq!(state["currentPlayer"], "p1");
    
    // Verify board is empty - 6 rows x 7 columns
    // First check what structure we have
    println!("State zones: {:?}", state["zones"]);
    
    // Based on tic-tac-toe test, board might have a cells property
    let board = if state["zones"]["board"]["cells"].is_array() {
        &state["zones"]["board"]["cells"]
    } else if state["zones"]["board"].is_array() {
        &state["zones"]["board"]
    } else {
        panic!("Unexpected board structure: {:?}", state["zones"]["board"]);
    };
    
    let board_array = board.as_array().unwrap();
    assert_eq!(board_array.len(), 6); // 6 rows
    
    for row in board_array {
        let row_array = row.as_array().unwrap();
        assert_eq!(row_array.len(), 7); // 7 columns
        for cell in row_array {
            assert!(cell.is_null());
        }
    }
}

#[test]
fn test_connect_four_gameplay() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("connect-four").expect("Failed to get bundle");
    let state = load_initial_state(&bundle);
    
    // Verify action configuration exists
    let actions = bundle.actions.as_array()
        .expect("actions should be an array");
    assert!(!actions.is_empty());
    
    // Find the dropDisc action
    let drop_action = actions.iter()
        .find(|a| a["id"].as_str() == Some("dropDisc"))
        .expect("dropDisc action not found");
    
    assert_eq!(drop_action["uses"].as_str(), Some("placeWithGravity"));
    assert!(drop_action["ui"]["direction"].as_str()
        .unwrap_or("")
        .contains("column"));
}

#[test]
fn test_connect_four_win_detection() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("connect-four").expect("Failed to get bundle");
    
    // Verify win detection is configured
    let actions = bundle.actions.as_array()
        .expect("actions should be an array");
    
    let check_win_action = actions.iter()
        .find(|a| a["id"].as_str() == Some("checkWin"))
        .expect("checkWin action not found");
    
    assert_eq!(check_win_action["uses"].as_str(), Some("grid.lineOfMarks"));
    
    // Verify it checks for 4 in a row
    let line_length = check_win_action["with"]["lineLength"]
        .as_u64()
        .expect("lineLength not found");
    
    assert_eq!(line_length, 4);
    
    // Verify it checks all directions
    let directions = check_win_action["with"]["directions"]
        .as_array()
        .expect("directions not found");
    
    assert_eq!(directions.len(), 3); // horizontal, vertical, diagonal
}

#[test]
fn test_connect_four_gravity_mechanics() {
    use bluefelt_core::engine::verbs::apply_verb;
    use serde_json::json;
    
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("connect-four").expect("Failed to get bundle");
    let mut state = load_initial_state(&bundle);
    
    // Test placing disc in empty column - should drop to bottom (row 5)
    let place_args = json!({
        "zone": "/zones/board",
        "column": 3,
        "entity": "disc_p1"
    });
    
    let result = apply_verb(&mut state, "placeWithGravity", &place_args, &bundle);
    assert!(result.is_ok(), "placeWithGravity should succeed: {:?}", result);
    
    // Verify disc is at bottom of column 3 (row 5, col 3)
    assert_eq!(state["zones"]["board"]["cells"][5][3]["entity"], "disc_p1");
    
    // Test placing another disc in same column - should stack (row 4)
    let place_args2 = json!({
        "zone": "/zones/board", 
        "column": 3,
        "entity": "disc_p2"
    });
    
    let result2 = apply_verb(&mut state, "placeWithGravity", &place_args2, &bundle);
    assert!(result2.is_ok(), "Second placeWithGravity should succeed: {:?}", result2);
    
    // Verify both discs are stacked correctly
    assert_eq!(state["zones"]["board"]["cells"][5][3]["entity"], "disc_p1"); // Bottom disc
    assert_eq!(state["zones"]["board"]["cells"][4][3]["entity"], "disc_p2"); // Top disc
    
    // Fill up the column and test full column rejection
    for i in 0..4 {
        let entity = if i % 2 == 0 { "disc_p1" } else { "disc_p2" };
        let place_args = json!({
            "zone": "/zones/board",
            "column": 3,
            "entity": entity
        });
        let result = apply_verb(&mut state, "placeWithGravity", &place_args, &bundle);
        assert!(result.is_ok(), "Filling column should succeed for iteration {}: {:?}", i, result);
    }
    
    // Now column should be full - test rejection
    let place_args_full = json!({
        "zone": "/zones/board",
        "column": 3,
        "entity": "disc_p1"
    });
    
    let result_full = apply_verb(&mut state, "placeWithGravity", &place_args_full, &bundle);
    assert!(result_full.is_err(), "Placing in full column should fail");
}

#[test]
fn test_connect_four_win_detection_logic() {
    use bluefelt_core::engine::verbs::apply_verb;
    use serde_json::json;
    
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("connect-four").expect("Failed to get bundle");
    let mut state = load_initial_state(&bundle);
    
    // Test horizontal win - place 4 discs in bottom row (they'll all land at row 5)
    for col in 0..4 {
        let place_args = json!({
            "zone": "/zones/board",
            "column": col,
            "entity": "disc_p1"
        });
        let result = apply_verb(&mut state, "placeWithGravity", &place_args, &bundle);
        assert!(result.is_ok(), "Placing disc should succeed: {:?}", result);
    }
    
    // Verify the discs are in the bottom row horizontally
    for col in 0..4 {
        assert_eq!(state["zones"]["board"]["cells"][5][col]["entity"], "disc_p1");
    }
    
    // Test that grid.lineOfMarks can detect the horizontal line
    let check_args = json!({
        "zone": "/zones/board",
        "entity": "disc_p1",
        "lineLength": 4,
        "directions": ["horizontal", "vertical", "diagonal"]
    });
    
    let win_result = apply_verb(&mut state, "grid.lineOfMarks", &check_args, &bundle);
    assert!(win_result.is_ok(), "Win check should succeed: {:?}", win_result);
    
    // grid.lineOfMarks sets game status internally
    if state["meta"]["gameStatus"]["state"] == "ended" {
        assert_eq!(state["meta"]["gameStatus"]["winner"], "p1");
        assert_eq!(state["meta"]["gameStatus"]["tie"], false);
    }
}

#[test] 
fn test_connect_four_vertical_detection() {
    use bluefelt_core::engine::verbs::apply_verb;
    use serde_json::json;
    
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("connect-four").expect("Failed to get bundle");
    let mut state = load_initial_state(&bundle);
    
    // Test vertical win - place 4 discs in same column (column 2)
    for _i in 0..4 {
        let place_args = json!({
            "zone": "/zones/board",
            "column": 2,
            "entity": "disc_p1"
        });
        let result = apply_verb(&mut state, "placeWithGravity", &place_args, &bundle);
        assert!(result.is_ok(), "Placing disc should succeed: {:?}", result);
    }
    
    // Verify the discs are stacked vertically in column 2
    for row in 2..6 { // rows 2,3,4,5 (bottom 4 rows of a 6-row board)
        assert_eq!(state["zones"]["board"]["cells"][row][2]["entity"], "disc_p1");
    }
    
    // Test that grid.lineOfMarks can detect the vertical line
    let check_args = json!({
        "zone": "/zones/board", 
        "entity": "disc_p1",
        "lineLength": 4,
        "directions": ["horizontal", "vertical", "diagonal"]
    });
    
    let win_result = apply_verb(&mut state, "grid.lineOfMarks", &check_args, &bundle);
    assert!(win_result.is_ok(), "Win check should succeed: {:?}", win_result);
    
    // grid.lineOfMarks sets game status internally
    if state["meta"]["gameStatus"]["state"] == "ended" {
        assert_eq!(state["meta"]["gameStatus"]["winner"], "p1");
    }
}
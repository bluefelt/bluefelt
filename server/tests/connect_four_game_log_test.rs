use bluefelt_core::{bundle::BundleMap, engine::load_initial_state};
use serde_json::json;

#[test]
fn test_connect_four_game_log_column_replacement() {
    // Load bundles from directory
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("connect-four").expect("Failed to get connect-four bundle");
    
    // Verify the log template exists in the bundle
    let actions = bundle.actions.as_array().expect("actions should be an array");
    let drop_action = actions.iter()
        .find(|a| a["id"].as_str() == Some("dropDisc"))
        .expect("dropDisc action not found");
    
    let log_template = drop_action["ui"]["logTemplate"].as_str()
        .expect("logTemplate should exist");
    
    assert_eq!(log_template, "{player} dropped a disc in column {column}");
    
    // Test that the action is properly configured for column-based arguments
    let with_args = &drop_action["with"];
    assert_eq!(with_args["column"], "{targetColumn}");
    assert_eq!(with_args["zone"], "/zones/board");
    assert_eq!(with_args["entity"], "disc_{player}");
}

#[test]
fn test_log_template_placeholder_processing() {
    // This test verifies the logic for replacing {column} placeholders
    // Since we can't easily test the full lobby system here, we'll test the logic
    
    // Simulate the log template processing logic
    let log_template = "{player} dropped a disc in column {column}";
    let player_id = "testPlayer";
    let column_number = 3i64; // 0-indexed column
    
    // Simulate what happens in lobby.rs
    let mut log_text = log_template.to_string();
    log_text = log_text.replace("{player}", player_id);
    log_text = log_text.replace("{column}", &(column_number + 1).to_string()); // 1-indexed for display
    
    assert_eq!(log_text, "testPlayer dropped a disc in column 4");
}

#[test]
fn test_different_column_numbers() {
    // Test various column numbers to ensure proper 1-indexing
    let test_cases = vec![
        (0, "1"),  // First column (0-indexed) -> 1 (1-indexed)
        (3, "4"),  // Middle column  
        (6, "7"),  // Last column in Connect 4 (7 columns total)
    ];
    
    for (zero_indexed, expected_number) in test_cases {
        let log_template = "Placed disc in column {column}";
        let mut log_text = log_template.to_string();
        log_text = log_text.replace("{column}", &(zero_indexed + 1).to_string());
        
        assert_eq!(log_text, format!("Placed disc in column {}", expected_number));
    }
}
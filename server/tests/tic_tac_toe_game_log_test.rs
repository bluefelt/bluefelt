use bluefelt_core::bundle::BundleMap;

#[test]
fn test_tic_tac_toe_game_log_coordinates() {
    // Load bundles from directory
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("tic-tac-toe").expect("Failed to get tic-tac-toe bundle");
    
    // Verify the updated log template exists in the bundle
    let actions = bundle.actions.as_array().expect("actions should be an array");
    let place_action = actions.iter()
        .find(|a| a["id"].as_str() == Some("placeMarker"))
        .expect("placeMarker action not found");
    
    let log_template = place_action["ui"]["logTemplate"].as_str()
        .expect("logTemplate should exist");
    
    // Verify it now includes coordinate placeholders
    assert_eq!(log_template, "{player} placed their mark at ({row}, {col})");
}

#[test]
fn test_tic_tac_toe_log_template_processing() {
    // Test the log template processing logic for tic-tac-toe coordinates
    let log_template = "{player} placed their mark at ({row}, {col})";
    let player_id = "testPlayer";
    let row = 1i64; // 0-indexed
    let col = 2i64; // 0-indexed
    
    // Simulate what happens in lobby.rs
    let mut log_text = log_template.to_string();
    log_text = log_text.replace("{player}", player_id);
    log_text = log_text.replace("{row}", &(row + 1).to_string()); // 1-indexed for display
    log_text = log_text.replace("{col}", &(col + 1).to_string()); // 1-indexed for display
    
    assert_eq!(log_text, "testPlayer placed their mark at (2, 3)");
}

#[test]
fn test_tic_tac_toe_coordinate_examples() {
    // Test various tic-tac-toe board positions
    let test_cases = vec![
        ((0, 0), "(1, 1)"), // Top-left corner
        ((1, 1), "(2, 2)"), // Center
        ((2, 2), "(3, 3)"), // Bottom-right corner
        ((0, 2), "(1, 3)"), // Top-right corner
        ((2, 0), "(3, 1)"), // Bottom-left corner
    ];
    
    for ((zero_row, zero_col), expected_coords) in test_cases {
        let log_template = "Mark at ({row}, {col})";
        let mut log_text = log_template.to_string();
        log_text = log_text.replace("{row}", &(zero_row + 1).to_string());
        log_text = log_text.replace("{col}", &(zero_col + 1).to_string());
        
        assert_eq!(log_text, format!("Mark at {}", expected_coords));
    }
}
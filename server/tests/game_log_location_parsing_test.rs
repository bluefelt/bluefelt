use regex::Regex;

#[test]
fn test_location_path_parsing() {
    // Test the regex pattern for extracting row/col from location paths
    let regex = Regex::new(r"/zones/[^/]+/cells/(\d+)/(\d+)").unwrap();
    
    let test_cases = vec![
        ("/zones/board/cells/0/0", Some((0, 0))),
        ("/zones/board/cells/1/2", Some((1, 2))),
        ("/zones/board/cells/2/1", Some((2, 1))),
        ("/zones/da-board/cells/0/2", Some((0, 2))), // Different zone name
        ("/zones/board/0/0", None), // Missing "cells"
        ("/zones/board/cells/a/b", None), // Non-numeric
        ("invalid", None),
    ];
    
    for (location, expected) in test_cases {
        let result = regex.captures(location)
            .and_then(|captures| {
                let row_str = captures.get(1)?.as_str();
                let col_str = captures.get(2)?.as_str();
                let row = row_str.parse::<i64>().ok()?;
                let col = col_str.parse::<i64>().ok()?;
                Some((row, col))
            });
        
        assert_eq!(result, expected, "Failed for location: {}", location);
    }
}

#[test]
fn test_log_template_with_location_parsing() {
    // Simulate the complete log template processing for tic-tac-toe
    let log_template = "{player} placed their mark at ({row}, {col})";
    let player_id = "testPlayer";
    let location = "/zones/board/cells/1/2"; // 0-indexed
    
    // Simulate what happens in lobby.rs
    let mut log_text = log_template.to_string();
    log_text = log_text.replace("{player}", player_id);
    
    // Extract row and col from location
    let regex = Regex::new(r"/zones/[^/]+/cells/(\d+)/(\d+)").unwrap();
    if let Some(captures) = regex.captures(location) {
        if let (Some(row_match), Some(col_match)) = (captures.get(1), captures.get(2)) {
            if let (Ok(row), Ok(col)) = (row_match.as_str().parse::<i64>(), col_match.as_str().parse::<i64>()) {
                log_text = log_text.replace("{row}", &(row + 1).to_string()); // 1-indexed for display
                log_text = log_text.replace("{col}", &(col + 1).to_string()); // 1-indexed for display
            }
        }
    }
    
    assert_eq!(log_text, "testPlayer placed their mark at (2, 3)");
}

#[test]
fn test_tic_tac_toe_board_positions() {
    // Test all 9 positions on a tic-tac-toe board
    let positions = vec![
        // Top row
        ("/zones/board/cells/0/0", "(1, 1)"),
        ("/zones/board/cells/0/1", "(1, 2)"),
        ("/zones/board/cells/0/2", "(1, 3)"),
        // Middle row
        ("/zones/board/cells/1/0", "(2, 1)"),
        ("/zones/board/cells/1/1", "(2, 2)"),
        ("/zones/board/cells/1/2", "(2, 3)"),
        // Bottom row
        ("/zones/board/cells/2/0", "(3, 1)"),
        ("/zones/board/cells/2/1", "(3, 2)"),
        ("/zones/board/cells/2/2", "(3, 3)"),
    ];
    
    let regex = Regex::new(r"/zones/[^/]+/cells/(\d+)/(\d+)").unwrap();
    
    for (location, expected_coords) in positions {
        let mut result = "at ({row}, {col})".to_string();
        
        if let Some(captures) = regex.captures(location) {
            if let (Some(row_match), Some(col_match)) = (captures.get(1), captures.get(2)) {
                if let (Ok(row), Ok(col)) = (row_match.as_str().parse::<i64>(), col_match.as_str().parse::<i64>()) {
                    result = result.replace("{row}", &(row + 1).to_string());
                    result = result.replace("{col}", &(col + 1).to_string());
                }
            }
        }
        
        assert_eq!(result, format!("at {}", expected_coords), "Failed for {}", location);
    }
}
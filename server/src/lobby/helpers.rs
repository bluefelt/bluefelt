//! Helper functions for lobby operations

use serde_json;

/// Helper function to check if placing a piece at (row, col) would flip any opponent pieces
pub fn would_flip_any(board: &serde_json::Value, row: usize, col: usize, player_piece: &str) -> bool {
    let directions = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1)
    ];
    
    let board_array = match board.as_array() {
        Some(arr) => arr,
        None => return false,
    };
    
    let board_size = board_array.len();
    let opponent_piece = if player_piece.contains("_p1") {
        player_piece.replace("_p1", "_p2")
    } else {
        player_piece.replace("_p2", "_p1")
    };
    
    for (dr, dc) in directions.iter() {
        let mut r = row as i32 + dr;
        let mut c = col as i32 + dc;
        let mut found_opponent = false;
        
        while r >= 0 && r < board_size as i32 && c >= 0 && c < board_size as i32 {
            let row_idx = r as usize;
            let col_idx = c as usize;
            
            if let Some(row_array) = board_array[row_idx].as_array() {
                if col_idx < row_array.len() {
                    match row_array[col_idx].as_str() {
                        Some(piece) if piece == opponent_piece => {
                            found_opponent = true;
                        }
                        Some(piece) if piece == player_piece => {
                            if found_opponent {
                                return true; // Would flip at least one piece
                            }
                            break;
                        }
                        _ => break, // Empty cell or edge
                    }
                }
            }
            
            r += dr;
            c += dc;
        }
    }
    
    false
}

/// Helper function to get a value at a specific path in the JSON state
pub fn get_value_at_path<'a>(state: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in path_parts {
        if let Ok(index) = part.parse::<usize>() {
            // This is an array index
            if let Some(array) = current.as_array() {
                if index < array.len() {
                    current = &array[index];
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            // This is an object key
            current = current.get(part)?;
        }
    }
    
    Some(current)
}
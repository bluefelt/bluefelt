use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn check_game_end(state_json: &str) -> String {
    let state: Value = match serde_json::from_str(state_json) {
        Ok(v) => v,
        Err(_) => return json!([]).to_string(),
    };

    let mut patches = vec![];

    // Get the board state
    let board = match state["zones"]["board"].as_array() {
        Some(b) => b,
        None => return json!([]).to_string(),
    };

    // Check for winner
    if let Some(winner) = check_winner(&board) {
        patches.push(json!({
            "op": "add",
            "path": "/meta/gameStatus",
            "value": {
                "state": "ended",
                "winner": winner,
                "tie": false
            }
        }));
        return serde_json::to_string(&patches).unwrap_or_else(|_| "[]".to_string());
    }

    // Check for tie (board full)
    let is_full = board.iter().all(|row| {
        row.as_array()
            .map(|r| r.iter().all(|cell| !cell.is_null()))
            .unwrap_or(false)
    });

    if is_full {
        patches.push(json!({
            "op": "add",
            "path": "/meta/gameStatus",
            "value": {
                "state": "ended",
                "tie": true
            }
        }));
    }

    serde_json::to_string(&patches).unwrap_or_else(|_| "[]".to_string())
}

fn check_winner(board: &[Value]) -> Option<String> {
    // Convert board to a more manageable format
    let mut cells: Vec<Vec<Option<String>>> = vec![vec![None; 3]; 3];
    for (r, row) in board.iter().enumerate() {
        if let Some(row_array) = row.as_array() {
            for (c, cell) in row_array.iter().enumerate() {
                if r < 3 && c < 3 {
                    cells[r][c] = cell.as_str().map(|s| s.to_string());
                }
            }
        }
    }

    // Check rows
    for r in 0..3 {
        if let Some(winner) = check_line(&cells[r][0], &cells[r][1], &cells[r][2]) {
            return Some(winner);
        }
    }

    // Check columns
    for col in 0..3 {
        if let Some(winner) = check_line(&cells[0][col], &cells[1][col], &cells[2][col]) {
            return Some(winner);
        }
    }

    // Check diagonals
    if let Some(winner) = check_line(&cells[0][0], &cells[1][1], &cells[2][2]) {
        return Some(winner);
    }
    if let Some(winner) = check_line(&cells[0][2], &cells[1][1], &cells[2][0]) {
        return Some(winner);
    }

    None
}

fn check_line(a: &Option<String>, b: &Option<String>, c: &Option<String>) -> Option<String> {
    match (a, b, c) {
        (Some(mark_a), Some(mark_b), Some(mark_c)) if mark_a == mark_b && mark_b == mark_c => {
            // Convert mark to player ID (mark_x -> p1, mark_o -> p2)
            if mark_a == "mark_x" {
                Some("p1".to_string())
            } else if mark_a == "mark_o" {
                Some("p2".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

// Export the function for the game engine to call
#[no_mangle]
pub extern "C" fn checkGameEnd(state_ptr: *const u8, state_len: usize) -> *mut u8 {
    let state_slice = unsafe { std::slice::from_raw_parts(state_ptr, state_len) };
    let state_str = match std::str::from_utf8(state_slice) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let result = check_game_end(state_str);
    let result_bytes = result.into_bytes();
    let result_len = result_bytes.len();
    
    // Allocate memory for the result
    let result_ptr = result_bytes.as_ptr();
    std::mem::forget(result_bytes);
    
    // Return pointer with length encoded in first 4 bytes
    let mut output = Vec::with_capacity(4 + result_len);
    output.extend_from_slice(&(result_len as u32).to_le_bytes());
    output.extend_from_slice(unsafe { std::slice::from_raw_parts(result_ptr, result_len) });
    
    let output_ptr = output.as_ptr();
    std::mem::forget(output);
    output_ptr as *mut u8
}
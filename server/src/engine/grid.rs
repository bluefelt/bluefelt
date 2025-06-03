use serde_json::{json, Value};
use crate::engine::path::get_zone_ref;

const DEFAULT_LINE_LENGTH: u64 = 3;

pub fn apply_check_for_win(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args["zone"].as_str().ok_or("Missing 'zone' path")?;
    let entity_pattern = args["entity"].as_str().ok_or("Missing 'entity' pattern")?;
    let line_length = args["lineLength"].as_u64().unwrap_or(DEFAULT_LINE_LENGTH) as usize;
    let directions = args["directions"].as_array().ok_or("Missing 'directions' array")?;
    
    let zone = get_zone_ref(state, zone_path)?;
    let cells = zone["cells"].as_array()
        .ok_or("Zone is not a grid with cells")?;
    
    if cells.is_empty() {
        return Err("Grid has no rows".to_string());
    }
    
    let dimensions = get_grid_dimensions(cells)?;
    
    // Check for winning lines in each enabled direction
    for direction in directions {
        let dir_str = direction.as_str()
            .ok_or("Direction must be a string")?;
        
        if let Some(winner) = check_direction(cells, entity_pattern, line_length, &dimensions, dir_str)? {
            return set_game_winner(state, &winner);
        }
    }
    
    // Check for tie (board full with no winner)
    if is_board_full(cells, &dimensions)? {
        return set_game_tie(state);
    }
    
    // No winner and board not full - game continues
    Ok(vec![])
}

struct GridDimensions {
    rows: usize,
    cols: usize,
}

fn get_grid_dimensions(cells: &[Value]) -> Result<GridDimensions, String> {
    let rows = cells.len();
    let cols = cells[0].as_array()
        .ok_or("Grid row is not an array")?
        .len();
    Ok(GridDimensions { rows, cols })
}

fn check_direction(
    cells: &[Value],
    entity_pattern: &str,
    line_length: usize,
    dimensions: &GridDimensions,
    direction: &str,
) -> Result<Option<String>, String> {
    match direction {
        "horizontal" => check_horizontal_lines(cells, entity_pattern, line_length, dimensions),
        "vertical" => check_vertical_lines(cells, entity_pattern, line_length, dimensions),
        "diagonal" => check_diagonal_lines(cells, entity_pattern, line_length, dimensions),
        _ => Err(format!("Unknown direction: {}", direction))
    }
}

fn check_horizontal_lines(
    cells: &[Value],
    entity_pattern: &str,
    line_length: usize,
    dimensions: &GridDimensions,
) -> Result<Option<String>, String> {
    for row in 0..dimensions.rows {
        for start_col in 0..=(dimensions.cols.saturating_sub(line_length)) {
            if let Some(winner) = check_line_at_position(
                cells, entity_pattern, line_length, row, start_col, 0, 1
            )? {
                return Ok(Some(winner));
            }
        }
    }
    Ok(None)
}

fn check_vertical_lines(
    cells: &[Value],
    entity_pattern: &str,
    line_length: usize,
    dimensions: &GridDimensions,
) -> Result<Option<String>, String> {
    for col in 0..dimensions.cols {
        for start_row in 0..=(dimensions.rows.saturating_sub(line_length)) {
            if let Some(winner) = check_line_at_position(
                cells, entity_pattern, line_length, start_row, col, 1, 0
            )? {
                return Ok(Some(winner));
            }
        }
    }
    Ok(None)
}

fn check_diagonal_lines(
    cells: &[Value],
    entity_pattern: &str,
    line_length: usize,
    dimensions: &GridDimensions,
) -> Result<Option<String>, String> {
    // Check main diagonal (top-left to bottom-right)
    for row in 0..=(dimensions.rows.saturating_sub(line_length)) {
        for col in 0..=(dimensions.cols.saturating_sub(line_length)) {
            if let Some(winner) = check_line_at_position(
                cells, entity_pattern, line_length, row, col, 1, 1
            )? {
                return Ok(Some(winner));
            }
        }
    }
    
    // Check anti-diagonal (top-right to bottom-left)
    for row in 0..=(dimensions.rows.saturating_sub(line_length)) {
        for col in (line_length - 1)..dimensions.cols {
            if let Some(winner) = check_line_at_position(
                cells, entity_pattern, line_length, row, col, 1, -1
            )? {
                return Ok(Some(winner));
            }
        }
    }
    
    Ok(None)
}

fn check_line_at_position(
    cells: &[Value], 
    entity_pattern: &str, 
    line_length: usize, 
    start_row: usize, 
    start_col: usize, 
    row_delta: i32, 
    col_delta: i32
) -> Result<Option<String>, String> {
    let mut first_entity: Option<String> = None;
    
    for i in 0..line_length {
        let row = (start_row as i32 + (i as i32 * row_delta)) as usize;
        let col = (start_col as i32 + (i as i32 * col_delta)) as usize;
        
        let cell = cells.get(row)
            .and_then(|r| r.as_array())
            .and_then(|r| r.get(col))
            .ok_or("Cell position out of bounds")?;
        
        let entity_id = extract_entity_id(cell);
        
        if let Some(id) = entity_id {
            if matches_pattern(&id, entity_pattern) {
                match &first_entity {
                    Some(first) if first != &id => return Ok(None), // Different entities
                    None => first_entity = Some(id),
                    _ => {} // Same entity, continue
                }
            } else {
                return Ok(None); // Entity doesn't match pattern
            }
        } else {
            return Ok(None); // Empty cell
        }
    }
    
    first_entity
        .map(|entity| extract_player_from_entity(&entity))
        .transpose()
        .map(|opt| opt.flatten())
}

fn extract_entity_id(cell: &Value) -> Option<String> {
    cell.as_object()
        .and_then(|obj| obj.get("entity"))
        .and_then(|e| e.as_str())
        .map(|s| s.to_string())
}

fn matches_pattern(entity_id: &str, pattern: &str) -> bool {
    if pattern.contains("{player}") {
        let base_pattern = pattern.replace("{player}", "");
        entity_id.starts_with(&base_pattern)
    } else {
        entity_id == pattern
    }
}

fn extract_player_from_entity(entity_id: &str) -> Result<Option<String>, String> {
    if let Some(pos) = entity_id.rfind("_p") {
        let player_part = &entity_id[pos + 1..];
        Ok(Some(player_part.to_string()))
    } else {
        Ok(None)
    }
}

fn is_board_full(cells: &[Value], dimensions: &GridDimensions) -> Result<bool, String> {
    for row in 0..dimensions.rows {
        for col in 0..dimensions.cols {
            let cell = cells.get(row)
                .and_then(|r| r.as_array())
                .and_then(|r| r.get(col))
                .ok_or("Cell position out of bounds")?;
            
            if extract_entity_id(cell).is_none() {
                return Ok(false); // Found empty cell
            }
        }
    }
    Ok(true) // All cells filled
}

fn set_game_winner(state: &mut Value, winner: &str) -> Result<Vec<Value>, String> {
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    
    let game_status = json!({
        "state": "ended",
        "winner": winner,
        "tie": false
    });
    
    state_obj.insert("gameStatus".to_string(), game_status.clone());
    
    Ok(vec![json!({
        "op": "replace",
        "path": "/game/gameStatus",
        "value": game_status
    })])
}

fn set_game_tie(state: &mut Value) -> Result<Vec<Value>, String> {
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    
    let game_status = json!({
        "state": "ended",
        "winner": null,
        "tie": true
    });
    
    state_obj.insert("gameStatus".to_string(), game_status.clone());
    
    Ok(vec![json!({
        "op": "replace",
        "path": "/game/gameStatus",
        "value": game_status
    })])}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_grid_line_of_marks_horizontal() {
        let mut state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [{"entity": "mark_p1"}, {"entity": "mark_p1"}, {"entity": "mark_p1"}],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            },
            "gameStatus": {
                "state": "playing",
                "winner": null,
                "tie": false
            }
        });

        let args = json!({
            "zone": "zones/board",
            "entity": "mark_{player}",
            "lineLength": 3,
            "directions": ["horizontal"]
        });

        let result = apply_check_for_win(&mut state, &args).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(state["gameStatus"]["state"], "ended");
        assert_eq!(state["gameStatus"]["winner"], "p1");
        assert_eq!(state["gameStatus"]["tie"], false);
    }

    #[test]
    fn test_grid_line_of_marks_vertical() {
        let mut state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [{"entity": "mark_p2"}, null, null],
                        [{"entity": "mark_p2"}, null, null],
                        [{"entity": "mark_p2"}, null, null]
                    ]
                }
            },
            "gameStatus": {
                "state": "playing",
                "winner": null,
                "tie": false
            }
        });

        let args = json!({
            "zone": "zones/board",
            "entity": "mark_{player}",
            "lineLength": 3,
            "directions": ["vertical"]
        });

        let result = apply_check_for_win(&mut state, &args).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(state["gameStatus"]["winner"], "p2");
    }

    #[test]
    fn test_grid_line_of_marks_diagonal() {
        let mut state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [{"entity": "mark_p1"}, null, null],
                        [null, {"entity": "mark_p1"}, null],
                        [null, null, {"entity": "mark_p1"}]
                    ]
                }
            },
            "gameStatus": {
                "state": "playing",
                "winner": null,
                "tie": false
            }
        });

        let args = json!({
            "zone": "zones/board",
            "entity": "mark_{player}",
            "lineLength": 3,
            "directions": ["diagonal"]
        });

        let result = apply_check_for_win(&mut state, &args).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(state["gameStatus"]["winner"], "p1");
    }

    #[test]
    fn test_grid_line_of_marks_tie() {
        let mut state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [{"entity": "mark_p1"}, {"entity": "mark_p2"}, {"entity": "mark_p1"}],
                        [{"entity": "mark_p2"}, {"entity": "mark_p1"}, {"entity": "mark_p2"}],
                        [{"entity": "mark_p2"}, {"entity": "mark_p1"}, {"entity": "mark_p2"}]
                    ]
                }
            },
            "gameStatus": {
                "state": "playing",
                "winner": null,
                "tie": false
            }
        });

        let args = json!({
            "zone": "zones/board",
            "entity": "mark_{player}",
            "lineLength": 3,
            "directions": ["horizontal", "vertical", "diagonal"]
        });

        let result = apply_check_for_win(&mut state, &args).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(state["gameStatus"]["state"], "ended");
        assert_eq!(state["gameStatus"]["winner"], Value::Null);
        assert_eq!(state["gameStatus"]["tie"], true);
    }

    #[test]
    fn test_grid_line_of_marks_no_winner() {
        let mut state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [{"entity": "mark_p1"}, null, null],
                        [null, {"entity": "mark_p2"}, null],
                        [null, null, null]
                    ]
                }
            },
            "meta": {}
        });

        let args = json!({
            "zone": "zones/board",
            "entity": "mark_{player}",
            "lineLength": 3,
            "directions": ["horizontal", "vertical", "diagonal"]
        });

        let result = apply_check_for_win(&mut state, &args).unwrap();
        
        assert_eq!(result.len(), 0);
        assert!(state["meta"]["gameStatus"].is_null());
    }
}
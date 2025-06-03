use crate::bundle::Bundle;
use serde_json::{json, Value, Map};
use rand::seq::SliceRandom;
use rand::rng;

/* --------------------------------------------------------------------------
   Load initial state from bundle
   ----------------------------------------------------------------------- */
pub fn load_initial_state(bundle: &Bundle) -> Value {
    let player_count = bundle
        .manifest
        .metadata
        .players
        .max;
    let mut players = Vec::new();
    for i in 1..=player_count {
        players.push(json!({"id": format!("p{}", i)}));
    }

    let mut zones = Map::new();
    if let Some(arr) = bundle.zones.as_array() {
        for zone in arr {
            let id = zone["id"].as_str().unwrap_or("");
            // Support both 'type' (new) and 'shape' (old) 
            let zone_type = zone["type"].as_str()
                .or_else(|| zone["shape"].as_str())
                .unwrap_or("");
            let contents = zone
                .get("contents")
                .cloned()
                .unwrap_or_else(|| Value::String("empty".to_string()));
            let per_player = id.contains("{player}");
            let ids: Vec<String> = if per_player {
                players
                    .iter()
                    .map(|p| id.replace("{player}", p["id"].as_str().unwrap()))
                    .collect()
            } else {
                vec![id.to_string()]
            };

            for pid in ids {
                println!("  Creating zone: {}", pid);
                let mut content_spec = contents.clone();
                if per_player {
                    let player_id = pid.split('_').last().unwrap_or("");
                    if let Some(map) = contents.as_object() {
                        if let Some(specific) = map.get(player_id) {
                            content_spec = specific.clone();
                        }
                    }
                    
                    // Replace {player} placeholders in the content_spec
                    if let Some(obj) = content_spec.as_object_mut() {
                        if let Some(entity) = obj.get_mut("entity") {
                            if let Some(entity_str) = entity.as_str() {
                                *entity = Value::String(entity_str.replace("{player}", player_id));
                            }
                        }
                    }
                }

                let mut value = match zone_type {
                    "grid" => init_grid(zone, &content_spec),
                    "list" | "deck" => init_list(&content_spec),
                    _ => Value::Null,
                };
                
                // Apply deck shuffling if specified
                if zone_type == "deck" {
                    if let Some(deck_props) = zone.get("deckProps") {
                        if deck_props.get("shuffle").and_then(|s| s.as_bool()).unwrap_or(false) {
                            if let Some(items) = value.get_mut("items").and_then(|i| i.as_array_mut()) {
                                let mut rng = rng();
                                items.shuffle(&mut rng);
                                println!("  Shuffled deck for zone: {}", pid);
                            }
                        }
                    }
                }
                
                zones.insert(pid, value);
            }
        }
    }

    // Initialize all phase sets to their initial phase
    let mut phase_states = json!({});
    
    if let Some(phase_sets) = bundle.phases.as_array() {
        for phase_set in phase_sets {
            if let Some(set_id) = phase_set["id"].as_str() {
                // Find the initial phase in this set
                if let Some(phases) = phase_set["phases"].as_array() {
                    let initial = phases.iter()
                        .find(|p| p["initial"].as_bool().unwrap_or(false))
                        .or_else(|| phases.first())
                        .and_then(|p| p["id"].as_str())
                        .unwrap_or("null");
                    
                    phase_states[set_id] = json!(initial);
                }
            }
        }
    }

    json!({
        "zones": zones,
        "players": players,
        "tick": 0,
        "turn": 0,
        "currentPlayer": "p1",
        "gameStatus": {
            "state": "playing",
            "winner": null,
            "tie": false
        },
        "phases": phase_states
    })
}

fn init_grid(zone: &Value, contents: &Value) -> Value {
    let rows = zone["rows"].as_u64().unwrap_or(3) as usize;
    let cols = zone["cols"].as_u64().unwrap_or(3) as usize;
    
    let mut grid = Vec::new();
    for _r in 0..rows {
        let mut row = Vec::new();
        for _c in 0..cols {
            let cell_value = if contents.as_str() == Some("empty") {
                Value::Null
            } else {
                contents.clone()
            };
            row.push(cell_value);
        }
        grid.push(Value::Array(row));
    }
    
    json!({
        "type": "grid",
        "cells": grid
    })
}

fn init_list(contents: &Value) -> Value {
    let items = if contents.as_str() == Some("empty") {
        Vec::new()
    } else if let Some(entity_id) = contents.get("entity").and_then(|e| e.as_str()) {
        let count = contents.get("count").and_then(|c| c.as_u64()).unwrap_or(1);
        (0..count).map(|_| json!({"entity": entity_id})).collect()
    } else if let Some(arr) = contents.as_array() {
        arr.clone()
    } else {
        vec![contents.clone()]
    };
    
    json!({
        "type": "list",
        "items": items
    })
}

/* --------------------------------------------------------------------------
   Apply verb functions 
   ----------------------------------------------------------------------- */

pub fn apply_verb(
    state: &mut Value,
    verb: &str,
    args: &Value,
    bundle: &Bundle,
) -> Result<Vec<Value>, String> {
    match verb {
        "draw" => apply_draw(state, args),
        "moveEntity" => apply_move_entity(state, args),
        "place" => apply_place(state, args),
        "nextTurn" => apply_next_turn(state, args, bundle),
        "setPhase" => apply_set_phase(state, args),
        "grid.lineOfMarks" => apply_check_for_win(state, args),
        _ => Err(format!("Unknown verb: {}", verb)),
    }
}

fn apply_draw(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;
    let count = args["count"].as_u64().unwrap_or(1) as usize;

    let mut patches = Vec::new();

    for _ in 0..count {
        // Get the source zone
        let from_zone = get_zone_mut(state, from_path)?;
        let items = from_zone["items"].as_array_mut()
            .ok_or("Source zone is not a list/deck")?;
        
        if items.is_empty() {
            return Err("Cannot draw from empty deck".to_string());
        }

        // Remove item from source
        let item = items.remove(0);
        patches.push(json!({
            "op": "remove",
            "path": format!("{}/items/0", from_path)
        }));

        // Add to destination
        let to_zone = get_zone_mut(state, to_path)?;
        let to_items = to_zone["items"].as_array_mut()
            .ok_or("Destination zone is not a list")?;
        
        let insert_index = to_items.len();
        to_items.push(item.clone());
        patches.push(json!({
            "op": "add",
            "path": format!("{}/items/{}", to_path, insert_index),
            "value": item
        }));
    }

    Ok(patches)
}

fn apply_move_entity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;

    let mut patches = Vec::new();

    // Handle grid to grid moves
    if from_path.contains("/cells/") && to_path.contains("/cells/") {
        let from_value = get_cell_value(state, from_path)?;
        
        // Remove from source
        set_cell_value(state, from_path, Value::Null)?;
        patches.push(json!({
            "op": "replace",
            "path": from_path,
            "value": null
        }));

        // Add to destination
        set_cell_value(state, to_path, from_value.clone())?;
        patches.push(json!({
            "op": "replace",
            "path": to_path,
            "value": from_value
        }));
    }

    Ok(patches)
}

fn apply_place(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let location = args["location"].as_str().ok_or("Missing 'location' path")?;
    let entity = args["entity"].as_str().ok_or("Missing 'entity' id")?;
    
    let entity_value = json!({"entity": entity});
    set_cell_value(state, location, entity_value.clone())?;
    
    Ok(vec![json!({
        "op": "replace", 
        "path": format!("/game{}", location),
        "value": entity_value
    })])
}

fn apply_next_turn(state: &mut Value, _args: &Value, bundle: &Bundle) -> Result<Vec<Value>, String> {
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    
    // Increment tick
    let current_tick = state_obj["tick"].as_u64().unwrap_or(0);
    state_obj.insert("tick".to_string(), json!(current_tick + 1));
    
    // Advance turn
    let player_count = bundle.manifest.metadata.players.max;
    let current_turn = state_obj["turn"].as_u64().unwrap_or(0);
    let next_turn = (current_turn + 1) % player_count as u64;
    let next_player = format!("p{}", next_turn + 1);
    
    state_obj.insert("turn".to_string(), json!(next_turn));
    state_obj.insert("currentPlayer".to_string(), json!(next_player));
    
    Ok(vec![
        json!({
            "op": "replace",
            "path": "/game/tick",
            "value": current_tick + 1
        }),
        json!({
            "op": "replace",
            "path": "/game/turn",
            "value": next_turn
        }),
        json!({
            "op": "replace",
            "path": "/game/currentPlayer",
            "value": next_player
        })
    ])
}

fn apply_set_phase(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let phase_set = args["phaseSet"].as_str().ok_or("Missing 'phaseSet'")?;
    let phase = args["phase"].as_str().ok_or("Missing 'phase'")?;
    
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    let phases = state_obj.get_mut("phases").and_then(|p| p.as_object_mut())
        .ok_or("Missing phases in state")?;
    
    phases.insert(phase_set.to_string(), json!(phase));
    
    Ok(vec![json!({
        "op": "replace",
        "path": format!("/game/phases/{}", phase_set),
        "value": phase
    })])
}

fn apply_check_for_win(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args["zone"].as_str().ok_or("Missing 'zone' path")?;
    let entity_pattern = args["entity"].as_str().ok_or("Missing 'entity' pattern")?;
    let line_length = args["lineLength"].as_u64().unwrap_or(3) as usize;
    let directions = args["directions"].as_array().ok_or("Missing 'directions' array")?;
    
    // Get the grid from the specified zone
    let zone = get_zone_ref(state, zone_path)?;
    let cells = zone["cells"].as_array()
        .ok_or("Zone is not a grid with cells")?;
    
    if cells.is_empty() {
        return Err("Grid has no rows".to_string());
    }
    
    let rows = cells.len();
    let cols = cells[0].as_array()
        .ok_or("Grid row is not an array")?
        .len();
    
    // Check for winning lines in each enabled direction
    for direction in directions {
        let dir_str = direction.as_str()
            .ok_or("Direction must be a string")?;
        
        match dir_str {
            "horizontal" => {
                if let Some(winner) = check_horizontal_lines(cells, entity_pattern, line_length, rows, cols)? {
                    return set_game_winner(state, &winner);
                }
            },
            "vertical" => {
                if let Some(winner) = check_vertical_lines(cells, entity_pattern, line_length, rows, cols)? {
                    return set_game_winner(state, &winner);
                }
            },
            "diagonal" => {
                if let Some(winner) = check_diagonal_lines(cells, entity_pattern, line_length, rows, cols)? {
                    return set_game_winner(state, &winner);
                }
            },
            _ => return Err(format!("Unknown direction: {}", dir_str))
        }
    }
    
    // Check for tie (board full with no winner)
    if is_board_full(cells, rows, cols)? {
        return set_game_tie(state);
    }
    
    // No winner and board not full - game continues
    Ok(vec![])
}

fn get_zone_ref<'a>(state: &'a Value, zone_path: &str) -> Result<&'a Value, String> {
    let path_parts: Vec<&str> = zone_path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in path_parts {
        current = current.get(part)
            .ok_or_else(|| format!("Path not found: {}", zone_path))?;
    }
    
    Ok(current)
}

fn check_horizontal_lines(cells: &[Value], entity_pattern: &str, line_length: usize, rows: usize, cols: usize) -> Result<Option<String>, String> {
    for row in 0..rows {
        for start_col in 0..=(cols.saturating_sub(line_length)) {
            if let Some(winner) = check_line_at_position(cells, entity_pattern, line_length, row, start_col, 0, 1)? {
                return Ok(Some(winner));
            }
        }
    }
    Ok(None)
}

fn check_vertical_lines(cells: &[Value], entity_pattern: &str, line_length: usize, rows: usize, cols: usize) -> Result<Option<String>, String> {
    for col in 0..cols {
        for start_row in 0..=(rows.saturating_sub(line_length)) {
            if let Some(winner) = check_line_at_position(cells, entity_pattern, line_length, start_row, col, 1, 0)? {
                return Ok(Some(winner));
            }
        }
    }
    Ok(None)
}

fn check_diagonal_lines(cells: &[Value], entity_pattern: &str, line_length: usize, rows: usize, cols: usize) -> Result<Option<String>, String> {
    // Check main diagonal (top-left to bottom-right)
    for row in 0..=(rows.saturating_sub(line_length)) {
        for col in 0..=(cols.saturating_sub(line_length)) {
            if let Some(winner) = check_line_at_position(cells, entity_pattern, line_length, row, col, 1, 1)? {
                return Ok(Some(winner));
            }
        }
    }
    
    // Check anti-diagonal (top-right to bottom-left)
    for row in 0..=(rows.saturating_sub(line_length)) {
        for col in (line_length - 1)..cols {
            if let Some(winner) = check_line_at_position(cells, entity_pattern, line_length, row, col, 1, -1)? {
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
                if let Some(ref first) = first_entity {
                    if first != &id {
                        return Ok(None); // Different entities in line
                    }
                } else {
                    first_entity = Some(id);
                }
            } else {
                return Ok(None); // Entity doesn't match pattern or empty cell
            }
        } else {
            return Ok(None); // Empty cell
        }
    }
    
    first_entity.map(|entity| extract_player_from_entity(&entity)).transpose().map(|opt| opt.flatten())
}

fn extract_entity_id(cell: &Value) -> Option<String> {
    if let Some(entity_obj) = cell.as_object() {
        if let Some(entity_id) = entity_obj.get("entity").and_then(|e| e.as_str()) {
            return Some(entity_id.to_string());
        }
    }
    None
}

fn matches_pattern(entity_id: &str, pattern: &str) -> bool {
    // Simple pattern matching - for now just check if the pattern is a substring
    // Could be enhanced to support wildcards like "mark_{player}"
    if pattern.contains("{player}") {
        // Extract the base pattern without {player}
        let base_pattern = pattern.replace("{player}", "");
        entity_id.starts_with(&base_pattern)
    } else {
        entity_id == pattern
    }
}

fn extract_player_from_entity(entity_id: &str) -> Result<Option<String>, String> {
    // Extract player ID from entity like "mark_p1" -> "p1"
    if let Some(pos) = entity_id.rfind("_p") {
        let player_part = &entity_id[pos + 1..];
        return Ok(Some(player_part.to_string()));
    }
    Ok(None)
}

fn is_board_full(cells: &[Value], rows: usize, cols: usize) -> Result<bool, String> {
    for row in 0..rows {
        for col in 0..cols {
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
    })])
}

// Helper functions
fn get_zone_mut<'a>(state: &'a mut Value, zone_path: &str) -> Result<&'a mut Value, String> {
    let path_parts: Vec<&str> = zone_path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in path_parts {
        current = current.get_mut(part)
            .ok_or_else(|| format!("Path not found: {}", zone_path))?;
    }
    
    Ok(current)
}

fn get_cell_value(state: &Value, cell_path: &str) -> Result<Value, String> {
    let path_parts: Vec<&str> = cell_path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in &path_parts {
        if let Ok(index) = part.parse::<usize>() {
            // This is an array index
            if let Some(array) = current.as_array() {
                if index < array.len() {
                    current = &array[index];
                } else {
                    return Err(format!("Array index {} out of bounds (length: {})", index, array.len()));
                }
            } else {
                return Err(format!("Expected array for numeric index '{}'", part));
            }
        } else {
            // This is an object key
            current = current.get(part)
                .ok_or_else(|| format!("Path not found: '{}' in path '{}'", part, cell_path))?;
        }
    }
    
    Ok(current.clone())
}

fn set_cell_value(state: &mut Value, cell_path: &str, value: Value) -> Result<(), String> {
    let path_parts: Vec<&str> = cell_path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in &path_parts {
        if let Ok(index) = part.parse::<usize>() {
            // This is an array index
            if let Some(array) = current.as_array_mut() {
                if index < array.len() {
                    current = &mut array[index];
                } else {
                    return Err(format!("Array index {} out of bounds (length: {})", index, array.len()));
                }
            } else {
                return Err(format!("Expected array for numeric index '{}'", part));
            }
        } else {
            // This is an object key
            if let Some(obj) = current.as_object_mut() {
                current = obj.get_mut(*part)
                    .ok_or_else(|| format!("Path not found: '{}' in path '{}'", part, cell_path))?;
            } else {
                return Err(format!("Expected object for key '{}'", part));
            }
        }
    }
    
    // Replace the final value
    *current = value;
    Ok(())
}

// Additional functions needed by lobby module (stubs for now)
pub fn process_phases(bundle: &Bundle, state: &mut Value) -> Result<Vec<Value>, String> {
    let mut patches = Vec::new();
    
    // Get current phase states
    let phase_states = state["phases"].as_object()
        .ok_or("Missing phase states")?
        .clone();
    
    // Check each phase set for enterActions
    if let Some(phase_sets) = bundle.phases.as_array() {
        for (phase_set_id, current_phase_id) in phase_states.iter() {
            // Find the phase set definition
            if let Some(phase_set) = phase_sets.iter()
                .find(|ps| ps["id"].as_str() == Some(phase_set_id)) {
                
                // Find the current phase within the set
                if let Some(phases) = phase_set["phases"].as_array() {
                    if let Some(current_phase) = phases.iter()
                        .find(|p| p["id"].as_str() == Some(current_phase_id.as_str().unwrap_or(""))) {
                        
                        // Check for enterActions
                        if let Some(enter_actions) = current_phase["enterActions"].as_array() {
                            println!("[DEBUG process_phases] Found enterActions for phase {}.{}", 
                                phase_set_id, current_phase_id.as_str().unwrap_or(""));
                            
                            // Process each enter action
                            for action in enter_actions {
                                if let Some(transition_to) = action["transitionToPhase"].as_str() {
                                    // This is a phase transition action
                                    println!("[DEBUG process_phases] Transitioning to phase: {}", transition_to);
                                    
                                    // Update the phase state
                                    let phases = state["phases"].as_object_mut().unwrap();
                                    phases.insert(phase_set_id.clone(), json!(transition_to));
                                    
                                    // Create patch for the transition
                                    patches.push(json!({
                                        "op": "replace",
                                        "path": format!("/phases/{}", phase_set_id),
                                        "value": transition_to
                                    }));
                                }
                                // Handle other types of enter actions here if needed
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(patches)
}

pub fn apply_patch_to_state(state: &mut Value, patch: &Value) {
    // TODO: Implement JSON patch application
    // For now, this is a stub
    if let Some(op) = patch.get("op").and_then(|o| o.as_str()) {
        match op {
            "replace" => {
                if let (Some(path), Some(value)) = (patch.get("path"), patch.get("value")) {
                    if let Some(path_str) = path.as_str() {
                        let _ = set_value_at_path(state, path_str, value.clone());
                    }
                }
            }
            _ => {
                // TODO: Implement other patch operations
            }
        }
    }
}

pub fn apply_action(
    bundle: &Bundle,
    state: &mut Value,
    _player_id: &str,
    action: &Value,
) -> Result<Vec<Value>, String> {
    // Extract verb and args from action
    if let (Some(verb), Some(args)) = (
        action.get("verb").and_then(|v| v.as_str()),
        action.get("args")
    ) {
        apply_verb(state, verb, args, bundle)
    } else {
        Err("Invalid action format".to_string())
    }
}

fn set_value_at_path(state: &mut Value, path: &str, value: Value) -> Result<(), String> {
    let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in &path_parts {
        if let Ok(index) = part.parse::<usize>() {
            // This is an array index
            if let Some(array) = current.as_array_mut() {
                if index < array.len() {
                    current = &mut array[index];
                } else {
                    return Err(format!("Array index {} out of bounds (length: {})", index, array.len()));
                }
            } else {
                return Err(format!("Expected array for numeric index '{}'", part));
            }
        } else {
            // This is an object key
            if let Some(obj) = current.as_object_mut() {
                current = obj.get_mut(*part)
                    .ok_or_else(|| format!("Path not found: '{}' in path '{}'", part, path))?;
            } else {
                return Err(format!("Expected object for key '{}'", part));
            }
        }
    }
    
    // Replace the final value
    *current = value;
    Ok(())
}

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
        
        // Should detect win and set game status
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
        
        // Should detect win
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
        
        // Should detect diagonal win
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
        
        // Should detect tie (board full, no winner)
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
        
        // Should return empty - no winner, game continues
        assert_eq!(result.len(), 0);
        assert!(state["meta"]["gameStatus"].is_null());
    }

    #[test]
    fn test_apply_draw_basic() {
        let mut state = json!({
            "zones": {
                "deck": {
                    "type": "list",
                    "items": [
                        {"entity": "card1"},
                        {"entity": "card2"}
                    ]
                },
                "hand": {
                    "type": "list", 
                    "items": []
                }
            }
        });

        let args = json!({
            "from": "/zones/deck",
            "to": "/zones/hand",
            "count": 1
        });

        let result = apply_draw(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 2);
        
        // Check that card was moved
        let deck_items = state["zones"]["deck"]["items"].as_array().unwrap();
        let hand_items = state["zones"]["hand"]["items"].as_array().unwrap();
        
        assert_eq!(deck_items.len(), 1);
        assert_eq!(hand_items.len(), 1);
        assert_eq!(hand_items[0]["entity"], "card1");
    }

    #[test]
    fn test_apply_draw_empty_deck() {
        let mut state = json!({
            "zones": {
                "deck": {
                    "type": "list",
                    "items": []
                },
                "hand": {
                    "type": "list",
                    "items": []
                }
            }
        });

        let args = json!({
            "from": "/zones/deck",
            "to": "/zones/hand",
            "count": 1
        });

        let result = apply_draw(&mut state, &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty deck"));
    }

    #[test]
    fn test_apply_place() {
        let mut state = json!({
            "zones": {
                "board": {
                    "type": "grid",
                    "cells": [
                        [null, null, null],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            }
        });

        let args = json!({
            "location": "/zones/board/cells/0/0",
            "entity": "x_token"
        });

        let result = apply_place(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 1);
        
        // Check that entity was placed
        let cell_value = &state["zones"]["board"]["cells"][0][0];
        assert_eq!(cell_value["entity"], "x_token");
    }

    #[test]
    fn test_apply_move_entity() {
        let mut state = json!({
            "zones": {
                "board": {
                    "type": "grid",
                    "cells": [
                        [{"entity": "piece"}, null, null],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            }
        });

        let args = json!({
            "from": "/zones/board/cells/0/0",
            "to": "/zones/board/cells/1/1"
        });

        let result = apply_move_entity(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 2);
        
        // Check that entity was moved
        assert!(state["zones"]["board"]["cells"][0][0].is_null());
        assert_eq!(state["zones"]["board"]["cells"][1][1]["entity"], "piece");
    }

    #[test]
    fn test_set_cell_value_grid() {
        let mut state = json!({
            "zones": {
                "board": {
                    "cells": [[null, null], [null, null]]
                }
            }
        });

        let result = set_cell_value(&mut state, "/zones/board/cells/0/1", json!({"entity": "test"}));
        assert!(result.is_ok());
        
        assert_eq!(state["zones"]["board"]["cells"][0][1]["entity"], "test");
    }

    #[test]
    fn test_get_cell_value() {
        let state = json!({
            "zones": {
                "board": {
                    "cells": [[{"entity": "piece"}, null]]
                }
            }
        });

        let result = get_cell_value(&state, "/zones/board/cells/0/0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["entity"], "piece");
    }
}
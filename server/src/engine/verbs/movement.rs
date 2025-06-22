use serde_json::{json, Value};
use crate::engine::path::{get_zone_mut, get_cell_value, set_cell_value};

pub fn apply_move_entity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;

    println!("[moveEntity] Moving from '{}' to '{}'", from_path, to_path);

    let mut patches = Vec::new();

    // Handle grid to grid moves
    if from_path.contains("/cells/") && to_path.contains("/cells/") {
        println!("[moveEntity] Detected grid-to-grid move");
        move_between_grid_cells(state, from_path, to_path, &mut patches)?;
    } else {
        println!("[moveEntity] Warning: Paths don't contain '/cells/', no move performed");
        println!("[moveEntity] From path: {}", from_path);
        println!("[moveEntity] To path: {}", to_path);
    }

    println!("[moveEntity] Generated {} patches", patches.len());
    Ok(patches)
}

fn move_between_grid_cells(
    state: &mut Value,
    from_path: &str,
    to_path: &str,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    let from_value = get_cell_value(state, from_path)?;
    
    // Remove from source
    set_cell_value(state, from_path, Value::Null)?;
    patches.push(json!({
        "op": "replace",
        "path": from_path,
        "value": null,
        "_animation": {
            "type": "entity_remove",
            "duration": 250
        }
    }));

    // Add to destination with movement animation hint
    set_cell_value(state, to_path, from_value.clone())?;
    patches.push(json!({
        "op": "replace",
        "path": to_path,
        "value": from_value,
        "_animation": {
            "type": "entity_movement",
            "from": from_path,
            "to": to_path,
            "duration": 400,
            "isGravityDrop": is_gravity_drop(from_path, to_path)
        }
    }));

    Ok(())
}

/// Detect if this is a gravity drop movement (like Connect 4)
fn is_gravity_drop(from_path: &str, to_path: &str) -> bool {
    // Extract coordinates from paths like "/game/zones/board/cells/0/3"
    let from_coords = extract_coordinates(from_path);
    let to_coords = extract_coordinates(to_path);
    
    if let (Some((from_row, from_col)), Some((to_row, to_col))) = (from_coords, to_coords) {
        // Same column, moving downward (higher row number)
        from_col == to_col && to_row > from_row
    } else {
        false
    }
}

/// Extract row and column coordinates from a cell path
fn extract_coordinates(path: &str) -> Option<(usize, usize)> {
    // Parse paths like "/game/zones/board/cells/2/4" or "/zones/board/cells/2/4"
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 4 && parts[parts.len()-3] == "cells" {
        let row = parts[parts.len()-2].parse::<usize>().ok()?;
        let col = parts[parts.len()-1].parse::<usize>().ok()?;
        Some((row, col))
    } else {
        None
    }
}

pub fn apply_place(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let location = args["location"].as_str().ok_or("Missing 'location' path")?;
    let entity = args["entity"].as_str().ok_or("Missing 'entity' id")?;
    
    let entity_value = json!({"entity": entity});
    set_cell_value(state, location, entity_value.clone())?;
    
    Ok(vec![json!({
        "op": "replace", 
        "path": format!("/game{}", location),
        "value": entity_value,
        "_animation": {
            "type": "entity_spawn",
            "duration": 300
        }
    })])
}

pub fn apply_place_with_gravity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args["zone"].as_str().ok_or("Missing 'zone' path")?;
    
    // Handle both string and number for column (due to template replacement converting to string)
    let column = if let Some(col_num) = args["column"].as_u64() {
        col_num as usize
    } else if let Some(col_str) = args["column"].as_str() {
        col_str.parse::<usize>().map_err(|_| "Invalid 'column' value")?
    } else {
        return Err("Missing 'column' index".to_string());
    };
    
    let entity = args["entity"].as_str().ok_or("Missing 'entity' id")?;
    
    // Get the zone data
    let zone = get_zone_mut(state, zone_path)?;
    let cells = zone["cells"].as_array_mut()
        .ok_or("Zone is not a grid with cells")?;
    
    // Find the lowest empty row in the specified column (highest row index)
    // Row 0 is the top, so we want to find the highest available row index
    let mut target_row = None;
    for row_idx in (0..cells.len()).rev() {
        let row_array = cells[row_idx].as_array()
            .ok_or("Row is not an array")?;
        
        if column >= row_array.len() {
            return Err("Column index out of bounds".to_string());
        }
        
        if row_array[column].is_null() {
            target_row = Some(row_idx);
            break;
        }
    }
    
    let row = target_row.ok_or("Column is full")?;
    
    // Place the entity at the calculated position
    let entity_value = json!({"entity": entity});
    let row_array = cells[row].as_array_mut()
        .ok_or("Row is not an array")?;
    row_array[column] = entity_value.clone();
    
    Ok(vec![json!({
        "op": "replace",
        "path": format!("/game{}/cells/{}/{}", zone_path, row, column),
        "value": entity_value,
        "_animation": {
            "type": "entity_spawn",
            "duration": 600,
            "isGravityDrop": true,
            "fromPosition": { "row": 0, "col": column },
            "toPosition": { "row": row, "col": column }
        }
    })])
}

pub fn apply_move_selected(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let target_location = args["target"].as_str().ok_or("Missing 'target'")?;
    let player = args["player"].as_str().ok_or("Missing 'player'")?;
    
    
    // Get current selection and clone the data we need
    let (source_location, selected_entity) = {
        let selection_state = state.get("selection")
            .and_then(|s| s.get(player))
            .ok_or("No selection found for player")?;
        
        let source_location = selection_state["location"].as_str()
            .ok_or("Invalid selection location")?
            .to_string();
        let selected_entity = selection_state["entity"].clone();
        
        (source_location, selected_entity)
    };
    
    // Validate target is empty
    let target_value = get_cell_value(state, target_location)?;
    if !target_value.is_null() {
        return Err("Target location is not empty".to_string());
    }
    
    // Move the entity
    set_cell_value(state, &source_location, json!(null))?;
    set_cell_value(state, target_location, selected_entity.clone())?;
    
    // Clear selection
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    if let Some(selection) = state_obj.get_mut("selection").and_then(|s| s.as_object_mut()) {
        selection.remove(player);
    }
    
    Ok(vec![
        json!({
            "op": "replace",
            "path": format!("/game{}", source_location),
            "value": null
        }),
        json!({
            "op": "replace", 
            "path": format!("/game{}", target_location),
            "value": selected_entity
        }),
        json!({
            "op": "remove",
            "path": format!("/game/selection/{}", player)
        })
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
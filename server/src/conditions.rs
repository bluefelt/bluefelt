use serde_json::Value;

// Helper function to get a value at a specific path in the JSON state
fn get_value_at_path<'a>(state: &'a Value, path: &str) -> Option<&'a Value> {
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

/// Evaluates a condition against the current game state
pub fn evaluate_condition(
    condition: &Value,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    let condition_type = condition["condition"]
        .as_str()
        .ok_or("Missing condition type")?;
    
    let with_obj = condition.get("with").and_then(|w| w.as_object());
    
    match condition_type {
        "zone.isEmpty" => evaluate_zone_is_empty(with_obj, state, args),
        "player.isActor" => evaluate_player_is_actor(state, current_actor),
        "zone.count" => evaluate_zone_count(with_obj, state, current_actor),
        "entity.owner" => evaluate_entity_owner(with_obj, state, args, current_actor),
        "phase.is" => evaluate_phase_is(with_obj, state),
        "resource.value" => evaluate_resource_value(with_obj, state, current_actor),
        "entity.selected" => evaluate_entity_selected(with_obj, state, args, current_actor),
        _ => Err(format!("Unknown condition type: {}", condition_type)),
    }
}

fn evaluate_zone_is_empty(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for zone.isEmpty")?;
    let zone_template = with_obj.get("zone")
        .and_then(|z| z.as_str())
        .ok_or("Missing zone in condition")?;
    
    // Replace {target} with the actual location from args
    let zone_path = if zone_template == "{target}" {
        if let Some(target) = args.get("target").and_then(|t| t.as_str()) {
            println!("[DEBUG zone.isEmpty] Replacing {{target}} with: {}", target);
            target.to_string()
        } else if let Some(location) = args.get("location").and_then(|l| l.as_str()) {
            println!("[DEBUG zone.isEmpty] Using location as target: {}", location);
            location.to_string()
        } else {
            println!("[DEBUG zone.isEmpty] No target or location found in args: {:?}", args);
            return Err("Missing target in args for {target} placeholder".to_string());
        }
    } else {
        zone_template.to_string()
    };
    
    // Check if the zone/cell is actually empty
    let is_empty = if zone_path.contains("/cells/") {
        // For grid cells, check if the cell is null
        get_value_at_path(state, &zone_path).map(|v| v.is_null()).unwrap_or(false)
    } else {
        // For other zones, check if empty
        if let Some(zone_value) = get_value_at_path(state, &zone_path) {
            if let Some(items) = zone_value.get("items").and_then(|i| i.as_array()) {
                items.is_empty()
            } else if let Some(cells) = zone_value.get("cells").and_then(|c| c.as_array()) {
                // For grid zones, check if all cells are empty
                cells.iter().all(|row| {
                    row.as_array()
                        .map(|r| r.iter().all(|cell| cell.is_null()))
                        .unwrap_or(true)
                })
            } else {
                zone_value.is_null()
            }
        } else {
            false
        }
    };
    
    Ok(is_empty)
}

fn evaluate_player_is_actor(state: &Value, current_actor: &str) -> Result<bool, String> {
    let current_player = state.get("currentPlayer")
        .and_then(|cp| cp.as_str())
        .unwrap_or("");
    
    Ok(current_player == current_actor)
}

fn evaluate_zone_count(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for zone.count")?;
    let zone_path = with_obj.get("zone")
        .and_then(|z| z.as_str())
        .ok_or("Missing zone in condition")?;
    
    // Replace {player} with current actor
    let zone_path = zone_path.replace("{player}", current_actor);
    
    // Get the zone
    let zone_value = get_value_at_path(state, &zone_path)
        .ok_or_else(|| format!("Zone not found: {}", zone_path))?;
    
    // Get and process entity filter
    let entity_filter_raw = with_obj.get("entity").and_then(|e| e.as_str()).unwrap_or("");
    let entity_filter_processed = entity_filter_raw.replace("{player}", current_actor);
    println!("[DEBUG zone.count] Checking zone: {}, entity: {} -> {}, actor: {}", zone_path, entity_filter_raw, entity_filter_processed, current_actor);
    
    // Count entities based on zone type
    let count = if let Some(cells) = zone_value.get("cells").and_then(|c| c.as_array()) {
        // Grid zone - count non-null cells
        
        cells.iter().map(|row| {
            row.as_array()
                .map(|r| r.iter().filter(|cell| {
                    if cell.is_null() {
                        false
                    } else if !entity_filter_processed.is_empty() {
                        // Check if entity matches the processed filter
                        if let Some(entity_id) = cell.get("entity").and_then(|e| e.as_str()) {
                            entity_id == entity_filter_processed
                        } else {
                            false
                        }
                    } else {
                        true // Count all non-null cells if no filter
                    }
                }).count())
                .unwrap_or(0)
        }).sum()
    } else if let Some(items) = zone_value.get("items").and_then(|i| i.as_array()) {
        // List zone - count items
        if !entity_filter_processed.is_empty() {
            items.iter().filter(|item| {
                if let Some(entity_id) = item.as_str() {
                    entity_id == entity_filter_processed
                } else {
                    false
                }
            }).count()
        } else {
            items.len()
        }
    } else {
        0
    };
    
    // Check against operator and value
    let operator = with_obj.get("operator")
        .and_then(|o| o.as_str())
        .unwrap_or("==");
    let value = with_obj.get("value")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    
    let result = match operator {
        "==" => count == value,
        "!=" => count != value,
        ">" => count > value,
        "<" => count < value,
        ">=" => count >= value,
        "<=" => count <= value,
        _ => return Err(format!("Unknown operator: {}", operator)),
    };
    
    println!("[DEBUG zone.count] Count: {}, operator: {}, value: {}, result: {}", count, operator, value, result);
    
    Ok(result)
}

fn evaluate_entity_owner(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for entity.owner")?;
    
    // Get entity from args or with object
    let entity_path = if let Some(entity_template) = with_obj.get("entity").and_then(|e| e.as_str()) {
        // Replace {selected} with actual entity from args
        if entity_template.contains("{selected}") {
            if let Some(selected) = args.get("entity").and_then(|e| e.as_str()) {
                entity_template.replace("{selected}", selected)
            } else {
                return Err("Missing selected entity in args".to_string());
            }
        } else if entity_template == "{entityAtLocation}" {
            // Look up the entity at the provided location
            if let Some(location) = args.get("location").and_then(|l| l.as_str()) {
                println!("[DEBUG entity.owner] Looking for entity at location: {}", location);
                
                // Parse the location path to get zone and coordinates
                let parts: Vec<&str> = location.split('/').collect();
                if parts.len() >= 5 && parts[1] == "zones" && parts[3] == "cells" {
                    let zone_id = parts[2];
                    if let (Ok(row), Ok(col)) = (parts[4].parse::<usize>(), parts[5].parse::<usize>()) {
                        // Navigate to the cell in the state
                        let cell_value = state
                            .get("zones")
                            .or_else(|| state.get("game").and_then(|g| g.get("zones")))
                            .and_then(|zones| zones.get(zone_id))
                            .and_then(|zone| zone.get("cells"))
                            .and_then(|cells| cells.get(row))
                            .and_then(|row_cells| row_cells.get(col));
                            
                        if let Some(cell) = cell_value {
                            if let Some(entity) = cell.get("entity").and_then(|e| e.as_str()) {
                                println!("[DEBUG entity.owner] Found entity {} at location {}", entity, location);
                                entity.to_string()
                            } else {
                                println!("[DEBUG entity.owner] No entity found at location {}", location);
                                return Err(format!("No entity at location {}", location));
                            }
                        } else {
                            println!("[DEBUG entity.owner] Could not find cell at location {}", location);
                            return Err(format!("Invalid location {}", location));
                        }
                    } else {
                        return Err("Invalid cell coordinates".to_string());
                    }
                } else {
                    return Err("Invalid location format".to_string());
                }
            } else {
                return Err("Missing location in args".to_string());
            }
        } else {
            entity_template.to_string()
        }
    } else {
        return Err("Missing entity in condition".to_string());
    };
    
    // Get expected owner
    let expected_owner = with_obj.get("owner")
        .and_then(|o| o.as_str())
        .unwrap_or("{player}")
        .replace("{player}", current_actor);
    
    println!("[DEBUG entity.owner] Checking entity: {}, expected owner: {}", entity_path, expected_owner);
    
    // For now, check if entity ID contains the owner
    // This is a simple implementation - could be enhanced to check entity properties
    let entity_owned_by = if entity_path.contains("_p1") {
        "p1"
    } else if entity_path.contains("_p2") {
        "p2"
    } else {
        ""
    };
    
    println!("[DEBUG entity.owner] Entity owned by: {}, matches: {}", entity_owned_by, entity_owned_by == expected_owner);
    
    Ok(entity_owned_by == expected_owner)
}

fn evaluate_phase_is(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for phase.is")?;
    let phase_set = with_obj.get("phaseSet")
        .and_then(|ps| ps.as_str())
        .ok_or("Missing phaseSet in condition")?;
    let expected_phase = with_obj.get("phase")
        .and_then(|p| p.as_str())
        .ok_or("Missing phase in condition")?;
    
    // Get current phase from state
    let current_phase = state.get("phases")
        .and_then(|phases| phases.get(phase_set))
        .and_then(|phase| phase.as_str())
        .unwrap_or("");
    
    Ok(current_phase == expected_phase)
}

fn evaluate_resource_value(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for resource.value")?;
    let resource_template = with_obj.get("resource")
        .and_then(|r| r.as_str())
        .ok_or("Missing resource in condition")?;
    
    // Replace {player} with current actor
    let resource_path = format!("/zones/{}", resource_template.replace("{player}", current_actor));
    
    // Get the resource zone
    let resource_zone = get_value_at_path(state, &resource_path)
        .ok_or_else(|| format!("Resource not found: {}", resource_path))?;
    
    // Get the current value
    let current_value = resource_zone.get("value")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    // Check against operator and value
    let operator = with_obj.get("operator")
        .and_then(|o| o.as_str())
        .unwrap_or("==");
    let value = with_obj.get("value")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    let result = match operator {
        "==" => current_value == value,
        "!=" => current_value != value,
        ">" => current_value > value,
        "<" => current_value < value,
        ">=" => current_value >= value,
        "<=" => current_value <= value,
        _ => return Err(format!("Unknown operator: {}", operator)),
    };
    
    Ok(result)
}

fn evaluate_movement_adjacent(
    _with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    // Get source location from player's selection
    let selection_state = state.get("selection")
        .and_then(|s| s.get(current_actor))
        .ok_or("No selection found for player")?;
    
    let source_location = selection_state["location"].as_str()
        .ok_or("Invalid selection location")?;
    
    // Get target location from args
    let target_location = args.get("target")
        .and_then(|t| t.as_str())
        .ok_or("Missing target location in args")?;
    
    // Parse both locations to get coordinates
    let source_coords = parse_grid_location(source_location)?;
    let target_coords = parse_grid_location(target_location)?;
    
    // Check if locations are adjacent (including diagonals)
    let row_diff = (source_coords.0 as i32 - target_coords.0 as i32).abs();
    let col_diff = (source_coords.1 as i32 - target_coords.1 as i32).abs();
    
    // Adjacent means distance of 1 in any direction (including diagonals)
    let is_adjacent = (row_diff <= 1) && (col_diff <= 1) && (row_diff + col_diff > 0);
    
    Ok(is_adjacent)
}

fn evaluate_entity_selected(
    _with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    _args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    // Check if player has an entity selected
    let has_selection = state.get("selection")
        .and_then(|s| s.get(current_actor))
        .is_some();
    
    Ok(has_selection)
}

fn parse_grid_location(location: &str) -> Result<(usize, usize), String> {
    // Parse "/zones/board/cells/row/col" format
    let parts: Vec<&str> = location.split('/').filter(|p| !p.is_empty()).collect();
    
    if parts.len() < 5 || parts[0] != "zones" || parts[2] != "cells" {
        return Err(format!("Invalid grid location format: {}", location));
    }
    
    let row = parts[3].parse::<usize>()
        .map_err(|_| format!("Invalid row in location: {}", location))?;
    let col = parts[4].parse::<usize>()
        .map_err(|_| format!("Invalid col in location: {}", location))?;
    
    Ok((row, col))
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::evaluate_condition;

    #[test]
    fn test_zone_is_empty() {
        let state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [null, {"entity": "piece_p1"}, null],
                        [null, null, null],
                        [{"entity": "piece_p2"}, null, null]
                    ]
                }
            }
        });

        // Test empty cell
        let condition = json!({
            "condition": "zone.isEmpty",
            "with": {
                "zone": "{target}"
            }
        });
        let args = json!({
            "location": "/zones/board/cells/0/0"
        });
        assert_eq!(evaluate_condition(&condition, &state, &args, "p1").unwrap(), true);

        // Test non-empty cell
        let args = json!({
            "location": "/zones/board/cells/0/1"
        });
        assert_eq!(evaluate_condition(&condition, &state, &args, "p1").unwrap(), false);
    }

    #[test]
    fn test_player_is_actor() {
        let state = json!({
            "currentPlayer": "p1"
        });

        let condition = json!({
            "condition": "player.isActor"
        });

        // Current player is actor
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);

        // Current player is not actor
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p2").unwrap(), false);
    }

    #[test]
    fn test_zone_count() {
        let state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [null, {"entity": "piece_p1"}, null],
                        [null, {"entity": "piece_p1"}, null],
                        [{"entity": "piece_p2"}, null, {"entity": "piece_p1"}]
                    ]
                }
            }
        });

        // Count all pieces for p1
        let condition = json!({
            "condition": "zone.count",
            "with": {
                "zone": "/zones/board",
                "entity": "piece_p1",
                "operator": "==",
                "value": 3
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);

        // Count with >= operator
        let condition = json!({
            "condition": "zone.count",
            "with": {
                "zone": "/zones/board",
                "entity": "piece_p1",
                "operator": ">=",
                "value": 3
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);

        // Count pieces for p2
        let condition = json!({
            "condition": "zone.count",
            "with": {
                "zone": "/zones/board",
                "entity": "piece_p2",
                "operator": "==",
                "value": 1
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p2").unwrap(), true);

        // Count with pattern matching
        let condition = json!({
            "condition": "zone.count",
            "with": {
                "zone": "/zones/board",
                "entity": "piece_{player}",
                "operator": ">=",
                "value": 1
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);
    }

    #[test]
    fn test_entity_owner() {
        let condition = json!({
            "condition": "entity.owner",
            "with": {
                "entity": "{selected}",
                "owner": "{player}"
            }
        });

        // Test p1's piece
        let args = json!({
            "entity": "piece_p1"
        });
        assert_eq!(evaluate_condition(&condition, &json!({}), &args, "p1").unwrap(), true);
        assert_eq!(evaluate_condition(&condition, &json!({}), &args, "p2").unwrap(), false);

        // Test p2's piece
        let args = json!({
            "entity": "piece_p2"
        });
        assert_eq!(evaluate_condition(&condition, &json!({}), &args, "p1").unwrap(), false);
        assert_eq!(evaluate_condition(&condition, &json!({}), &args, "p2").unwrap(), true);
    }

    #[test]
    fn test_phase_is() {
        let state = json!({
            "phases": {
                "game": "placement",
                "turn": "draw"
            }
        });

        // Test correct phase
        let condition = json!({
            "condition": "phase.is",
            "with": {
                "phaseSet": "game",
                "phase": "placement"
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);

        // Test incorrect phase
        let condition = json!({
            "condition": "phase.is",
            "with": {
                "phaseSet": "game",
                "phase": "movement"
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), false);

        // Test different phase set
        let condition = json!({
            "condition": "phase.is",
            "with": {
                "phaseSet": "turn",
                "phase": "draw"
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);
    }

    #[test]
    fn test_resource_value() {
        let state = json!({
            "zones": {
                "gold_p1": {
                    "type": "resource",
                    "value": 10
                },
                "gold_p2": {
                    "type": "resource",
                    "value": 5
                }
            }
        });

        // Test exact value
        let condition = json!({
            "condition": "resource.value",
            "with": {
                "resource": "gold_{player}",
                "operator": "==",
                "value": 10
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p2").unwrap(), false);

        // Test >= operator
        let condition = json!({
            "condition": "resource.value",
            "with": {
                "resource": "gold_{player}",
                "operator": ">=",
                "value": 5
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p2").unwrap(), true);
    }
}
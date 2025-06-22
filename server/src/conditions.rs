use serde_json::{json, Value};

// Replace template variables in condition parameters
fn replace_condition_templates(value: &Value, state: &Value, current_actor: &str, args: &Value) -> Value {
    match value {
        Value::String(s) => {
            let mut result = s.clone();
            
            // Replace {player} with current actor
            if result.contains("{player}") {
                result = result.replace("{player}", current_actor);
            }
            
            // Replace {actor} with current actor
            if result.contains("{actor}") {
                result = result.replace("{actor}", current_actor);
            }
            
            // Replace {args.*} with values from args
            if result.contains("{args.") {
                // Find all {args.X} patterns
                let re = regex::Regex::new(r"\{args\.([^}]+)\}").unwrap();
                for cap in re.captures_iter(&result.clone()) {
                    if let Some(arg_name) = cap.get(1) {
                        if let Some(arg_value) = args.get(arg_name.as_str()) {
                            if let Some(val_str) = arg_value.as_str() {
                                result = result.replace(&cap[0], val_str);
                            }
                        }
                    }
                }
            }
            
            // Replace {target} with location from args
            if result.contains("{target}") {
                if let Some(location) = args.get("location").and_then(|l| l.as_str()) {
                    result = result.replace("{target}", location);
                }
            }
            
            Value::String(result)
        }
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), replace_condition_templates(v, state, current_actor, args));
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| replace_condition_templates(v, state, current_actor, args)).collect())
        }
        _ => value.clone()
    }
}

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
    
    // Replace templates in the 'with' object
    let with_value = condition.get("with").cloned().unwrap_or(json!({}));
    let processed_with = replace_condition_templates(&with_value, state, current_actor, args);
    let with_obj = processed_with.as_object();
    
    // Debug log
    if condition_type == "zone.isEmpty" {
        println!("[DEBUG evaluate_condition] Original with: {:?}", with_value);
        println!("[DEBUG evaluate_condition] Processed with: {:?}", processed_with);
        println!("[DEBUG evaluate_condition] Args: {:?}", args);
    }
    
    let result = match condition_type {
        "zone.isEmpty" => evaluate_zone_is_empty(with_obj, state, args),
        "player.isActor" => evaluate_player_is_actor(state, current_actor),
        "zone.count" => evaluate_zone_count(with_obj, state, current_actor),
        "entity.owner" => evaluate_entity_owner(with_obj, state, args, current_actor),
        "phase.is" => evaluate_phase_is(with_obj, state),
        "resource.value" => evaluate_resource_value(with_obj, state, current_actor),
        "entity.selected" => evaluate_entity_selected(with_obj, state, args, current_actor),
        "zone.hasMatching" => evaluate_zone_has_matching(with_obj, state, current_actor),
        "zone.countWhere" => evaluate_zone_count_where(with_obj, state, current_actor),
        "game.notEnded" => evaluate_game_not_ended(state),
        "value.notEquals" => evaluate_value_not_equals(with_obj, args, current_actor),
        "valueEquals" => evaluate_value_equals(condition, with_obj, state, args, current_actor),
        "entity.hasProperty" => evaluate_entity_has_property(with_obj, args),
        "movement.adjacent" => evaluate_movement_adjacent(with_obj, state, args, current_actor),
        "movement.orthogonal" => evaluate_movement_orthogonal(with_obj, state, args, current_actor),
        "movement.diagonal" => evaluate_movement_diagonal(with_obj, state, args, current_actor),
        "movement.inLine" => evaluate_movement_in_line(with_obj, state, args, current_actor),
        "path.clear" => evaluate_path_clear(with_obj, state, args),
        "grid.inBounds" => evaluate_grid_in_bounds(with_obj, state, args),
        "grid.distance" => evaluate_grid_distance(with_obj, state, args, current_actor),
        "numeric.compare" => evaluate_numeric_compare(with_obj, state, current_actor),
        "grid.lineOfMarks" => evaluate_grid_line_of_marks(with_obj, state, current_actor),
        _ if condition_type.starts_with("phase.") && condition_type.ends_with(".isActive") => {
            // Handle dynamic phase conditions like "phase.game.playing.isActive"
            evaluate_dynamic_phase_condition(condition_type, state)
        },
        _ => Err(format!("Unknown condition type: {}", condition_type)),
    }?;
    
    // Check if the condition should be negated
    if condition.get("negate").and_then(|n| n.as_bool()).unwrap_or(false) {
        Ok(!result)
    } else {
        Ok(result)
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
    
    // Debug logging removed
    
    // Get current actor from args (if available)
    let current_actor = args.get("player").and_then(|p| p.as_str()).unwrap_or("");
    
    // The template should already be replaced, so use it directly
    let mut zone_path = zone_template.to_string();
    
    // Replace {player} with current actor
    if zone_path.contains("{player}") {
        zone_path = zone_path.replace("{player}", current_actor);
    }
    
    // Check if we need to check all cells/items
    let check_all = with_obj.get("checkAll").and_then(|v| v.as_bool()).unwrap_or(false);
    
    // Check if this is a conditional empty check (e.g., no cards matching certain criteria)
    if let Some(matching) = with_obj.get("matching") {
        // This is checking if the zone has no items matching certain criteria
        if let Some(zone_value) = get_value_at_path(state, &zone_path) {
            if let Some(items) = zone_value.get("items").and_then(|i| i.as_array()) {
                // Check if we have anyOf conditions
                if let Some(any_of) = matching.get("anyOf").and_then(|a| a.as_array()) {
                    // Return true if NO items match ANY of the conditions
                    for item in items {
                        if let Some(entity_id) = item.get("entity").and_then(|e| e.as_str()) {
                            // Check each condition in anyOf
                            for condition in any_of {
                                if let Some(condition_obj) = condition.as_object() {
                                    if check_single_matching_condition(entity_id, condition_obj, state)? {
                                        // Found a matching item, so zone is NOT empty of matching items
                                        return Ok(false);
                                    }
                                }
                            }
                        }
                    }
                    // No items matched any condition, so zone IS empty of matching items
                    Ok(true)
                } else {
                    // Handle simple matching (legacy)
                    Ok(items.is_empty())
                }
            } else {
                Ok(true) // Zone has no items array, so it's empty
            }
        } else {
            Ok(true) // Zone doesn't exist, so it's empty
        }
    } else if check_all {
        // When checkAll is true, check if ALL cells/items are non-empty
        // This is used for tie detection - we want to know if the board is FULL
        if let Some(zone_value) = get_value_at_path(state, &zone_path) {
            if let Some(cells) = zone_value.get("cells").and_then(|c| c.as_array()) {
                // For grid zones, check if all cells are non-empty
                let mut total_cells = 0;
                let mut filled_cells = 0;
                
                for row in cells {
                    if let Some(row_array) = row.as_array() {
                        for cell in row_array {
                            total_cells += 1;
                            if !cell.is_null() && cell.is_object() && cell.get("entity").is_some() {
                                filled_cells += 1;
                            }
                        }
                    }
                }
                
                println!("[DEBUG zone.isEmpty checkAll] Zone: {}, total cells: {}, filled cells: {}", 
                    zone_path, total_cells, filled_cells);
                
                // Zone is empty if not all cells are filled
                let is_empty = filled_cells < total_cells;
                println!("[DEBUG zone.isEmpty checkAll] Is empty: {}", is_empty);
                Ok(is_empty)
            } else if let Some(items) = zone_value.get("items").and_then(|i| i.as_array()) {
                // For list zones, check if empty
                Ok(items.is_empty())
            } else {
                Ok(true) // Unknown zone type, consider empty
            }
        } else {
            Ok(true) // Zone doesn't exist, so it's empty
        }
    } else {
        // Check if the zone/cell is actually empty
        let is_empty = if zone_path.contains("/cells/") {
            // For grid cells, check if the cell is null or has no entity
            get_value_at_path(state, &zone_path)
                .map(|v| v.is_null() || (v.is_object() && v.get("entity").is_none()))
                .unwrap_or(false)
        } else {
            // For other zones, check if empty
            if let Some(zone_value) = get_value_at_path(state, &zone_path) {
                if let Some(items) = zone_value.get("items").and_then(|i| i.as_array()) {
                    items.is_empty()
                } else if let Some(cells) = zone_value.get("cells").and_then(|c| c.as_array()) {
                    // For grid zones, check if all cells are empty
                    cells.iter().all(|row| {
                        row.as_array()
                            .map(|r| r.iter().all(|cell| {
                                // A cell is empty if it's null OR if it doesn't have an entity
                                cell.is_null() || 
                                (cell.is_object() && cell.get("entity").is_none())
                            }))
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
}

fn evaluate_player_is_actor(state: &Value, current_actor: &str) -> Result<bool, String> {
    let current_player = state.get("currentPlayer")
        .and_then(|cp| cp.as_str())
        .unwrap_or("");
    
    println!("[DEBUG player.isActor] currentPlayer: {}, current_actor: {}, matches: {}", 
        current_player, current_actor, current_player == current_actor);
    
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
        } else if entity_template == "{entityAtLocation}" || entity_template == "{entity}" {
            // First check if entityAtLocation is directly provided in args
            if let Some(entity_from_args) = args.get("entityAtLocation").and_then(|e| e.as_str()) {
                println!("[DEBUG entity.owner] Using entityAtLocation from args: {}", entity_from_args);
                entity_from_args.to_string()
            } else {
                // Otherwise look up the entity at the provided location
                // Try both "location" and "target" fields since some actions use different field names
                let location = args.get("location").and_then(|l| l.as_str())
                    .or_else(|| args.get("target").and_then(|t| t.as_str()));
                    
                if let Some(location) = location {
                    println!("[DEBUG entity.owner] Looking for entity at location: {}", location);
                    
                    // Parse the location path to get zone and coordinates
                    let parts: Vec<&str> = location.split('/').collect();
                    if parts.len() >= 5 && parts[1] == "zones" && parts[3] == "cells" {
                        let zone_id = parts[2];
                        if let (Ok(row), Ok(col)) = (parts[4].parse::<usize>(), parts[5].parse::<usize>()) {
                            // Navigate to the cell in the state - check both 'zones' and 'game/zones' paths
                            let cell_value = state
                                .get("zones")
                                .and_then(|zones| zones.get(zone_id))
                                .and_then(|zone| zone.get("cells"))
                                .and_then(|cells| cells.get(row))
                                .and_then(|row_cells| row_cells.get(col))
                                .or_else(|| {
                                    // Try game/zones path as fallback
                                    state
                                        .get("game")
                                        .and_then(|game| game.get("zones"))
                                        .and_then(|zones| zones.get(zone_id))
                                        .and_then(|zone| zone.get("cells"))
                                        .and_then(|cells| cells.get(row))
                                        .and_then(|row_cells| row_cells.get(col))
                                });
                                
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
                    return Err("Missing location or target in args".to_string());
                }
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
    
    // Check if using enhanced phase system
    if let Some(current_phases) = state["phases"]["current"].as_object() {
        // Enhanced phase system
        let current_phase = current_phases.get(phase_set)
            .and_then(|p| p.as_str())
            .unwrap_or("");
        println!("[DEBUG phase.is] Enhanced system - phaseSet: {}, current: {}, expected: {}, match: {}", 
                phase_set, current_phase, expected_phase, current_phase == expected_phase);
        Ok(current_phase == expected_phase)
    } else {
        // Legacy phase system
        let current_phase = state.get("phases")
            .and_then(|phases| phases.get(phase_set))
            .and_then(|phase| phase.as_str())
            .unwrap_or("");
        println!("[DEBUG phase.is] Legacy system - phaseSet: {}, current: {}, expected: {}, match: {}", 
                phase_set, current_phase, expected_phase, current_phase == expected_phase);
        Ok(current_phase == expected_phase)
    }
}

fn evaluate_dynamic_phase_condition(
    condition_type: &str,
    state: &Value,
) -> Result<bool, String> {
    // Parse condition format: "phase.{set}.{phase}.isActive"
    let parts: Vec<&str> = condition_type.split('.').collect();
    if parts.len() != 4 || parts[0] != "phase" || parts[3] != "isActive" {
        return Err(format!("Invalid phase condition format: {}", condition_type));
    }
    
    let phase_set = parts[1];
    let phase_id = parts[2];
    
    // Check if using enhanced phase system
    if let Some(current_phases) = state["phases"]["current"].as_object() {
        // Enhanced phase system
        let current_phase = current_phases.get(phase_set)
            .and_then(|p| p.as_str())
            .unwrap_or("");
        Ok(current_phase == phase_id)
    } else {
        // Legacy phase system
        let current_phase = state["phases"][phase_set]
            .as_str()
            .unwrap_or("");
        Ok(current_phase == phase_id)
    }
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
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    // Get source and target locations
    let (source_location, target_location) = get_movement_locations(with_obj, state, args, current_actor)?;
    
    // Parse both locations to get coordinates
    let source_coords = parse_grid_location(&source_location)?;
    let target_coords = parse_grid_location(&target_location)?;
    
    // Check if locations are adjacent (including diagonals by default)
    let row_diff = (source_coords.0 as i32 - target_coords.0 as i32).abs();
    let col_diff = (source_coords.1 as i32 - target_coords.1 as i32).abs();
    
    // Check if diagonal movement is allowed (default: true)
    let allow_diagonal = with_obj
        .and_then(|obj| obj.get("allowDiagonal"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let is_adjacent = if allow_diagonal {
        // Adjacent means distance of 1 in any direction (including diagonals)
        (row_diff <= 1) && (col_diff <= 1) && (row_diff + col_diff > 0)
    } else {
        // Adjacent means orthogonally adjacent only
        (row_diff == 1 && col_diff == 0) || (row_diff == 0 && col_diff == 1)
    };
    
    Ok(is_adjacent)
}

fn evaluate_movement_orthogonal(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    // Get source and target locations
    let (source_location, target_location) = get_movement_locations(with_obj, state, args, current_actor)?;
    
    // Parse both locations to get coordinates
    let source_coords = parse_grid_location(&source_location)?;
    let target_coords = parse_grid_location(&target_location)?;
    
    // Check if movement is orthogonal (horizontal or vertical)
    let is_orthogonal = (source_coords.0 == target_coords.0) || (source_coords.1 == target_coords.1);
    let is_same_position = source_coords == target_coords;
    
    Ok(is_orthogonal && !is_same_position)
}

fn evaluate_movement_diagonal(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    // Get source and target locations
    let (source_location, target_location) = get_movement_locations(with_obj, state, args, current_actor)?;
    
    // Parse both locations to get coordinates
    let source_coords = parse_grid_location(&source_location)?;
    let target_coords = parse_grid_location(&target_location)?;
    
    // Check if movement is diagonal
    let row_diff = (source_coords.0 as i32 - target_coords.0 as i32).abs();
    let col_diff = (source_coords.1 as i32 - target_coords.1 as i32).abs();
    
    // Diagonal means equal row and column distance
    Ok(row_diff == col_diff && row_diff > 0)
}

fn evaluate_movement_in_line(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    // Get source and target locations
    let (source_location, target_location) = get_movement_locations(with_obj, state, args, current_actor)?;
    
    // Parse both locations to get coordinates
    let source_coords = parse_grid_location(&source_location)?;
    let target_coords = parse_grid_location(&target_location)?;
    
    // Check if movement is in a straight line (orthogonal or diagonal)
    let row_diff = (source_coords.0 as i32 - target_coords.0 as i32).abs();
    let col_diff = (source_coords.1 as i32 - target_coords.1 as i32).abs();
    
    // In line means orthogonal or diagonal
    let is_orthogonal = (source_coords.0 == target_coords.0) || (source_coords.1 == target_coords.1);
    let is_diagonal = row_diff == col_diff;
    let is_same_position = source_coords == target_coords;
    
    Ok((is_orthogonal || is_diagonal) && !is_same_position)
}

fn evaluate_path_clear(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for path.clear")?;
    
    // Get source and target from args or with object
    let source = if let Some(src) = with_obj.get("from").and_then(|s| s.as_str()) {
        src.to_string()
    } else if let Some(src) = args.get("from").and_then(|s| s.as_str()) {
        src.to_string()
    } else {
        return Err("Missing 'from' location for path.clear".to_string());
    };
    
    let target = if let Some(tgt) = with_obj.get("to").and_then(|s| s.as_str()) {
        tgt.to_string()
    } else if let Some(tgt) = args.get("target").and_then(|s| s.as_str()) {
        tgt.to_string()
    } else if let Some(tgt) = args.get("to").and_then(|s| s.as_str()) {
        tgt.to_string()
    } else {
        return Err("Missing 'to' location for path.clear".to_string());
    };
    
    // Parse locations
    let source_coords = parse_grid_location(&source)?;
    let target_coords = parse_grid_location(&target)?;
    
    // Calculate path between source and target
    let path_cells = calculate_path(source_coords, target_coords)?;
    
    // Check if all cells in the path are empty (excluding source and target)
    for (i, (row, col)) in path_cells.iter().enumerate() {
        // Skip source and optionally target
        if i == 0 {
            continue; // Skip source
        }
        if i == path_cells.len() - 1 && with_obj.get("excludeTarget").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue; // Skip target if excludeTarget is true
        }
        
        // Build cell path
        let cell_path = format!("/zones/board/cells/{}/{}", row, col);
        
        // Check if cell is empty
        if let Some(cell_value) = get_value_at_path(state, &cell_path) {
            if !cell_value.is_null() {
                return Ok(false); // Path is blocked
            }
        }
    }
    
    Ok(true)
}

fn evaluate_grid_in_bounds(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for grid.inBounds")?;
    
    // Get location from args or with object
    let location = if let Some(loc) = args.get("target").and_then(|s| s.as_str()) {
        loc
    } else if let Some(loc) = args.get("location").and_then(|s| s.as_str()) {
        loc
    } else if let Some(loc) = with_obj.get("location").and_then(|s| s.as_str()) {
        loc
    } else {
        return Err("Missing location for grid.inBounds".to_string());
    };
    
    // Get grid dimensions from state or with object
    let grid_zone = with_obj.get("zone")
        .and_then(|z| z.as_str())
        .unwrap_or("/zones/board");
    
    if let Some(zone_value) = get_value_at_path(state, grid_zone) {
        if let Some(cells) = zone_value.get("cells").and_then(|c| c.as_array()) {
            let rows = cells.len();
            let cols = cells.get(0)
                .and_then(|row| row.as_array())
                .map(|r| r.len())
                .unwrap_or(0);
            
            // Try to parse location
            if let Ok((row, col)) = parse_grid_location(location) {
                return Ok(row < rows && col < cols);
            }
        }
    }
    
    Ok(false)
}

fn evaluate_grid_distance(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for grid.distance")?;
    
    // Get source and target locations
    let (source_location, target_location) = get_movement_locations(Some(with_obj), state, args, current_actor)?;
    
    // Parse locations
    let source_coords = parse_grid_location(&source_location)?;
    let target_coords = parse_grid_location(&target_location)?;
    
    // Calculate distance based on type
    let distance_type = with_obj.get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("euclidean");
    
    let distance = match distance_type {
        "manhattan" => {
            // Manhattan distance (orthogonal moves only)
            (source_coords.0 as i32 - target_coords.0 as i32).abs() +
            (source_coords.1 as i32 - target_coords.1 as i32).abs()
        }
        "chebyshev" => {
            // Chebyshev distance (king moves in chess)
            let row_diff = (source_coords.0 as i32 - target_coords.0 as i32).abs();
            let col_diff = (source_coords.1 as i32 - target_coords.1 as i32).abs();
            row_diff.max(col_diff)
        }
        _ => {
            // Euclidean distance (squared, to avoid floating point)
            let row_diff = (source_coords.0 as i32 - target_coords.0 as i32).abs();
            let col_diff = (source_coords.1 as i32 - target_coords.1 as i32).abs();
            row_diff * row_diff + col_diff * col_diff
        }
    };
    
    // Get operator and value
    let operator = with_obj.get("operator")
        .and_then(|o| o.as_str())
        .unwrap_or("==");
    let value = with_obj.get("value")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;
    
    // For euclidean distance, square the comparison value
    let value = if distance_type == "euclidean" {
        value * value
    } else {
        value
    };
    
    // Apply operator
    match operator {
        "==" => Ok(distance == value),
        "!=" => Ok(distance != value),
        ">" => Ok(distance > value),
        "<" => Ok(distance < value),
        ">=" => Ok(distance >= value),
        "<=" => Ok(distance <= value),
        _ => Err(format!("Unknown operator: {}", operator)),
    }
}

// Helper function to get movement source and target locations
fn get_movement_locations(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<(String, String), String> {
    // Get source location
    let source_location = if let Some(obj) = with_obj {
        if let Some(from) = obj.get("from").and_then(|f| f.as_str()) {
            // Explicit from in condition
            from.to_string()
        } else if obj.get("useSelection").and_then(|u| u.as_bool()).unwrap_or(true) {
            // Use player's selection (default behavior)
            state.get("selection")
                .and_then(|s| s.get(current_actor))
                .and_then(|s| s.get("location"))
                .and_then(|l| l.as_str())
                .ok_or("No selection found for player")?
                .to_string()
        } else {
            // Must have from in args
            args.get("from")
                .and_then(|f| f.as_str())
                .ok_or("Missing source location")?
                .to_string()
        }
    } else {
        // Default: use selection
        state.get("selection")
            .and_then(|s| s.get(current_actor))
            .and_then(|s| s.get("location"))
            .and_then(|l| l.as_str())
            .ok_or("No selection found for player")?
            .to_string()
    };
    
    // Get target location
    let target_location = if let Some(obj) = with_obj {
        if let Some(to) = obj.get("to").and_then(|t| t.as_str()) {
            to.to_string()
        } else {
            args.get("target")
                .or_else(|| args.get("to"))
                .or_else(|| args.get("location"))
                .and_then(|t| t.as_str())
                .ok_or("Missing target location")?
                .to_string()
        }
    } else {
        args.get("target")
            .or_else(|| args.get("to"))
            .or_else(|| args.get("location"))
            .and_then(|t| t.as_str())
            .ok_or("Missing target location")?
            .to_string()
    };
    
    Ok((source_location, target_location))
}

// Calculate path between two grid coordinates
fn calculate_path(
    source: (usize, usize),
    target: (usize, usize),
) -> Result<Vec<(usize, usize)>, String> {
    let mut path = vec![source];
    
    let row_diff = target.0 as i32 - source.0 as i32;
    let col_diff = target.1 as i32 - source.1 as i32;
    
    // Determine if movement is orthogonal or diagonal
    if row_diff == 0 || col_diff == 0 {
        // Orthogonal movement
        let steps = row_diff.abs().max(col_diff.abs());
        let row_step = if row_diff == 0 { 0 } else { row_diff / row_diff.abs() };
        let col_step = if col_diff == 0 { 0 } else { col_diff / col_diff.abs() };
        
        for i in 1..=steps {
            let row = (source.0 as i32 + i * row_step) as usize;
            let col = (source.1 as i32 + i * col_step) as usize;
            path.push((row, col));
        }
    } else if row_diff.abs() == col_diff.abs() {
        // Diagonal movement
        let steps = row_diff.abs();
        let row_step = row_diff / row_diff.abs();
        let col_step = col_diff / col_diff.abs();
        
        for i in 1..=steps {
            let row = (source.0 as i32 + i * row_step) as usize;
            let col = (source.1 as i32 + i * col_step) as usize;
            path.push((row, col));
        }
    } else {
        return Err("Path is not in a straight line".to_string());
    }
    
    Ok(path)
}

fn evaluate_entity_selected(
    _with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    _args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    // Check if player has an entity selected - check both possible paths
    let selection = state.get("selection")
        .or_else(|| state.get("game").and_then(|g| g.get("selection")));
    
    let has_selection = selection
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
    fn test_entity_owner_at_location() {
        let state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [{"entity": "piece_p1"}, {"entity": "piece_p2"}, null],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            }
        });

        let condition = json!({
            "condition": "entity.owner",
            "with": {
                "entity": "{entityAtLocation}",
                "owner": "{player}"
            }
        });

        // Test with location field
        let args_with_location = json!({
            "location": "/zones/board/cells/0/0"
        });
        assert_eq!(evaluate_condition(&condition, &state, &args_with_location, "p1").unwrap(), true);
        assert_eq!(evaluate_condition(&condition, &state, &args_with_location, "p2").unwrap(), false);

        // Test with target field (should work as fallback)
        let args_with_target = json!({
            "target": "/zones/board/cells/0/1"
        });
        assert_eq!(evaluate_condition(&condition, &state, &args_with_target, "p1").unwrap(), false);
        assert_eq!(evaluate_condition(&condition, &state, &args_with_target, "p2").unwrap(), true);

        // Test with empty cell
        let args_empty = json!({
            "target": "/zones/board/cells/0/2"
        });
        assert!(evaluate_condition(&condition, &state, &args_empty, "p1").is_err());
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

    #[test]
    fn test_grid_line_of_marks() {
        // Create a 3x3 board with diagonal win for p1
        let state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [{"entity": "mark_p1"}, {"entity": "mark_p2"}, null],
                        [null, {"entity": "mark_p1"}, null],
                        [null, null, {"entity": "mark_p1"}]
                    ]
                }
            }
        });

        // Test diagonal win for p1
        let condition = json!({
            "condition": "grid.lineOfMarks",
            "with": {
                "zone": "zones/board",
                "entity": "mark_{player}",
                "lineLength": 3,
                "directions": ["diagonal"]
            }
        });
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p1").unwrap(), true);
        assert_eq!(evaluate_condition(&condition, &state, &json!({}), "p2").unwrap(), false);

        // Test horizontal win
        let state_horizontal = json!({
            "zones": {
                "board": {
                    "cells": [
                        [{"entity": "mark_p1"}, {"entity": "mark_p1"}, {"entity": "mark_p1"}],
                        [null, {"entity": "mark_p2"}, null],
                        [null, null, {"entity": "mark_p2"}]
                    ]
                }
            }
        });

        let condition_horizontal = json!({
            "condition": "grid.lineOfMarks",
            "with": {
                "zone": "zones/board",
                "entity": "mark_{player}",
                "lineLength": 3,
                "directions": ["horizontal"]
            }
        });
        assert_eq!(evaluate_condition(&condition_horizontal, &state_horizontal, &json!({}), "p1").unwrap(), true);
        assert_eq!(evaluate_condition(&condition_horizontal, &state_horizontal, &json!({}), "p2").unwrap(), false);
    }
}

// Helper function to check a single matching condition
pub fn check_single_matching_condition(
    entity_id: &str,
    condition: &serde_json::Map<String, Value>,
    state: &Value,
) -> Result<bool, String> {
    // Extract property from the condition
    let property = condition.get("property")
        .and_then(|p| p.as_str())
        .ok_or("Missing property in matching condition")?;
    
    println!("[DEBUG check_single_matching] Checking entity {} for property {}", entity_id, property);
    
    // Check if this is a matchesTop condition
    if let Some(matches_top_zone) = condition.get("matchesTop").and_then(|m| m.as_str()) {
        println!("[DEBUG check_single_matching] Checking matchesTop against zone: {}", matches_top_zone);
        // Get the top card from the specified zone
        let top_zone = get_value_at_path(state, matches_top_zone)
            .ok_or_else(|| format!("Zone not found for matchesTop: {}", matches_top_zone))?;
        
        if let Some(items) = top_zone.get("items").and_then(|i| i.as_array()) {
            if let Some(top_item) = items.last() {
                if let Some(top_entity_id) = top_item.get("entity").and_then(|e| e.as_str()) {
                    println!("[DEBUG check_single_matching] Top card in {} is: {}", matches_top_zone, top_entity_id);
                    // Compare properties between entity and top card
                    if entity_id.starts_with("card_") && top_entity_id.starts_with("card_") {
                        let entity_parts: Vec<&str> = entity_id.split('_').collect();
                        let top_parts: Vec<&str> = top_entity_id.split('_').collect();
                        
                        if entity_parts.len() >= 3 && top_parts.len() >= 3 {
                            let (entity_value, top_value) = match property {
                                "rank" => (entity_parts[2], top_parts[2]),
                                "suit" => (entity_parts[1], top_parts[1]),
                                _ => return Ok(false),
                            };
                            println!("[DEBUG check_single_matching] Comparing {} vs {} for property {}", entity_value, top_value, property);
                            return Ok(entity_value == top_value);
                        }
                    }
                }
            } else {
                println!("[DEBUG check_single_matching] No items in zone {}", matches_top_zone);
            }
        }
        Ok(false)
    } else if let Some(value) = condition.get("value").and_then(|v| v.as_str()) {
        // Simple value matching
        if entity_id.starts_with("card_") {
            let parts: Vec<&str> = entity_id.split('_').collect();
            if parts.len() >= 3 {
                let entity_value = match property {
                    "rank" => parts[2],
                    "suit" => parts[1],
                    _ => return Ok(false),
                };
                Ok(entity_value == value)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    } else {
        Err("Matching condition must have either 'value' or 'matchesTop'".to_string())
    }
}

fn evaluate_zone_has_matching(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for zone.hasMatching")?;
    let zone_path = with_obj.get("zone")
        .and_then(|z| z.as_str())
        .ok_or("Missing zone in condition")?;
    
    // Replace {player} with current actor
    let zone_path_processed = zone_path.replace("{player}", current_actor);
    
    // Get zone from state
    let zone_value = get_value_at_path(state, &zone_path_processed)
        .ok_or_else(|| format!("Zone not found: {}", zone_path_processed))?;
    
    // Check if we have anyOf conditions
    if let Some(any_of) = with_obj.get("anyOf").and_then(|a| a.as_array()) {
        println!("[DEBUG zone.hasMatching] Processing anyOf conditions for zone: {}", zone_path_processed);
        // Handle anyOf - return true if ANY condition matches
        if let Some(items) = zone_value.get("items").and_then(|i| i.as_array()) {
            println!("[DEBUG zone.hasMatching] Zone has {} items", items.len());
            for item in items {
                if let Some(entity_id) = item.get("entity").and_then(|e| e.as_str()) {
                    println!("[DEBUG zone.hasMatching] Checking entity: {}", entity_id);
                    // Check each condition in anyOf
                    for condition in any_of {
                        if let Some(condition_obj) = condition.as_object() {
                            println!("[DEBUG zone.hasMatching] Checking condition: {:?}", condition_obj);
                            match check_single_matching_condition(entity_id, condition_obj, state) {
                                Ok(true) => {
                                    println!("[DEBUG zone.hasMatching] Found match!");
                                    return Ok(true);
                                }
                                Ok(false) => {
                                    println!("[DEBUG zone.hasMatching] No match");
                                }
                                Err(e) => {
                                    println!("[DEBUG zone.hasMatching] Error checking condition: {}", e);
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("[DEBUG zone.hasMatching] No matches found in anyOf conditions");
        Ok(false)
    } else {
        // Handle simple property/value matching (legacy)
        let property = with_obj.get("property")
            .and_then(|p| p.as_str())
            .ok_or("Missing property in condition")?;
        let value = with_obj.get("value")
            .and_then(|v| v.as_str())
            .ok_or("Missing value in condition")?;
        
        // Check if zone has any items matching the property value
        if let Some(items) = zone_value.get("items").and_then(|i| i.as_array()) {
            for item in items {
                if let Some(entity_id) = item.get("entity").and_then(|e| e.as_str()) {
                    // For card entities, extract property from entity ID
                    if entity_id.starts_with("card_") {
                        let parts: Vec<&str> = entity_id.split('_').collect();
                        if parts.len() >= 3 {
                            let entity_value = match property {
                                "rank" => parts[2],  // rank is the third part: card_suit_rank
                                "suit" => parts[1],  // suit is the second part: card_suit_rank
                                _ => continue,
                            };
                            if entity_value == value {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }
}

fn evaluate_zone_count_where(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for zone.countWhere")?;
    let zone_path = with_obj.get("zone")
        .and_then(|z| z.as_str())
        .ok_or("Missing zone in condition")?;
    let property = with_obj.get("property")
        .and_then(|p| p.as_str())
        .ok_or("Missing property in condition")?;
    let value = with_obj.get("value")
        .and_then(|v| v.as_str())
        .ok_or("Missing value in condition")?;
    let operator = with_obj.get("operator")
        .and_then(|o| o.as_str())
        .ok_or("Missing operator in condition")?;
    let count_value = with_obj.get("count")
        .and_then(|c| c.as_u64())
        .ok_or("Missing count in condition")? as usize;
    
    // Replace {player} with current actor
    let zone_path_processed = zone_path.replace("{player}", current_actor);
    
    // Get zone from state
    let zone_value = get_value_at_path(state, &zone_path_processed)
        .ok_or_else(|| format!("Zone not found: {}", zone_path_processed))?;
    
    // Count items matching the property value
    let mut count = 0;
    if let Some(items) = zone_value.get("items").and_then(|i| i.as_array()) {
        for item in items {
            if let Some(entity_id) = item.get("entity").and_then(|e| e.as_str()) {
                // For card entities, extract property from entity ID
                if entity_id.starts_with("card_") {
                    let parts: Vec<&str> = entity_id.split('_').collect();
                    if parts.len() >= 3 {
                        let entity_value = match property {
                            "rank" => parts[2],  // rank is the third part: card_suit_rank
                            "suit" => parts[1],  // suit is the second part: card_suit_rank
                            _ => continue,
                        };
                        if entity_value.to_lowercase() == value.to_lowercase() {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    
    // Apply operator
    let result = match operator {
        "==" => count == count_value,
        "!=" => count != count_value,
        ">" => count > count_value,
        "<" => count < count_value,
        ">=" => count >= count_value,
        "<=" => count <= count_value,
        _ => return Err(format!("Unknown operator: {}", operator)),
    };
    
    Ok(result)
}

fn evaluate_game_not_ended(state: &Value) -> Result<bool, String> {
    // Check if gameStatus exists at root or under game
    let game_status = state.get("gameStatus")
        .or_else(|| state.get("game").and_then(|g| g.get("gameStatus")));
    
    if let Some(status) = game_status {
        // New simplified format: gameStatus is a string
        if let Some(status_str) = status.as_str() {
            // Game is ended if status is anything other than "playing"
            Ok(status_str == "playing")
        } else if let Some(status_state) = status.get("state").and_then(|s| s.as_str()) {
            // Legacy format support
            Ok(status_state != "ended")
        } else {
            // If neither format matches, assume game is not ended
            Ok(true)
        }
    } else {
        // If gameStatus doesn't exist, game is not ended
        Ok(true)
    }
}

fn evaluate_value_not_equals(
    with_obj: Option<&serde_json::Map<String, Value>>,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for value.notEquals")?;
    
    // Get the first value (which might be from args)
    let value1 = if let Some(arg_path) = with_obj.get("value1").and_then(|v| v.as_str()) {
        // If it starts with {args., extract from args
        if arg_path.starts_with("{args.") && arg_path.ends_with("}") {
            let key = &arg_path[6..arg_path.len()-1];
            args.get(key).cloned().unwrap_or(Value::Null)
        } else {
            Value::String(arg_path.to_string())
        }
    } else {
        return Err("Missing value1 in value.notEquals condition".to_string());
    };
    
    // Get the second value
    let value2 = if let Some(val) = with_obj.get("value2") {
        // If it's {player}, replace with current actor
        if let Some(str_val) = val.as_str() {
            if str_val == "{player}" {
                Value::String(current_actor.to_string())
            } else {
                val.clone()
            }
        } else {
            val.clone()
        }
    } else {
        return Err("Missing value2 in value.notEquals condition".to_string());
    };
    
    println!("[DEBUG value.notEquals] Comparing value1: {:?} with value2: {:?}", value1, value2);
    
    // Compare the values
    let result = value1 != value2;
    println!("[DEBUG value.notEquals] Result: {}", result);
    
    Ok(result)
}

fn evaluate_value_equals(
    condition: &Value,
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    args: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    // This function needs to handle both formats:
    // 1. New format with "with": {"value1": "...", "value2": "..."}
    // 2. Legacy format with "path": "/test", "value": true (used in tests)
    
    // Check for legacy format first
    if let (Some(path_str), Some(expected_value)) = (
        condition.get("path").and_then(|p| p.as_str()),
        condition.get("value")
    ) {
        // Legacy format: get value from state at path and compare
        let actual_value = get_value_at_path(state, path_str);
        let result = actual_value.map(|v| v == expected_value).unwrap_or(false);
        
        println!("[DEBUG valueEquals] Legacy format - path: {}, expected: {:?}, actual: {:?}, result: {}", 
                 path_str, expected_value, actual_value, result);
        
        return Ok(result);
    }
    
    // New format with value1/value2 in with object
    if let Some(with_obj) = with_obj {
        let value1 = if let Some(arg_path) = with_obj.get("value1").and_then(|v| v.as_str()) {
            // If it starts with {args., extract from args
            if arg_path.starts_with("{args.") && arg_path.ends_with("}") {
                let key = &arg_path[6..arg_path.len()-1];
                args.get(key).cloned().unwrap_or(Value::Null)
            } else {
                Value::String(arg_path.to_string())
            }
        } else {
            return Err("Missing value1 in valueEquals condition".to_string());
        };
        
        // Get the second value
        let value2 = if let Some(val) = with_obj.get("value2") {
            // If it's {player}, replace with current actor
            if let Some(str_val) = val.as_str() {
                if str_val == "{player}" {
                    Value::String(current_actor.to_string())
                } else {
                    val.clone()
                }
            } else {
                val.clone()
            }
        } else {
            return Err("Missing value2 in valueEquals condition".to_string());
        };
        
        println!("[DEBUG valueEquals] New format - value1: {:?}, value2: {:?}", value1, value2);
        
        // Compare the values
        let result = value1 == value2;
        println!("[DEBUG valueEquals] Result: {}", result);
        
        Ok(result)
    } else {
        return Err("Missing both legacy format (path/value) and new format (with object) for valueEquals".to_string());
    }
}

fn evaluate_entity_has_property(
    with_obj: Option<&serde_json::Map<String, Value>>,
    args: &Value,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for entity.hasProperty")?;
    
    // Get entity from args
    let entity_template = with_obj.get("entity")
        .and_then(|e| e.as_str())
        .ok_or("Missing entity in condition")?;
    
    let entity_id = if entity_template.starts_with("{args.") && entity_template.ends_with("}") {
        let key = &entity_template[6..entity_template.len()-1];
        args.get(key)
            .and_then(|e| e.as_str())
            .ok_or("Missing entity in args")?
    } else {
        entity_template
    };
    
    // Get property and value to check
    let property = with_obj.get("property")
        .and_then(|p| p.as_str())
        .ok_or("Missing property in condition")?;
    let expected_value = with_obj.get("value")
        .and_then(|v| v.as_str())
        .ok_or("Missing value in condition")?;
    
    // For card entities, extract property from entity ID
    if entity_id.starts_with("card_") {
        let parts: Vec<&str> = entity_id.split('_').collect();
        if parts.len() >= 3 {
            let actual_value = match property {
                "rank" => parts[2],
                "suit" => parts[1],
                _ => return Ok(false),
            };
            Ok(actual_value == expected_value)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

fn evaluate_numeric_compare(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    let with_obj = with_obj.ok_or("Missing 'with' object for numeric.compare")?;
    
    // Get the left value (expression or path)
    let left_val = if let Some(expr) = with_obj.get("expression").and_then(|e| e.as_str()) {
        // Evaluate expression
        let mut variables = std::collections::HashMap::new();
        
        // Add any variables from the condition
        if let Some(vars) = with_obj.get("variables").and_then(|v| v.as_object()) {
            for (key, val) in vars {
                if let Some(path) = val.as_str() {
                    let path = path.replace("{player}", current_actor);
                    if let Ok(num_val) = crate::engine::verbs::math::get_numeric_value(state, &path) {
                        variables.insert(key.clone(), num_val);
                    }
                } else if let Some(num) = val.as_f64() {
                    variables.insert(key.clone(), num);
                }
            }
        }
        
        crate::engine::verbs::math::evaluate_expression(expr, &variables)
            .map_err(|e| format!("Failed to evaluate expression: {}", e))?
    } else if let Some(path) = with_obj.get("path").and_then(|p| p.as_str()) {
        // Get value from path
        let path = path.replace("{player}", current_actor);
        crate::engine::verbs::math::get_numeric_value(state, &path)
            .map_err(|e| format!("Failed to get numeric value: {}", e))?
    } else {
        return Err("numeric.compare requires either 'expression' or 'path'".to_string());
    };
    
    // Get the comparison operator
    let operator = with_obj.get("operator")
        .and_then(|o| o.as_str())
        .ok_or("Missing operator in numeric.compare")?;
    
    // Get the right value
    let right_val = with_obj.get("value")
        .and_then(|v| v.as_f64())
        .or_else(|| with_obj.get("value").and_then(|v| v.as_i64()).map(|i| i as f64))
        .ok_or("Missing value in numeric.compare")?;
    
    // Perform comparison
    let result = match operator {
        "==" => left_val == right_val,
        "!=" => left_val != right_val,
        ">" => left_val > right_val,
        "<" => left_val < right_val,
        ">=" => left_val >= right_val,
        "<=" => left_val <= right_val,
        _ => return Err(format!("Unknown operator: {}", operator))
    };
    
    Ok(result)
}

fn evaluate_grid_line_of_marks(
    with_obj: Option<&serde_json::Map<String, Value>>,
    state: &Value,
    current_actor: &str,
) -> Result<bool, String> {
    println!("[DEBUG evaluate_grid_line_of_marks] Called for player: {}", current_actor);
    let with_obj = with_obj.ok_or("Missing 'with' object for grid.lineOfMarks")?;
    
    // Extract parameters
    let zone_path = with_obj.get("zone")
        .and_then(|z| z.as_str())
        .ok_or("Missing 'zone' path")?;
    let entity_pattern = with_obj.get("entity")
        .and_then(|e| e.as_str())
        .ok_or("Missing 'entity' pattern")?;
    let line_length = with_obj.get("lineLength")
        .and_then(|l| l.as_u64())
        .unwrap_or(3) as usize;
    let directions = with_obj.get("directions")
        .and_then(|d| d.as_array())
        .ok_or("Missing 'directions' array")?;
    
    // Replace {player} in entity pattern
    let entity_pattern = entity_pattern.replace("{player}", current_actor);
    println!("[DEBUG evaluate_grid_line_of_marks] Entity pattern: {}", entity_pattern);
    println!("[DEBUG evaluate_grid_line_of_marks] Zone path: {}", zone_path);
    
    // Get the zone
    let zone = get_value_at_path(state, zone_path)
        .ok_or_else(|| format!("Zone not found: {}", zone_path))?;
    let cells = zone.get("cells")
        .and_then(|c| c.as_array())
        .ok_or("Zone is not a grid with cells")?;
    
    if cells.is_empty() {
        return Ok(false);
    }
    
    // Get grid dimensions
    let rows = cells.len();
    let cols = cells[0].as_array()
        .ok_or("Grid row is not an array")?
        .len();
    
    // Debug: Print board state
    println!("[DEBUG evaluate_grid_line_of_marks] Board state:");
    for (r, row) in cells.iter().enumerate() {
        if let Some(row_array) = row.as_array() {
            let mut row_str = String::new();
            for (c, cell) in row_array.iter().enumerate() {
                if let Some(entity) = cell.get("entity").and_then(|e| e.as_str()) {
                    if entity == "mark_p1" {
                        row_str.push('X');
                    } else if entity == "mark_p2" {
                        row_str.push('O');
                    } else {
                        row_str.push('?');
                    }
                } else {
                    row_str.push('-');
                }
                row_str.push(' ');
            }
            println!("[DEBUG evaluate_grid_line_of_marks]   Row {}: {}", r, row_str);
        }
    }
    
    // Check for winning lines in each enabled direction
    for direction in directions {
        let dir_str = direction.as_str()
            .ok_or("Direction must be a string")?;
        
        if check_grid_direction(cells, &entity_pattern, line_length, rows, cols, dir_str)? {
            return Ok(true);
        }
    }
    
    Ok(false)
}

fn check_grid_direction(
    cells: &[Value],
    entity_pattern: &str,
    line_length: usize,
    rows: usize,
    cols: usize,
    direction: &str,
) -> Result<bool, String> {
    match direction {
        "horizontal" => check_horizontal_lines_condition(cells, entity_pattern, line_length, rows, cols),
        "vertical" => check_vertical_lines_condition(cells, entity_pattern, line_length, rows, cols),
        "diagonal" => check_diagonal_lines_condition(cells, entity_pattern, line_length, rows, cols),
        _ => Err(format!("Unknown direction: {}", direction))
    }
}

fn check_horizontal_lines_condition(
    cells: &[Value],
    entity_pattern: &str,
    line_length: usize,
    rows: usize,
    cols: usize,
) -> Result<bool, String> {
    for row in 0..rows {
        for start_col in 0..=(cols.saturating_sub(line_length)) {
            if check_line_at_position_condition(
                cells, entity_pattern, line_length, row, start_col, 0, 1
            )? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn check_vertical_lines_condition(
    cells: &[Value],
    entity_pattern: &str,
    line_length: usize,
    rows: usize,
    cols: usize,
) -> Result<bool, String> {
    for col in 0..cols {
        for start_row in 0..=(rows.saturating_sub(line_length)) {
            if check_line_at_position_condition(
                cells, entity_pattern, line_length, start_row, col, 1, 0
            )? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn check_diagonal_lines_condition(
    cells: &[Value],
    entity_pattern: &str,
    line_length: usize,
    rows: usize,
    cols: usize,
) -> Result<bool, String> {
    // Check main diagonal (top-left to bottom-right)
    for row in 0..=(rows.saturating_sub(line_length)) {
        for col in 0..=(cols.saturating_sub(line_length)) {
            if check_line_at_position_condition(
                cells, entity_pattern, line_length, row, col, 1, 1
            )? {
                return Ok(true);
            }
        }
    }
    
    // Check anti-diagonal (top-right to bottom-left)
    for row in 0..=(rows.saturating_sub(line_length)) {
        for col in (line_length - 1)..cols {
            if check_line_at_position_condition(
                cells, entity_pattern, line_length, row, col, 1, -1
            )? {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

fn check_line_at_position_condition(
    cells: &[Value], 
    entity_pattern: &str, 
    line_length: usize, 
    start_row: usize, 
    start_col: usize, 
    row_delta: i32, 
    col_delta: i32
) -> Result<bool, String> {
    let mut first_entity: Option<String> = None;
    
    for i in 0..line_length {
        let row = (start_row as i32 + (i as i32 * row_delta)) as usize;
        let col = (start_col as i32 + (i as i32 * col_delta)) as usize;
        
        let cell = cells.get(row)
            .and_then(|r| r.as_array())
            .and_then(|r| r.get(col))
            .ok_or("Cell position out of bounds")?;
        
        let entity_id = extract_entity_id_condition(cell);
        
        if let Some(id) = entity_id {
            if matches_pattern_condition(&id, entity_pattern) {
                match &first_entity {
                    Some(first) if first != &id => return Ok(false), // Different entities
                    None => first_entity = Some(id),
                    _ => {} // Same entity, continue
                }
            } else {
                return Ok(false); // Entity doesn't match pattern
            }
        } else {
            return Ok(false); // Empty cell
        }
    }
    
    Ok(first_entity.is_some())
}

fn extract_entity_id_condition(cell: &Value) -> Option<String> {
    cell.as_object()
        .and_then(|obj| obj.get("entity"))
        .and_then(|e| e.as_str())
        .map(|s| s.to_string())
}

fn matches_pattern_condition(entity_id: &str, pattern: &str) -> bool {
    println!("[DEBUG matches_pattern_condition] entity_id: '{}', pattern: '{}'", entity_id, pattern);
    let result = if pattern.contains("{player}") {
        let base_pattern = pattern.replace("{player}", "");
        println!("[DEBUG matches_pattern_condition] base_pattern: '{}', starts_with: {}", base_pattern, entity_id.starts_with(&base_pattern));
        entity_id.starts_with(&base_pattern)
    } else {
        let equals = entity_id == pattern;
        println!("[DEBUG matches_pattern_condition] exact match: {}", equals);
        equals
    };
    println!("[DEBUG matches_pattern_condition] final result: {}", result);
    result
}


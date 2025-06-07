use serde_json::{json, Value};
use crate::bundle::Bundle;
use crate::engine::path::{get_zone_mut, get_cell_value, set_cell_value};
use crate::engine::grid::apply_check_for_win;
use chrono;

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
        "placeWithGravity" => apply_place_with_gravity(state, args),
        "nextTurn" => apply_next_turn(state, args, bundle),
        "setPhase" => apply_set_phase(state, args),
        "grid.lineOfMarks" => apply_check_for_win(state, args),
        "selectEntity" => apply_select_entity(state, args),
        "moveSelected" => apply_move_selected(state, args),
        "clearSelection" => apply_clear_selection(state, args),
        "queryEntities" => apply_query_entities(state, args),
        "transferMatching" => apply_transfer_matching(state, args),
        "presentChoice" => apply_present_choice(state, args),
        "makeSelection" => apply_make_selection(state, args),
        "setState" => apply_set_state(state, args),
        "conditionalAction" => {
            // For conditionalAction, we need to pass the current player context
            // Get it from currentPlayer in state
            let current_player = state.get("currentPlayer")
                .and_then(|p| p.as_str())
                .unwrap_or("p1")
                .to_string();
            apply_conditional_action(state, args, bundle, &current_player)
        },
        "formPairs" => apply_form_pairs(state, args),
        "calculateWinner" => apply_calculate_winner(state, args),
        "selectPlayer" => apply_select_player(state, args),
        "checkPlayerCards" => apply_check_player_cards(state, args, bundle),
        "drawWithReshuffle" => apply_draw_with_reshuffle(state, args),
        "validateMeld" => apply_validate_meld(state, args),
        "matchCard" => apply_match_card(state, args),
        "removePairs" => apply_remove_pairs(state, args),
        "makeBid" => apply_make_bid(state, args),
        "playToTrick" => apply_play_to_trick(state, args),
        "resolveTrick" => apply_resolve_trick(state, args),
        _ => Err(format!("Unknown verb: {}", verb)),
    }
}

fn apply_draw(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;
    let count = args["count"].as_u64().unwrap_or(1) as usize;

    let mut patches = Vec::new();

    for _ in 0..count {
        draw_single_item(state, from_path, to_path, &mut patches)?;
    }

    Ok(patches)
}

fn draw_single_item(
    state: &mut Value,
    from_path: &str,
    to_path: &str,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
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

    Ok(())
}

fn apply_move_entity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;

    let mut patches = Vec::new();

    // Handle grid to grid moves
    if from_path.contains("/cells/") && to_path.contains("/cells/") {
        move_between_grid_cells(state, from_path, to_path, &mut patches)?;
    }

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
        "value": null
    }));

    // Add to destination
    set_cell_value(state, to_path, from_value.clone())?;
    patches.push(json!({
        "op": "replace",
        "path": to_path,
        "value": from_value
    }));

    Ok(())
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

fn apply_place_with_gravity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args["zone"].as_str().ok_or("Missing 'zone' path")?;
    let column = args["column"].as_u64().ok_or("Missing 'column' index")? as usize;
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
        "value": entity_value
    })])
}

fn apply_next_turn(state: &mut Value, _args: &Value, bundle: &Bundle) -> Result<Vec<Value>, String> {
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    
    // Increment tick
    let current_tick = state_obj["tick"].as_u64().unwrap_or(0);
    let new_tick = current_tick + 1;
    state_obj.insert("tick".to_string(), json!(new_tick));
    
    // Advance turn
    let player_count = bundle.manifest.metadata.players.max;
    let current_turn = state_obj["turn"].as_u64().unwrap_or(0);
    let next_turn = (current_turn + 1) % player_count as u64;
    let next_player = format!("p{}", next_turn + 1);
    
    state_obj.insert("turn".to_string(), json!(next_turn));
    state_obj.insert("currentPlayer".to_string(), json!(next_player.clone()));
    
    Ok(vec![
        json!({
            "op": "replace",
            "path": "/game/tick",
            "value": new_tick
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
    let phases = state_obj.get_mut("phases")
        .and_then(|p| p.as_object_mut())
        .ok_or("Missing phases in state")?;
    
    phases.insert(phase_set.to_string(), json!(phase));
    
    Ok(vec![json!({
        "op": "replace",
        "path": format!("/game/phases/{}", phase_set),
        "value": phase
    })])
}

fn apply_select_entity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let location = args["location"].as_str().ok_or("Missing 'location'")?;
    let player = args["player"].as_str().ok_or("Missing 'player'")?;
    
    // Validate that there's an entity at this location
    if !location.starts_with("/zones/") {
        return Err("Invalid location path".to_string());
    }
    
    // Parse location to get zone path
    let path_parts: Vec<&str> = location.split('/').filter(|p| !p.is_empty()).collect();
    if path_parts.len() < 2 {
        return Err("Invalid location format".to_string());
    }
    
    // Get the entity at the location
    let entity = get_cell_value(state, location)?;
    if entity.is_null() {
        return Err("No entity at specified location".to_string());
    }
    
    // Store selection in player's selection state
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    
    // Initialize selection state if it doesn't exist
    if !state_obj.contains_key("selection") {
        state_obj.insert("selection".to_string(), json!({}));
    }
    
    let selection = state_obj.get_mut("selection")
        .and_then(|s| s.as_object_mut())
        .ok_or("Invalid selection state")?;
    
    selection.insert(player.to_string(), json!({
        "location": location,
        "entity": entity
    }));
    
    Ok(vec![json!({
        "op": "replace",
        "path": format!("/game/selection/{}", player),
        "value": {
            "location": location,
            "entity": entity
        }
    })])
}

fn apply_move_selected(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
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

fn apply_clear_selection(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let player = args["player"].as_str().ok_or("Missing 'player'")?;
    
    // Clear player's selection
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    if let Some(selection) = state_obj.get_mut("selection").and_then(|s| s.as_object_mut()) {
        selection.remove(player);
    }
    
    Ok(vec![json!({
        "op": "remove",
        "path": format!("/game/selection/{}", player)
    })])
}

fn apply_query_entities(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args["zone"].as_str().ok_or("Missing 'zone' path")?;
    let property = args["property"].as_str().ok_or("Missing 'property'")?;
    let store_path = args["storePath"].as_str().ok_or("Missing 'storePath'")?;
    
    // Get the source zone
    let zone = get_zone_mut(state, zone_path)?;
    let items = zone["items"].as_array()
        .ok_or("Source zone is not a list")?;
    
    // Find unique property values
    let mut unique_values = std::collections::HashSet::new();
    for item in items {
        if let Some(entity_id) = item.get("entity").and_then(|e| e.as_str()) {
            // For card entities, extract property from entity ID
            // e.g., "card_hearts_2" -> suit=hearts, rank=2
            if entity_id.starts_with("card_") {
                let parts: Vec<&str> = entity_id.split('_').collect();
                if parts.len() >= 3 {
                    let value = match property {
                        "rank" => parts[2],  // rank is the third part
                        "suit" => parts[1],  // suit is the second part
                        _ => continue,
                    };
                    unique_values.insert(value.to_string());
                }
            }
        }
    }
    
    // Convert to sorted vector for consistent ordering
    let mut values: Vec<String> = unique_values.into_iter().collect();
    values.sort();
    
    // Store results at the specified path
    let results: Vec<Value> = values.into_iter().map(|v| json!(v)).collect();
    
    // Store the results directly in state 
    if store_path.starts_with("/temp/") {
        let temp_key = store_path.strip_prefix("/temp/").unwrap_or("data");
        if let Some(state_obj) = state.as_object_mut() {
            let mut patches = Vec::new();
            let temp_existed = state_obj.contains_key("temp");
            if !temp_existed {
                state_obj.insert("temp".to_string(), json!({}));
                patches.push(json!({
                    "op": "add",
                    "path": "/game/temp",
                    "value": {}
                }));
            }
            if let Some(temp_obj) = state_obj.get_mut("temp").and_then(|t| t.as_object_mut()) {
                let key_existed = temp_obj.contains_key(temp_key);
                temp_obj.insert(temp_key.to_string(), json!(results.clone()));
                
                // Use appropriate operation based on whether key existed
                let op = if key_existed { "replace" } else { "add" };
                patches.push(json!({
                    "op": op,
                    "path": format!("/game{}", store_path),
                    "value": results
                }));
            }
            return Ok(patches);
        }
    } else if store_path.starts_with("/selection/") {
        let selection_key = store_path.strip_prefix("/selection/").unwrap_or("data");
        if let Some(state_obj) = state.as_object_mut() {
            // Selection object should exist from initial state
            if let Some(selection_obj) = state_obj.get_mut("selection").and_then(|s| s.as_object_mut()) {
                let key_existed = selection_obj.contains_key(selection_key);
                selection_obj.insert(selection_key.to_string(), json!(results.clone()));
                
                // Use appropriate operation based on whether key existed
                let op = if key_existed { "replace" } else { "add" };
                return Ok(vec![json!({
                    "op": op,
                    "path": format!("/game{}", store_path),
                    "value": results
                })]);
            }
        }
    }
    
    // Create patch to store the results (fallback for other paths)
    Ok(vec![json!({
        "op": "replace",
        "path": format!("/game{}", store_path),
        "value": results
    })])
}

fn apply_transfer_matching(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;
    let property = args["property"].as_str().ok_or("Missing 'property'")?;
    let value = args["value"].as_str().ok_or("Missing 'value'")?;
    
    let mut patches = Vec::new();
    
    // Get source zone
    let from_zone = get_zone_mut(state, from_path)?;
    let from_items = from_zone["items"].as_array_mut()
        .ok_or("Source zone is not a list")?;
    
    // Find and remove matching entities
    let mut matching_entities = Vec::new();
    let mut i = 0;
    while i < from_items.len() {
        let mut matches = false;
        if let Some(entity_id) = from_items[i].get("entity").and_then(|e| e.as_str()) {
            // Check if entity matches the property value
            if entity_id.starts_with("card_") {
                let parts: Vec<&str> = entity_id.split('_').collect();
                if parts.len() >= 3 {
                    let entity_value = match property {
                        "rank" => parts[2],  // rank is the third part: card_suit_rank
                        "suit" => parts[1],  // suit is the second part: card_suit_rank
                        _ => "",
                    };
                    matches = entity_value == value;
                }
            }
        }
        
        if matches {
            matching_entities.push(from_items.remove(i));
        } else {
            i += 1;
        }
    }
    
    // Create patch for source zone
    patches.push(json!({
        "op": "replace",
        "path": format!("/game{}/items", from_path),
        "value": from_items
    }));
    
    // Add entities to target zone
    let to_zone = get_zone_mut(state, to_path)?;
    let to_items = to_zone["items"].as_array_mut()
        .ok_or("Target zone is not a list")?;
    
    for entity in matching_entities {
        to_items.push(entity);
    }
    
    // Create patch for target zone
    patches.push(json!({
        "op": "replace",
        "path": format!("/game{}/items", to_path),
        "value": to_items
    }));
    
    Ok(patches)
}

fn apply_present_choice(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args["zone"].as_str().ok_or("Missing 'zone' path")?;
    let options = args["options"].as_object().ok_or("Missing 'options'")?;
    let prompt = args.get("prompt").and_then(|p| p.as_str()).unwrap_or("Make a choice");
    
    // Generate choice options based on type
    let choice_items = if let Some(option_type) = options.get("type").and_then(|t| t.as_str()) {
        match option_type {
            "property" => {
                // Extract property values from a source zone
                let property = options.get("property").and_then(|p| p.as_str()).ok_or("Missing property")?;
                let from_zone_path = options.get("fromZone").and_then(|z| z.as_str()).ok_or("Missing fromZone")?;
                
                // Get source zone and extract property values first (avoid double borrow)
                let items = {
                    let from_zone = get_zone_mut(state, from_zone_path)?;
                    from_zone["items"].as_array().cloned().ok_or("Source zone is not a list")?
                };
                
                let mut values = std::collections::HashSet::new();
                for item in items {
                    if let Some(entity_id) = item.get("entity").and_then(|e| e.as_str()) {
                        if entity_id.starts_with("card_") {
                            let parts: Vec<&str> = entity_id.split('_').collect();
                            if parts.len() >= 3 {
                                let value = match property {
                                    "rank" => parts[2],  // rank is the third part: card_suit_rank
                                    "suit" => parts[1],  // suit is the second part: card_suit_rank
                                    _ => continue,
                                };
                                values.insert(value.to_string());
                            }
                        }
                    }
                }
                
                let mut sorted_values: Vec<String> = values.into_iter().collect();
                sorted_values.sort();
                
                sorted_values.into_iter().map(|v| json!({
                    "id": v,
                    "label": v
                })).collect()
            }
            _ => Vec::new()
        }
    } else {
        Vec::new()
    };
    
    // Update the choice zone with options
    let zone = get_zone_mut(state, zone_path)?;
    if let Some(zone_obj) = zone.as_object_mut() {
        zone_obj.insert("items".to_string(), json!(choice_items));
        zone_obj.insert("prompt".to_string(), json!(prompt));
    }
    
    // Create patch for choice zone
    Ok(vec![json!({
        "op": "replace",
        "path": format!("/game{}/items", zone_path),
        "value": choice_items
    })])
}

fn apply_make_selection(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let selection = args["selection"].as_str().ok_or("Missing 'selection'")?;
    let store_path = args.get("storePath").and_then(|p| p.as_str()).unwrap_or("/temp/selection");
    
    // Store the selection in temporary state
    if store_path.starts_with("/temp/") {
        let temp_key = store_path.strip_prefix("/temp/").unwrap_or("selection");
        if let Some(state_obj) = state.as_object_mut() {
            if !state_obj.contains_key("temp") {
                state_obj.insert("temp".to_string(), json!({}));
            }
            if let Some(temp_obj) = state_obj.get_mut("temp").and_then(|t| t.as_object_mut()) {
                temp_obj.insert(temp_key.to_string(), json!(selection));
            }
        }
    }
    
    // Create patch for the selection
    Ok(vec![json!({
        "op": "replace",
        "path": format!("/game{}", store_path),
        "value": selection
    })])
}

fn apply_set_state(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let path = args["path"].as_str().ok_or("Missing 'path'")?;
    let value = args.get("value").ok_or("Missing 'value'")?;
    
    let mut patches = Vec::new();
    
    // Store the value in state
    if path.starts_with("/selection/") {
        let selection_key = path.strip_prefix("/selection/").unwrap_or("data");
        if let Some(state_obj) = state.as_object_mut() {
            // Selection object should always exist from initial state
            if let Some(selection_obj) = state_obj.get_mut("selection").and_then(|t| t.as_object_mut()) {
                let key_existed = selection_obj.contains_key(selection_key);
                selection_obj.insert(selection_key.to_string(), value.clone());
                
                // Use appropriate operation based on whether key existed
                if key_existed {
                    patches.push(json!({
                        "op": "replace",
                        "path": format!("/game{}", path),
                        "value": value
                    }));
                } else {
                    patches.push(json!({
                        "op": "add",
                        "path": format!("/game{}", path),
                        "value": value
                    }));
                }
            } else {
                return Err("Selection object not found in state".to_string());
            }
        }
    } else if path.starts_with("/temp/") {
        // Still support temp for backward compatibility
        let temp_key = path.strip_prefix("/temp/").unwrap_or("data");
        if let Some(state_obj) = state.as_object_mut() {
            let temp_existed = state_obj.contains_key("temp");
            if !temp_existed {
                state_obj.insert("temp".to_string(), json!({}));
                // Add patch to create temp object
                patches.push(json!({
                    "op": "add",
                    "path": "/game/temp",
                    "value": {}
                }));
            }
            if let Some(temp_obj) = state_obj.get_mut("temp").and_then(|t| t.as_object_mut()) {
                let key_existed = temp_obj.contains_key(temp_key);
                temp_obj.insert(temp_key.to_string(), value.clone());
                
                // Use appropriate operation based on whether key existed
                if key_existed {
                    patches.push(json!({
                        "op": "replace",
                        "path": format!("/game{}", path),
                        "value": value
                    }));
                } else {
                    patches.push(json!({
                        "op": "add",
                        "path": format!("/game{}", path),
                        "value": value
                    }));
                }
            }
        }
    } else {
        // For other paths, just create a simple replace patch
        patches.push(json!({
            "op": "replace",
            "path": format!("/game{}", path),
            "value": value
        }));
    }
    
    Ok(patches)
}

fn apply_conditional_action(state: &mut Value, args: &Value, bundle: &Bundle, current_actor: &str) -> Result<Vec<Value>, String> {
    let condition = args.get("condition").ok_or("Missing 'condition'")?;
    let if_true = args.get("ifTrue").and_then(|t| t.as_array());
    let if_false = args.get("ifFalse").and_then(|f| f.as_array());
    
    // Evaluate the condition
    let condition_result = if let Some(conditions) = condition.as_array() {
        // Evaluate all conditions (AND logic)
        conditions.iter().all(|cond| {
            match crate::conditions::evaluate_condition(cond, state, args, current_actor) {
                Ok(result) => result,
                Err(e) => {
                    println!("Error evaluating condition: {}", e);
                    false
                }
            }
        })
    } else {
        // Single condition
        match crate::conditions::evaluate_condition(condition, state, args, current_actor) {
            Ok(result) => result,
            Err(e) => {
                println!("Error evaluating condition: {}", e);
                false
            }
        }
    };
    
    let actions_to_execute = if condition_result {
        println!("[DEBUG conditionalAction] Condition TRUE, executing ifTrue actions");
        if_true
    } else {
        println!("[DEBUG conditionalAction] Condition FALSE, executing ifFalse actions");
        if_false
    };
    
    let mut all_patches = Vec::new();
    
    if let Some(actions) = actions_to_execute {
        for action in actions {
            if let Some(action_name) = action.get("action").and_then(|a| a.as_str()) {
                let empty_args = json!({});
                let action_args = action.get("with").unwrap_or(&empty_args);
                
                // Find the action in the bundle and execute it
                if let Some(actions_array) = bundle.actions.as_array() {
                    if let Some(action_def) = actions_array.iter().find(|a| a["id"].as_str() == Some(action_name)) {
                        if let Some(verb) = action_def["uses"].as_str() {
                            // Merge action args with definition's "with" params
                            let mut final_args = action_def.get("with").cloned().unwrap_or(json!({}));
                            if let (Some(final_obj), Some(args_obj)) = (final_args.as_object_mut(), action_args.as_object()) {
                                for (k, v) in args_obj {
                                    final_obj.insert(k.clone(), v.clone());
                                }
                            }
                            
                            // Replace templates in the args
                            final_args = crate::engine::patches::replace_template_vars(&final_args, state);
                            final_args = crate::engine::patches::replace_actor_template(&final_args, current_actor);
                            
                            // Execute the verb
                            println!("[DEBUG conditionalAction] Executing action {} with verb {}", action_name, verb);
                            match apply_verb(state, verb, &final_args, bundle) {
                                Ok(mut patches) => {
                                    println!("[DEBUG conditionalAction] Action {} produced {} patches", action_name, patches.len());
                                    all_patches.append(&mut patches)
                                },
                                Err(e) => return Err(format!("Failed to execute conditional action '{}': {}", action_name, e)),
                            }
                        }
                    } else {
                        // If it's not an action ID, treat it as a verb name directly
                        match apply_verb(state, action_name, action_args, bundle) {
                            Ok(mut patches) => all_patches.append(&mut patches),
                            Err(e) => return Err(format!("Failed to execute conditional action '{}': {}", action_name, e)),
                        }
                    }
                } else {
                    return Err("Bundle missing actions array".to_string());
                }
            }
        }
    }
    
    Ok(all_patches)
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::bundle::{Bundle, Manifest, ManifestMetadata, PlayersRange};

    #[allow(dead_code)]
    fn create_test_bundle() -> Bundle {
        Bundle {
            game_id: "test".to_string(),
            manifest: Manifest {
                game_id: "test".to_string(),
                version: "1.0".to_string(),
                spec_version: "0.1".to_string(),
                metadata: ManifestMetadata {
                    name: "Test Game".to_string(),
                    author: "Test Author".to_string(),
                    players: PlayersRange { min: 2, max: 2 },
                    description: "Test game".to_string(),
                },
                phases: None,
                setup: None,
                zone_groups: None,
            },
            entities: Value::Null,
            zones: Value::Null,
            actions: Value::Null,
            phases: Value::Null,
            _hooks: None,
        }
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
}

fn apply_form_pairs(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let player = args["player"].as_str().ok_or("Missing 'player'")?;
    
    let hand_path = format!("/zones/hand_{}", player);
    let pairs_path = format!("/zones/pairs_{}", player);
    
    let mut patches = Vec::new();
    
    // Get player's hand
    let hand_zone = get_zone_mut(state, &hand_path)?;
    let hand_items = hand_zone["items"].as_array_mut()
        .ok_or("Hand zone is not a list")?;
    
    // Count cards by rank
    let mut rank_counts: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    for (index, item) in hand_items.iter().enumerate() {
        if let Some(entity_id) = item.get("entity").and_then(|e| e.as_str()) {
            if entity_id.starts_with("card_") {
                let parts: Vec<&str> = entity_id.split('_').collect();
                if parts.len() >= 3 {
                    let rank = parts[2].to_string();  // rank is the third part: card_suit_rank
                    rank_counts.entry(rank).or_insert_with(Vec::new).push(index);
                }
            }
        }
    }
    
    // Find ranks with 2+ cards (pairs)
    let mut pairs_formed = false;
    let mut cards_to_remove = Vec::new();
    let mut pairs_to_add = Vec::new();
    let mut formed_ranks = Vec::new();
    
    for (rank, indices) in rank_counts {
        if indices.len() >= 2 {
            // Found a pair (2 or more cards of same rank)
            // Take only the first 2 cards to form a pair
            pairs_formed = true;
            formed_ranks.push(rank.clone());
            for i in 0..2 {
                let index = indices[i];
                cards_to_remove.push(index);
                pairs_to_add.push(hand_items[index].clone());
            }
        }
    }
    
    if pairs_formed {
        // Remove cards from hand (in reverse order to maintain indices)
        cards_to_remove.sort_by(|a, b| b.cmp(a));
        for index in cards_to_remove {
            hand_items.remove(index);
        }
        
        // Update hand
        patches.push(json!({
            "op": "replace",
            "path": format!("/game{}/items", hand_path),
            "value": hand_items
        }));
        
        // Add to pairs zone
        let pairs_zone = get_zone_mut(state, &pairs_path)?;
        let pairs_items = pairs_zone["items"].as_array_mut()
            .ok_or("Pairs zone is not a list")?;
        
        for card in pairs_to_add {
            pairs_items.push(card);
        }
        
        patches.push(json!({
            "op": "replace", 
            "path": format!("/game{}/items", pairs_path),
            "value": pairs_items
        }));
        
        // Generate log messages for each pair formed
        for rank in formed_ranks {
            let log_message = format!("{} forms a pair of {}s", player, rank);
            patches.push(json!({
                "op": "add",
                "path": "/ui/gameLog/-",
                "value": {
                    "auto": true,
                    "message": log_message,
                    "timestamp": chrono::Utc::now().format("%H:%M").to_string()
                }
            }));
        }
    }
    
    Ok(patches)
}

fn apply_calculate_winner(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let mut max_pairs = 0;
    let mut winner = None;
    let mut tie = false;
    
    // Check each player's pairs
    for player_num in 1..=4 {
        let player_id = format!("p{}", player_num);
        let pairs_path = format!("/zones/pairs_{}", player_id);
        
        if let Ok(pairs_zone) = get_zone_mut(state, &pairs_path) {
            if let Some(pairs_items) = pairs_zone["items"].as_array() {
                let pairs_count = pairs_items.len();
                if pairs_count > max_pairs {
                    max_pairs = pairs_count;
                    winner = Some(player_id);
                    tie = false;
                } else if pairs_count == max_pairs && max_pairs > 0 {
                    tie = true;
                }
            }
        }
    }
    
    // Set game status
    let game_status = if tie {
        json!({
            "state": "ended",
            "winner": null,
            "tie": true
        })
    } else if let Some(winner_id) = winner {
        json!({
            "state": "ended", 
            "winner": winner_id,
            "tie": false
        })
    } else {
        json!({
            "state": "active",
            "winner": null,
            "tie": false
        })
    };
    
    // Update state
    if let Some(state_obj) = state.as_object_mut() {
        state_obj.insert("gameStatus".to_string(), game_status.clone());
    }
    
    Ok(vec![json!({
        "op": "replace",
        "path": "/game/gameStatus",
        "value": game_status
    })])
}

fn apply_select_player(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let target_player = args["targetPlayer"].as_str().ok_or("Missing 'targetPlayer'")?;
    
    // Store the target player selection in temporary state
    if let Some(state_obj) = state.as_object_mut() {
        if !state_obj.contains_key("temp") {
            state_obj.insert("temp".to_string(), json!({}));
        }
        if let Some(temp_obj) = state_obj.get_mut("temp").and_then(|t| t.as_object_mut()) {
            temp_obj.insert("targetPlayer".to_string(), json!(target_player));
        }
    }
    
    Ok(vec![json!({
        "op": "replace",
        "path": "/game/temp/targetPlayer",
        "value": target_player
    })])
}

fn apply_check_player_cards(state: &mut Value, args: &Value, bundle: &Bundle) -> Result<Vec<Value>, String> {
    // This function will be handled by the conditionalAction logic in the actions.yaml
    // For now, just return empty patches since the logic is in the action definition
    Ok(Vec::new())
}

// ===== NEW CARD GAME VERBS =====

/// Draw cards with automatic reshuffling if deck runs out
fn apply_draw_with_reshuffle(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;
    let count = args["count"].as_u64().unwrap_or(1) as usize;
    let reshuffle_from = args.get("reshuffleFrom").and_then(|v| v.as_str());

    let mut patches = Vec::new();
    
    for i in 0..count {
        // Check if deck is empty and reshuffle is enabled
        let from_zone = get_zone_mut(state, from_path)?;
        let items = from_zone["items"].as_array().ok_or("Source is not a deck")?;
        
        if items.is_empty() && reshuffle_from.is_some() {
            // Reshuffle discard pile back into deck
            let discard_path = reshuffle_from.unwrap();
            reshuffle_deck(state, discard_path, from_path, &mut patches)?;
        }
        
        // Now try to draw
        match draw_single_item(state, from_path, to_path, &mut patches) {
            Ok(_) => {},
            Err(e) => {
                if i > 0 {
                    // Partial success - drew some cards
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }
    
    Ok(patches)
}

fn reshuffle_deck(state: &mut Value, from_path: &str, to_path: &str, patches: &mut Vec<Value>) -> Result<(), String> {
    let from_zone = get_zone_mut(state, from_path)?;
    let items = from_zone["items"].as_array_mut()
        .ok_or("Discard pile is not a list")?;
    
    if items.is_empty() {
        return Err("Cannot reshuffle empty discard pile".to_string());
    }
    
    // Move all cards from discard to deck
    let cards: Vec<Value> = items.drain(..).collect();
    patches.push(json!({
        "op": "replace",
        "path": format!("{}/items", from_path),
        "value": []
    }));
    
    // Shuffle the cards (would need RNG from state)
    // For now, just move them as-is
    let to_zone = get_zone_mut(state, to_path)?;
    let to_items = to_zone["items"].as_array_mut()
        .ok_or("Deck is not a list")?;
    
    for (i, card) in cards.into_iter().enumerate() {
        to_items.push(card.clone());
        patches.push(json!({
            "op": "add",
            "path": format!("{}/items/{}", to_path, i),
            "value": card
        }));
    }
    
    Ok(())
}

/// Validate if a set of cards forms a valid meld (set or run)
fn apply_validate_meld(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let cards = args["cards"].as_array().ok_or("Missing 'cards' array")?;
    let meld_type = args.get("type").and_then(|v| v.as_str());
    let result_path = args["resultPath"].as_str().ok_or("Missing 'resultPath'")?;
    
    let is_valid = match meld_type {
        Some("set") => validate_set(cards),
        Some("run") => validate_run(cards),
        _ => validate_set(cards) || validate_run(cards), // Either type
    };
    
    Ok(vec![json!({
        "op": "replace",
        "path": result_path,
        "value": is_valid
    })])
}

fn validate_set(cards: &[Value]) -> bool {
    if cards.len() < 3 {
        return false;
    }
    
    // All cards must have same rank
    let first_rank = cards[0]["rank"].as_str();
    if first_rank.is_none() {
        return false;
    }
    
    cards.iter().all(|card| {
        card["rank"].as_str() == first_rank
    })
}

fn validate_run(cards: &[Value]) -> bool {
    if cards.len() < 3 {
        return false;
    }
    
    // All cards must be same suit and consecutive ranks
    let first_suit = cards[0]["suit"].as_str();
    if first_suit.is_none() {
        return false;
    }
    
    // Check all same suit
    if !cards.iter().all(|card| card["suit"].as_str() == first_suit) {
        return false;
    }
    
    // Get ranks and sort
    let mut ranks: Vec<i32> = cards.iter()
        .filter_map(|card| {
            let rank = card["rank"].as_str()?;
            match rank {
                "A" => Some(1),  // Can also be 14 in some games
                "J" => Some(11),
                "Q" => Some(12),
                "K" => Some(13),
                num => num.parse().ok(),
            }
        })
        .collect();
    
    if ranks.len() != cards.len() {
        return false;
    }
    
    ranks.sort();
    
    // Check consecutive
    for i in 1..ranks.len() {
        if ranks[i] != ranks[i-1] + 1 {
            // Check for Ace-high run (Q-K-A)
            if i == ranks.len() - 1 && ranks[0] == 1 && ranks[i-1] == 13 {
                continue;
            }
            return false;
        }
    }
    
    true
}

/// Check if a card matches specified criteria (rank and/or suit)
fn apply_match_card(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let card_path = args["card"].as_str().ok_or("Missing 'card' path")?;
    let match_rank = args.get("matchRank").and_then(|v| v.as_str());
    let match_suit = args.get("matchSuit").and_then(|v| v.as_str());
    let result_path = args["resultPath"].as_str().ok_or("Missing 'resultPath'")?;
    
    // Get the card
    let card = get_cell_value(state, card_path)?;
    
    let mut matches = true;
    
    if let Some(rank) = match_rank {
        matches = matches && card["rank"].as_str() == Some(rank);
    }
    
    if let Some(suit) = match_suit {
        matches = matches && card["suit"].as_str() == Some(suit);
    }
    
    Ok(vec![json!({
        "op": "replace",
        "path": result_path,
        "value": matches
    })])
}

/// Remove all pairs from a player's hand
fn apply_remove_pairs(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let hand_path = args["hand"].as_str().ok_or("Missing 'hand' path")?;
    let pairs_path = args.get("pairsZone").and_then(|v| v.as_str());
    
    let hand_zone = get_zone_mut(state, hand_path)?;
    let items = hand_zone["items"].as_array_mut()
        .ok_or("Hand is not a list")?;
    
    let mut patches = Vec::new();
    let mut pairs_found = Vec::new();
    
    // Find all pairs by rank
    let mut rank_counts: std::collections::HashMap<String, Vec<(usize, Value)>> = std::collections::HashMap::new();
    
    for (i, card) in items.iter().enumerate() {
        if let Some(rank) = card["rank"].as_str() {
            rank_counts.entry(rank.to_string())
                .or_insert(Vec::new())
                .push((i, card.clone()));
        }
    }
    
    // Collect pairs
    for (rank, cards) in rank_counts {
        if cards.len() >= 2 {
            // Take pairs (2 at a time)
            for chunk in cards.chunks(2) {
                if chunk.len() == 2 {
                    pairs_found.push((chunk[0].0, chunk[1].0));
                }
            }
        }
    }
    
    // Remove pairs from hand (in reverse order to maintain indices)
    let mut indices_to_remove: Vec<usize> = pairs_found.iter()
        .flat_map(|(a, b)| vec![*a, *b])
        .collect();
    indices_to_remove.sort_by(|a, b| b.cmp(a));
    
    for idx in indices_to_remove {
        items.remove(idx);
        patches.push(json!({
            "op": "remove",
            "path": format!("{}/items/{}", hand_path, idx)
        }));
    }
    
    // If pairs zone specified, add pairs there
    if let Some(zone_path) = pairs_path {
        let pairs_zone = get_zone_mut(state, zone_path)?;
        let pairs_items = pairs_zone["items"].as_array_mut()
            .ok_or("Pairs zone is not a list")?;
        
        for (i, _pair) in pairs_found.iter().enumerate() {
            pairs_items.push(json!({"id": format!("pair_{}", i)}));
            patches.push(json!({
                "op": "add", 
                "path": format!("{}/items/{}", zone_path, i),
                "value": {"id": format!("pair_{}", i)}
            }));
        }
    }
    
    Ok(patches)
}

/// Make a bid (for games like Oh Hell)
fn apply_make_bid(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let player = args["player"].as_str().ok_or("Missing 'player'")?;
    let bid = args["bid"].as_u64().ok_or("Missing 'bid'")? as i32;
    let bid_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("tricks");
    
    let mut patches = Vec::new();
    
    // Store bid in state
    let bid_path = format!("/game/bids/{}", player);
    patches.push(json!({
        "op": "replace",
        "path": bid_path,
        "value": {
            "amount": bid,
            "type": bid_type
        }
    }));
    
    // Check for hook rule if specified
    if let Some(true) = args.get("enforceHook").and_then(|v| v.as_bool()) {
        // Calculate total bids
        let bids = state["game"]["bids"].as_object();
        let total_bids: i32 = bids.map(|b| {
            b.values()
                .filter_map(|v| v["amount"].as_i64())
                .sum::<i64>() as i32
        }).unwrap_or(0);
        
        let available_tricks = args.get("availableTricks")
            .and_then(|v| v.as_i64())
            .unwrap_or(13) as i32;
        
        if total_bids == available_tricks {
            return Err("Total bids cannot equal available tricks (hook rule)".to_string());
        }
    }
    
    Ok(patches)
}

/// Play a card to the current trick
fn apply_play_to_trick(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let player = args["player"].as_str().ok_or("Missing 'player'")?;
    let card_path = args["card"].as_str().ok_or("Missing 'card' path")?;
    let trick_path = args.get("trickPath")
        .and_then(|v| v.as_str())
        .unwrap_or("/game/currentTrick");
    
    let mut patches = Vec::new();
    
    // Get the card
    let card = get_cell_value(state, card_path)?.clone();
    
    // Remove from player's hand
    patches.push(json!({
        "op": "remove",
        "path": card_path
    }));
    
    // Add to current trick
    let trick_card_path = format!("{}/{}", trick_path, player);
    patches.push(json!({
        "op": "replace",
        "path": trick_card_path,
        "value": card
    }));
    
    Ok(patches)
}

/// Resolve a completed trick and determine winner
fn apply_resolve_trick(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let trick_path = args.get("trickPath")
        .and_then(|v| v.as_str())
        .unwrap_or("/game/currentTrick");
    let trump_suit = args.get("trumpSuit").and_then(|v| v.as_str());
    let led_suit_path = args.get("ledSuit").and_then(|v| v.as_str());
    
    let mut patches = Vec::new();
    
    // Get current trick
    let trick = state.pointer(trick_path)
        .ok_or("Trick not found")?
        .as_object()
        .ok_or("Trick is not an object")?;
    
    // Determine led suit
    let led_suit = if let Some(path) = led_suit_path {
        state.pointer(path).and_then(|v| v.as_str())
    } else {
        // Get from first card played
        trick.values().next()
            .and_then(|card| card["suit"].as_str())
    };
    
    // Find winning card
    let mut winner = None;
    let mut highest_value = -1;
    
    for (player, card) in trick.iter() {
        let suit = card["suit"].as_str().unwrap_or("");
        let rank_value = get_card_trick_value(card, suit, led_suit, trump_suit);
        
        if rank_value > highest_value {
            highest_value = rank_value;
            winner = Some(player.clone());
        }
    }
    
    if let Some(winner) = winner {
        // Store trick winner
        patches.push(json!({
            "op": "replace",
            "path": "/game/lastTrickWinner",
            "value": winner
        }));
        
        // Clear current trick
        patches.push(json!({
            "op": "replace",
            "path": trick_path,
            "value": {}
        }));
        
        // Increment tricks won
        let won_path = format!("/game/tricksWon/{}", winner);
        let current_won = state.pointer(&won_path)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        patches.push(json!({
            "op": "replace",
            "path": won_path,
            "value": current_won + 1
        }));
    }
    
    Ok(patches)
}

fn get_card_trick_value(card: &Value, suit: &str, led_suit: Option<&str>, trump_suit: Option<&str>) -> i32 {
    let base_value = match card["rank"].as_str().unwrap_or("") {
        "2" => 2,
        "3" => 3,
        "4" => 4,
        "5" => 5,
        "6" => 6,
        "7" => 7,
        "8" => 8,
        "9" => 9,
        "10" => 10,
        "J" => 11,
        "Q" => 12,
        "K" => 13,
        "A" => 14,
        _ => 0,
    };
    
    // Trump suit beats all
    if let Some(trump) = trump_suit {
        if suit == trump {
            return base_value + 100;
        }
    }
    
    // Must follow suit to win (unless trump)
    if let Some(led) = led_suit {
        if suit == led {
            return base_value;
        }
    }
    
    // Off suit, can't win
    0
}
use crate::bundle::Bundle;
use crate::conditions::evaluate_condition;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Action map computation that handles simplified state structure  
pub fn compute_action_map(
    state: &Value,
    bundle: &Bundle,
) -> serde_json::Map<String, Value> {
    let mut player_action_maps = serde_json::Map::new();
    
    println!("[action_map] Starting compute_action_map");
    println!("[action_map] State keys: {:?}", state.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    
    // Check if game has ended using simplified gameStatus
    if is_game_ended(state) {
        println!("[action_map] Game has ended, returning empty action maps");
        return create_empty_action_maps(state);
    }
    
    // Get current player
    let current_player = state.get("currentPlayer")
        .and_then(|cp| cp.as_str())
        .unwrap_or("");
    
    println!("[action_map] Current turn player: {}", current_player);
    
    // Get players array
    let empty_players = Vec::new();
    let players = state.get("players")
        .and_then(|p| p.as_array())
        .unwrap_or(&empty_players);
    
    println!("[action_map] Found {} players", players.len());
    
    // Compute action map for each player
    for (idx, _player) in players.iter().enumerate() {
        let player_id = format!("p{}", idx + 1);
        println!("[action_map] Computing actions for player {}", player_id);
        
        // Use a simplified action map computation
        // that focuses on basic actions without complex phase handling
        let player_actions = compute_player_actions(state, bundle, &player_id);
        
        player_action_maps.insert(player_id, Value::Object(player_actions));
    }
    
    player_action_maps
}

/// Check if game has ended using simplified gameStatus field
fn is_game_ended(state: &Value) -> bool {
    println!("[action_map] Checking if game ended. State has gameStatus: {:?}", 
             state.get("gameStatus"));
    
    if let Some(game_status) = state.get("gameStatus") {
        // Check if it's an object with state field
        if let Some(status_state) = game_status.get("state").and_then(|s| s.as_str()) {
            let ended = status_state == "ended";
            println!("[action_map] Game status state: '{}', ended: {}", status_state, ended);
            ended
        } else if let Some(status_str) = game_status.as_str() {
            // Fallback for string status
            let ended = status_str != "playing";
            println!("[action_map] Game status string: '{}', ended: {}", status_str, ended);
            ended
        } else {
            println!("[action_map] Game status is not string or object with state field");
            false
        }
    } else {
        println!("[action_map] No gameStatus field found");
        false
    }
}

/// Create empty action maps for all players when game has ended
fn create_empty_action_maps(state: &Value) -> serde_json::Map<String, Value> {
    let mut empty_maps = serde_json::Map::new();
    
    if let Some(players) = state.get("players").and_then(|p| p.as_array()) {
        for (idx, _) in players.iter().enumerate() {
            let player_id = format!("p{}", idx + 1);
            empty_maps.insert(player_id, json!({}));
        }
    }
    
    empty_maps
}

/// Compute actions available to a specific player
fn compute_player_actions(
    state: &Value,
    bundle: &Bundle,
    player_id: &str,
) -> serde_json::Map<String, Value> {
    let mut action_map = serde_json::Map::new();
    
    // Get current player
    let current_player = state.get("currentPlayer")
        .and_then(|cp| cp.as_str())
        .unwrap_or("");
    
    // Only compute actions for the current player
    if player_id != current_player {
        return action_map;
    }
    
    // Get all actions from bundle
    let actions = match &bundle.actions {
        Value::Array(arr) => arr,
        _ => return action_map,
    };
    
    println!("[action_map] Processing {} actions for current player {}", actions.len(), player_id);
    
    // Process each action
    for action in actions {
        let action_id = action.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Skip auto actions
        let is_auto = action.get("auto")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if is_auto {
            continue;
        }
        
        // Get action type early to check for place actions
        let action_type = action.get("uses")
            .or_else(|| action.get("builtin"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Check if action has when conditions
        if let Some(when_conditions) = action.get("when").and_then(|w| w.as_array()) {
            let mut all_conditions_met = true;
            
            // For place-type actions, we need to handle conditions differently
            // Some conditions depend on the specific location which isn't known yet
            let is_place_action = action_type == "place" || action_type == "placeWithGravity";
            
            for condition in when_conditions {
                // Check if this condition has template variables that would be resolved later
                let has_template_args = if let Some(with_obj) = condition.get("with") {
                    let with_str = with_obj.to_string();
                    with_str.contains("{args.")
                } else {
                    false
                };
                
                // Skip template-based conditions for place actions during action map generation
                if is_place_action && has_template_args {
                    println!("[action_map] Skipping template-based condition for place action {}", action_id);
                    continue;
                }
                
                match evaluate_condition(condition, state, &json!({}), player_id) {
                    Ok(true) => continue,
                    Ok(false) => {
                        all_conditions_met = false;
                        break;
                    }
                    Err(e) => {
                        println!("[action_map] Error evaluating condition for action {}: {}", action_id, e);
                        all_conditions_met = false;
                        break;
                    }
                }
            }
            
            if !all_conditions_met {
                println!("[action_map] Skipping action {} due to unmet conditions", action_id);
                continue;
            }
        }
        
        println!("[action_map] Processing action {} of type {}", action_id, action_type);
        
        // Handle different action types
        match action_type {
            "place" => {
                // Get the action's with field for proper entity and argument names
                let with_clause = action.get("with");
                
                // Extract entity template from action definition
                let default_entity = format!("piece_{}", player_id);
                let entity_template = with_clause
                    .and_then(|w| w.get("entity"))
                    .and_then(|e| e.as_str())
                    .unwrap_or(&default_entity);
                
                // Replace {player} in entity template
                let entity = entity_template.replace("{player}", player_id);
                
                // For place actions, find all empty cells on the board
                if let Some(zones) = state.get("zones") {
                    if let Some(board) = zones.get("board") {
                        if let Some(cells) = board.get("cells").and_then(|c| c.as_array()) {
                            for (row_idx, row) in cells.iter().enumerate() {
                                if let Some(row_array) = row.as_array() {
                                    for (col_idx, cell) in row_array.iter().enumerate() {
                                        if cell.is_null() {
                                            let location = format!("/zones/board/cells/{}/{}", row_idx, col_idx);
                                            action_map.insert(location.clone(), json!({
                                                "action": action_id,
                                                "args": {
                                                    "location": location,
                                                    "entity": entity
                                                }
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "placeWithGravity" => {
                // For Connect Four style games - check columns
                if let Some(zones) = state.get("zones") {
                    if let Some(board) = zones.get("board") {
                        if let Some(cells) = board.get("cells").and_then(|c| c.as_array()) {
                            // Check each column to see if it has space
                            if let Some(first_row) = cells.first().and_then(|r| r.as_array()) {
                                for col_idx in 0..first_row.len() {
                                    // Check if column has any empty cell
                                    let has_space = cells.iter().any(|row| {
                                        row.as_array()
                                            .and_then(|r| r.get(col_idx))
                                            .map(|cell| cell.is_null())
                                            .unwrap_or(false)
                                    });
                                    
                                    if has_space {
                                        // Use cell path for the top row
                                        let location = format!("/zones/board/cells/0/{}", col_idx);
                                        action_map.insert(location, json!({
                                            "action": action_id,
                                            "args": {
                                                "targetColumn": col_idx,
                                                "player": player_id
                                            }
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // Check if this is a multi-step action
                let is_multi_step = action.get("isMultiStep")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                if is_multi_step {
                    // For multi-step actions, we need to check if the first step has choice zones
                    // and generate individual choice entries instead of a single multi-step entry
                    println!("[action_map] Processing multi-step action: {}", action_id);
                    
                    // Check if we should generate choice-based actions by looking at choice zones
                    if let Some(choice_actions) = generate_choice_based_actions(state, action, player_id) {
                        for (key, value) in choice_actions {
                            action_map.insert(key, value);
                        }
                    } else {
                        // Fallback to multi-step format if no choice zones found
                        let direction = action.get("ui")
                            .and_then(|ui| ui.get("direction"))
                            .and_then(|d| d.as_str())
                            .unwrap_or("Select an option");
                        
                        let multi_step_key = format!("_multiStep_{}", action_id);
                        action_map.insert(multi_step_key, json!({
                            "action": action_id,
                            "type": "multiStep",
                            "direction": direction,
                            "args": {}
                        }));
                    }
                } else {
                    println!("[action_map] Skipping action type: {}", action_type);
                }
            }
        }
    }
    
    println!("[action_map] Generated {} actions for player {}", action_map.len(), player_id);
    action_map
}

/// Generate choice-based actions for multi-step actions when choice zones exist
fn generate_choice_based_actions(
    state: &Value,
    action: &Value,
    player_id: &str,
) -> Option<serde_json::Map<String, Value>> {
    let mut choice_actions = serde_json::Map::new();
    
    // Get the first step to see if it involves choice zones
    let steps = action.get("steps").and_then(|s| s.as_array())?;
    let first_step = steps.first()?;
    
    // Check if first step is a choice selection
    let step_type = first_step.get("as").and_then(|t| t.as_str())?;
    if step_type != "bf.selectChoice" {
        return None;
    }
    
    // Get the choice zone pattern from the first step
    let choice_zone_pattern = first_step.get("with")
        .and_then(|w| w.get("choiceZone"))
        .and_then(|cz| cz.as_str())?;
    
    // Replace {actor} with the actual player
    let choice_zone = choice_zone_pattern.replace("{actor}", player_id);
    
    println!("[action_map] Looking for choice zone: {}", choice_zone);
    
    // Look for the choice zone in state
    // Choice zones are typically at /zones/choice_{player} or similar
    let zone_path = if choice_zone.starts_with("/zones/") {
        &choice_zone[7..] // Remove "/zones/" prefix
    } else {
        &choice_zone
    };
    
    let zones = state.get("zones")?;
    let choice_zone_data = zones.get(zone_path)?;
    
    println!("[action_map] Found choice zone data: {:?}", choice_zone_data);
    
    // Look for choices in the zone
    if let Some(choices) = choice_zone_data.get("choices").and_then(|c| c.as_object()) {
        let action_id = action.get("id").and_then(|id| id.as_str()).unwrap_or("unknown");
        let direction = first_step.get("ui")
            .and_then(|ui| ui.get("direction"))
            .and_then(|d| d.as_str())
            .unwrap_or("Select an option");
        
        for (choice_key, _choice_value) in choices {
            let location = format!("/zones/{}/{}", zone_path, choice_key);
            let step_id = first_step.get("id").and_then(|id| id.as_str()).unwrap_or("step");
            
            // Action name should be just the step ID, not prefixed with the main action ID
            // Create args based on the step type and choice
            let mut args = json!({
                "location": location,
                "player": player_id
            });
            
            // Add step-specific arguments based on the step ID
            if step_id == "selectRank" {
                args.as_object_mut().unwrap().insert("rank".to_string(), json!(choice_key));
            } else if step_id == "selectPlayer" {
                args.as_object_mut().unwrap().insert("targetPlayer".to_string(), json!(choice_key));
            } else {
                // Generic choice for other steps
                args.as_object_mut().unwrap().insert("choice".to_string(), json!(choice_key));
            }
            
            choice_actions.insert(location.clone(), json!({
                "action": step_id,
                "direction": direction,
                "args": args
            }));
        }
        
        println!("[action_map] Generated {} choice actions", choice_actions.len());
        return Some(choice_actions);
    }
    
    None
}
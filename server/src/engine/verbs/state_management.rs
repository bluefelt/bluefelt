use serde_json::{json, Value};
use crate::bundle::Bundle;

pub fn apply_set_state(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
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
        // For other paths, store the value directly in the state root
        // Remove leading slash if present
        let key = path.strip_prefix('/').unwrap_or(path);
        
        if let Some(state_obj) = state.as_object_mut() {
            state_obj.insert(key.to_string(), value.clone());
        }
        
        // Create patch for client sync
        patches.push(json!({
            "op": "replace",
            "path": format!("/game{}", path),
            "value": value
        }));
    }
    
    Ok(patches)
}

pub fn apply_set_phase(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let phase_set = args["phaseSet"].as_str().ok_or("Missing 'phaseSet'")?;
    let phase = args["phase"].as_str().ok_or("Missing 'phase'")?;
    
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    let phases = state_obj.get_mut("phases")
        .and_then(|p| p.as_object_mut())
        .ok_or("Missing phases in state")?;
    
    // Update the current phase structure to match client expectations
    if let Some(current) = phases.get_mut("current").and_then(|c| c.as_object_mut()) {
        current.insert(phase_set.to_string(), json!(phase));
    } else {
        // Create current structure if it doesn't exist
        phases.insert("current".to_string(), json!({
            phase_set: phase
        }));
    }
    
    Ok(vec![json!({
        "op": "replace",
        "path": format!("/game/phases/current/{}", phase_set),
        "value": phase
    })])
}

pub fn apply_next_turn(state: &mut Value, _args: &Value, bundle: &Bundle) -> Result<Vec<Value>, String> {
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    
    // Increment tick
    let current_tick = state_obj["tick"].as_u64().unwrap_or(0);
    let new_tick = current_tick + 1;
    state_obj.insert("tick".to_string(), json!(new_tick));
    
    // Advance turn - use actual player count from state, not max players
    let actual_player_count = state_obj.get("players")
        .and_then(|p| p.as_array())
        .map(|arr| arr.len())
        .unwrap_or(bundle.manifest.metadata.players.max as usize) as u64;
    
    let current_turn = state_obj["turn"].as_u64().unwrap_or(0);
    let next_turn = (current_turn + 1) % actual_player_count;
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
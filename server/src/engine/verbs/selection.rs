use serde_json::{json, Value};
use crate::engine::path::get_cell_value;

pub fn apply_select_entity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
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

pub fn apply_clear_selection(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
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

pub fn apply_select_player(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
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


#[cfg(test)]
#[path = "selection_tests.rs"]
mod tests;
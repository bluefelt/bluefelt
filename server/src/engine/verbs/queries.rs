use serde_json::{json, Value};
use crate::bundle::Bundle;
use crate::engine::path::get_zone_mut;

pub fn apply_query_entities(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
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

pub fn apply_check_player_cards(state: &mut Value, args: &Value, bundle: &Bundle) -> Result<Vec<Value>, String> {
    // This function will be handled by the conditionalAction logic in the actions.yaml
    // For now, just return empty patches since the logic is in the action definition
    Ok(Vec::new())
}
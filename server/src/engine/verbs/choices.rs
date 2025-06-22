use serde_json::{json, Value};
use crate::engine::path::get_zone_mut;

pub fn apply_present_choice(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
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

pub fn apply_make_selection(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
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
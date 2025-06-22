//! Transfer entity verb - works with simplified zone structure

use serde_json::{json, Value};
use crate::bundle::Bundle;

pub fn execute(args: &Value, bundle: &Bundle, state: &mut Value) -> Result<Vec<Value>, String> {
    let from_zone = args["from"].as_str()
        .ok_or("transferEntity requires 'from' zone")?;
    let to_zone = args["to"].as_str()
        .ok_or("transferEntity requires 'to' zone")?;
    let entity = args.get("entity")
        .ok_or("transferEntity requires 'entity'")?;
    
    let mut patches = Vec::new();
    
    // Remove from source zone
    let removed = remove_entity_from_zone(state, from_zone, entity)?;
    if removed {
        patches.push(json!({
            "op": "remove",
            "path": format!("/zones/{}/items/-", from_zone),
            "value": entity
        }));
    }
    
    // Add to destination zone
    add_entity_to_zone(state, to_zone, entity.clone())?;
    patches.push(json!({
        "op": "add",
        "path": format!("/zones/{}/items/-", to_zone),
        "value": entity
    }));
    
    Ok(patches)
}

fn remove_entity_from_zone(state: &mut Value, zone_id: &str, entity: &Value) -> Result<bool, String> {
    let zone = state["zones"].get_mut(zone_id)
        .ok_or_else(|| format!("Zone not found: {}", zone_id))?;
    
    // Handle different zone shapes
    if let Some(items) = zone.get_mut("items").and_then(|i| i.as_array_mut()) {
        // List/stack zone
        if let Some(pos) = items.iter().position(|e| e == entity) {
            items.remove(pos);
            return Ok(true);
        }
    } else if let Some(cells) = zone.get_mut("cells") {
        // Grid zone - search through cells
        if let Some(grid) = cells.as_array_mut() {
            for row in grid {
                if let Some(row_array) = row.as_array_mut() {
                    for cell in row_array {
                        if cell == entity {
                            *cell = json!(null);
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    
    Ok(false)
}

fn add_entity_to_zone(state: &mut Value, zone_id: &str, entity: Value) -> Result<(), String> {
    let zone = state["zones"].get_mut(zone_id)
        .ok_or_else(|| format!("Zone not found: {}", zone_id))?;
    
    // Handle different zone shapes
    if let Some(items) = zone.get_mut("items").and_then(|i| i.as_array_mut()) {
        // List/stack zone - just append
        items.push(entity);
    } else if let Some(cells) = zone.get_mut("cells") {
        // Grid zone - find first empty cell
        if let Some(grid) = cells.as_array_mut() {
            for row in grid {
                if let Some(row_array) = row.as_array_mut() {
                    for cell in row_array {
                        if cell.is_null() {
                            *cell = entity;
                            return Ok(());
                        }
                    }
                }
            }
        }
        return Err("No empty cells in grid zone".to_string());
    }
    
    Ok(())
}
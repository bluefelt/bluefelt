//! Message formatting utilities for different client capabilities

use serde_json::{json, Value, Map};

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageFormat {
    /// Standard format with nested JSON and path-based action maps
    Standard,
    /// Simplified format for clients with limited JSON support (like Unity)
    Simple,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateFormat {
    /// JSON Patch format (default)
    Patch,
    /// Full state updates
    Full,
}

impl From<&str> for MessageFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "simple" => MessageFormat::Simple,
            _ => MessageFormat::Standard,
        }
    }
}

impl From<&str> for UpdateFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "full" => UpdateFormat::Full,
            _ => UpdateFormat::Patch,
        }
    }
}

/// Convert zones from nested format to array format for simple clients
pub fn simplify_zones(zones: &Value) -> Value {
    if let Some(zones_obj) = zones.as_object() {
        let mut zones_array = Vec::new();
        
        for (id, zone_data) in zones_obj {
            let mut zone_obj = Map::new();
            zone_obj.insert("id".to_string(), json!(id));
            
            // Determine zone type and structure
            if let Some(arr) = zone_data.as_array() {
                if !arr.is_empty() && arr[0].is_array() {
                    // Grid zone
                    zone_obj.insert("type".to_string(), json!("grid"));
                    zone_obj.insert("contents".to_string(), zone_data.clone());
                    zone_obj.insert("rows".to_string(), json!(arr.len()));
                    if let Some(first_row) = arr[0].as_array() {
                        zone_obj.insert("cols".to_string(), json!(first_row.len()));
                    }
                } else {
                    // List zone (simple array)
                    zone_obj.insert("type".to_string(), json!("list"));
                    zone_obj.insert("contents".to_string(), zone_data.clone());
                }
            } else if let Some(obj) = zone_data.as_object() {
                if obj.contains_key("items") {
                    // List zone with items
                    zone_obj.insert("type".to_string(), json!("list"));
                    zone_obj.insert("contents".to_string(), obj["items"].clone());
                } else {
                    // Unknown structure, preserve as-is
                    zone_obj.insert("type".to_string(), json!("unknown"));
                    zone_obj.insert("contents".to_string(), zone_data.clone());
                }
            } else {
                // Single value zone
                zone_obj.insert("type".to_string(), json!("single"));
                zone_obj.insert("contents".to_string(), zone_data.clone());
            }
            
            zones_array.push(json!(zone_obj));
        }
        
        json!(zones_array)
    } else {
        zones.clone()
    }
}

/// Convert action map from path-based to structured format
pub fn simplify_action_map(action_map: &Value) -> Value {
    if let Some(player_actions) = action_map.as_object() {
        let mut simplified = Map::new();
        
        for (player_id, actions) in player_actions {
            if let Some(actions_obj) = actions.as_object() {
                let mut player_actions = Vec::new();
                let mut action_groups: std::collections::HashMap<String, Vec<Value>> = std::collections::HashMap::new();
                
                // Group actions by action ID
                for (path, action_info) in actions_obj {
                    if let Some(action_obj) = action_info.as_object() {
                        if let Some(action_id) = action_obj.get("action").and_then(|a| a.as_str()) {
                            let target = parse_action_path(path);
                            action_groups.entry(action_id.to_string())
                                .or_insert_with(Vec::new)
                                .push(target);
                        }
                    }
                }
                
                // Convert grouped actions to structured format
                for (action_id, targets) in action_groups {
                    let direction = actions_obj.values()
                        .find_map(|v| {
                            if v["action"] == action_id {
                                v["direction"].as_str()
                            } else {
                                None
                            }
                        })
                        .unwrap_or("Select");
                    
                    player_actions.push(json!({
                        "id": action_id,
                        "targets": targets,
                        "description": direction
                    }));
                }
                
                simplified.insert(player_id.clone(), json!(player_actions));
            } else {
                // Empty actions
                simplified.insert(player_id.clone(), json!([]));
            }
        }
        
        json!(simplified)
    } else {
        action_map.clone()
    }
}

/// Parse an action path like "/zones/board/0/1" into structured format
fn parse_action_path(path: &str) -> Value {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    
    if parts.len() >= 2 && parts[0] == "zones" {
        let zone_id = parts[1];
        
        if parts.len() == 2 {
            // Whole zone action
            json!({
                "zone": zone_id,
                "type": "zone"
            })
        } else if parts.len() == 3 {
            // List zone with index
            if let Ok(index) = parts[2].parse::<u64>() {
                json!({
                    "zone": zone_id,
                    "index": index,
                    "type": "index"
                })
            } else {
                json!({
                    "zone": zone_id,
                    "type": "zone"
                })
            }
        } else if parts.len() == 4 {
            // Grid zone with row/col
            if let (Ok(row), Ok(col)) = (parts[2].parse::<u64>(), parts[3].parse::<u64>()) {
                json!({
                    "zone": zone_id,
                    "position": [row, col],
                    "type": "grid"
                })
            } else {
                json!({
                    "zone": zone_id,
                    "type": "zone"
                })
            }
        } else {
            json!({
                "path": path,
                "type": "unknown"
            })
        }
    } else {
        json!({
            "path": path,
            "type": "unknown"
        })
    }
}

/// Format a welcome message based on client capabilities
pub fn format_welcome_message(
    base_message: Value,
    format: MessageFormat,
) -> Value {
    match format {
        MessageFormat::Standard => base_message,
        MessageFormat::Simple => {
            let mut simplified = base_message.clone();
            
            // Simplify zones in state if present
            if let Some(state) = simplified.get_mut("state") {
                if let Some(zones) = state.get("zones") {
                    state["zones"] = simplify_zones(zones);
                }
            }
            
            // Simplify action map in meta
            if let Some(meta) = simplified.get_mut("meta") {
                if let Some(action_map) = meta.get("actionMap") {
                    meta["availableActions"] = simplify_action_map(action_map);
                    // Keep both for compatibility during transition
                    // meta.as_object_mut().unwrap().remove("actionMap");
                }
                
                // Add explicit zone type information
                if let Some(zones) = meta.get("zones") {
                    meta["zoneTypes"] = build_zone_types(zones);
                }
            }
            
            simplified
        }
    }
}

/// Build zone type information from zone metadata
fn build_zone_types(zones: &Value) -> Value {
    if let Some(zones_array) = zones.as_array() {
        let mut zone_types = Map::new();
        
        for zone in zones_array {
            if let Some(zone_obj) = zone.as_object() {
                if let Some(id) = zone_obj.get("id").and_then(|id| id.as_str()) {
                    let zone_type = zone_obj.get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("unknown");
                    
                    zone_types.insert(id.to_string(), json!({
                        "type": zone_type,
                        "name": zone_obj.get("name").cloned().unwrap_or(json!(id))
                    }));
                }
            }
        }
        
        json!(zone_types)
    } else {
        json!({})
    }
}

/// Convert a JSON Patch diff to a full state update
pub fn patch_to_full_state(
    current_state: &Value,
    patch: &Value,
    tick: u64,
) -> Value {
    let mut updated_state = current_state.clone();
    
    // Apply patches to get the new state
    if let Some(patches) = patch.as_array() {
        for p in patches {
            apply_single_patch(&mut updated_state, p);
        }
    }
    
    json!({
        "type": "fullState",
        "tick": tick,
        "game": updated_state.get("game").cloned().unwrap_or(json!({})),
        "ui": updated_state.get("ui").cloned().unwrap_or(json!({}))
    })
}

/// Apply a single patch operation to a value
fn apply_single_patch(target: &mut Value, patch: &Value) {
    if let (Some(op), Some(path)) = (
        patch.get("op").and_then(|o| o.as_str()),
        patch.get("path").and_then(|p| p.as_str())
    ) {
        let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        
        match op {
            "replace" | "add" => {
                if let Some(value) = patch.get("value") {
                    set_value_at_path(target, &path_parts, value.clone());
                }
            }
            "remove" => {
                remove_value_at_path(target, &path_parts);
            }
            _ => {}
        }
    }
}

/// Set a value at a given path in a JSON structure
fn set_value_at_path(target: &mut Value, path: &[&str], value: Value) {
    if path.is_empty() {
        *target = value;
        return;
    }
    
    let mut current = target;
    for (i, segment) in path.iter().enumerate() {
        if i == path.len() - 1 {
            // Last segment - set the value
            if let Some(obj) = current.as_object_mut() {
                obj.insert(segment.to_string(), value);
            } else if let Some(arr) = current.as_array_mut() {
                if let Ok(index) = segment.parse::<usize>() {
                    if index < arr.len() {
                        arr[index] = value;
                    }
                } else if *segment == "-" {
                    arr.push(value);
                }
            }
            return;
        }
        
        // Navigate to the next level
        current = if let Ok(index) = segment.parse::<usize>() {
            if let Some(arr) = current.as_array_mut() {
                if index < arr.len() {
                    &mut arr[index]
                } else {
                    return;
                }
            } else {
                return;
            }
        } else if let Some(obj) = current.as_object_mut() {
            obj.entry(segment.to_string()).or_insert(json!({}))
        } else {
            return;
        };
    }
}

/// Remove a value at a given path in a JSON structure
fn remove_value_at_path(target: &mut Value, path: &[&str]) {
    if path.is_empty() {
        return;
    }
    
    let mut current = target;
    for (i, segment) in path.iter().enumerate() {
        if i == path.len() - 1 {
            // Last segment - remove the value
            if let Some(obj) = current.as_object_mut() {
                obj.remove(*segment);
            } else if let Some(arr) = current.as_array_mut() {
                if let Ok(index) = segment.parse::<usize>() {
                    if index < arr.len() {
                        arr.remove(index);
                    }
                }
            }
            return;
        }
        
        // Navigate to the next level
        current = if let Ok(index) = segment.parse::<usize>() {
            if let Some(arr) = current.as_array_mut() {
                if index < arr.len() {
                    &mut arr[index]
                } else {
                    return;
                }
            } else {
                return;
            }
        } else if let Some(obj) = current.as_object_mut() {
            if let Some(next) = obj.get_mut(*segment) {
                next
            } else {
                return;
            }
        } else {
            return;
        };
    }
}
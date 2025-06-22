//! Entity-centric UI system
//! 
//! This module handles adding UI hints and interaction metadata directly to entities,
//! replacing the centralized action map approach.

use serde_json::{json, Value};
use crate::bundle::Bundle;
use std::collections::HashMap;

/// Enhance entities with UI metadata based on current game state
pub fn enhance_entities_with_ui(
    state: &Value,
    bundle: &Bundle,
    player_id: &str,
) -> Result<HashMap<String, Value>, String> {
    let mut entity_ui = HashMap::new();
    
    // Get available actions for this player
    let available_actions = get_available_actions(state, bundle, player_id)?;
    
    // Process each zone and its entities
    if let Some(zones) = state.get("zones").and_then(|z| z.as_object()) {
        for (zone_id, zone) in zones {
            process_zone_entities(
                zone_id,
                zone,
                &available_actions,
                player_id,
                &mut entity_ui,
            )?;
        }
    }
    
    Ok(entity_ui)
}

/// Get available actions for a player
fn get_available_actions(
    state: &Value,
    bundle: &Bundle,
    player_id: &str,
) -> Result<Vec<Value>, String> {
    let mut available = Vec::new();
    
    // Get actions from bundle
    if let Some(actions) = bundle.actions.as_array() {
        for action in actions {
            if is_action_available(action, state, player_id)? {
                available.push(action.clone());
            }
        }
    }
    
    Ok(available)
}

/// Check if an action is available for a player
fn is_action_available(
    action: &Value,
    state: &Value,
    player_id: &str,
) -> Result<bool, String> {
    // Check player restrictions
    if let Some(allowed_players) = action.get("allowedPlayers") {
        if !check_player_allowed(allowed_players, state, player_id)? {
            return Ok(false);
        }
    }
    
    // Check phase restrictions
    if let Some(allowed_phases) = action.get("allowedPhases") {
        if !check_phase_allowed(allowed_phases, state)? {
            return Ok(false);
        }
    }
    
    // Check conditions
    if let Some(conditions) = action.get("conditions") {
        if !check_conditions_met(conditions, state, player_id)? {
            return Ok(false);
        }
    }
    
    Ok(true)
}

/// Process entities in a zone and add UI hints
fn process_zone_entities(
    zone_id: &str,
    zone: &Value,
    available_actions: &[Value],
    player_id: &str,
    entity_ui: &mut HashMap<String, Value>,
) -> Result<(), String> {
    // Handle grid zones
    if let Some(cells) = zone.get("cells").and_then(|c| c.as_array()) {
        for (row_idx, row) in cells.iter().enumerate() {
            if let Some(row_array) = row.as_array() {
                for (col_idx, entity) in row_array.iter().enumerate() {
                    if !entity.is_null() {
                        process_entity(
                            entity,
                            zone_id,
                            Some((row_idx, col_idx)),
                            available_actions,
                            player_id,
                            entity_ui,
                        )?;
                    }
                }
            }
        }
    }
    
    // Handle list zones
    if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
        for (idx, entity) in items.iter().enumerate() {
            process_entity(
                entity,
                zone_id,
                None,
                available_actions,
                player_id,
                entity_ui,
            )?;
        }
    }
    
    Ok(())
}

/// Process a single entity and determine its UI properties
fn process_entity(
    entity: &Value,
    zone_id: &str,
    position: Option<(usize, usize)>,
    available_actions: &[Value],
    player_id: &str,
    entity_ui: &mut HashMap<String, Value>,
) -> Result<(), String> {
    let entity_id = entity.as_str()
        .or_else(|| entity.get("id").and_then(|id| id.as_str()))
        .unwrap_or("unknown");
    
    let mut ui_props = json!({
        "zone": zone_id,
        "position": position,
        "interactions": []
    });
    
    // Check which actions can be performed on this entity
    for action in available_actions {
        if can_action_target_entity(action, entity, zone_id, position)? {
            let interaction = json!({
                "actionId": action["id"],
                "verb": action["verb"],
                "label": action.get("label").unwrap_or(&json!(action["id"])),
                "style": determine_action_style(action, entity),
            });
            
            ui_props["interactions"].as_array_mut()
                .unwrap()
                .push(interaction);
        }
    }
    
    // Add visual state based on entity properties
    add_visual_state(&mut ui_props, entity, player_id)?;
    
    entity_ui.insert(entity_id.to_string(), ui_props);
    
    Ok(())
}

/// Check if an action can target a specific entity
fn can_action_target_entity(
    action: &Value,
    entity: &Value,
    zone_id: &str,
    position: Option<(usize, usize)>,
) -> Result<bool, String> {
    // Check zone restrictions
    if let Some(allowed_zones) = action.get("allowedZones").and_then(|z| z.as_array()) {
        let zone_allowed = allowed_zones.iter()
            .any(|z| z.as_str() == Some(zone_id));
        if !zone_allowed {
            return Ok(false);
        }
    }
    
    // Check entity type restrictions
    if let Some(entity_filter) = action.get("entityFilter") {
        if !matches_entity_filter(entity, entity_filter)? {
            return Ok(false);
        }
    }
    
    Ok(true)
}

/// Determine visual style for an action
fn determine_action_style(action: &Value, entity: &Value) -> Value {
    let verb = action.get("verb").and_then(|v| v.as_str()).unwrap_or("");
    
    match verb {
        "moveEntity" => json!({
            "highlight": "movement",
            "cursor": "pointer"
        }),
        "selectEntity" => json!({
            "highlight": "selection",
            "cursor": "pointer"
        }),
        "placeEntity" => json!({
            "highlight": "placement",
            "cursor": "crosshair"
        }),
        _ => json!({
            "highlight": "default",
            "cursor": "pointer"
        })
    }
}

/// Add visual state properties to entity UI
fn add_visual_state(
    ui_props: &mut Value,
    entity: &Value,
    player_id: &str,
) -> Result<(), String> {
    // Check ownership
    if let Some(owner) = entity.get("owner").and_then(|o| o.as_str()) {
        ui_props["isOwned"] = json!(owner == player_id);
        ui_props["owner"] = json!(owner);
    }
    
    // Check selection state
    if let Some(selected) = entity.get("selected").and_then(|s| s.as_bool()) {
        ui_props["isSelected"] = json!(selected);
    }
    
    // Add entity-specific visual hints
    if let Some(entity_type) = entity.get("type").and_then(|t| t.as_str()) {
        match entity_type {
            "card" => {
                ui_props["displayStyle"] = json!("card");
                if let Some(face_up) = entity.get("faceUp").and_then(|f| f.as_bool()) {
                    ui_props["faceUp"] = json!(face_up);
                }
            }
            "piece" => {
                ui_props["displayStyle"] = json!("piece");
            }
            "token" => {
                ui_props["displayStyle"] = json!("token");
            }
            _ => {}
        }
    }
    
    Ok(())
}

// Helper functions
fn check_player_allowed(allowed: &Value, state: &Value, player_id: &str) -> Result<bool, String> {
    match allowed {
        Value::String(s) if s == "all" => Ok(true),
        Value::String(s) if s == "currentPlayer" => {
            let current = state.get("currentPlayer")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            Ok(current == player_id)
        }
        Value::String(s) if s == "none" => Ok(false),
        Value::Array(arr) => {
            Ok(arr.iter().any(|v| v.as_str() == Some(player_id)))
        }
        _ => Ok(true)
    }
}

fn check_phase_allowed(allowed: &Value, state: &Value) -> Result<bool, String> {
    if allowed.is_null() {
        return Ok(true);
    }
    
    let phases = state.get("phases").ok_or("No phases in state")?;
    
    // Check each allowed phase pattern
    if let Some(allowed_phases) = allowed.as_array() {
        for phase_pattern in allowed_phases {
            if let Some(pattern) = phase_pattern.as_str() {
                // Check if any current phase matches the pattern
                if let Some(phases_obj) = phases.as_object() {
                    for (phase_set, current_phase) in phases_obj {
                        if let Some(current) = current_phase.as_str() {
                            let full_phase = format!("{}.{}", phase_set, current);
                            if full_phase == pattern || current == pattern {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        Ok(false)
    } else if let Some(pattern) = allowed.as_str() {
        // Single phase pattern
        if let Some(phases_obj) = phases.as_object() {
            for (phase_set, current_phase) in phases_obj {
                if let Some(current) = current_phase.as_str() {
                    let full_phase = format!("{}.{}", phase_set, current);
                    if full_phase == pattern || current == pattern {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    } else {
        Ok(true)
    }
}

fn check_conditions_met(conditions: &Value, state: &Value, player_id: &str) -> Result<bool, String> {
    if conditions.is_null() {
        return Ok(true);
    }
    
    // For now, just check basic conditions
    // Full condition evaluation would use the conditions module
    if let Some(cond_obj) = conditions.as_object() {
        // Check game.notEnded condition
        if cond_obj.contains_key("game.notEnded") {
            let game_status = state.get("gameStatus")
                .and_then(|s| s.as_str())
                .unwrap_or("playing");
            if game_status != "playing" {
                return Ok(false);
            }
        }
        
        // Check isCurrentPlayer condition
        if cond_obj.contains_key("isCurrentPlayer") {
            let current = state.get("currentPlayer")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            if current != player_id {
                return Ok(false);
            }
        }
    }
    
    Ok(true)
}

fn matches_entity_filter(entity: &Value, filter: &Value) -> Result<bool, String> {
    if filter.is_null() {
        return Ok(true);
    }
    
    if let Some(filter_obj) = filter.as_object() {
        // Check owner filter
        if let Some(owner_filter) = filter_obj.get("owner") {
            let entity_owner = entity.get("owner").and_then(|o| o.as_str()).unwrap_or("");
            let filter_owner = owner_filter.as_str().unwrap_or("");
            if entity_owner != filter_owner {
                return Ok(false);
            }
        }
        
        // Check type filter
        if let Some(type_filter) = filter_obj.get("type") {
            let entity_type = entity.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match type_filter {
                Value::String(s) => {
                    if entity_type != s {
                        return Ok(false);
                    }
                }
                Value::Array(arr) => {
                    if !arr.iter().any(|t| t.as_str() == Some(entity_type)) {
                        return Ok(false);
                    }
                }
                _ => {}
            }
        }
        
        // Check property filters
        for (key, value) in filter_obj {
            if key != "owner" && key != "type" {
                if entity.get(key) != Some(value) {
                    return Ok(false);
                }
            }
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_ui_enhancement() {
        let state = json!({
            "zones": {
                "board": {
                    "cells": [
                        ["p1_piece", null, null],
                        [null, null, null],
                        [null, null, "p2_piece"]
                    ]
                },
                "hand": {
                    "items": ["card1", "card2"]
                }
            }
        });
        
        let bundle = Bundle {
            game_id: "test".to_string(),
            manifest: crate::bundle::Manifest {
                game_id: "test".to_string(),
                version: "1.0".to_string(),
                spec_version: "1.0".to_string(),
                metadata: crate::bundle::ManifestMetadata {
                    name: "Test".to_string(),
                    author: "Test".to_string(),
                    description: "Test".to_string(),
                    players: crate::bundle::PlayersRange { min: 2, max: 2 },
                },
                phases: None,
                setup: None,
                zone_groups: None,
            },
            entities: Value::Null,
            zones: Value::Null,
            actions: Value::Null,
            phases: Value::Null,
        };
        let result = enhance_entities_with_ui(&state, &bundle, "p1");
        
        assert!(result.is_ok());
        let entity_ui = result.unwrap();
        assert!(entity_ui.contains_key("p1_piece"));
        assert!(entity_ui.contains_key("card1"));
    }
}
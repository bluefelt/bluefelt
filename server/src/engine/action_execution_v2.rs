//! Simplified action execution for entity-centric UI
//!
//! This module handles executing actions based on entity interactions
//! rather than centralized action maps.

use serde_json::{json, Value};
use crate::bundle::Bundle;
use crate::engine::verbs::apply_verb;
use crate::engine::patches::process_phases;

/// Execute an entity interaction
pub fn execute_entity_interaction(
    state: &mut Value,
    bundle: &Bundle,
    player_id: &str,
    entity_id: &str,
    action_id: &str,
    additional_args: Option<Value>,
) -> Result<Vec<Value>, String> {
    println!("[execute_entity_interaction] player={}, entity={}, action={}", 
        player_id, entity_id, action_id);
    
    // Find the action definition
    let action = find_action(bundle, action_id)?;
    
    // Verify player can perform this action
    verify_player_allowed(&action, state, player_id)?;
    
    // Build action arguments
    let mut args = build_action_args(&action, entity_id, additional_args)?;
    
    // Execute the action verb
    let verb = action["verb"].as_str()
        .ok_or("Action missing verb")?;
    
    let mut patches = apply_verb(state, verb, &args, bundle)?;
    
    // Process any phase transitions
    let phase_patches = process_phases(bundle, state)?;
    patches.extend(phase_patches);
    
    Ok(patches)
}

/// Find action definition in bundle
fn find_action(bundle: &Bundle, action_id: &str) -> Result<Value, String> {
    if let Some(actions) = bundle.actions.as_array() {
        for action in actions {
            if action["id"].as_str() == Some(action_id) {
                return Ok(action.clone());
            }
        }
    }
    
    Err(format!("Action not found: {}", action_id))
}

/// Verify player is allowed to perform action
fn verify_player_allowed(
    action: &Value,
    state: &Value,
    player_id: &str,
) -> Result<(), String> {
    // Check if it's the player's turn
    if let Some(current_player) = state.get("currentPlayer").and_then(|p| p.as_str()) {
        if current_player != player_id {
            return Err("Not your turn".to_string());
        }
    }
    
    // Check phase restrictions
    if let Some(allowed_phases) = action.get("allowedPhases") {
        // TODO: Implement phase checking
    }
    
    Ok(())
}

/// Build action arguments from entity and additional data
fn build_action_args(
    action: &Value,
    entity_id: &str,
    additional_args: Option<Value>,
) -> Result<Value, String> {
    let mut args = if let Some(template) = action.get("args") {
        template.clone()
    } else {
        json!({})
    };
    
    // Add entity reference
    if let Some(args_obj) = args.as_object_mut() {
        args_obj.insert("entity".to_string(), json!(entity_id));
        
        // Merge additional arguments
        if let Some(additional) = additional_args {
            if let Some(add_obj) = additional.as_object() {
                for (key, value) in add_obj {
                    args_obj.insert(key.clone(), value.clone());
                }
            }
        }
    }
    
    Ok(args)
}

/// Execute a zone interaction (for empty cells/spaces)
pub fn execute_zone_interaction(
    state: &mut Value,
    bundle: &Bundle,
    player_id: &str,
    zone_id: &str,
    position: Option<(usize, usize)>,
    action_id: &str,
    additional_args: Option<Value>,
) -> Result<Vec<Value>, String> {
    println!("[execute_zone_interaction] player={}, zone={}, action={}", 
        player_id, zone_id, action_id);
    
    // Find the action definition
    let action = find_action(bundle, action_id)?;
    
    // Verify player can perform this action
    verify_player_allowed(&action, state, player_id)?;
    
    // Build action arguments
    let mut args = if let Some(template) = action.get("args") {
        template.clone()
    } else {
        json!({})
    };
    
    // Add zone and position info
    if let Some(args_obj) = args.as_object_mut() {
        args_obj.insert("zone".to_string(), json!(zone_id));
        if let Some((row, col)) = position {
            args_obj.insert("position".to_string(), json!([row, col]));
        }
        
        // Merge additional arguments
        if let Some(additional) = additional_args {
            if let Some(add_obj) = additional.as_object() {
                for (key, value) in add_obj {
                    args_obj.insert(key.clone(), value.clone());
                }
            }
        }
    }
    
    // Execute the action verb
    let verb = action["verb"].as_str()
        .ok_or("Action missing verb")?;
    
    let mut patches = apply_verb(state, verb, &args, bundle)?;
    
    // Process any phase transitions
    let phase_patches = process_phases(bundle, state)?;
    patches.extend(phase_patches);
    
    Ok(patches)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_execute_entity_interaction() {
        let mut state = json!({
            "currentPlayer": "p1",
            "zones": {
                "board": {
                    "cells": [["piece1", null], [null, null]]
                }
            },
            "phases": {
                "game": "play"
            }
        });
        
        let bundle = Bundle {
            actions: json!([{
                "id": "move",
                "verb": "moveEntity",
                "args": {
                    "from": "/zones/board/cells/0/0",
                    "to": "/zones/board/cells/0/1"
                }
            }]),
            phases: json!({
                "play": {}
            }),
            ..Default::default()
        };
        
        let result = execute_entity_interaction(
            &mut state,
            &bundle,
            "p1",
            "piece1",
            "move",
            Some(json!({"targetPosition": [0, 1]}))
        );
        
        // Should succeed but actual move logic would be in the verb
        assert!(result.is_ok());
    }
}
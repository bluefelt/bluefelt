use serde_json::{json, Value};
use crate::bundle::Bundle;
use crate::engine::path::set_value_at_path;
use crate::engine::verbs::apply_verb;

pub fn process_phases(bundle: &Bundle, state: &mut Value) -> Result<Vec<Value>, String> {
    let mut patches = Vec::new();
    
    // Get current phase states
    let phase_states = state["phases"].as_object()
        .ok_or("Missing phase states")?
        .clone();
    
    // Check each phase set for enterActions
    if let Some(phase_sets) = bundle.phases.as_array() {
        for (phase_set_id, current_phase_id) in phase_states.iter() {
            process_phase_set(
                phase_sets,
                phase_set_id,
                current_phase_id,
                state,
                &mut patches
            )?;
        }
    }
    
    Ok(patches)
}

fn process_phase_set(
    phase_sets: &[Value],
    phase_set_id: &str,
    current_phase_id: &Value,
    state: &mut Value,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    // Find the phase set definition
    if let Some(phase_set) = phase_sets.iter()
        .find(|ps| ps["id"].as_str() == Some(phase_set_id)) {
        
        // Find the current phase within the set
        if let Some(phases) = phase_set["phases"].as_array() {
            if let Some(current_phase) = phases.iter()
                .find(|p| p["id"].as_str() == Some(current_phase_id.as_str().unwrap_or(""))) {
                
                process_enter_actions(
                    current_phase,
                    phase_set_id,
                    current_phase_id,
                    state,
                    patches
                )?;
            }
        }
    }
    
    Ok(())
}

fn process_enter_actions(
    current_phase: &Value,
    phase_set_id: &str,
    current_phase_id: &Value,
    state: &mut Value,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    if let Some(enter_actions) = current_phase["enterActions"].as_array() {
        println!("[DEBUG process_phases] Found enterActions for phase {}.{}", 
            phase_set_id, current_phase_id.as_str().unwrap_or(""));
        
        // Process each enter action
        for action in enter_actions {
            if let Some(transition_to) = action["transitionToPhase"].as_str() {
                process_phase_transition(
                    phase_set_id,
                    transition_to,
                    state,
                    patches
                )?;
            }
            // Handle other types of enter actions here if needed
        }
    }
    
    Ok(())
}

fn process_phase_transition(
    phase_set_id: &str,
    transition_to: &str,
    state: &mut Value,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    println!("[DEBUG process_phases] Transitioning to phase: {}", transition_to);
    
    // Update the phase state
    let phases = state["phases"].as_object_mut()
        .ok_or("Missing phases object")?;
    phases.insert(phase_set_id.to_string(), json!(transition_to));
    
    // Create patch for the transition
    patches.push(json!({
        "op": "replace",
        "path": format!("/phases/{}", phase_set_id),
        "value": transition_to
    }));
    
    Ok(())
}

pub fn apply_patch_to_state(state: &mut Value, patch: &Value) {
    if let Some(op) = patch.get("op").and_then(|o| o.as_str()) {
        match op {
            "replace" => apply_replace_patch(state, patch),
            "add" => apply_add_patch(state, patch),
            "remove" => apply_remove_patch(state, patch),
            _ => {
                // TODO: Implement other patch operations if needed
                println!("[WARN] Unhandled patch operation: {}", op);
            }
        }
    }
}

fn apply_replace_patch(state: &mut Value, patch: &Value) {
    if let (Some(path), Some(value)) = (patch.get("path"), patch.get("value")) {
        if let Some(path_str) = path.as_str() {
            let _ = set_value_at_path(state, path_str, value.clone());
        }
    }
}

fn apply_add_patch(_state: &mut Value, patch: &Value) {
    // TODO: Implement add operation
    if let Some(path) = patch.get("path").and_then(|p| p.as_str()) {
        println!("[WARN] Add patch operation not yet implemented for path: {}", path);
    }
}

fn apply_remove_patch(_state: &mut Value, patch: &Value) {
    // TODO: Implement remove operation
    if let Some(path) = patch.get("path").and_then(|p| p.as_str()) {
        println!("[WARN] Remove patch operation not yet implemented for path: {}", path);
    }
}

pub fn apply_action(
    bundle: &Bundle,
    state: &mut Value,
    _player_id: &str,
    action: &Value,
) -> Result<Vec<Value>, String> {
    // Extract verb and args from action
    if let (Some(verb), Some(args)) = (
        action.get("verb").and_then(|v| v.as_str()),
        action.get("args")
    ) {
        apply_verb(state, verb, args, bundle)
    } else {
        Err("Invalid action format".to_string())
    }
}
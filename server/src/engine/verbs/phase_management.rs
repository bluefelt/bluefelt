use serde_json::{json, Value};
use crate::bundle::Bundle;
use crate::engine::enhanced_phases::{execute_phase_transition, check_phase_condition, get_current_phase};

/// Transition to a specific phase within a phase set
/// 
/// Example: phase.game.transitionTo with args { phase: "movement" }
pub fn transition_phase(state: &mut Value, args: &Value, bundle: &Bundle) -> Result<Vec<Value>, String> {
    let mut patches = Vec::new();
    
    // Extract phase set from the verb path (this would be passed in context)
    // For now, we'll require explicit phase set in args
    let phase_set = args["phaseSet"].as_str()
        .ok_or("Missing required parameter 'phaseSet'")?;
    
    let target_phase = args["phase"].as_str()
        .ok_or("Missing required parameter 'phase'")?;
    
    // Get current phase for exit actions
    let current_phase = get_current_phase(state, phase_set);
    
    // Execute exit actions for current phase
    if let Some(current) = current_phase {
        execute_phase_exit_actions(state, bundle, phase_set, &current, &mut patches)?;
    }
    
    // Perform the transition
    let transition_target = format!("{}.{}", phase_set, target_phase);
    execute_phase_transition(state, &transition_target, &mut patches)?;
    
    Ok(patches)
}

/// Execute exit actions for a phase
fn execute_phase_exit_actions(
    state: &mut Value,
    bundle: &Bundle,
    phase_set: &str,
    phase_id: &str,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    // Find phase definition
    if let Some(phase_sets) = bundle.phases.as_array() {
        if let Some(set_def) = phase_sets.iter()
            .find(|ps| ps["id"].as_str() == Some(phase_set)) {
            
            if let Some(phases) = set_def["phases"].as_array() {
                if let Some(phase_def) = phases.iter()
                    .find(|p| p["id"].as_str() == Some(phase_id)) {
                    
                    // Execute exit actions
                    if let Some(exit_actions) = phase_def["exitActions"].as_array() {
                        for action in exit_actions {
                            execute_phase_action(state, bundle, action, patches)?;
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Execute a phase action (similar to enhanced_phases but accessible from verbs)
fn execute_phase_action(
    state: &mut Value,
    bundle: &Bundle,
    action: &Value,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    use crate::engine::verbs::apply_verb;
    
    if let Some(action_str) = action.as_str() {
        // Simple action string - execute as verb
        let action_patches = apply_verb(state, action_str, &json!({}), bundle)?;
        patches.extend(action_patches);
    } else if let Some(action_obj) = action.as_object() {
        // Action object with verb and parameters
        if let Some(verb) = action_obj.get("action").and_then(|v| v.as_str()) {
            let default_args = json!({});
            let args = action_obj.get("with").unwrap_or(&default_args);
            let action_patches = apply_verb(state, verb, args, bundle)?;
            patches.extend(action_patches);
        }
    }
    
    Ok(())
}

/// Check if a specific phase is active
/// 
/// Used in conditions like: phase.game.playing.isActive
pub fn is_phase_active(state: &Value, phase_set: &str, phase_id: &str) -> bool {
    state["phases"]["current"][phase_set]
        .as_str()
        .map(|current| current == phase_id)
        .unwrap_or(false)
}

/// Get phase information for UI display
pub fn get_phase_info(state: &Value, bundle: &Bundle) -> Value {
    let mut phase_info = json!({});
    
    if let Some(current_phases) = state["phases"]["current"].as_object() {
        for (set_id, current_phase) in current_phases {
            if let Some(phase_def) = find_phase_definition(bundle, set_id, current_phase.as_str().unwrap_or("")) {
                phase_info[set_id] = json!({
                    "current": current_phase,
                    "name": phase_def["name"],
                    "description": phase_def["description"],
                    "ui": phase_def["ui"]
                });
            }
        }
    }
    
    phase_info
}

/// Find a phase definition in the bundle
fn find_phase_definition<'a>(bundle: &'a Bundle, phase_set: &str, phase_id: &str) -> Option<&'a Value> {
    bundle.phases.as_array()?
        .iter()
        .find(|ps| ps["id"].as_str() == Some(phase_set))?
        ["phases"].as_array()?
        .iter()
        .find(|p| p["id"].as_str() == Some(phase_id))
}
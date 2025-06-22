use serde::{Deserialize, Serialize};
use serde_json::{json, Value, Map};
use std::collections::{HashMap, HashSet};
use crate::bundle::Bundle;
use crate::engine::verbs::apply_verb;

/// Enhanced phase state that supports multiple active phase sets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseState {
    /// Current phase for each phase set (e.g., {"game": "playing", "turn": "draw"})
    pub current_phases: HashMap<String, String>,
    /// Transition history for debugging
    pub transition_history: Vec<PhaseTransition>,
    /// Phases that have executed enter actions this iteration
    pub entered_phases: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub phase_set: String,
    pub from_phase: String,
    pub to_phase: String,
    pub timestamp: u64,
}

/// Enhanced phase definition with enter/exit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPhase {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub initial: bool,
    pub enter_actions: Vec<Value>,
    pub exit_actions: Vec<Value>,
    pub possible_actions: Vec<String>,
    pub ui: Option<PhaseUI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseUI {
    pub display: Option<String>,
    pub prompt: Option<String>,
}

/// Initialize phase states for enhanced system
pub fn initialize_enhanced_phases(phases_def: &Value) -> Value {
    let mut phase_states = HashMap::new();
    
    // Handle both old and new phase formats
    match phases_def {
        // New format: Array of phase sets
        Value::Array(phase_sets) => {
            for phase_set in phase_sets {
                if let Some(set_id) = phase_set["id"].as_str() {
                    let initial_phase = find_initial_phase_in_set(&phase_set);
                    phase_states.insert(set_id.to_string(), initial_phase.to_string());
                }
            }
        }
        // Old format: Object with phase set keys
        Value::Object(phase_sets_map) => {
            for (set_id, phases) in phase_sets_map {
                if let Value::Array(phases_arr) = phases {
                    let initial_phase = find_initial_phase(phases_arr);
                    phase_states.insert(set_id.clone(), initial_phase.to_string());
                }
            }
        }
        _ => {}
    }
    
    // Return enhanced phase state structure
    json!({
        "current": phase_states,
        "history": [],
        "entered": []
    })
}

/// Find the initial phase in a phase set
fn find_initial_phase_in_set(phase_set: &Value) -> &str {
    if let Some(phases) = phase_set["phases"].as_array() {
        find_initial_phase(phases)
    } else {
        "unknown"
    }
}

/// Find the initial phase in an array of phases
fn find_initial_phase(phases: &[Value]) -> &str {
    phases.iter()
        .find(|p| p["initial"].as_bool().unwrap_or(false))
        .or_else(|| phases.first())
        .and_then(|p| p["id"].as_str())
        .unwrap_or("unknown")
}

/// Process enter and exit actions for phase transitions
pub fn process_phase_transitions(
    bundle: &Bundle,
    state: &mut Value,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    let phase_state = state["phases"].clone();
    
    // Get current phases
    let current_phases = match phase_state["current"].as_object() {
        Some(phases) => phases,
        None => return Ok(()), // No enhanced phase state
    };
    
    // Get entered phases to avoid re-processing
    let mut entered_phases: HashSet<String> = phase_state["entered"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();
    
    // Process each phase set
    if let Some(phase_sets) = bundle.phases.as_array() {
        for phase_set in phase_sets {
            let set_id = match phase_set["id"].as_str() {
                Some(id) => id,
                None => continue,
            };
            
            let current_phase_id = match current_phases.get(set_id) {
                Some(Value::String(id)) => id.as_str(),
                _ => continue,
            };
            
            let phase_key = format!("{}.{}", set_id, current_phase_id);
            
            // Skip if already processed
            if entered_phases.contains(&phase_key) {
                continue;
            }
            
            // Find and execute enter actions
            if let Some(phases) = phase_set["phases"].as_array() {
                if let Some(current_phase) = phases.iter()
                    .find(|p| p["id"].as_str() == Some(current_phase_id)) {
                    
                    // Execute enter actions
                    if let Some(enter_actions) = current_phase["enterActions"].as_array() {
                        println!("[DEBUG process_phase_transitions] Found {} enterActions for phase {}.{}", 
                            enter_actions.len(), set_id, current_phase_id);
                        
                        // Track initial patch count
                        let initial_patch_count = patches.len();
                        
                        for action in enter_actions {
                            println!("[DEBUG process_phase_transitions] Executing enterAction: {:?}", action);
                            execute_phase_action(bundle, state, action, patches)?;
                        }
                        
                        // Apply any patches generated by the actions to the state
                        // This is crucial for phase transitions to take effect
                        if patches.len() > initial_patch_count {
                            println!("[DEBUG process_phase_transitions] Applying {} patches to state", 
                                patches.len() - initial_patch_count);
                            for i in initial_patch_count..patches.len() {
                                crate::engine::patches::apply_patch_to_state(state, &patches[i]);
                            }
                        }
                        
                        // Mark as entered
                        entered_phases.insert(phase_key);
                        
                        // Update entered phases in state
                        let entered_array: Vec<Value> = entered_phases.iter()
                            .map(|s| json!(s))
                            .collect();
                        let entered_patch = json!({
                            "op": "replace",
                            "path": "/phases/entered",
                            "value": entered_array
                        });
                        patches.push(entered_patch.clone());
                        
                        // Apply the entered patch to state as well
                        crate::engine::patches::apply_patch_to_state(state, &entered_patch);
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Execute a phase action (enter or exit)
fn execute_phase_action(
    bundle: &Bundle,
    state: &mut Value,
    action: &Value,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    // Handle different action formats
    if let Some(action_str) = action.as_str() {
        // Simple action string
        if action_str == "transitionToPhase" {
            // Legacy format, skip
            return Ok(());
        }
        
        // First, try to find it as an action in the bundle
        if let Some(actions) = bundle.actions.as_array() {
            if let Some(action_def) = actions.iter()
                .find(|a| a["id"].as_str() == Some(action_str)) {
                
                // Found the action definition, execute it
                if let (Some(verb), Some(args)) = (
                    action_def["uses"].as_str(),
                    action_def.get("with")
                ) {
                    println!("[DEBUG execute_phase_action] Executing action {} with verb {}", action_str, verb);
                    let action_patches = apply_verb(state, verb, args, bundle)?;
                    patches.extend(action_patches);
                    
                    // Process "then" actions if present
                    if let Some(then_actions) = action_def.get("then").and_then(|t| t.as_array()) {
                        for then_action in then_actions {
                            execute_phase_action(bundle, state, then_action, patches)?;
                        }
                    }
                    
                    return Ok(());
                }
            }
        }
        
        // If not found as action, try as verb (for backward compatibility)
        let action_patches = apply_verb(state, action_str, &json!({}), bundle)?;
        patches.extend(action_patches);
    } else if let Some(action_obj) = action.as_object() {
        // Action object with verb and parameters
        if let Some(verb) = action_obj.get("action").and_then(|v| v.as_str()) {
            // Check if this is an action ID reference
            if let Some(actions) = bundle.actions.as_array() {
                if let Some(action_def) = actions.iter()
                    .find(|a| a["id"].as_str() == Some(verb)) {
                    
                    // Found the action definition, execute it
                    if let (Some(actual_verb), Some(args)) = (
                        action_def["uses"].as_str(),
                        action_def.get("with")
                    ) {
                        println!("[DEBUG execute_phase_action] Executing action {} with verb {}", verb, actual_verb);
                        let action_patches = apply_verb(state, actual_verb, args, bundle)?;
                        patches.extend(action_patches);
                        
                        // Process "then" actions if present
                        if let Some(then_actions) = action_def.get("then").and_then(|t| t.as_array()) {
                            for then_action in then_actions {
                                execute_phase_action(bundle, state, then_action, patches)?;
                            }
                        }
                        
                        return Ok(());
                    }
                }
            }
            
            // Not an action ID, execute as verb
            let default_args = json!({});
            let args = action_obj.get("with").unwrap_or(&default_args);
            let action_patches = apply_verb(state, verb, args, bundle)?;
            patches.extend(action_patches);
        } else if let Some(transition_target) = action_obj.get("transitionToPhase").and_then(|v| v.as_str()) {
            // Phase transition
            execute_phase_transition(state, transition_target, patches)?;
        }
    }
    
    Ok(())
}

/// Execute a phase transition
pub fn execute_phase_transition(
    state: &mut Value,
    target: &str,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    // Parse target format: "phaseSet.phaseId"
    let parts: Vec<&str> = target.split('.').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid phase transition target: {}", target));
    }
    
    let phase_set = parts[0];
    let phase_id = parts[1];
    
    // Update phase state
    let phase_patch = json!({
        "op": "replace",
        "path": format!("/phases/current/{}", phase_set),
        "value": phase_id
    });
    patches.push(phase_patch.clone());
    
    // Apply immediately to state to ensure subsequent processing sees the change
    crate::engine::patches::apply_patch_to_state(state, &phase_patch);
    
    // Clear entered phases to allow new phase's enter actions
    let entered_patch = json!({
        "op": "replace",
        "path": "/phases/entered",
        "value": []
    });
    patches.push(entered_patch.clone());
    
    // Apply immediately to state
    crate::engine::patches::apply_patch_to_state(state, &entered_patch);
    
    // Add to transition history
    let timestamp = chrono::Utc::now().timestamp_millis() as u64;
    patches.push(json!({
        "op": "add",
        "path": "/phases/history/-",
        "value": {
            "phaseSet": phase_set,
            "toPhase": phase_id,
            "timestamp": timestamp
        }
    }));
    
    Ok(())
}

/// Built-in phase condition checker
pub fn check_phase_condition(state: &Value, condition: &str) -> bool {
    // Parse condition format: "phase.{set}.{phase}.isActive"
    if let Some(captures) = regex::Regex::new(r"phase\.(\w+)\.(\w+)\.isActive")
        .unwrap()
        .captures(condition) {
        
        let phase_set = &captures[1];
        let phase_id = &captures[2];
        
        // Check current phase
        if let Some(current_phases) = state["phases"]["current"].as_object() {
            if let Some(current) = current_phases.get(phase_set).and_then(|v| v.as_str()) {
                return current == phase_id;
            }
        }
    }
    
    false
}

/// Get current phase for a phase set
pub fn get_current_phase(state: &Value, phase_set: &str) -> Option<String> {
    state["phases"]["current"][phase_set]
        .as_str()
        .map(|s| s.to_string())
}

/// Get all active phases
pub fn get_active_phases(state: &Value) -> HashMap<String, String> {
    state["phases"]["current"]
        .as_object()
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| {
                    v.as_str().map(|s| (k.clone(), s.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}
use serde_json::{json, Value};
use crate::bundle::Bundle;
use crate::engine::path::set_value_at_path;
use crate::engine::verbs::apply_verb;
use crate::engine::phase_validation::{validate_phase_state};
use crate::engine::enhanced_phases::process_phase_transitions;
use regex;
use chrono;
use std::collections::HashSet;

const MAX_PHASE_ITERATIONS: usize = 50;

// Animation delay configuration
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub enabled: bool,
    pub delay_ms: u64,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        AnimationConfig {
            enabled: false,
            delay_ms: 100,
        }
    }
}

// Result of a discrete state update
#[derive(Debug, Clone)]
pub struct DiscreteUpdate {
    pub patches: Vec<Value>,
    pub delay_ms: u64,
    pub description: Option<String>,
}

pub fn process_phases(bundle: &Bundle, state: &mut Value) -> Result<Vec<Value>, String> {
    process_phases_with_animation(bundle, state, &AnimationConfig::default())
}

pub fn process_phases_with_animation(
    bundle: &Bundle, 
    state: &mut Value,
    animation_config: &AnimationConfig
) -> Result<Vec<Value>, String> {
    let mut patches = Vec::new();
    let mut iteration_count = 0;
    let mut processed_phases = HashSet::new();
    
    // Validate phase state before processing
    validate_phase_state(&state["phases"], &bundle.phases)
        .map_err(|e| format!("Phase state validation failed: {}", e))?;
    
    // Check if using enhanced phase system
    let use_enhanced = state["phases"].get("current").is_some();
    
    if use_enhanced {
        // Use enhanced phase processing with enter/exit actions
        loop {
            iteration_count += 1;
            
            // Safety check for excessive iterations
            if iteration_count > MAX_PHASE_ITERATIONS {
                return Err(format!(
                    "Phase processing exceeded maximum iterations ({}). Possible infinite loop in phase transitions.",
                    MAX_PHASE_ITERATIONS
                ));
            }
            
            let initial_patch_count = patches.len();
            
            // Process phase transitions with enter/exit actions
            println!("[DEBUG process_phases] Before process_phase_transitions, patches: {}", patches.len());
            process_phase_transitions(bundle, state, &mut patches)?;
            println!("[DEBUG process_phases] After process_phase_transitions, patches: {}", patches.len());
            
            // If no new patches were generated, we're done
            if patches.len() == initial_patch_count {
                println!("[DEBUG process_phases] No new patches, breaking loop");
                break;
            }
        }
    } else {
        // Use legacy phase processing
        loop {
            iteration_count += 1;
            
            // Safety check for excessive iterations
            if iteration_count > MAX_PHASE_ITERATIONS {
                return Err(format!(
                    "Phase processing exceeded maximum iterations ({}). Possible infinite loop in phase transitions.",
                    MAX_PHASE_ITERATIONS
                ));
            }
            
            let initial_patch_count = patches.len();
            
            // Get current phase states
            let phase_states = state["phases"].as_object()
                .ok_or("Missing phase states")?
                .clone();
            
            // Check each phase set for enterActions
            if let Some(phase_sets) = bundle.phases.as_array() {
                for (phase_set_id, current_phase_id) in phase_states.iter() {
                    let phase_key = format!("{}.{}", 
                        phase_set_id, 
                        current_phase_id.as_str().unwrap_or("")
                    );
                    
                    // Skip if we've already processed this exact phase combination
                    if processed_phases.contains(&phase_key) {
                        continue;
                    }
                    
                    let initial_patches_for_phase = patches.len();
                    
                    process_phase_set(
                        bundle,
                        phase_sets,
                        phase_set_id,
                        current_phase_id,
                        state,
                        &mut patches,
                        animation_config
                    )?;
                    
                    // Only mark as processed if it produced patches (state changes)
                    if patches.len() > initial_patches_for_phase {
                        processed_phases.insert(phase_key);
                    }
                }
            }
            
            // If no new patches were generated, we're done
            if patches.len() == initial_patch_count {
                break;
            }
            
            // Clear processed phases for next iteration to detect state changes
            processed_phases.clear();
        }
    }
    
    Ok(patches)
}

// Process verb and return discrete updates for animation
pub fn apply_verb_with_animation(
    state: &mut Value,
    verb: &str,
    args: &Value,
    bundle: &Bundle,
    animation_config: &AnimationConfig
) -> Result<Vec<DiscreteUpdate>, String> {
    // For now, just wrap regular patches in DiscreteUpdate
    // In the future, certain verbs can return multiple discrete updates
    let patches = apply_verb(state, verb, args, bundle)?;
    
    if !animation_config.enabled {
        // No animation, return all patches at once
        return Ok(vec![DiscreteUpdate {
            patches,
            delay_ms: 0,
            description: None,
        }]);
    }
    
    // Check if this verb supports discrete updates
    match verb {
        "moveEntity" => {
            // For card movement, we might want to animate each card separately
            if let Some(from_path) = args["from"].as_str() {
                if from_path.contains("/cards/") || from_path.contains("/hand/") {
                    return Ok(patches.into_iter().map(|patch| {
                        DiscreteUpdate {
                            patches: vec![patch],
                            delay_ms: animation_config.delay_ms,
                            description: Some("Moving card".to_string()),
                        }
                    }).collect());
                }
            }
        }
        "shuffle" => {
            // For shuffling, we might animate the shuffle process
            return Ok(vec![DiscreteUpdate {
                patches,
                delay_ms: animation_config.delay_ms * 3, // Longer delay for shuffle
                description: Some("Shuffling cards".to_string()),
            }]);
        }
        "formPairs" => {
            // For pair formation, animate each pair separately
            if patches.len() > 1 {
                let mut updates = Vec::new();
                for (i, patch) in patches.into_iter().enumerate() {
                    updates.push(DiscreteUpdate {
                        patches: vec![patch],
                        delay_ms: animation_config.delay_ms,
                        description: Some(format!("Forming pair {}", i + 1)),
                    });
                }
                return Ok(updates);
            }
        }
        _ => {}
    }
    
    // Default: return all patches at once
    Ok(vec![DiscreteUpdate {
        patches,
        delay_ms: 0,
        description: None,
    }])
}

fn process_phase_set(
    bundle: &Bundle,
    phase_sets: &[Value],
    phase_set_id: &str,
    current_phase_id: &Value,
    state: &mut Value,
    patches: &mut Vec<Value>,
    animation_config: &AnimationConfig,
) -> Result<(), String> {
    // Find the phase set definition
    if let Some(phase_set) = phase_sets.iter()
        .find(|ps| ps["id"].as_str() == Some(phase_set_id)) {
        
        // Find the current phase within the set
        if let Some(phases) = phase_set["phases"].as_array() {
            println!("[DEBUG process_phase_set] Looking for phase {} in {} phases", 
                current_phase_id.as_str().unwrap_or(""), phases.len());
            
            if let Some(current_phase) = phases.iter()
                .find(|p| p["id"].as_str() == Some(current_phase_id.as_str().unwrap_or(""))) {
                
                println!("[DEBUG process_phase_set] Found phase, calling process_enter_actions");
                process_enter_actions(
                    bundle,
                    current_phase,
                    phase_set_id,
                    current_phase_id,
                    state,
                    patches,
                    animation_config
                )?;
            } else {
                println!("[DEBUG process_phase_set] Phase {} not found in phase set {}", 
                    current_phase_id.as_str().unwrap_or(""), phase_set_id);
            }
        }
    }
    
    Ok(())
}

fn process_enter_actions(
    bundle: &Bundle,
    current_phase: &Value,
    phase_set_id: &str,
    current_phase_id: &Value,
    state: &mut Value,
    patches: &mut Vec<Value>,
    animation_config: &AnimationConfig,
) -> Result<(), String> {
    println!("[DEBUG process_enter_actions] Checking phase {}.{} for enterActions", 
        phase_set_id, current_phase_id.as_str().unwrap_or(""));
    
    if let Some(enter_actions) = current_phase["enterActions"].as_array() {
        println!("[DEBUG process_enter_actions] Found {} enterActions for phase {}.{}", 
            enter_actions.len(), phase_set_id, current_phase_id.as_str().unwrap_or(""));
        
        // Process each enter action
        for (idx, action) in enter_actions.iter().enumerate() {
            println!("[DEBUG process_enter_actions] Processing enterAction[{}]: {:?}", idx, action);
            if let Some(transition_to) = action["transitionToPhase"].as_str() {
                process_phase_transition(
                    phase_set_id,
                    transition_to,
                    state,
                    patches
                )?;
            } else if let Some(action_id) = action.as_str() {
                // Handle string action references (like "dealCards")
                println!("[DEBUG process_phases] Processing enterAction: {}", action_id);
                
                // Find the action in the bundle
                if let Some(actions) = bundle.actions.as_array() {
                    if let Some(action_def) = actions.iter()
                        .find(|a| a["id"].as_str() == Some(action_id)) {
                        
                        process_action_with_then(
                            bundle,
                            state,
                            action_def,
                            action_id,
                            patches,
                            animation_config
                        )?;
                    } else {
                        println!("[ERROR process_phases] Action {} not found in bundle", action_id);
                        return Err(format!("Action {} not found", action_id));
                    }
                } else {
                    return Err("Bundle missing actions array".to_string());
                }
            } else if let Some(action_id) = action["action"].as_str() {
                // Handle object format with action field (like {"action": "dealCards"})
                println!("[DEBUG process_phases] Processing enterAction object: {}", action_id);
                
                // Find the action in the bundle
                if let Some(actions) = bundle.actions.as_array() {
                    if let Some(action_def) = actions.iter()
                        .find(|a| a["id"].as_str() == Some(action_id)) {
                        
                        process_action_with_then(
                            bundle,
                            state,
                            action_def,
                            action_id,
                            patches,
                            animation_config
                        )?;
                    } else {
                        println!("[ERROR process_phases] Action {} not found in bundle", action_id);
                        return Err(format!("Action {} not found", action_id));
                    }
                } else {
                    return Err("Bundle missing actions array".to_string());
                }
            }
            // Handle other types of enter actions here if needed
        }
    }
    
    Ok(())
}

pub fn replace_template_vars(value: &Value, state: &Value) -> Value {
    match value {
        Value::String(s) => {
            let mut result = s.clone();
            
            // Replace {player} with currentPlayer
            if result.contains("{player}") {
                if let Some(current_player) = state["currentPlayer"].as_str() {
                    result = result.replace("{player}", current_player);
                }
            }
            
            // Replace {currentPlayer} with currentPlayer
            if result.contains("{currentPlayer}") {
                if let Some(current_player) = state["currentPlayer"].as_str() {
                    result = result.replace("{currentPlayer}", current_player);
                }
            }
            
            // Replace {selection.X} with values from selection
            if result.contains("{selection.") {
                if let Some(selection) = state["selection"].as_object() {
                    for (key, val) in selection {
                        let template = format!("{{selection.{}}}", key);
                        if result.contains(&template) {
                            if let Some(str_val) = val.as_str() {
                                result = result.replace(&template, str_val);
                            }
                        }
                    }
                }
            }
            
            Value::String(result)
        }
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), replace_template_vars(v, state));
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| replace_template_vars(v, state)).collect())
        }
        _ => value.clone()
    }
}

fn process_action_with_then(
    bundle: &Bundle,
    state: &mut Value,
    action_def: &Value,
    action_id: &str,
    patches: &mut Vec<Value>,
    animation_config: &AnimationConfig,
) -> Result<(), String> {
    // Get verb and args from action definition
    if let (Some(verb), Some(args)) = (
        action_def["uses"].as_str(),
        action_def.get("with")
    ) {
        println!("[DEBUG process_phases] Executing action {} with verb {} and args {:?}", 
            action_id, verb, args);
        
        // Map preset verbs to actual verbs (same as in lobby.rs)
        let actual_verb = match verb {
            "presets.entity.move" => "moveEntity",
            "cards.deal" => {
                println!("[ERROR process_phases] cards.deal should have been expanded during build");
                return Err(format!("Action {} uses cards.deal which should have been expanded", action_id));
            }
            _ => verb
        };
        
        // Replace template variables in args
        let processed_args = replace_template_vars(args, state);
        println!("[DEBUG process_phases] Processed args: {:?}", processed_args);
        
        // Apply the action to the state
        // Note: For auto actions in enterActions, we don't need an actor
        // Apply verb with animation support
        let discrete_updates = apply_verb_with_animation(state, actual_verb, &processed_args, bundle, animation_config)?;
        
        // Convert discrete updates to patches
        let mut verb_patches = Vec::new();
        for update in discrete_updates {
            verb_patches.extend(update.patches);
            // TODO: In future, we could yield control here for animation delays
        }
        
        let num_patches = verb_patches.len();
        let has_patches = !verb_patches.is_empty();
        patches.extend(verb_patches);
        println!("[DEBUG process_phases] Action {} generated {} patches", 
            action_id, num_patches);
        
        // Generate log entry for automatic actions if they have a logTemplate
        if has_patches {
            if let Some(log_template) = action_def["ui"]["logTemplate"].as_str() {
                println!("[DEBUG process_phases] Generating log for automatic action {}", action_id);
                let log_patch = generate_log_patch(state, action_def, &processed_args, log_template);
                println!("[DEBUG process_phases] Generated log patch: {:?}", log_patch);
                patches.push(log_patch);
            } else {
                println!("[DEBUG process_phases] No logTemplate found for action {}", action_id);
            }
        }
        
        // Process "then" actions if the initial action succeeded
        if has_patches {
            if let Some(then_actions) = action_def.get("then").and_then(|t| t.as_array()) {
                println!("[DEBUG process_phases] Found {} 'then' actions for {}", then_actions.len(), action_id);
                
                for then_action in then_actions {
                    if let Some(then_action_id) = then_action["action"].as_str() {
                        println!("[DEBUG process_phases] Processing 'then' action: {}", then_action_id);
                        
                        // Find the then action definition
                        if let Some(actions) = bundle.actions.as_array() {
                            if let Some(then_action_def) = actions.iter().find(|a| a["id"].as_str() == Some(then_action_id)) {
                                // Recursively process this action with its own "then" actions
                                process_action_with_then(
                                    bundle,
                                    state,
                                    then_action_def,
                                    then_action_id,
                                    patches,
                                    animation_config
                                )?;
                            } else {
                                println!("[ERROR process_phases] 'then' action {} not found in bundle", then_action_id);
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("[ERROR process_phases] Action {} missing 'uses' or 'with' fields", action_id);
        return Err(format!("Action {} missing required fields", action_id));
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
    
    // Parse the transition target: "game.placement" -> phaseSet="game", phase="placement"
    let (target_phase_set, target_phase) = if transition_to.contains('.') {
        let parts: Vec<&str> = transition_to.split('.').collect();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            return Err(format!("Invalid phase transition format: {}", transition_to));
        }
    } else {
        // If no dot, assume it's within the current phase set
        (phase_set_id, transition_to)
    };
    
    // Update the phase state
    let phases = state["phases"].as_object_mut()
        .ok_or("Missing phases object")?;
    phases.insert(target_phase_set.to_string(), json!(target_phase));
    
    // Create patch for the transition
    patches.push(json!({
        "op": "replace",
        "path": format!("/phases/{}", target_phase_set),
        "value": target_phase
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

fn apply_add_patch(state: &mut Value, patch: &Value) {
    if let (Some(path), Some(value)) = (
        patch.get("path").and_then(|p| p.as_str()),
        patch.get("value")
    ) {
        // Split the path to get parent and target
        let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        if path_parts.is_empty() {
            println!("[ERROR] Invalid empty path for add operation");
            return;
        }
        
        // Navigate to parent container (array or object)
        let mut current = state;
        for i in 0..path_parts.len() - 1 {
            let part = path_parts[i];
            current = if let Ok(index) = part.parse::<usize>() {
                // Navigate through array
                if let Some(array) = current.as_array_mut() {
                    if index < array.len() {
                        &mut array[index]
                    } else {
                        println!("[ERROR] Array index {} out of bounds in path {}", index, path);
                        return;
                    }
                } else {
                    println!("[ERROR] Expected array at {} in path {}", part, path);
                    return;
                }
            } else {
                // Navigate through object
                if let Some(obj) = current.as_object_mut() {
                    if let Some(next) = obj.get_mut(part) {
                        next
                    } else {
                        println!("[ERROR] Key {} not found in path {}", part, path);
                        return;
                    }
                } else {
                    println!("[ERROR] Expected object at {} in path {}", part, path);
                    return;
                }
            };
        }
        
        // Add value to the parent container
        let target = path_parts[path_parts.len() - 1];
        if let Ok(index) = target.parse::<usize>() {
            // Adding to array at specific index
            if let Some(array) = current.as_array_mut() {
                if index <= array.len() {
                    array.insert(index, value.clone());
                } else {
                    // If index is beyond array length, append
                    array.push(value.clone());
                }
            } else {
                println!("[ERROR] Expected array for numeric index {} in add operation", target);
            }
        } else {
            // Adding to object
            if let Some(obj) = current.as_object_mut() {
                obj.insert(target.to_string(), value.clone());
            } else {
                println!("[ERROR] Expected object for key {} in add operation", target);
            }
        }
    }
}

fn apply_remove_patch(state: &mut Value, patch: &Value) {
    if let Some(path) = patch.get("path").and_then(|p| p.as_str()) {
        println!("[DEBUG] Applying remove patch for path: {}", path);
        // Split the path to get parent and target
        let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        if path_parts.is_empty() {
            println!("[ERROR] Invalid empty path for remove operation");
            return;
        }
        
        // Navigate to parent container (array or object)
        let mut current = state;
        for i in 0..path_parts.len() - 1 {
            let part = path_parts[i];
            current = if let Ok(index) = part.parse::<usize>() {
                // Navigate through array
                if let Some(array) = current.as_array_mut() {
                    if index < array.len() {
                        &mut array[index]
                    } else {
                        println!("[ERROR] Array index {} out of bounds in path {}", index, path);
                        return;
                    }
                } else {
                    println!("[ERROR] Expected array at {} in path {}", part, path);
                    return;
                }
            } else {
                // Navigate through object
                if let Some(obj) = current.as_object_mut() {
                    if let Some(next) = obj.get_mut(part) {
                        next
                    } else {
                        println!("[ERROR] Key {} not found in path {}", part, path);
                        return;
                    }
                } else {
                    println!("[ERROR] Expected object at {} in path {}", part, path);
                    return;
                }
            };
        }
        
        // Remove from parent container
        let target = path_parts[path_parts.len() - 1];
        if let Ok(index) = target.parse::<usize>() {
            // Removing from array at specific index
            if let Some(array) = current.as_array_mut() {
                if index < array.len() {
                    array.remove(index);
                } else {
                    println!("[ERROR] Array index {} out of bounds for remove operation (array length: {})", index, array.len());
                }
            } else {
                println!("[ERROR] Expected array for numeric index {} in remove operation", target);
            }
        } else {
            // Removing from object
            if let Some(obj) = current.as_object_mut() {
                obj.remove(target);
            } else {
                println!("[ERROR] Expected object for key {} in remove operation", target);
            }
        }
    }
}

pub fn apply_action(
    bundle: &Bundle,
    state: &mut Value,
    player_id: &str,
    action: &Value,
) -> Result<Vec<Value>, String> {
    apply_action_with_animation(bundle, state, player_id, action, &AnimationConfig::default())
}

pub fn apply_action_with_animation(
    bundle: &Bundle,
    state: &mut Value,
    player_id: &str,
    action: &Value,
    animation_config: &AnimationConfig,
) -> Result<Vec<Value>, String> {
    // Extract verb and args from action
    if let (Some(verb), Some(args)) = (
        action.get("verb").and_then(|v| v.as_str()),
        action.get("args")
    ) {
        // Replace template variables in args with actor context
        let mut processed_args = replace_template_vars(args, state);
        
        // Also handle {actor} template with the player_id
        processed_args = replace_actor_template(&processed_args, player_id);
        
        // Apply verb with animation support
        let discrete_updates = apply_verb_with_animation(state, verb, &processed_args, bundle, animation_config)?;
        
        // For now, flatten discrete updates into patches
        // In the future, we might return DiscreteUpdate directly
        let mut patches = Vec::new();
        for update in discrete_updates {
            patches.extend(update.patches);
        }
        
        Ok(patches)
    } else {
        Err("Invalid action format".to_string())
    }
}

pub fn replace_actor_template(value: &Value, actor_id: &str) -> Value {
    match value {
        Value::String(s) => {
            Value::String(s.replace("{actor}", actor_id))
        }
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), replace_actor_template(v, actor_id));
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| replace_actor_template(v, actor_id)).collect())
        }
        _ => value.clone()
    }
}

fn generate_log_patch(state: &Value, action_def: &Value, args: &Value, log_template: &str) -> Value {
    let mut log_text = log_template.to_string();
    
    // Replace {player} with current player name
    if let Some(current_player_id) = state["currentPlayer"].as_str() {
        // First try to get the actual player name from the players array
        let player_name = if let Some(players) = state["players"].as_array() {
            players.iter()
                .find(|p| p["id"].as_str() == Some(current_player_id))
                .and_then(|p| p["name"].as_str())
                .map(|n| n.to_string())
                .unwrap_or_else(|| current_player_id.to_string())
        } else {
            current_player_id.to_string()
        };
        log_text = log_text.replace("{player}", &player_name);
        
        // Also replace any direct p1/p2 references with player names
        if current_player_id == "p1" {
            log_text = log_text.replace(" p1 ", &format!(" {} ", player_name));
            log_text = log_text.replace(" p1.", &format!(" {}.", player_name));
            log_text = log_text.replace(" p1!", &format!(" {}!", player_name));
            log_text = log_text.replace(" p1,", &format!(" {},", player_name));
            log_text = log_text.replace(" p1's", &format!(" {}'s", player_name));
        }
    }
    
    // Replace all p1/p2 references with actual player names
    if let Some(players) = state["players"].as_array() {
        for player in players {
            if let (Some(id), Some(name)) = (player["id"].as_str(), player["name"].as_str()) {
                // Replace various patterns where the player ID might appear
                log_text = log_text.replace(&format!(" {} ", id), &format!(" {} ", name));
                log_text = log_text.replace(&format!(" {}.", id), &format!(" {}.", name));
                log_text = log_text.replace(&format!(" {}!", id), &format!(" {}!", name));
                log_text = log_text.replace(&format!(" {},", id), &format!(" {},", name));
                log_text = log_text.replace(&format!(" {}'s", id), &format!(" {}'s", name));
                log_text = log_text.replace(&format!(" {}", id), &format!(" {}", name)); // At end of string
                log_text = log_text.replace(&format!("{} ", id), &format!("{} ", name)); // At start of string
                
                // Also replace when followed by a newline or at the very end
                if log_text.ends_with(id) {
                    let prefix = &log_text[..log_text.len() - id.len()];
                    log_text = format!("{}{}", prefix, name);
                }
            }
        }
    }
    
    // Replace {nextPlayer} if this is a nextTurn action
    if action_def["uses"].as_str() == Some("nextTurn") {
        if let Some(current_player_id) = state["currentPlayer"].as_str() {
            // Get player order to determine next player
            let next_player_id = if let Some(player_order) = state["playerOrder"].as_array() {
                let player_ids: Vec<&str> = player_order.iter()
                    .filter_map(|p| p.as_str())
                    .collect();
                
                if let Some(current_index) = player_ids.iter().position(|&p| p == current_player_id) {
                    let next_index = (current_index + 1) % player_ids.len();
                    player_ids.get(next_index).copied()
                } else {
                    None
                }
            } else {
                // Fallback for 2-player games
                Some(if current_player_id == "p1" { "p2" } else { "p1" })
            };
            
            // Get the actual name for the next player
            if let Some(next_id) = next_player_id {
                let next_player_name = if let Some(players) = state["players"].as_array() {
                    players.iter()
                        .find(|p| p["id"].as_str() == Some(next_id))
                        .and_then(|p| p["name"].as_str())
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| next_id.to_string())
                } else {
                    next_id.to_string()
                };
                log_text = log_text.replace("{nextPlayer}", &next_player_name);
            }
        }
    }
    
    // Replace args values
    if let Some(args_obj) = args.as_object() {
        for (key, value) in args_obj {
            if let Some(str_val) = value.as_str() {
                let pattern = format!("{{{}}}", key);
                log_text = log_text.replace(&pattern, str_val);
            } else if let Some(num_val) = value.as_i64() {
                let pattern = format!("{{{}}}", key);
                log_text = log_text.replace(&pattern, &num_val.to_string());
            }
        }
    }
    
    // Handle special cases for specific verbs
    if action_def["uses"].as_str() == Some("formPairs") {
        // If the log template contains {rank}, try to get it from args or selection
        if log_text.contains("{rank}") {
            if let Some(rank) = args["rank"].as_str() {
                log_text = log_text.replace("{rank}", rank);
            } else if let Some(selection) = state["selection"].as_object() {
                if let Some(rank) = selection["selectedRank"].as_str() {
                    log_text = log_text.replace("{rank}", rank);
                }
            }
        }
    }
    
    // Replace selection values
    if let Some(selection) = state["selection"].as_object() {
        for (key, value) in selection {
            if let Some(str_val) = value.as_str() {
                let pattern = format!("{{selection.{}}}", key);
                log_text = log_text.replace(&pattern, str_val);
            }
        }
    }
    
    // Replace any args.X patterns
    let args_pattern = regex::Regex::new(r"\{args\.(\w+)\}").unwrap();
    let mut new_log_text = log_text.clone();
    for cap in args_pattern.captures_iter(&log_text) {
        if let Some(field_name) = cap.get(1) {
            let field = field_name.as_str();
            if let Some(value) = args[field].as_str() {
                let pattern = format!("{{args.{}}}", field);
                new_log_text = new_log_text.replace(&pattern, value);
            }
        }
    }
    log_text = new_log_text;
    
    // Create timestamp
    let timestamp = chrono::Local::now().format("%H:%M").to_string();
    
    // Create log entry patch
    json!({
        "op": "add",
        "path": "/ui/gameLog/-",
        "value": {
            "message": log_text,
            "timestamp": timestamp,
            "auto": true
        }
    })
}
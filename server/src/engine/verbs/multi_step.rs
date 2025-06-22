use serde_json::{json, Value, Map};
use std::collections::HashMap;
use crate::bundle::Bundle;
use crate::engine::patches::replace_template_vars;
use regex::Regex;

/// Represents a multi-step action definition from YAML
#[derive(Debug, Clone)]
pub struct MultiStepAction {
    pub id: String,
    pub cancellable: bool,
    pub confirm_before_finalizing: bool,
    pub confirmation_prompt: Option<String>,
    pub state_store: Vec<String>,
    pub steps: Vec<MultiStepActionStep>,
    pub result: MultiStepActionResult,
}

/// Represents a single step within a multi-step action
#[derive(Debug, Clone)]
pub struct MultiStepActionStep {
    pub id: String,
    pub action_type: String, // The "as" field (e.g., "bf.selectEntity")
    pub with: Value,
    pub ui: Option<Value>,
    pub store: String, // Which state variable to store the result in
    pub when: Option<Vec<Value>>, // Conditions for this step
}

/// Represents the final result action of a multi-step sequence
#[derive(Debug, Clone)]
pub struct MultiStepActionResult {
    pub action_type: String, // The "as" field
    pub with: Value,
    pub ui: Option<Value>,
}

/// Represents the current state of a player's multi-step action
#[derive(Debug, Clone)]
pub struct MultiStepState {
    pub action_id: String,
    pub current_step: usize,
    pub stored_values: HashMap<String, Value>,
    pub can_cancel: bool,
    pub deferred_logs: Vec<Value>,
    pub created_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
}

/// Parse a multi-step action from YAML/JSON value
pub fn parse_multi_step_action(action: &Value) -> Result<MultiStepAction, String> {
    let id = action.get("id")
        .and_then(|v| v.as_str())
        .ok_or("Multi-step action missing 'id' field")?
        .to_string();

    let cancellable = action.get("cancellable")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let confirm_before_finalizing = action.get("confirmBeforeFinalizing")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let confirmation_prompt = action.get("ui")
        .and_then(|ui| ui.get("confirmationPrompt"))
        .and_then(|p| p.as_str())
        .map(|s| s.to_string());

    // Parse state store variables
    let state_store = action.get("stateStore")
        .and_then(|v| v.as_array())
        .ok_or("Multi-step action missing 'stateStore' array")?
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect();

    // Parse steps
    let steps_array = action.get("steps")
        .and_then(|v| v.as_array())
        .ok_or("Multi-step action missing 'steps' array")?;

    let mut steps = Vec::new();
    for step_value in steps_array {
        let step = parse_multi_step_step(step_value)?;
        steps.push(step);
    }

    // Parse result
    let result_value = action.get("result")
        .ok_or("Multi-step action missing 'result' field")?;
    let result = parse_multi_step_result(result_value)?;

    Ok(MultiStepAction {
        id,
        cancellable,
        confirm_before_finalizing,
        confirmation_prompt,
        state_store,
        steps,
        result,
    })
}

/// Parse a single step from YAML/JSON
fn parse_multi_step_step(step: &Value) -> Result<MultiStepActionStep, String> {
    let id = step.get("id")
        .and_then(|v| v.as_str())
        .ok_or("Step missing 'id' field")?
        .to_string();

    let action_type = step.get("as")
        .and_then(|v| v.as_str())
        .ok_or("Step missing 'as' field")?
        .to_string();

    let with = step.get("with")
        .cloned()
        .unwrap_or(json!({}));

    let ui = step.get("ui").cloned();

    let store = step.get("store")
        .and_then(|v| v.as_str())
        .ok_or("Step missing 'store' field")?
        .to_string();

    let when = step.get("when")
        .and_then(|v| v.as_array())
        .map(|arr| arr.clone());

    Ok(MultiStepActionStep {
        id,
        action_type,
        with,
        ui,
        store,
        when,
    })
}

/// Parse the result action from YAML/JSON
fn parse_multi_step_result(result: &Value) -> Result<MultiStepActionResult, String> {
    let action_type = result.get("as")
        .and_then(|v| v.as_str())
        .ok_or("Result missing 'as' field")?
        .to_string();

    let with = result.get("with")
        .cloned()
        .unwrap_or(json!({}));

    let ui = result.get("ui").cloned();

    Ok(MultiStepActionResult {
        action_type,
        with,
        ui,
    })
}

/// Format a board cell path into a readable coordinate
fn format_board_cell_path(path: &str) -> String {
    // Extract row and col from path like "/zones/board/cells/1/2"
    if let Some(captures) = regex::Regex::new(r"/zones/board/cells/(\d+)/(\d+)").unwrap().captures(path) {
        if let (Some(row_match), Some(col_match)) = (captures.get(1), captures.get(2)) {
            if let (Ok(row), Ok(col)) = (row_match.as_str().parse::<i32>(), col_match.as_str().parse::<i32>()) {
                return format!("({}, {})", col, row);
            }
        }
    }
    // Fallback to original path if parsing fails
    path.to_string()
}

/// Apply template substitution to a value using stored state
pub fn apply_multi_step_templates(value: &Value, stored_values: &HashMap<String, Value>) -> Value {
    apply_multi_step_templates_with_formatting(value, stored_values, true)
}

/// Apply template substitution without formatting (for action execution)
pub fn apply_multi_step_templates_no_format(value: &Value, stored_values: &HashMap<String, Value>) -> Value {
    apply_multi_step_templates_with_formatting(value, stored_values, false)
}

/// Apply template substitution to a value using stored state, with optional formatting
fn apply_multi_step_templates_with_formatting(value: &Value, stored_values: &HashMap<String, Value>, format_paths: bool) -> Value {
    match value {
        Value::String(s) => {
            let mut result = s.clone();
            
            // Handle dotted notation like {key.property}
            let re = regex::Regex::new(r"\{([^}]+)\}").unwrap();
            for cap in re.captures_iter(s) {
                if let Some(full_match) = cap.get(0) {
                    if let Some(path_match) = cap.get(1) {
                        let path = path_match.as_str();
                        let placeholder = full_match.as_str();
                        
                        // Check if this is a dotted path like "key.property"
                        if path.contains('.') {
                            let parts: Vec<&str> = path.split('.').collect();
                            if parts.len() == 2 {
                                let key = parts[0];
                                let property = parts[1];
                                
                                if let Some(obj) = stored_values.get(key).and_then(|v| v.as_object()) {
                                    if let Some(prop_val) = obj.get(property) {
                                        let val_str = match prop_val {
                                            Value::String(s) => s.clone(),
                                            Value::Number(n) => n.to_string(),
                                            Value::Bool(b) => b.to_string(),
                                            _ => prop_val.to_string(),
                                        };
                                        result = result.replace(placeholder, &val_str);
                                    }
                                }
                            }
                        } else {
                            // Handle simple placeholders like {key}
                            if let Some(val) = stored_values.get(path) {
                                let val_str = if let Some(s) = val.as_str() {
                                    // Check if this is a board cell path and format it nicely
                                    if format_paths && s.contains("/zones/board/cells/") {
                                        format_board_cell_path(s)
                                    } else {
                                        s.to_string()
                                    }
                                } else if let Some(obj) = val.as_object() {
                                    // For objects, try to extract a meaningful representation
                                    if let Some(location) = obj.get("location").and_then(|l| l.as_str()) {
                                        if format_paths && location.contains("/zones/board/cells/") {
                                            format_board_cell_path(location)
                                        } else {
                                            location.to_string()
                                        }
                                    } else if let Some(row) = obj.get("row").and_then(|r| r.as_i64()) {
                                        if let Some(col) = obj.get("col").and_then(|c| c.as_i64()) {
                                            format!("({}, {})", col, row)
                                        } else {
                                            val.to_string()
                                        }
                                    } else {
                                        val.to_string()
                                    }
                                } else {
                                    // For other types, serialize to JSON string
                                    val.to_string()
                                };
                                result = result.replace(placeholder, &val_str);
                            }
                        }
                    }
                }
            }
            Value::String(result)
        },
        Value::Object(obj) => {
            let mut new_obj = Map::new();
            for (key, val) in obj {
                new_obj.insert(key.clone(), apply_multi_step_templates_with_formatting(val, stored_values, format_paths));
            }
            Value::Object(new_obj)
        },
        Value::Array(arr) => {
            Value::Array(
                arr.iter()
                    .map(|v| apply_multi_step_templates_with_formatting(v, stored_values, format_paths))
                    .collect()
            )
        },
        _ => value.clone(),
    }
}

/// Validate that a multi-step action is properly formed
pub fn validate_multi_step_action(action: &MultiStepAction) -> Result<(), String> {
    // Check that all stored variables are actually used
    let mut used_vars = std::collections::HashSet::new();
    
    // Check variables used in result
    collect_template_variables(&action.result.with, &mut used_vars);
    if let Some(ui) = &action.result.ui {
        collect_template_variables(ui, &mut used_vars);
    }

    // Check variables used in confirmation prompt
    if let Some(prompt) = &action.confirmation_prompt {
        for var in &action.state_store {
            let placeholder = format!("{{{}}}", var);
            if prompt.contains(&placeholder) {
                used_vars.insert(var.clone());
            }
        }
    }

    // Verify all state_store variables are actually stored by steps
    let stored_vars: std::collections::HashSet<String> = 
        action.steps.iter().map(|step| step.store.clone()).collect();

    for var in &action.state_store {
        if !stored_vars.contains(var) {
            return Err(format!("State variable '{}' declared but not stored by any step", var));
        }
    }

    // Verify all stored variables are declared in state_store
    for var in &stored_vars {
        if !action.state_store.contains(var) {
            return Err(format!("Step stores variable '{}' but it's not declared in stateStore", var));
        }
    }

    // Verify steps reference valid actions (basic check)
    for step in &action.steps {
        if !step.action_type.starts_with("bf.") {
            return Err(format!("Step '{}' uses unknown action type '{}'", step.id, step.action_type));
        }
    }

    if !action.result.action_type.starts_with("bf.") {
        return Err(format!("Result uses unknown action type '{}'", action.result.action_type));
    }

    Ok(())
}

/// Helper function to collect template variables from a JSON value
fn collect_template_variables(value: &Value, used_vars: &mut std::collections::HashSet<String>) {
    match value {
        Value::String(s) => {
            // Simple regex-like extraction of {variable} patterns
            let mut chars = s.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '{' {
                    let mut var_name = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == '}' {
                            chars.next(); // consume the '}'
                            used_vars.insert(var_name);
                            break;
                        } else {
                            var_name.push(chars.next().unwrap());
                        }
                    }
                }
            }
        },
        Value::Object(obj) => {
            for val in obj.values() {
                collect_template_variables(val, used_vars);
            }
        },
        Value::Array(arr) => {
            for val in arr {
                collect_template_variables(val, used_vars);
            }
        },
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_multi_step_action() {
        let action_json = json!({
            "id": "movePiece",
            "type": "multiStep",
            "cancellable": true,
            "confirmBeforeFinalizing": true,
            "ui": {
                "confirmationPrompt": "Move {selectedPiece} to {destination}?"
            },
            "stateStore": ["selectedPiece", "destination"],
            "steps": [
                {
                    "id": "selectPiece",
                    "as": "bf.selectEntity",
                    "with": {"source": "board"},
                    "store": "selectedPiece"
                },
                {
                    "id": "selectDestination", 
                    "as": "bf.selectMapSpace",
                    "with": {"zone": "board"},
                    "store": "destination"
                }
            ],
            "result": {
                "as": "bf.moveEntity",
                "with": {
                    "entity": "{selectedPiece}",
                    "destination": "{destination}"
                }
            }
        });

        let action = parse_multi_step_action(&action_json).unwrap();
        assert_eq!(action.id, "movePiece");
        assert_eq!(action.steps.len(), 2);
        assert_eq!(action.state_store.len(), 2);
        assert!(action.cancellable);
        assert!(action.confirm_before_finalizing);
    }
}

/// Response types for multi-step execution
#[derive(Debug, Clone)]
pub enum MultiStepResponse {
    StepReady {
        step_id: String,
        action_type: String,
        action_map: HashMap<String, Value>,
        direction: Option<String>,
        can_cancel: bool,
    },
    ConfirmationRequired {
        prompt: String,
    },
    Completed {
        patches: Vec<Value>,
        log_entry: Option<String>,
    },
    Cancelled,
}

/// Execute the next step in a multi-step action
pub fn execute_multi_step_next(
    bundle: &Bundle,
    state: &mut Value,
    player_id: &str,
    multi_step_state: &mut MultiStepState,
    action_def: &MultiStepAction,
) -> Result<MultiStepResponse, String> {
    println!("[DEBUG MultiStep] execute_multi_step_next: player={}, current_step={}, total_steps={}", 
        player_id, multi_step_state.current_step, action_def.steps.len());
    
    // Check if we're at the confirmation stage
    if multi_step_state.current_step >= action_def.steps.len() {
        println!("[DEBUG MultiStep] At confirmation stage or beyond");
        if action_def.confirm_before_finalizing {
            // Generate confirmation prompt with template substitution
            let prompt = if let Some(template) = &action_def.confirmation_prompt {
                println!("[MultiStep] Template before substitution: {}", template);
                println!("[MultiStep] Stored values: {:?}", multi_step_state.stored_values);
                let prompt_value = Value::String(template.clone());
                let substituted = apply_multi_step_templates(&prompt_value, &multi_step_state.stored_values);
                println!("[MultiStep] Template after substitution: {:?}", substituted);
                substituted.as_str().unwrap_or("Confirm action?").to_string()
            } else {
                "Confirm action?".to_string()
            };
            
            return Ok(MultiStepResponse::ConfirmationRequired { prompt });
        } else {
            // Execute the final result
            return execute_multi_step_finalize(bundle, state, player_id, multi_step_state, action_def);
        }
    }
    
    // Get current step
    let current_step = &action_def.steps[multi_step_state.current_step];
    println!("[DEBUG MultiStep] Current step: {} (type: {})", current_step.id, current_step.action_type);
    
    // Check conditions if any
    if let Some(conditions) = &current_step.when {
        println!("[DEBUG MultiStep] Step has {} conditions to check", conditions.len());
        // TODO: Evaluate conditions
        // For now, we'll skip condition evaluation
    }
    
    // Generate action map for this step
    println!("[DEBUG MultiStep] Generating action map for step");
    let action_map = generate_step_action_map(bundle, state, player_id, current_step, &multi_step_state.stored_values)?;
    
    // Prepare UI direction
    let direction = current_step.ui
        .as_ref()
        .and_then(|ui| ui.get("direction"))
        .and_then(|d| d.as_str())
        .map(|s| {
            let dir_value = Value::String(s.to_string());
            let substituted = apply_multi_step_templates(&dir_value, &multi_step_state.stored_values);
            substituted.as_str().unwrap_or(s).to_string()
        });
    
    Ok(MultiStepResponse::StepReady {
        step_id: current_step.id.clone(),
        action_type: current_step.action_type.clone(),
        action_map,
        direction,
        can_cancel: multi_step_state.can_cancel,
    })
}

/// Execute a step selection (when player makes a choice)
pub fn execute_multi_step_selection(
    bundle: &Bundle,
    state: &mut Value,
    player_id: &str,
    multi_step_state: &mut MultiStepState,
    action_def: &MultiStepAction,
    selection: &Value,
) -> Result<MultiStepResponse, String> {
    if multi_step_state.current_step >= action_def.steps.len() {
        return Err("No active step to process selection".to_string());
    }
    
    let current_step = &action_def.steps[multi_step_state.current_step];
    
    // Store the selection value
    // For multi-step actions that need paths, prefer storing just the location path
    let value_to_store = if let Some(location) = selection.get("location").and_then(|l| l.as_str()) {
        // If location is a full path (starts with /zones/), store it directly
        if location.starts_with("/zones/") {
            Value::String(location.to_string())
        } else {
            // Otherwise store the full selection object
            selection.clone()
        }
    } else {
        selection.clone()
    };
    
    multi_step_state.stored_values.insert(current_step.store.clone(), value_to_store);
    
    // Move to next step
    multi_step_state.current_step += 1;
    
    // Execute next step
    execute_multi_step_next(bundle, state, player_id, multi_step_state, action_def)
}

/// Finalize a multi-step action (execute the result)
pub fn execute_multi_step_finalize(
    bundle: &Bundle,
    state: &mut Value,
    player_id: &str,
    multi_step_state: &MultiStepState,
    action_def: &MultiStepAction,
) -> Result<MultiStepResponse, String> {
    // Apply template substitution to the result action (without formatting paths)
    let processed_args = apply_multi_step_templates_no_format(&action_def.result.with, &multi_step_state.stored_values);
    
    // Execute the result action
    // Remove "bf." prefix if present for verb lookup
    let verb = if action_def.result.action_type.starts_with("bf.") {
        &action_def.result.action_type[3..]
    } else {
        &action_def.result.action_type
    };
    
    println!("[MultiStep] Executing result verb '{}' with args: {:?}", verb, processed_args);
    
    let patches = crate::engine::verbs::apply_verb(
        state,
        verb,
        &processed_args,
        bundle
    )?;
    
    println!("[MultiStep] Result verb returned {} patches", patches.len());
    for (i, patch) in patches.iter().enumerate() {
        println!("[MultiStep] Patch {}: {:?}", i, patch);
    }
    
    // Generate log entry if specified
    let log_entry = if let Some(ui) = &action_def.result.ui {
        if let Some(log_template) = ui.get("logTemplate").and_then(|l| l.as_str()) {
            // First apply multi-step template substitution
            let log_value = Value::String(log_template.to_string());
            let substituted = apply_multi_step_templates(&log_value, &multi_step_state.stored_values);
            let mut log_text = substituted.as_str().unwrap_or(log_template).to_string();
            
            // Then apply standard template variables like {player}
            if let Some(current_player_id) = state.get("currentPlayer").and_then(|cp| cp.as_str()) {
                let player_name = if let Some(players) = state.get("players").and_then(|p| p.as_array()) {
                    players.iter()
                        .find(|p| p.get("id").and_then(|id| id.as_str()) == Some(current_player_id))
                        .and_then(|p| p.get("name").and_then(|n| n.as_str()))
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| current_player_id.to_string())
                } else {
                    current_player_id.to_string()
                };
                log_text = log_text.replace("{player}", &player_name);
                
                // Also handle {actor} which is the same as {player} in single-player actions
                log_text = log_text.replace("{actor}", &player_name);
            }
            
            Some(log_text)
        } else {
            None
        }
    } else {
        None
    };
    
    Ok(MultiStepResponse::Completed {
        patches,
        log_entry,
    })
}

/// Helper function to replace {actor} with the actual player ID in a value
fn replace_actor_in_value(value: &mut Value, player_id: &str) {
    match value {
        Value::String(s) => {
            *s = s.replace("{actor}", player_id);
        }
        Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                replace_actor_in_value(v, player_id);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                replace_actor_in_value(v, player_id);
            }
        }
        _ => {}
    }
}

/// Generate action map for a specific step
fn generate_step_action_map(
    bundle: &Bundle,
    state: &Value,
    player_id: &str,
    step: &MultiStepActionStep,
    stored_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, String> {
    println!("[DEBUG MultiStep] generate_step_action_map: type={}, player={}", step.action_type, player_id);
    
    // Apply template substitution to step parameters
    let mut processed_with = apply_multi_step_templates(&step.with, stored_values);
    
    // Also replace {actor} with the current player ID
    if let Value::Object(ref mut obj) = processed_with {
        for (_, value) in obj.iter_mut() {
            replace_actor_in_value(value, player_id);
        }
    }
    
    println!("[DEBUG MultiStep] Processed step parameters: {:?}", processed_with);
    
    // Based on the step type, generate appropriate action map entries
    match step.action_type.as_str() {
        "bf.selectEntity" => generate_select_entity_actions(bundle, state, player_id, &processed_with),
        "bf.selectMapSpace" | "bf.selectLocation" => generate_select_map_space_actions(bundle, state, player_id, &processed_with),
        "bf.selectChoice" => generate_select_choice_actions(bundle, state, player_id, &processed_with),
        _ => Err(format!("Unknown multi-step action type: {}", step.action_type)),
    }
}

/// Generate actions for entity selection
fn generate_select_entity_actions(
    _bundle: &Bundle,
    state: &Value,
    player_id: &str,
    params: &Value,
) -> Result<HashMap<String, Value>, String> {
    println!("[DEBUG MultiStep] generate_select_entity_actions: player={}", player_id);
    let mut actions = HashMap::new();
    
    // Get source zone (can be specified as 'zone' or 'source')
    let source = params.get("zone")
        .or_else(|| params.get("source"))
        .and_then(|s| s.as_str())
        .ok_or("selectEntity missing 'zone' or 'source' parameter")?;
    
    // Remove /zones/ prefix if present
    let zone_name = if source.starts_with("/zones/") {
        &source[7..]
    } else {
        source
    };
    println!("[DEBUG MultiStep] Source zone: {}", zone_name);
    
    // Get entity filter (e.g., "piece_{player}")
    let entity_filter = params.get("entityFilter")
        .and_then(|e| e.as_str());
    println!("[DEBUG MultiStep] Entity filter: {:?}", entity_filter);
    
    // Process entity filter to replace {player} with actual player ID
    let processed_filter = entity_filter.map(|filter| {
        filter.replace("{player}", player_id)
            .replace("{actor}", player_id)
    });
    println!("[DEBUG MultiStep] Processed entity filter: {:?}", processed_filter);
    
    // Get conditions
    let conditions = params.get("conditions")
        .and_then(|c| c.as_array());
    println!("[DEBUG MultiStep] Conditions: {:?}", conditions);
    
    // Find entities in the source zone
    if let Some(zone_data) = state.get("zones").and_then(|z| z.get(zone_name)) {
        println!("[DEBUG MultiStep] Found zone data: {:?}", zone_data);
        // Handle different zone types
        if let Some(cells) = zone_data.get("cells").and_then(|c| c.as_array()) {
            // Grid zone
            for (row_idx, row) in cells.iter().enumerate() {
                if let Some(row_array) = row.as_array() {
                    for (col_idx, cell) in row_array.iter().enumerate() {
                        if let Some(entity) = cell.get("entity").and_then(|e| e.as_str()) {
                            // Check if entity matches the filter
                            let matches_filter = if let Some(ref filter) = processed_filter {
                                entity == filter
                            } else {
                                true // No filter means all entities are valid
                            };
                            
                            if matches_filter && check_entity_conditions(entity, player_id, conditions) {
                                let location = format!("/zones/{}/cells/{}/{}", zone_name, row_idx, col_idx);
                                actions.insert(location.clone(), json!({
                                    "action": "multiStepSelect",
                                    "args": {
                                        "location": location,
                                        "entity": entity,
                                        "row": row_idx,
                                        "col": col_idx
                                    }
                                }));
                                println!("[DEBUG MultiStep] Added action for entity {} at {}", entity, location);
                            }
                        }
                    }
                }
            }
        } else if let Some(items) = zone_data.get("items").and_then(|i| i.as_array()) {
            // Card zone
            for (idx, item) in items.iter().enumerate() {
                if let Some(entity) = item.as_str() {
                    // Check if entity matches the filter
                    let matches_filter = if let Some(ref filter) = processed_filter {
                        entity == filter
                    } else {
                        true // No filter means all entities are valid
                    };
                    
                    if matches_filter && check_entity_conditions(entity, player_id, conditions) {
                        let location = format!("/zones/{}/items/{}", zone_name, idx);
                        actions.insert(location, json!({
                            "action": "multiStepSelect",
                            "args": {
                                "entity": entity,
                                "index": idx
                            }
                        }));
                    }
                }
            }
        }
    }
    
    Ok(actions)
}

/// Generate actions for map space selection
fn generate_select_map_space_actions(
    _bundle: &Bundle,
    state: &Value,
    _player_id: &str,
    params: &Value,
) -> Result<HashMap<String, Value>, String> {
    let mut actions = HashMap::new();
    
    // Get target zone
    let zone = params.get("zone")
        .and_then(|z| z.as_str())
        .ok_or("selectMapSpace missing 'zone' parameter")?;
    
    // Remove /zones/ prefix if present
    let zone_name = if zone.starts_with("/zones/") {
        &zone[7..]
    } else {
        zone
    };
    
    // Check if we should only show empty cells
    let empty_only = params.get("emptyOnly")
        .and_then(|e| e.as_bool())
        .unwrap_or(false);
    
    // Get conditions
    let conditions = params.get("conditions")
        .and_then(|c| c.as_array());
    
    // Find valid spaces in the zone
    if let Some(zone_data) = state.get("zones").and_then(|z| z.get(zone_name)) {
        if let Some(cells) = zone_data.get("cells").and_then(|c| c.as_array()) {
            for (row_idx, row) in cells.iter().enumerate() {
                if let Some(row_array) = row.as_array() {
                    for (col_idx, cell) in row_array.iter().enumerate() {
                        // Check empty_only condition first
                        let is_empty = cell.is_null() || cell.get("entity").is_none();
                        if empty_only && !is_empty {
                            continue; // Skip non-empty cells if emptyOnly is true
                        }
                        
                        if check_space_conditions(cell, conditions) {
                            let location = format!("/zones/{}/cells/{}/{}", zone_name, row_idx, col_idx);
                            actions.insert(location.clone(), json!({
                                "action": "multiStepSelect",
                                "args": {
                                    "location": location,
                                    "row": row_idx,
                                    "col": col_idx
                                }
                            }));
                            println!("[DEBUG MultiStep] Added map space action at {}", location);
                        }
                    }
                }
            }
        }
    }
    
    Ok(actions)
}

/// Generate actions for choice selection
fn generate_select_choice_actions(
    _bundle: &Bundle,
    state: &Value,
    player_id: &str,
    params: &Value,
) -> Result<HashMap<String, Value>, String> {
    println!("[DEBUG MultiStep] generate_select_choice_actions called with params: {:?}", params);
    let mut actions = HashMap::new();
    
    // Check for dynamicChoices parameter first
    if let Some(dynamic_config) = params.get("dynamicChoices") {
        println!("[DEBUG MultiStep] Found dynamicChoices config: {:?}", dynamic_config);
        if dynamic_config.get("type").and_then(|t| t.as_str()) == Some("uniqueProperty") {
            // Extract unique property values from a zone
            if let (Some(from_zone), Some(property)) = (
                dynamic_config.get("fromZone").and_then(|z| z.as_str()),
                dynamic_config.get("property").and_then(|p| p.as_str())
            ) {
                let zone_id = from_zone.trim_start_matches("/zones/");
                if let Some(zone) = state.get("zones").and_then(|z| z.get(zone_id)) {
                    if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
                        let mut unique_values = std::collections::HashSet::new();
                        
                        // Collect unique property values
                        for item in items {
                            let entity = if let Some(e) = item.get("entity") {
                                e.as_str()
                            } else {
                                item.as_str()
                            };
                                
                            if let Some(entity_str) = entity {
                                if entity_str.starts_with("card_") {
                                    let parts: Vec<&str> = entity_str.split('_').collect();
                                    if parts.len() >= 3 {
                                        let value = match property {
                                            "rank" => parts[2],
                                            "suit" => parts[1],
                                            _ => continue,
                                        };
                                        unique_values.insert(value.to_string());
                                    }
                                }
                            }
                        }
                        
                        // Get labels if provided
                        let labels = dynamic_config.get("labels").and_then(|l| l.as_object());
                        
                        // Create actions for each unique value
                        for value in unique_values {
                            let label = labels
                                .and_then(|l| l.get(&value))
                                .and_then(|v| v.as_str())
                                .unwrap_or(&value);
                                
                            let location = format!("/ranks/{}", value);
                            actions.insert(location, json!({
                                "action": "multiStepSelect",
                                "args": {
                                    "choice": value,
                                    "label": label
                                }
                            }));
                        }
                    }
                }
            }
        }
    }
    
    // Check for choiceZone parameter (dynamic choices from a zone)
    if let Some(choice_zone_path) = params.get("choiceZone").and_then(|z| z.as_str()) {
        // Get the choice zone from state
        let zone = state.get("zones")
            .and_then(|zones| {
                let zone_id = choice_zone_path.trim_start_matches("/zones/");
                zones.get(zone_id)
            });
            
        if let Some(zone) = zone {
            // Get items from the choice zone
            if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
                for item in items {
                    if let Some(item_obj) = item.as_object() {
                        if let Some(id) = item_obj.get("id").and_then(|i| i.as_str()) {
                            let location = format!("{}/{}", choice_zone_path, id);
                            actions.insert(location, json!({
                                "action": "multiStepSelect",
                                "args": item_obj
                            }));
                        }
                    }
                }
            }
        }
    }
    
    // Also support static choices from parameters
    if let Some(choices) = params.get("choices").and_then(|c| c.as_array()) {
        for choice in choices {
            if let Some(choice_id) = choice.as_str() {
                let location = format!("/choices/{}", choice_id);
                actions.insert(location, json!({
                    "action": "multiStepSelect",
                    "args": {
                        "choice": choice_id
                    }
                }));
            }
        }
    }
    
    Ok(actions)
}

/// Check if an entity matches the given conditions
fn check_entity_conditions(entity: &str, player_id: &str, conditions: Option<&Vec<Value>>) -> bool {
    if let Some(conditions) = conditions {
        for condition in conditions {
            if let Some(owner) = condition.get("owner").and_then(|o| o.as_str()) {
                if owner == "{actor}" {
                    // Check if entity belongs to the current player
                    if !entity.ends_with(&format!("_{}", player_id)) {
                        return false;
                    }
                } else if !entity.ends_with(&format!("_{}", owner)) {
                    return false;
                }
            }
            // Add more condition checks as needed
        }
    }
    true
}

/// Check if a space matches the given conditions
fn check_space_conditions(cell: &Value, conditions: Option<&Vec<Value>>) -> bool {
    if let Some(conditions) = conditions {
        for condition in conditions {
            if let Some(is_empty) = condition.get("isEmpty").and_then(|e| e.as_bool()) {
                if is_empty {
                    // Check if cell is empty
                    if cell.get("entity").is_some() {
                        return false;
                    }
                } else {
                    // Check if cell is not empty
                    if cell.get("entity").is_none() {
                        return false;
                    }
                }
            }
            // Add more condition checks as needed
        }
    }
    true
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_parse_multi_step_action() {
        let action_json = json!({
            "id": "movePiece",
            "type": "multiStep",
            "cancellable": true,
            "confirmBeforeFinalizing": true,
            "ui": {
                "confirmationPrompt": "Move {selectedPiece} to {destination}?"
            },
            "stateStore": ["selectedPiece", "destination"],
            "steps": [
                {
                    "id": "selectPiece",
                    "as": "bf.selectEntity",
                    "with": {"source": "board"},
                    "store": "selectedPiece"
                },
                {
                    "id": "selectDestination", 
                    "as": "bf.selectMapSpace",
                    "with": {"zone": "board"},
                    "store": "destination"
                }
            ],
            "result": {
                "as": "bf.moveEntity",
                "with": {
                    "entity": "{selectedPiece}",
                    "destination": "{destination}"
                }
            }
        });

        let action = parse_multi_step_action(&action_json).unwrap();
        assert_eq!(action.id, "movePiece");
        assert_eq!(action.steps.len(), 2);
        assert_eq!(action.state_store.len(), 2);
        assert!(action.cancellable);
        assert!(action.confirm_before_finalizing);
    }

    #[test]
    fn test_apply_templates() {
        let mut stored_values = HashMap::new();
        stored_values.insert("selectedPiece".to_string(), json!("piece_1"));
        stored_values.insert("destination".to_string(), json!("cell_2_3"));

        let template = json!({
            "entity": "{selectedPiece}",
            "destination": "{destination}",
            "message": "Move {selectedPiece} to {destination}"
        });

        let result = apply_multi_step_templates(&template, &stored_values);
        
        assert_eq!(result["entity"], "piece_1");
        assert_eq!(result["destination"], "cell_2_3");
        assert_eq!(result["message"], "Move piece_1 to cell_2_3");
    }
    
    #[test]
    fn test_board_path_formatting() {
        let mut stored_values = HashMap::new();
        stored_values.insert("selectedPiece".to_string(), json!("/zones/board/cells/0/0"));
        stored_values.insert("destination".to_string(), json!("/zones/board/cells/1/1"));

        // Test with formatting (for UI display)
        let ui_template = json!({
            "message": "Move piece from {selectedPiece} to {destination}"
        });
        let ui_result = apply_multi_step_templates(&ui_template, &stored_values);
        assert_eq!(ui_result["message"], "Move piece from (0, 0) to (1, 1)");

        // Test without formatting (for action execution)
        let action_template = json!({
            "from": "{selectedPiece}",
            "to": "{destination}"
        });
        let action_result = apply_multi_step_templates_no_format(&action_template, &stored_values);
        assert_eq!(action_result["from"], "/zones/board/cells/0/0");
        assert_eq!(action_result["to"], "/zones/board/cells/1/1");
    }

    #[test]
    fn test_validate_multi_step_action() {
        let action = MultiStepAction {
            id: "test".to_string(),
            cancellable: true,
            confirm_before_finalizing: false,
            confirmation_prompt: None,
            state_store: vec!["var1".to_string(), "var2".to_string()],
            steps: vec![
                MultiStepActionStep {
                    id: "step1".to_string(),
                    action_type: "bf.selectEntity".to_string(),
                    with: json!({}),
                    ui: None,
                    store: "var1".to_string(),
                    when: None,
                },
                MultiStepActionStep {
                    id: "step2".to_string(),
                    action_type: "bf.selectMapSpace".to_string(),
                    with: json!({}),
                    ui: None,
                    store: "var2".to_string(),
                    when: None,
                }
            ],
            result: MultiStepActionResult {
                action_type: "bf.moveEntity".to_string(),
                with: json!({"entity": "{var1}", "destination": "{var2}"}),
                ui: None,
            },
        };

        assert!(validate_multi_step_action(&action).is_ok());
    }
}

use serde_json::Value;
use std::collections::{HashSet, HashMap};
use crate::bundle::Bundle;

pub struct ValidationError {
    pub game_id: String,
    pub error_type: String,
    pub message: String,
    pub location: String,
}

impl ValidationError {
    fn new(game_id: &str, error_type: &str, message: String, location: String) -> Self {
        Self {
            game_id: game_id.to_string(),
            error_type: error_type.to_string(),
            message,
            location,
        }
    }
}

pub fn validate_bundle(bundle: &Bundle) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    
    // Collect all valid identifiers
    let phase_ids = collect_phase_ids(&bundle.phases);
    let mut action_ids = collect_action_ids(&bundle.actions);
    let mut zone_ids = collect_zone_ids(&bundle.zones);
    let mut entity_ids = collect_entity_ids(&bundle.entities);
    
    // Add built-in actions that are always available
    add_builtin_actions(&mut action_ids);
    
    // Add expanded player-specific zones and entities based on max players
    let max_players = bundle.manifest.metadata.players.max;
    expand_player_specific_ids(&mut zone_ids, &mut entity_ids, &bundle.zones, &bundle.entities, max_players);
    
    // Validate phase references
    validate_phase_references(bundle, &phase_ids, &mut errors);
    
    // Validate action references
    validate_action_references(bundle, &action_ids, &mut errors);
    
    // Validate zone references
    validate_zone_references(bundle, &zone_ids, &mut errors);
    
    // Validate entity references
    validate_entity_references(bundle, &entity_ids, &zone_ids, &mut errors);
    
    // Validate phase transitions for loops and conflicts
    validate_phase_transition_logic(bundle, &phase_ids, &mut errors);
    
    errors
}

fn collect_phase_ids(phases: &Value) -> HashMap<String, HashSet<String>> {
    let mut phase_map = HashMap::new();
    
    if let Some(phase_sets) = phases.as_array() {
        for phase_set in phase_sets {
            if let Some(set_id) = phase_set["id"].as_str() {
                let mut phase_ids = HashSet::new();
                
                if let Some(phases) = phase_set["phases"].as_array() {
                    for phase in phases {
                        if let Some(phase_id) = phase["id"].as_str() {
                            phase_ids.insert(phase_id.to_string());
                        }
                    }
                }
                
                phase_map.insert(set_id.to_string(), phase_ids);
            }
        }
    }
    
    phase_map
}

fn collect_action_ids(actions: &Value) -> HashSet<String> {
    let mut action_ids = HashSet::new();
    
    if let Some(action_list) = actions.as_array() {
        for action in action_list {
            if let Some(id) = action["id"].as_str() {
                action_ids.insert(id.to_string());
            }
        }
    }
    
    action_ids
}

fn collect_zone_ids(zones: &Value) -> HashSet<String> {
    let mut zone_ids = HashSet::new();
    
    if let Some(zone_list) = zones.as_array() {
        for zone in zone_list {
            if let Some(id) = zone["id"].as_str() {
                zone_ids.insert(id.to_string());
            }
        }
    }
    
    zone_ids
}

fn collect_entity_ids(entities: &Value) -> HashSet<String> {
    let mut entity_ids = HashSet::new();
    
    if let Some(entity_list) = entities.as_array() {
        for entity in entity_list {
            if let Some(id) = entity["id"].as_str() {
                entity_ids.insert(id.to_string());
            }
        }
    }
    
    entity_ids
}

fn add_builtin_actions(action_ids: &mut HashSet<String>) {
    // Add common built-in actions that are always available
    let builtins = [
        "turn.advance",
        "turn.reset",
        "phase.set",
        "phase.advance",
        "game.end",
        "score.update",
        "entity.move",
        "entity.select",
        "entity.moveSelected",
        "deck.draw",
        "deck.transfer",
        "zone.reset",
        "transitionToPhase",
        // Built-in verbs from engine
        "draw",
        "moveEntity",
        "place",
        "placeWithGravity",
        "nextTurn",
        "setPhase",
        "grid.lineOfMarks",
        "selectEntity",
        "moveSelected",
        "clearSelection",
        "queryEntities",
        "transferMatching",
        "presentChoice",
        "makeSelection",
        "setState",
        "conditionalAction",
    ];
    
    for builtin in &builtins {
        action_ids.insert(builtin.to_string());
    }
}

fn expand_player_specific_ids(
    zone_ids: &mut HashSet<String>,
    entity_ids: &mut HashSet<String>,
    zones: &Value,
    entities: &Value,
    max_players: u32,
) {
    // Expand zone IDs with {player} pattern
    if let Some(zone_list) = zones.as_array() {
        for zone in zone_list {
            if let Some(id) = zone["id"].as_str() {
                if id.contains("{player}") {
                    for player_num in 1..=max_players {
                        let expanded_id = id.replace("{player}", &format!("p{}", player_num));
                        zone_ids.insert(expanded_id);
                    }
                }
            }
        }
    }
    
    // Expand entity IDs with {player} pattern
    if let Some(entity_list) = entities.as_array() {
        for entity in entity_list {
            if let Some(id) = entity["id"].as_str() {
                if id.contains("{player}") {
                    for player_num in 1..=max_players {
                        let expanded_id = id.replace("{player}", &format!("p{}", player_num));
                        entity_ids.insert(expanded_id);
                    }
                }
            }
        }
    }
}

fn validate_phase_references(bundle: &Bundle, phase_ids: &HashMap<String, HashSet<String>>, errors: &mut Vec<ValidationError>) {
    // Check transitionToPhase in actions
    if let Some(phase_sets) = bundle.phases.as_array() {
        for phase_set in phase_sets {
            let set_id = phase_set["id"].as_str().unwrap_or("unknown");
            
            if let Some(phases) = phase_set["phases"].as_array() {
                for phase in phases {
                    let phase_id = phase["id"].as_str().unwrap_or("unknown");
                    
                    // Check enterActions
                    if let Some(enter_actions) = phase["enterActions"].as_array() {
                        for (idx, action) in enter_actions.iter().enumerate() {
                            if let Some(target) = action.get("transitionToPhase").and_then(|t| t.as_str()) {
                                validate_phase_transition(bundle, target, phase_ids, 
                                    &format!("{}.{}.enterActions[{}]", set_id, phase_id, idx), errors);
                            }
                        }
                    }
                    
                    // Check exitActions
                    if let Some(exit_actions) = phase["exitActions"].as_array() {
                        for (idx, action) in exit_actions.iter().enumerate() {
                            if let Some(target) = action.get("transitionToPhase").and_then(|t| t.as_str()) {
                                validate_phase_transition(bundle, target, phase_ids, 
                                    &format!("{}.{}.exitActions[{}]", set_id, phase_id, idx), errors);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Check transitionToPhase in action definitions
    if let Some(actions) = bundle.actions.as_array() {
        for action in actions {
            let action_id = action["id"].as_str().unwrap_or("unknown");
            
            // Check 'to' field for transitionToPhase actions
            if let Some(to) = action["to"].as_str() {
                validate_phase_transition(bundle, to, phase_ids, 
                    &format!("action.{}.to", action_id), errors);
            }
        }
    }
}

fn validate_phase_transition(bundle: &Bundle, target: &str, phase_ids: &HashMap<String, HashSet<String>>, location: &str, errors: &mut Vec<ValidationError>) {
    // Parse phase transition format: "phaseSet.phaseId"
    if let Some(dot_pos) = target.find('.') {
        let phase_set = &target[..dot_pos];
        let phase_id = &target[dot_pos + 1..];
        
        if let Some(set_phases) = phase_ids.get(phase_set) {
            if !set_phases.contains(phase_id) {
                errors.push(ValidationError::new(
                    &bundle.game_id,
                    "invalid_phase_reference",
                    format!("Phase '{}' does not exist in phase set '{}'", phase_id, phase_set),
                    location.to_string(),
                ));
            }
        } else {
            errors.push(ValidationError::new(
                &bundle.game_id,
                "invalid_phase_set",
                format!("Phase set '{}' does not exist", phase_set),
                location.to_string(),
            ));
        }
    } else {
        errors.push(ValidationError::new(
            &bundle.game_id,
            "invalid_phase_format",
            format!("Invalid phase transition format '{}'. Expected 'phaseSet.phaseId'", target),
            location.to_string(),
        ));
    }
}

fn validate_action_references(bundle: &Bundle, action_ids: &HashSet<String>, errors: &mut Vec<ValidationError>) {
    // Check possibleActions in phases
    if let Some(phase_sets) = bundle.phases.as_array() {
        for phase_set in phase_sets {
            let set_id = phase_set["id"].as_str().unwrap_or("unknown");
            
            if let Some(phases) = phase_set["phases"].as_array() {
                for phase in phases {
                    let phase_id = phase["id"].as_str().unwrap_or("unknown");
                    
                    // Check possibleActions
                    if let Some(possible_actions) = phase["possibleActions"].as_array() {
                        for (idx, action) in possible_actions.iter().enumerate() {
                            if let Some(action_id) = action.as_str() {
                                if !action_ids.contains(action_id) {
                                    errors.push(ValidationError::new(
                                        &bundle.game_id,
                                        "invalid_action_reference",
                                        format!("Action '{}' does not exist", action_id),
                                        format!("{}.{}.possibleActions[{}]", set_id, phase_id, idx),
                                    ));
                                }
                            }
                        }
                    }
                    
                    // Check enterActions (that aren't transitionToPhase)
                    if let Some(enter_actions) = phase["enterActions"].as_array() {
                        for (idx, action) in enter_actions.iter().enumerate() {
                            if let Some(action_id) = action.as_str() {
                                if !action_ids.contains(action_id) {
                                    errors.push(ValidationError::new(
                                        &bundle.game_id,
                                        "invalid_action_reference",
                                        format!("Action '{}' does not exist", action_id),
                                        format!("{}.{}.enterActions[{}]", set_id, phase_id, idx),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Check action triggers/then references
    if let Some(actions) = bundle.actions.as_array() {
        for action in actions {
            let action_id = action["id"].as_str().unwrap_or("unknown");
            
            // Check triggers
            if let Some(triggers) = action.get("triggers").or_else(|| action.get("then")) {
                if let Some(trigger_list) = triggers.as_array() {
                    for (idx, trigger) in trigger_list.iter().enumerate() {
                        let trigger_action = if let Some(action_str) = trigger.as_str() {
                            action_str
                        } else if let Some(action_obj) = trigger["action"].as_str() {
                            action_obj
                        } else {
                            continue;
                        };
                        
                        if !action_ids.contains(trigger_action) {
                            errors.push(ValidationError::new(
                                &bundle.game_id,
                                "invalid_action_reference",
                                format!("Triggered action '{}' does not exist", trigger_action),
                                format!("action.{}.triggers[{}]", action_id, idx),
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn validate_zone_references(bundle: &Bundle, zone_ids: &HashSet<String>, errors: &mut Vec<ValidationError>) {
    // Check zone references in actions
    if let Some(actions) = bundle.actions.as_array() {
        for action in actions {
            let action_id = action["id"].as_str().unwrap_or("unknown");
            
            // Skip validation for expanded actions (e.g., dealCards_p1_1)
            if action_id.contains("_p") && action_id.chars().filter(|&c| c == '_').count() >= 2 {
                continue;
            }
            
            // Check 'with' parameters for zone references
            if let Some(with) = action.get("with") {
                // Check source zone
                if let Some(source) = with["source"].as_str() {
                    // Skip validation if the zone contains any template variables (they'll be resolved at runtime)
                    if !source.contains("{") && !zone_ids.contains(source) {
                        errors.push(ValidationError::new(
                            &bundle.game_id,
                            "invalid_zone_reference",
                            format!("Zone '{}' does not exist", source),
                            format!("action.{}.with.source", action_id),
                        ));
                    }
                }
                
                // Check target zone
                if let Some(target) = with.get("target") {
                    let zone_name = if let Some(zone_str) = target.as_str() {
                        zone_str
                    } else if let Some(zone_obj) = target.get("zone").and_then(|z| z.as_str()) {
                        zone_obj
                    } else {
                        continue;
                    };
                    
                    // "any" is a special zone identifier for worker placement games
                    // Skip validation if the zone contains any template variables (they'll be resolved at runtime)
                    if zone_name != "any" && !zone_name.contains("{") && !zone_ids.contains(zone_name) {
                        errors.push(ValidationError::new(
                            &bundle.game_id,
                            "invalid_zone_reference",
                            format!("Zone '{}' does not exist", zone_name),
                            format!("action.{}.with.target", action_id),
                        ));
                    }
                }
            }
        }
    }
    
    // Check zone references in manifest zone groups
    if let Some(zone_groups) = &bundle.manifest.zone_groups {
        for group in zone_groups {
            for (idx, zone) in group.zones.iter().enumerate() {
                if !zone.contains("{player}") && !zone_ids.contains(zone) {
                    errors.push(ValidationError::new(
                        &bundle.game_id,
                        "invalid_zone_reference",
                        format!("Zone '{}' does not exist", zone),
                        format!("manifest.zoneGroups.{}.zones[{}]", group.id, idx),
                    ));
                }
            }
        }
    }
}

fn validate_entity_references(bundle: &Bundle, entity_ids: &HashSet<String>, _zone_ids: &HashSet<String>, errors: &mut Vec<ValidationError>) {
    // Check entity references in zone contents
    if let Some(zones) = bundle.zones.as_array() {
        for zone in zones {
            let zone_id = zone["id"].as_str().unwrap_or("unknown");
            
            if let Some(contents) = zone.get("contents") {
                if let Some(content_array) = contents.as_array() {
                    for (idx, entity) in content_array.iter().enumerate() {
                        if let Some(entity_id) = entity.as_str() {
                            if !entity_id.contains("{player}") && !entity_ids.contains(entity_id) {
                                errors.push(ValidationError::new(
                                    &bundle.game_id,
                                    "invalid_entity_reference",
                                    format!("Entity '{}' does not exist", entity_id),
                                    format!("zone.{}.contents[{}]", zone_id, idx),
                                ));
                            }
                        }
                    }
                } else if let Some(content_obj) = contents.as_object() {
                    // Check entity field in content objects
                    if let Some(entity_id) = content_obj.get("entity").and_then(|e| e.as_str()) {
                        if !entity_id.contains("{player}") && !entity_ids.contains(entity_id) {
                            errors.push(ValidationError::new(
                                &bundle.game_id,
                                "invalid_entity_reference",
                                format!("Entity '{}' does not exist", entity_id),
                                format!("zone.{}.contents.entity", zone_id),
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn validate_phase_transition_logic(bundle: &Bundle, _phase_ids: &HashMap<String, HashSet<String>>, errors: &mut Vec<ValidationError>) {
    // Build a map of phase enterActions
    let mut phase_transitions: HashMap<String, Vec<String>> = HashMap::new();
    let mut phase_actions: HashMap<String, Vec<String>> = HashMap::new();
    
    if let Some(phase_sets) = bundle.phases.as_array() {
        for phase_set in phase_sets {
            let set_id = phase_set["id"].as_str().unwrap_or("unknown");
            
            if let Some(phases) = phase_set["phases"].as_array() {
                for phase in phases {
                    let phase_id = phase["id"].as_str().unwrap_or("unknown");
                    let full_phase_id = format!("{}.{}", set_id, phase_id);
                    
                    // Collect transitions and actions from enterActions
                    if let Some(enter_actions) = phase["enterActions"].as_array() {
                        let mut transitions = Vec::new();
                        let mut actions = Vec::new();
                        
                        for action in enter_actions {
                            if let Some(target) = action.get("transitionToPhase").and_then(|t| t.as_str()) {
                                transitions.push(target.to_string());
                            } else if let Some(action_str) = action.as_str() {
                                actions.push(action_str.to_string());
                            }
                        }
                        
                        phase_transitions.insert(full_phase_id.clone(), transitions);
                        phase_actions.insert(full_phase_id, actions);
                    }
                }
            }
        }
    }
    
    // Check for multiple conflicting transitions in the same phase
    for (phase_id, transitions) in &phase_transitions {
        if transitions.len() > 1 {
            // Check if all transitions are to the same phase set
            let phase_sets: HashSet<&str> = transitions.iter()
                .filter_map(|t| t.split('.').next())
                .collect();
            
            if phase_sets.len() > 1 {
                errors.push(ValidationError::new(
                    &bundle.game_id,
                    "conflicting_phase_transitions",
                    format!("Phase '{}' has conflicting transitions to different phase sets: {:?}", phase_id, transitions),
                    format!("phase.{}.enterActions", phase_id),
                ));
            }
        }
    }
    
    // Check for potential infinite loops in phase transitions
    for (start_phase, _) in &phase_transitions {
        let mut visited = HashSet::new();
        let mut current = start_phase.clone();
        let mut path = vec![start_phase.clone()];
        
        loop {
            if visited.contains(&current) {
                // Found a loop
                errors.push(ValidationError::new(
                    &bundle.game_id,
                    "phase_transition_loop",
                    format!("Potential infinite loop detected in phase transitions: {} -> ... -> {}", 
                        path.join(" -> "), current),
                    format!("phase.{}.enterActions", start_phase),
                ));
                break;
            }
            
            visited.insert(current.clone());
            
            // Get the first transition from this phase (if any)
            if let Some(transitions) = phase_transitions.get(&current) {
                if let Some(first_transition) = transitions.first() {
                    // Check if this phase only has transitions (no other actions)
                    let has_only_transitions = phase_actions.get(&current)
                        .map(|actions| actions.is_empty())
                        .unwrap_or(true);
                    
                    if has_only_transitions && transitions.len() == 1 {
                        // This phase only transitions to another phase, continue checking
                        current = first_transition.clone();
                        path.push(current.clone());
                        
                        // Safety check: stop after 20 transitions to prevent infinite loops in validation
                        if path.len() > 20 {
                            errors.push(ValidationError::new(
                                &bundle.game_id,
                                "excessive_phase_transitions",
                                format!("Excessive phase transition chain detected starting from '{}'", start_phase),
                                format!("phase.{}.enterActions", start_phase),
                            ));
                            break;
                        }
                    } else {
                        // Phase has other actions or multiple transitions, stop checking
                        break;
                    }
                } else {
                    // No transitions from this phase
                    break;
                }
            } else {
                // Phase not found or has no transitions
                break;
            }
        }
    }
}

pub fn print_validation_errors(errors: &[ValidationError]) {
    if errors.is_empty() {
        return;
    }
    
    println!("\n⚠️  Validation Errors Found:");
    println!("══════════════════════════");
    
    let mut errors_by_game: HashMap<String, Vec<&ValidationError>> = HashMap::new();
    for error in errors {
        errors_by_game.entry(error.game_id.clone()).or_default().push(error);
    }
    
    for (game_id, game_errors) in errors_by_game {
        println!("\n📦 Game: {}", game_id);
        for error in game_errors {
            println!("  ❌ {}: {}", error.error_type, error.message);
            println!("     Location: {}", error.location);
        }
    }
    
    println!("\n{} total validation error(s) found.\n", errors.len());
}
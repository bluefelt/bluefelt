use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Validation result for phase definitions
#[derive(Debug, Clone)]
pub struct PhaseValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Phase transition graph for circular reference detection
#[derive(Debug, Clone)]
pub struct PhaseTransitionGraph {
    edges: HashMap<String, Vec<String>>,
    phase_sets: HashSet<String>,
}

impl PhaseTransitionGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            phase_sets: HashSet::new(),
        }
    }

    /// Add a phase transition edge
    pub fn add_transition(&mut self, from_phase: &str, to_phase: &str) {
        self.edges.entry(from_phase.to_string())
            .or_insert_with(Vec::new)
            .push(to_phase.to_string());
    }

    /// Register a phase set ID
    pub fn add_phase_set(&mut self, phase_set_id: &str) {
        self.phase_sets.insert(phase_set_id.to_string());
    }

    /// Detect circular references using depth-first search
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for node in self.edges.keys() {
            if !visited.contains(node) {
                self.dfs_detect_cycle(node, &mut visited, &mut recursion_stack, &mut cycles, Vec::new());
            }
        }

        cycles
    }

    fn dfs_detect_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
        mut path: Vec<String>,
    ) {
        visited.insert(node.to_string());
        recursion_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = self.edges.get(node) {
            for neighbor in neighbors {
                if recursion_stack.contains(neighbor) {
                    // Found a cycle - extract the cycle from the path
                    if let Some(cycle_start) = path.iter().position(|p| p == neighbor) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                } else if !visited.contains(neighbor) {
                    self.dfs_detect_cycle(neighbor, visited, recursion_stack, cycles, path.clone());
                }
            }
        }

        recursion_stack.remove(node);
    }
}

/// Validates phase definitions for safety and correctness
pub fn validate_phase_definitions(phases: &Value) -> PhaseValidationResult {
    let mut result = PhaseValidationResult {
        is_valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    let mut graph = PhaseTransitionGraph::new();
    let mut phase_set_ids = HashSet::new();
    let mut all_phases = HashMap::new(); // phase_set.phase_id -> phase_definition

    // First pass: collect all phases and validate structure
    if let Some(phase_sets) = phases.as_array() {
        for phase_set in phase_sets {
            if let Err(err) = validate_phase_set(phase_set, &mut result, &mut graph, &mut phase_set_ids, &mut all_phases) {
                result.errors.push(err);
                result.is_valid = false;
            }
        }
    } else {
        result.errors.push("Phases must be an array".to_string());
        result.is_valid = false;
        return result;
    }

    // Second pass: validate phase transitions and detect cycles
    validate_phase_transitions(&mut graph, &all_phases, &mut result);

    // Detect circular references
    let cycles = graph.detect_cycles();
    if !cycles.is_empty() {
        result.is_valid = false;
        for cycle in cycles {
            result.errors.push(format!("Circular phase transition detected: {}", cycle.join(" -> ")));
        }
    }

    result
}

fn validate_phase_set(
    phase_set: &Value,
    result: &mut PhaseValidationResult,
    graph: &mut PhaseTransitionGraph,
    phase_set_ids: &mut HashSet<String>,
    all_phases: &mut HashMap<String, Value>,
) -> Result<(), String> {
    // Validate phase set structure
    let phase_set_id = phase_set["id"].as_str()
        .ok_or("Phase set missing 'id' field")?;

    if phase_set_ids.contains(phase_set_id) {
        return Err(format!("Duplicate phase set ID: {}", phase_set_id));
    }
    phase_set_ids.insert(phase_set_id.to_string());
    graph.add_phase_set(phase_set_id);

    // Validate phases within the set
    let phases = phase_set["phases"].as_array()
        .ok_or("Phase set missing 'phases' array")?;

    let mut initial_phases = 0;
    let mut phase_ids = HashSet::new();

    for phase in phases {
        let phase_id = phase["id"].as_str()
            .ok_or(format!("Phase in set '{}' missing 'id' field", phase_set_id))?;

        if phase_ids.contains(phase_id) {
            return Err(format!("Duplicate phase ID '{}' in set '{}'", phase_id, phase_set_id));
        }
        phase_ids.insert(phase_id.to_string());

        let full_phase_id = format!("{}.{}", phase_set_id, phase_id);
        all_phases.insert(full_phase_id, phase.clone());

        // Check for initial phase
        if phase.get("initial").and_then(|v| v.as_bool()).unwrap_or(false) {
            initial_phases += 1;
        }

        // Validate phase structure
        validate_individual_phase(phase, phase_set_id, phase_id, result)?;
    }

    // Validate initial phase count
    match initial_phases {
        0 => result.warnings.push(format!("Phase set '{}' has no initial phase", phase_set_id)),
        1 => {}, // Good
        _ => return Err(format!("Phase set '{}' has multiple initial phases", phase_set_id)),
    }

    Ok(())
}

fn validate_individual_phase(
    phase: &Value,
    phase_set_id: &str,
    phase_id: &str,
    result: &mut PhaseValidationResult,
) -> Result<(), String> {
    // Validate enterActions if present
    if let Some(enter_actions) = phase.get("enterActions") {
        if let Some(actions_array) = enter_actions.as_array() {
            for (idx, action) in actions_array.iter().enumerate() {
                validate_enter_action(action, phase_set_id, phase_id, idx, result)?;
            }
        } else {
            return Err(format!("enterActions in phase '{}.{}' must be an array", phase_set_id, phase_id));
        }
    }

    // Validate possibleActions if present
    if let Some(possible_actions) = phase.get("possibleActions") {
        if !possible_actions.is_array() {
            result.warnings.push(format!("possibleActions in phase '{}.{}' should be an array", phase_set_id, phase_id));
        }
    }

    Ok(())
}

fn validate_enter_action(
    action: &Value,
    phase_set_id: &str,
    phase_id: &str,
    action_idx: usize,
    result: &mut PhaseValidationResult,
) -> Result<(), String> {
    if action.is_string() {
        // String action reference - will be validated later against action definitions
        return Ok(());
    }

    if let Some(transition_to) = action.get("transitionToPhase") {
        if let Some(target_phase) = transition_to.as_str() {
            validate_phase_transition_syntax(target_phase, phase_set_id, phase_id, result)?;
        } else {
            return Err(format!(
                "transitionToPhase in enterAction[{}] of phase '{}.{}' must be a string",
                action_idx, phase_set_id, phase_id
            ));
        }
    } else if action.is_object() {
        // Object action definition - validate it has either a verb or known action type
        let has_valid_action = action.get("uses").is_some() 
            || action.get("transitionToPhase").is_some()
            || action.get("action").is_some();

        if !has_valid_action {
            result.warnings.push(format!(
                "enterAction[{}] in phase '{}.{}' may be missing action definition",
                action_idx, phase_set_id, phase_id
            ));
        }
    } else {
        return Err(format!(
            "enterAction[{}] in phase '{}.{}' must be a string or object",
            action_idx, phase_set_id, phase_id
        ));
    }

    Ok(())
}

fn validate_phase_transition_syntax(
    target_phase: &str,
    source_phase_set: &str,
    source_phase: &str,
    result: &mut PhaseValidationResult,
) -> Result<(), String> {
    if !target_phase.contains('.') {
        return Err(format!(
            "Invalid phase transition '{}' in phase '{}.{}': must use format 'phaseSet.phaseId'",
            target_phase, source_phase_set, source_phase
        ));
    }

    let parts: Vec<&str> = target_phase.split('.').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid phase transition '{}' in phase '{}.{}': must have exactly one dot",
            target_phase, source_phase_set, source_phase
        ));
    }

    if parts[0].is_empty() || parts[1].is_empty() {
        return Err(format!(
            "Invalid phase transition '{}' in phase '{}.{}': phaseSet and phaseId cannot be empty",
            target_phase, source_phase_set, source_phase
        ));
    }

    Ok(())
}

fn validate_phase_transitions(
    graph: &mut PhaseTransitionGraph,
    all_phases: &HashMap<String, Value>,
    result: &mut PhaseValidationResult,
) {
    // Build transition graph and validate transitions reference valid phases
    for (phase_id, phase_def) in all_phases {
        if let Some(enter_actions) = phase_def.get("enterActions").and_then(|ea| ea.as_array()) {
            for action in enter_actions {
                if let Some(transition_to) = action.get("transitionToPhase").and_then(|tt| tt.as_str()) {
                    // Add edge to graph
                    graph.add_transition(phase_id, transition_to);

                    // Validate target phase exists
                    if !all_phases.contains_key(transition_to) {
                        result.errors.push(format!(
                            "Phase '{}' references non-existent target phase '{}'",
                            phase_id, transition_to
                        ));
                        result.is_valid = false;
                    }
                }
            }
        }
    }
}

/// Runtime validation for phase state
pub fn validate_phase_state(phase_state: &Value, phase_definitions: &Value) -> Result<(), String> {
    // Check if using enhanced phase system
    if phase_state.get("current").is_some() {
        // Enhanced phase system validation
        let current_phases = phase_state["current"].as_object()
            .ok_or("Enhanced phase state 'current' must be an object")?;
        
        // Validate that all active phases exist in definitions
        if let Some(phase_sets) = phase_definitions.as_array() {
            for (phase_set_id, current_phase_id) in current_phases {
                let current_phase_str = current_phase_id.as_str()
                    .ok_or(format!("Phase state for '{}' must be a string", phase_set_id))?;

                // Find the phase set
                let phase_set = phase_sets.iter()
                    .find(|ps| ps["id"].as_str() == Some(phase_set_id))
                    .ok_or(format!("Phase set '{}' not found in definitions", phase_set_id))?;

                // Find the phase within the set
                let phases = phase_set["phases"].as_array()
                    .ok_or(format!("Phase set '{}' missing phases array", phase_set_id))?;

                let _phase = phases.iter()
                    .find(|p| p["id"].as_str() == Some(current_phase_str))
                    .ok_or(format!("Phase '{}' not found in phase set '{}'", current_phase_str, phase_set_id))?;
            }
        }
    } else {
        // Legacy phase system validation
        let phase_states = phase_state.as_object()
            .ok_or("Phase state must be an object")?;

        // Validate that all active phases exist in definitions
        if let Some(phase_sets) = phase_definitions.as_array() {
            for (phase_set_id, current_phase_id) in phase_states {
                let current_phase_str = current_phase_id.as_str()
                    .ok_or(format!("Phase state for '{}' must be a string", phase_set_id))?;

                // Find the phase set
                let phase_set = phase_sets.iter()
                    .find(|ps| ps["id"].as_str() == Some(phase_set_id))
                    .ok_or(format!("Phase set '{}' not found in definitions", phase_set_id))?;

                // Find the phase within the set
                let phases = phase_set["phases"].as_array()
                    .ok_or(format!("Phase set '{}' missing phases array", phase_set_id))?;

                let _phase = phases.iter()
                    .find(|p| p["id"].as_str() == Some(current_phase_str))
                    .ok_or(format!("Phase '{}' not found in phase set '{}'", current_phase_str, phase_set_id))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_phase_definitions() {
        let phases = json!([
            {
                "id": "game",
                "phases": [
                    {
                        "id": "setup",
                        "initial": true,
                        "enterActions": ["dealCards"]
                    },
                    {
                        "id": "playing",
                        "possibleActions": ["playCard"]
                    }
                ]
            }
        ]);

        let result = validate_phase_definitions(&phases);
        assert!(result.is_valid, "Expected valid phase definitions, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_circular_reference_detection() {
        let phases = json!([
            {
                "id": "game",
                "phases": [
                    {
                        "id": "phase1",
                        "initial": true,
                        "enterActions": [
                            {"transitionToPhase": "game.phase2"}
                        ]
                    },
                    {
                        "id": "phase2",
                        "enterActions": [
                            {"transitionToPhase": "game.phase1"}
                        ]
                    }
                ]
            }
        ]);

        let result = validate_phase_definitions(&phases);
        assert!(!result.is_valid, "Expected to detect circular reference");
        assert!(!result.errors.is_empty(), "Expected error for circular reference");
    }

    #[test]
    fn test_duplicate_phase_set_ids() {
        let phases = json!([
            {
                "id": "game",
                "phases": [{"id": "phase1", "initial": true}]
            },
            {
                "id": "game", // Duplicate
                "phases": [{"id": "phase2", "initial": true}]
            }
        ]);

        let result = validate_phase_definitions(&phases);
        assert!(!result.is_valid, "Expected to detect duplicate phase set ID");
    }

    #[test]
    fn test_invalid_transition_syntax() {
        let phases = json!([
            {
                "id": "game",
                "phases": [
                    {
                        "id": "phase1",
                        "initial": true,
                        "enterActions": [
                            {"transitionToPhase": "invalid_syntax"} // Missing dot
                        ]
                    }
                ]
            }
        ]);

        let result = validate_phase_definitions(&phases);
        assert!(!result.is_valid, "Expected to detect invalid transition syntax");
    }

    #[test]
    fn test_transition_to_nonexistent_phase() {
        let phases = json!([
            {
                "id": "game", 
                "phases": [
                    {
                        "id": "phase1",
                        "initial": true,
                        "enterActions": [
                            {"transitionToPhase": "game.nonexistent"}
                        ]
                    }
                ]
            }
        ]);

        let result = validate_phase_definitions(&phases);
        assert!(!result.is_valid, "Expected to detect transition to nonexistent phase");
    }
}
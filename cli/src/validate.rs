//! Validation logic for game files

use anyhow::{Context, Result};
use colored::Colorize;
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

use crate::bundle::BundleManifest;
use crate::yaml_includes;
use crate::yaml_shortcuts;

/// Validation result
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub file: String,
    pub message: String,
    pub line: Option<usize>,
}

#[derive(Debug)]
pub struct ValidationWarning {
    pub file: String,
    pub message: String,
    pub line: Option<usize>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn print_summary(&self) {
        if !self.warnings.is_empty() {
            println!("\n{}", "Warnings:".yellow().bold());
            for warning in &self.warnings {
                println!("  {} {}: {}", 
                    "⚠".yellow(),
                    warning.file.yellow(),
                    warning.message
                );
            }
        }

        if !self.errors.is_empty() {
            println!("\n{}", "Errors:".red().bold());
            for error in &self.errors {
                println!("  {} {}: {}", 
                    "✗".red(),
                    error.file.red(),
                    error.message
                );
            }
        }

        if self.is_valid() {
            println!("\n{}", "✓ Validation passed".green().bold());
        } else {
            println!("\n{}", format!("✗ Validation failed with {} errors", self.errors.len()).red().bold());
        }
    }
}

/// Main validation entry point
pub async fn run(game_path: impl AsRef<Path>, spec_version: String) -> Result<()> {
    let result = validate_game(game_path).await?;
    result.print_summary();
    
    if !result.is_valid() {
        std::process::exit(1);
    }
    
    Ok(())
}

/// Validate a complete game
pub async fn validate_game(game_path: impl AsRef<Path>) -> Result<ValidationResult> {
    let game_path = game_path.as_ref();
    let mut result = ValidationResult::default();
    
    info!("Validating game at {}", game_path.display());

    // Check manifest exists
    let manifest_path = game_path.join("manifest.yaml");
    if !manifest_path.exists() {
        result.errors.push(ValidationError {
            file: "manifest.yaml".to_string(),
            message: "Required file not found".to_string(),
            line: None,
        });
        return Ok(result);
    }

    // Load and validate manifest
    let manifest = match load_and_validate_manifest(&manifest_path, &mut result) {
        Ok(m) => m,
        Err(_) => return Ok(result),
    };

    // Validate other files
    validate_entities(game_path, &mut result);
    validate_zones(game_path, &mut result);
    validate_actions(game_path, &mut result);
    validate_phases(game_path, &mut result);

    // Cross-file validation
    validate_references(game_path, &mut result);

    Ok(result)
}

/// Load and validate manifest
fn load_and_validate_manifest(
    path: &Path,
    result: &mut ValidationResult,
) -> Result<BundleManifest> {
    let content = fs::read_to_string(path)?;
    
    match serde_yaml::from_str::<BundleManifest>(&content) {
        Ok(manifest) => {
            // Validate spec version
            if manifest.spec_version != "1" {
                result.warnings.push(ValidationWarning {
                    file: "manifest.yaml".to_string(),
                    message: format!("Spec version {} may not be fully supported", manifest.spec_version),
                    line: None,
                });
            }

            // Validate game ID format
            if !manifest.game_id.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
                result.errors.push(ValidationError {
                    file: "manifest.yaml".to_string(),
                    message: "gameId must contain only lowercase letters and hyphens".to_string(),
                    line: None,
                });
            }

            Ok(manifest)
        }
        Err(e) => {
            result.errors.push(ValidationError {
                file: "manifest.yaml".to_string(),
                message: format!("Failed to parse: {}", e),
                line: None,
            });
            Err(e.into())
        }
    }
}

/// Validate entities file
fn validate_entities(game_path: &Path, result: &mut ValidationResult) {
    let path = game_path.join("entities.yaml");
    if !path.exists() {
        return;
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "entities.yaml".to_string(),
                message: format!("Failed to read: {}", e),
                line: None,
            });
            return;
        }
    };

    // Parse YAML with includes
    let mut yaml: Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "entities.yaml".to_string(),
                message: format!("Failed to parse: {}", e),
                line: None,
            });
            return;
        }
    };

    // Process includes
    let mut included_files = HashSet::new();
    yaml = match yaml_includes::process_includes(yaml, game_path, &mut included_files) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "entities.yaml".to_string(),
                message: format!("Failed to process includes: {}", e),
                line: None,
            });
            return;
        }
    };
    
    // Process shortcuts
    yaml = match crate::yaml_shortcuts::expand_shortcuts(yaml) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "entities.yaml".to_string(),
                message: format!("Failed to expand shortcuts: {}", e),
                line: None,
            });
            return;
        }
    };

    match yaml {
        Value::Sequence(entities) => {
            for (i, entity) in entities.iter().enumerate() {
                if let Value::Mapping(map) = entity {
                    // Check required fields
                    if !map.contains_key(&Value::String("id".to_string())) {
                        result.errors.push(ValidationError {
                            file: "entities.yaml".to_string(),
                            message: format!("Entity {} missing required field 'id'", i),
                            line: Some(i + 1),
                        });
                    }
                }
            }
        }
        _ => {
            result.errors.push(ValidationError {
                file: "entities.yaml".to_string(),
                message: "Root must be an array".to_string(),
                line: None,
            });
        }
    }
}

/// Validate zones file
fn validate_zones(game_path: &Path, result: &mut ValidationResult) {
    let path = game_path.join("zones.yaml");
    if !path.exists() {
        return;
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "zones.yaml".to_string(),
                message: format!("Failed to read: {}", e),
                line: None,
            });
            return;
        }
    };

    // Parse YAML
    let mut yaml: Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "zones.yaml".to_string(),
                message: format!("Failed to parse: {}", e),
                line: None,
            });
            return;
        }
    };

    // Process includes
    let mut included_files = HashSet::new();
    yaml = match yaml_includes::process_includes(yaml, game_path, &mut included_files) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "zones.yaml".to_string(),
                message: format!("Failed to process includes: {}", e),
                line: None,
            });
            return;
        }
    };
    
    // Process shortcuts
    yaml = match yaml_shortcuts::expand_shortcuts(yaml) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "zones.yaml".to_string(),
                message: format!("Failed to expand shortcuts: {}", e),
                line: None,
            });
            return;
        }
    };

    match yaml {
        Value::Sequence(zones) => {
            for (i, zone) in zones.iter().enumerate() {
                if let Value::Mapping(map) = zone {
                    // Check required fields
                    if !map.contains_key(&Value::String("id".to_string())) {
                        result.errors.push(ValidationError {
                            file: "zones.yaml".to_string(),
                            message: format!("Zone {} missing required field 'id'", i),
                            line: Some(i + 1),
                        });
                    }

                    // Validate zone type
                    if let Some(Value::String(zone_type)) = map.get(&Value::String("type".to_string())) {
                        match zone_type.as_str() {
                            "grid" => {
                                // Check grid-specific fields (shape or gridProps)
                                let has_shape = map.contains_key(&Value::String("shape".to_string()));
                                let has_grid_props = map.contains_key(&Value::String("gridProps".to_string()));
                                
                                if !has_shape && !has_grid_props {
                                    result.errors.push(ValidationError {
                                        file: "zones.yaml".to_string(),
                                        message: format!("Grid zone {} missing required field 'shape' or 'gridProps'", i),
                                        line: Some(i + 1),
                                    });
                                }
                            }
                            "deck" | "list" => {
                                // These are valid
                            }
                            _ => {
                                result.warnings.push(ValidationWarning {
                                    file: "zones.yaml".to_string(),
                                    message: format!("Unknown zone type '{}'", zone_type),
                                    line: Some(i + 1),
                                });
                            }
                        }
                    }
                }
            }
        }
        _ => {
            result.errors.push(ValidationError {
                file: "zones.yaml".to_string(),
                message: "Root must be an array".to_string(),
                line: None,
            });
        }
    }
}

/// Validate actions file
fn validate_actions(game_path: &Path, result: &mut ValidationResult) {
    let path = game_path.join("actions.yaml");
    if !path.exists() {
        return;
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "actions.yaml".to_string(),
                message: format!("Failed to read: {}", e),
                line: None,
            });
            return;
        }
    };

    // Parse YAML
    let mut yaml: Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "actions.yaml".to_string(),
                message: format!("Failed to parse: {}", e),
                line: None,
            });
            return;
        }
    };
    
    // Process includes
    let mut included_files = HashSet::new();
    yaml = match yaml_includes::process_includes(yaml, game_path, &mut included_files) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "actions.yaml".to_string(),
                message: format!("Failed to process includes: {}", e),
                line: None,
            });
            return;
        }
    };
    
    // Process shortcuts
    yaml = match yaml_shortcuts::expand_shortcuts(yaml) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "actions.yaml".to_string(),
                message: format!("Failed to expand shortcuts: {}", e),
                line: None,
            });
            return;
        }
    };
    
    match yaml {
        Value::Sequence(actions) => {
            for (i, action) in actions.iter().enumerate() {
                if let Value::Mapping(map) = action {
                    // Check required fields
                    if !map.contains_key(&Value::String("id".to_string())) {
                        result.errors.push(ValidationError {
                            file: "actions.yaml".to_string(),
                            message: format!("Action {} missing required field 'id'", i),
                            line: Some(i + 1),
                        });
                    }

                    // Check action type and validation
                    if let Some(Value::String(action_type)) = map.get(&Value::String("type".to_string())) {
                        if action_type == "multiStep" {
                            validate_multi_step_action(map, i, result);
                        }
                    } else {
                        // Check for implementation in single-step actions
                        let has_impl = map.contains_key(&Value::String("uses".to_string())) ||
                                      map.contains_key(&Value::String("hook".to_string())) ||
                                      map.contains_key(&Value::String("conditions".to_string()));
                        
                        if !has_impl {
                            result.warnings.push(ValidationWarning {
                                file: "actions.yaml".to_string(),
                                message: format!("Action {} has no implementation", i),
                                line: Some(i + 1),
                            });
                        }
                    }
                }
            }
        }
        _ => {
            result.errors.push(ValidationError {
                file: "actions.yaml".to_string(),
                message: "Root must be an array".to_string(),
                line: None,
            });
        }
    }
}

/// Validate phases file
fn validate_phases(game_path: &Path, result: &mut ValidationResult) {
    let path = game_path.join("phases.yaml");
    if !path.exists() {
        return;
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "phases.yaml".to_string(),
                message: format!("Failed to read: {}", e),
                line: None,
            });
            return;
        }
    };

    // Parse YAML
    let mut yaml_value = match serde_yaml::from_str::<Value>(&content) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "phases.yaml".to_string(),
                message: format!("Failed to parse YAML: {}", e),
                line: None,
            });
            return;
        }
    };

    // Process includes
    let mut included_files = HashSet::new();
    yaml_value = match yaml_includes::process_includes(yaml_value, game_path, &mut included_files) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "phases.yaml".to_string(),
                message: format!("Failed to process includes: {}", e),
                line: None,
            });
            return;
        }
    };

    // Process shortcuts
    yaml_value = match yaml_shortcuts::expand_shortcuts(yaml_value) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "phases.yaml".to_string(),
                message: format!("Failed to expand shortcuts: {}", e),
                line: None,
            });
            return;
        }
    };

    // Convert to JSON Value for the server's validation
    let json_value = match serde_json::to_value(&yaml_value) {
        Ok(v) => v,
        Err(e) => {
            result.errors.push(ValidationError {
                file: "phases.yaml".to_string(),
                message: format!("Failed to convert to JSON: {}", e),
                line: None,
            });
            return;
        }
    };

    // Use the comprehensive phase validation from the server
    let validation_result = bluefelt_core::engine::validate_phase_definitions(&json_value);
    
    // Convert server validation results to CLI format
    for error in validation_result.errors {
        result.errors.push(ValidationError {
            file: "phases.yaml".to_string(),
            message: error,
            line: None,
        });
    }
    
    for warning in validation_result.warnings {
        result.warnings.push(ValidationWarning {
            file: "phases.yaml".to_string(),
            message: warning,
            line: None,
        });
    }

    // Additional CLI-specific validation for new format
    match yaml_value {
        // New format: Array of phase sets
        Value::Sequence(phase_sets) => {
            for (i, phase_set) in phase_sets.iter().enumerate() {
                if let Value::Mapping(set_map) = phase_set {
                    // Basic structure validation
                    if !set_map.contains_key(&Value::String("id".to_string())) {
                        result.errors.push(ValidationError {
                            file: "phases.yaml".to_string(),
                            message: format!("Phase set {} missing required field 'id'", i),
                            line: Some(i + 1),
                        });
                    }
                    
                    // Check phases within the set
                    if let Some(Value::Sequence(phases)) = set_map.get(&Value::String("phases".to_string())) {
                        for (j, phase) in phases.iter().enumerate() {
                            if let Value::Mapping(phase_map) = phase {
                                if !phase_map.contains_key(&Value::String("id".to_string())) {
                                    result.errors.push(ValidationError {
                                        file: "phases.yaml".to_string(),
                                        message: format!("Phase {} in set missing required field 'id'", j),
                                        line: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        // Old format: Mapping of phase sets
        Value::Mapping(phase_sets) => {
            for (set_name, phases) in phase_sets {
                if let (Value::String(set_name_str), Value::Sequence(phases_arr)) = (set_name, phases) {
                    for (i, phase) in phases_arr.iter().enumerate() {
                        if let Value::Mapping(map) = phase {
                            // Check required fields
                            if !map.contains_key(&Value::String("id".to_string())) {
                                result.errors.push(ValidationError {
                                    file: "phases.yaml".to_string(),
                                    message: format!("Phase {} in set '{}' missing required field 'id'", i, set_name_str),
                                    line: None,
                                });
                            }
                        }
                    }
                }
            }
        }
        _ => {
            result.errors.push(ValidationError {
                file: "phases.yaml".to_string(),
                message: "Root must be an array or mapping of phase sets".to_string(),
                line: None,
            });
        }
    }
}

/// Validate a multi-step action
fn validate_multi_step_action(action_map: &serde_yaml::Mapping, action_index: usize, result: &mut ValidationResult) {
    // Check required fields for multi-step actions
    
    // Check stateStore field
    if let Some(Value::Sequence(state_store)) = action_map.get(&Value::String("stateStore".to_string())) {
        let stored_vars: HashSet<String> = state_store.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
        
        // Check steps field
        if let Some(Value::Sequence(steps)) = action_map.get(&Value::String("steps".to_string())) {
            let mut step_stored_vars = HashSet::new();
            
            for (step_idx, step) in steps.iter().enumerate() {
                if let Value::Mapping(step_map) = step {
                    // Check required step fields
                    if !step_map.contains_key(&Value::String("id".to_string())) {
                        result.errors.push(ValidationError {
                            file: "actions.yaml".to_string(),
                            message: format!("Multi-step action {} step {} missing required field 'id'", action_index, step_idx),
                            line: None,
                        });
                    }
                    
                    if !step_map.contains_key(&Value::String("as".to_string())) {
                        result.errors.push(ValidationError {
                            file: "actions.yaml".to_string(),
                            message: format!("Multi-step action {} step {} missing required field 'as'", action_index, step_idx),
                            line: None,
                        });
                    } else if let Some(Value::String(as_type)) = step_map.get(&Value::String("as".to_string())) {
                        // Validate that 'as' field starts with 'bf.'
                        if !as_type.starts_with("bf.") {
                            result.errors.push(ValidationError {
                                file: "actions.yaml".to_string(),
                                message: format!("Multi-step action {} step {} 'as' field must start with 'bf.'", action_index, step_idx),
                                line: None,
                            });
                        }
                    }
                    
                    if !step_map.contains_key(&Value::String("store".to_string())) {
                        result.errors.push(ValidationError {
                            file: "actions.yaml".to_string(),
                            message: format!("Multi-step action {} step {} missing required field 'store'", action_index, step_idx),
                            line: None,
                        });
                    } else if let Some(Value::String(store_var)) = step_map.get(&Value::String("store".to_string())) {
                        step_stored_vars.insert(store_var.clone());
                        
                        // Check that stored variable is declared in stateStore
                        if !stored_vars.contains(store_var) {
                            result.errors.push(ValidationError {
                                file: "actions.yaml".to_string(),
                                message: format!("Multi-step action {} step {} stores variable '{}' not declared in stateStore", action_index, step_idx, store_var),
                                line: None,
                            });
                        }
                    }
                }
            }
            
            // Check that all stateStore variables are actually stored by steps
            for declared_var in &stored_vars {
                if !step_stored_vars.contains(declared_var) {
                    result.errors.push(ValidationError {
                        file: "actions.yaml".to_string(),
                        message: format!("Multi-step action {} declares variable '{}' in stateStore but no step stores it", action_index, declared_var),
                        line: None,
                    });
                }
            }
        } else {
            result.errors.push(ValidationError {
                file: "actions.yaml".to_string(),
                message: format!("Multi-step action {} missing required field 'steps'", action_index),
                line: None,
            });
        }
        
        // Check result field
        if let Some(Value::Mapping(result_map)) = action_map.get(&Value::String("result".to_string())) {
            if !result_map.contains_key(&Value::String("as".to_string())) {
                result.errors.push(ValidationError {
                    file: "actions.yaml".to_string(),
                    message: format!("Multi-step action {} result missing required field 'as'", action_index),
                    line: None,
                });
            } else if let Some(Value::String(as_type)) = result_map.get(&Value::String("as".to_string())) {
                // Validate that 'as' field starts with 'bf.'
                if !as_type.starts_with("bf.") {
                    result.errors.push(ValidationError {
                        file: "actions.yaml".to_string(),
                        message: format!("Multi-step action {} result 'as' field must start with 'bf.'", action_index),
                        line: None,
                    });
                }
            }
            
            // Check for template variables in result
            if let Some(Value::Mapping(with_map)) = result_map.get(&Value::String("with".to_string())) {
                validate_template_variables(with_map, &stored_vars, action_index, "result", result);
            }
        } else {
            result.errors.push(ValidationError {
                file: "actions.yaml".to_string(),
                message: format!("Multi-step action {} missing required field 'result'", action_index),
                line: None,
            });
        }
    } else {
        result.errors.push(ValidationError {
            file: "actions.yaml".to_string(),
            message: format!("Multi-step action {} missing required field 'stateStore'", action_index),
            line: None,
        });
    }
}

/// Validate template variables in multi-step action parameters
fn validate_template_variables(
    params: &serde_yaml::Mapping,
    declared_vars: &HashSet<String>,
    action_index: usize,
    context: &str,
    result: &mut ValidationResult,
) {
    for (key, value) in params {
        if let Value::String(str_val) = value {
            // Simple pattern matching for {variable} templates
            let mut chars = str_val.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '{' {
                    let mut var_name = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == '}' {
                            chars.next(); // consume the '}'
                            
                            // Check if this variable is declared
                            if !declared_vars.contains(&var_name) {
                                result.errors.push(ValidationError {
                                    file: "actions.yaml".to_string(),
                                    message: format!("Multi-step action {} {} uses undeclared variable '{{{}}}'", action_index, context, var_name),
                                    line: None,
                                });
                            }
                            break;
                        } else {
                            var_name.push(chars.next().unwrap());
                        }
                    }
                }
            }
        }
    }
}

/// Validate cross-file references
fn validate_references(game_path: &Path, result: &mut ValidationResult) {
    // TODO: Implement cross-file reference validation
    // - Check that zone IDs referenced in actions exist
    // - Check that entity IDs referenced in zones exist
    // - Check that phase IDs referenced in actions exist
    debug!("Cross-file reference validation not yet implemented");
}
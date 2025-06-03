//! Validation logic for game files

use anyhow::{Context, Result};
use colored::Colorize;
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

use crate::bundle::BundleManifest;

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

    match serde_yaml::from_str::<Value>(&content) {
        Ok(Value::Sequence(entities)) => {
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
        Ok(_) => {
            result.errors.push(ValidationError {
                file: "entities.yaml".to_string(),
                message: "Root must be an array".to_string(),
                line: None,
            });
        }
        Err(e) => {
            result.errors.push(ValidationError {
                file: "entities.yaml".to_string(),
                message: format!("Failed to parse: {}", e),
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

    match serde_yaml::from_str::<Value>(&content) {
        Ok(Value::Sequence(zones)) => {
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
        Ok(_) => {
            result.errors.push(ValidationError {
                file: "zones.yaml".to_string(),
                message: "Root must be an array".to_string(),
                line: None,
            });
        }
        Err(e) => {
            result.errors.push(ValidationError {
                file: "zones.yaml".to_string(),
                message: format!("Failed to parse: {}", e),
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

    match serde_yaml::from_str::<Value>(&content) {
        Ok(Value::Sequence(actions)) => {
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

                    // Check for implementation
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
        Ok(_) => {
            result.errors.push(ValidationError {
                file: "actions.yaml".to_string(),
                message: "Root must be an array".to_string(),
                line: None,
            });
        }
        Err(e) => {
            result.errors.push(ValidationError {
                file: "actions.yaml".to_string(),
                message: format!("Failed to parse: {}", e),
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

    match serde_yaml::from_str::<Value>(&content) {
        // New format: Array of phase sets
        Ok(Value::Sequence(phase_sets)) => {
            for (i, phase_set) in phase_sets.iter().enumerate() {
                if let Value::Mapping(set_map) = phase_set {
                    // Check phase set has id
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
        Ok(Value::Mapping(phase_sets)) => {
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
        Ok(_) => {
            result.errors.push(ValidationError {
                file: "phases.yaml".to_string(),
                message: "Root must be an array or mapping of phase sets".to_string(),
                line: None,
            });
        }
        Err(e) => {
            result.errors.push(ValidationError {
                file: "phases.yaml".to_string(),
                message: format!("Failed to parse: {}", e),
                line: None,
            });
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
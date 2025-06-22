/**
 * Action validation module for the CLI
 * 
 * Validates that action names in YAML files are consistent and follow conventions.
 * This prevents client-server action name mismatches during the build process.
 */

use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};

pub struct ActionValidator {
    known_issues: HashMap<String, Vec<String>>,
}

impl ActionValidator {
    pub fn new() -> Self {
        let mut known_issues = HashMap::new();
        
        // Document known action name changes to prevent regressions
        known_issues.insert("tic-tac-toe".to_string(), vec![
            "Action 'placeMarker' was renamed to 'placeMark' - ensure tests use correct name".to_string()
        ]);
        
        Self { known_issues }
    }

    /// Validate action names across all games
    pub fn validate_all_games(&self, games_dir: &Path) -> Result<()> {
        let mut all_errors = Vec::new();
        let mut game_actions = HashMap::new();

        // Collect all games and their actions
        for entry in fs::read_dir(games_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let game_name = entry.file_name().to_string_lossy().to_string();
                
                if let Ok(actions) = self.get_game_actions(&entry.path()) {
                    game_actions.insert(game_name.clone(), actions);
                    
                    // Validate individual game
                    if let Err(errors) = self.validate_game(&game_name, &entry.path()) {
                        all_errors.extend(errors);
                    }
                }
            }
        }

        // Cross-game validation
        if let Err(errors) = self.validate_cross_game_consistency(&game_actions) {
            all_errors.extend(errors);
        }

        if !all_errors.is_empty() {
            return Err(anyhow!("Action validation failed:\n{}", all_errors.join("\n")));
        }

        println!("✅ All action names validated successfully");
        Ok(())
    }

    /// Validate actions for a specific game
    fn validate_game(&self, game_name: &str, game_path: &Path) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Find the latest version directory
        let mut version_dirs: Vec<_> = fs::read_dir(game_path)
            .map_err(|e| vec![format!("Cannot read game directory {}: {}", game_name, e)])?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().ok()?.is_dir() {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .collect();

        version_dirs.sort();
        let latest_version = version_dirs.last()
            .ok_or_else(|| vec![format!("No version directory found for game {}", game_name)])?;

        // Validate actions.yaml
        let actions_file = latest_version.join("actions.yaml");
        if actions_file.exists() {
            match self.validate_actions_file(&actions_file) {
                Ok(action_names) => {
                    // Check for known problematic patterns
                    if let Err(pattern_errors) = self.check_action_patterns(game_name, &action_names) {
                        errors.extend(pattern_errors);
                    }
                }
                Err(file_errors) => {
                    errors.extend(file_errors);
                }
            }
        }

        // Validate phases.yaml references match actions.yaml
        let phases_file = latest_version.join("phases.yaml");
        if phases_file.exists() && actions_file.exists() {
            if let Err(ref_errors) = self.validate_phase_action_references(&phases_file, &actions_file) {
                errors.extend(ref_errors);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate individual actions.yaml file
    fn validate_actions_file(&self, actions_file: &Path) -> Result<Vec<String>, Vec<String>> {
        let content = fs::read_to_string(actions_file)
            .map_err(|e| vec![format!("Cannot read actions file {:?}: {}", actions_file, e)])?;

        let actions: Vec<Value> = serde_yaml::from_str(&content)
            .map_err(|e| vec![format!("Invalid YAML in {:?}: {}", actions_file, e)])?;

        let mut action_names = Vec::new();
        let mut errors = Vec::new();

        for (index, action) in actions.iter().enumerate() {
            if let Some(action_obj) = action.as_mapping() {
                if let Some(id_value) = action_obj.get(&Value::String("id".to_string())) {
                    if let Some(id) = id_value.as_str() {
                        action_names.push(id.to_string());
                        
                        // Validate action name format
                        if let Err(format_errors) = self.validate_action_name_format(id) {
                            errors.extend(format_errors.into_iter().map(|e| {
                                format!("Action {} (index {}): {}", id, index, e)
                            }));
                        }
                    } else {
                        errors.push(format!("Action at index {} has non-string 'id' field", index));
                    }
                } else {
                    errors.push(format!("Action at index {} missing 'id' field", index));
                }
            } else {
                errors.push(format!("Action at index {} is not an object", index));
            }
        }

        if errors.is_empty() {
            Ok(action_names)
        } else {
            Err(errors)
        }
    }

    /// Validate action name follows conventions
    fn validate_action_name_format(&self, action_name: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check naming conventions
        if action_name.is_empty() {
            errors.push("Action name cannot be empty".to_string());
        }

        if action_name.contains(' ') {
            errors.push("Action name should not contain spaces".to_string());
        }

        if action_name.chars().next().unwrap_or('a').is_uppercase() {
            errors.push("Action name should start with lowercase letter (camelCase)".to_string());
        }

        // Check for known problematic names
        let problematic_names = [
            "placeMarker", // Should be placeMark
            "dropPiece",   // Should be dropChecker or similar
        ];

        if problematic_names.contains(&action_name) {
            errors.push(format!("Action name '{}' is known to cause client-server mismatches", action_name));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check for problematic action name patterns
    fn check_action_patterns(&self, game_name: &str, action_names: &[String]) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Tic Tac Toe specific checks
        if game_name == "tic-tac-toe" {
            if action_names.contains(&"placeMarker".to_string()) {
                errors.push("Tic Tac Toe should use 'placeMark' not 'placeMarker'".to_string());
            }
            
            if !action_names.contains(&"placeMark".to_string()) {
                errors.push("Tic Tac Toe should have 'placeMark' action".to_string());
            }
        }

        // Connect Four specific checks
        if game_name == "connect-four" {
            let valid_actions = ["dropChecker", "drop"];
            let has_valid_drop = action_names.iter().any(|name| valid_actions.contains(&name.as_str()));
            
            if !has_valid_drop {
                errors.push(format!("Connect Four should have a drop action: {:?}", valid_actions));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate that phases.yaml references match actions.yaml
    fn validate_phase_action_references(&self, phases_file: &Path, actions_file: &Path) -> Result<(), Vec<String>> {
        let phases_content = fs::read_to_string(phases_file)
            .map_err(|e| vec![format!("Cannot read phases file: {}", e)])?;
        let actions_content = fs::read_to_string(actions_file)
            .map_err(|e| vec![format!("Cannot read actions file: {}", e)])?;

        let phases: Vec<Value> = serde_yaml::from_str(&phases_content)
            .map_err(|e| vec![format!("Invalid YAML in phases file: {}", e)])?;
        let actions: Vec<Value> = serde_yaml::from_str(&actions_content)
            .map_err(|e| vec![format!("Invalid YAML in actions file: {}", e)])?;

        // Extract action names from actions.yaml
        let mut action_names = HashSet::new();
        for action in &actions {
            if let Some(action_obj) = action.as_mapping() {
                if let Some(id) = action_obj.get(&Value::String("id".to_string()))
                    .and_then(|v| v.as_str()) {
                    action_names.insert(id.to_string());
                }
            }
        }

        // Check phase references
        let mut errors = Vec::new();
        for (phase_index, phase) in phases.iter().enumerate() {
            if let Some(phase_obj) = phase.as_mapping() {
                if let Some(possible_actions) = phase_obj.get(&Value::String("possibleActions".to_string())) {
                    if let Some(actions_array) = possible_actions.as_sequence() {
                        for (action_index, action_ref) in actions_array.iter().enumerate() {
                            if let Some(action_name) = action_ref.as_str() {
                                if !action_names.contains(action_name) {
                                    errors.push(format!(
                                        "Phase {} references unknown action '{}' at index {}",
                                        phase_index, action_name, action_index
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate consistency across games
    fn validate_cross_game_consistency(&self, game_actions: &HashMap<String, Vec<String>>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for similar games with inconsistent action names
        let similar_games = [
            (vec!["tic-tac-toe", "hex-tic-tac-toe"], "Both should use consistent mark placement actions"),
        ];

        for (game_group, description) in similar_games {
            let mut action_sets = Vec::new();
            
            for game_name in &game_group {
                if let Some(actions) = game_actions.get(*game_name) {
                    action_sets.push((game_name, actions));
                }
            }

            if action_sets.len() > 1 {
                // Check if they have similar but inconsistent action names
                let all_actions: HashSet<String> = action_sets.iter()
                    .flat_map(|(_, actions)| actions.iter().cloned())
                    .collect();

                // Look for potential mismatches like placeMark vs placeMarker
                for action in &all_actions {
                    if action.ends_with("er") {
                        let base_name = &action[..action.len() - 2];
                        if all_actions.contains(base_name) {
                            errors.push(format!(
                                "Found potentially inconsistent actions '{}' and '{}' in similar games: {}",
                                action, base_name, description
                            ));
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get all action names for a game
    fn get_game_actions(&self, game_path: &Path) -> Result<Vec<String>> {
        let mut version_dirs: Vec<_> = fs::read_dir(game_path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().ok()?.is_dir() {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .collect();

        version_dirs.sort();
        let latest_version = version_dirs.last()
            .ok_or_else(|| anyhow!("No version directory found"))?;

        let actions_file = latest_version.join("actions.yaml");
        if !actions_file.exists() {
            return Ok(Vec::new());
        }

        match self.validate_actions_file(&actions_file) {
            Ok(actions) => Ok(actions),
            Err(_) => Ok(Vec::new()), // Return empty on parse errors
        }
    }
}

/// CLI command to validate action names
pub fn validate_action_names(games_dir: &Path) -> Result<()> {
    println!("🔍 Validating action names across all games...");
    
    let validator = ActionValidator::new();
    validator.validate_all_games(games_dir)?;
    
    Ok(())
}
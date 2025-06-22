//! YAML include support - allows splitting game definitions across multiple files

use anyhow::{Context, Result};
use serde_yaml::{Value as YamlValue, Mapping};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

/// Process YAML with include directives
pub fn process_includes(
    yaml: YamlValue,
    base_path: &Path,
    included_files: &mut HashSet<PathBuf>,
) -> Result<YamlValue> {
    match yaml {
        YamlValue::Mapping(map) => process_mapping_includes(map, base_path, included_files),
        YamlValue::Sequence(seq) => {
            Ok(YamlValue::Sequence(
                seq.into_iter()
                    .map(|v| process_includes(v, base_path, included_files))
                    .collect::<Result<Vec<_>>>()?
            ))
        }
        YamlValue::String(s) => {
            // Check for include directive
            if s.starts_with("!include ") {
                let file_path = s.trim_start_matches("!include ").trim();
                load_included_file(file_path, base_path, included_files)
            } else {
                Ok(YamlValue::String(s))
            }
        }
        _ => Ok(yaml),
    }
}

fn process_mapping_includes(
    map: Mapping,
    base_path: &Path,
    included_files: &mut HashSet<PathBuf>,
) -> Result<YamlValue> {
    let mut result = Mapping::new();
    
    for (key, value) in map.into_iter() {
        let key_str = key.as_str().unwrap_or("");
        
        // Special handling for include directives
        if key_str == "$include" {
            match value {
                YamlValue::String(file_path) => {
                    // Include a single file
                    let included = load_included_file(&file_path, base_path, included_files)?;
                    if let YamlValue::Mapping(included_map) = included {
                        // Merge the included mapping into the current one
                        for (k, v) in included_map {
                            result.insert(k, v);
                        }
                    }
                }
                YamlValue::Sequence(files) => {
                    // Include multiple files
                    for file in files {
                        if let Some(file_path) = file.as_str() {
                            let included = load_included_file(file_path, base_path, included_files)?;
                            if let YamlValue::Mapping(included_map) = included {
                                for (k, v) in included_map {
                                    result.insert(k, v);
                                }
                            }
                        }
                    }
                }
                _ => return Err(anyhow::anyhow!("$include must be a string or array of strings")),
            }
        } else {
            // Process the value recursively
            let processed_value = process_includes(value, base_path, included_files)?;
            result.insert(key, processed_value);
        }
    }
    
    Ok(YamlValue::Mapping(result))
}

fn load_included_file(
    file_path: &str,
    base_path: &Path,
    included_files: &mut HashSet<PathBuf>,
) -> Result<YamlValue> {
    let full_path = if Path::new(file_path).is_absolute() {
        PathBuf::from(file_path)
    } else {
        base_path.join(file_path)
    };
    
    // Check for circular includes
    if included_files.contains(&full_path) {
        return Err(anyhow::anyhow!(
            "Circular include detected: {}",
            full_path.display()
        ));
    }
    
    included_files.insert(full_path.clone());
    
    // Load the file
    let content = fs::read_to_string(&full_path)
        .with_context(|| format!("Failed to read included file: {}", full_path.display()))?;
    
    // Parse YAML
    let yaml: YamlValue = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML from: {}", full_path.display()))?;
    
    // Process includes recursively
    process_includes(yaml, full_path.parent().unwrap_or(base_path), included_files)
}

/// Example of how YAML includes work:
/// 
/// main.yaml:
/// ```yaml
/// manifest:
///   game_id: my-game
///   version: 1.0
/// 
/// $include:
///   - entities/cards.yaml
///   - entities/tokens.yaml
/// 
/// zones: !include zones/all.yaml
/// 
/// actions:
///   $include: actions/
/// ```
/// 
/// This would:
/// 1. Include and merge entities from two files
/// 2. Include zones from a single file
/// 3. Include all YAML files from the actions/ directory
pub fn process_directory_includes(
    yaml: &mut YamlValue,
    base_path: &Path,
) -> Result<()> {
    if let YamlValue::Mapping(map) = yaml {
        for (_key, value) in map.iter_mut() {
            if let YamlValue::Mapping(inner_map) = value {
                if let Some(YamlValue::String(dir_path)) = inner_map.get(&YamlValue::String("$include".to_string())) {
                    if dir_path.ends_with('/') {
                        // Include all YAML files from directory
                        let full_dir = base_path.join(dir_path);
                        let mut included_values = Vec::new();
                        
                        if full_dir.is_dir() {
                            for entry in fs::read_dir(&full_dir)? {
                                let entry = entry?;
                                let path = entry.path();
                                if path.extension().and_then(|s| s.to_str()) == Some("yaml") ||
                                   path.extension().and_then(|s| s.to_str()) == Some("yml") {
                                    let content = fs::read_to_string(&path)?;
                                    let yaml_content: YamlValue = serde_yaml::from_str(&content)?;
                                    included_values.push(yaml_content);
                                }
                            }
                        }
                        
                        // Replace the $include directive with the combined content
                        *value = YamlValue::Sequence(included_values);
                    }
                }
            }
        }
    }
    
    Ok(())
}
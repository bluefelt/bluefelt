use std::process::Command;
use std::fs;
use serde_json::Value;

fn main() {
    // Test that shortcuts are actually processed
    
    // 1. Create a simple test directory
    fs::create_dir_all("test_shortcuts_output").ok();
    
    // 2. Build the test game
    let output = Command::new("cargo")
        .args(&["run", "-q", "--", "build", "test_shortcut_game/1.0", "-o", "test_shortcuts_output"])
        .output()
        .expect("Failed to run build");
        
    if !output.status.success() {
        eprintln!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
        // Continue anyway to see what was produced
    }
    
    // 3. Check if output files were created
    let output_dirs: Vec<_> = fs::read_dir("test_shortcuts_output")
        .expect("Failed to read output dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
        
    if output_dirs.is_empty() {
        eprintln!("No output directories created!");
        return;
    }
    
    let bundle_dir = output_dirs[0].path();
    println!("Checking bundle at: {:?}", bundle_dir);
    
    // 4. Check the processed files
    
    // Check entities.json
    if let Ok(entities_json) = fs::read_to_string(bundle_dir.join("entities.json")) {
        let entities: Value = serde_json::from_str(&entities_json).unwrap();
        println!("\nEntities.json:");
        println!("{}", serde_json::to_string_pretty(&entities).unwrap());
        
        // Check if standard deck was expanded
        if let Some(arr) = entities.as_array() {
            println!("Number of entities: {}", arr.len());
            if arr.len() == 52 {
                println!("✓ Standard deck shortcut was expanded!");
            }
        }
    }
    
    // Check actions.json  
    if let Ok(actions_json) = fs::read_to_string(bundle_dir.join("actions.json")) {
        let actions: Value = serde_json::from_str(&actions_json).unwrap();
        println!("\nActions.json:");
        println!("{}", serde_json::to_string_pretty(&actions).unwrap());
        
        // Check if shortcuts were expanded
        if let Some(arr) = actions.as_array() {
            for action in arr {
                if let Some(id) = action.get("id").and_then(|v| v.as_str()) {
                    println!("\nChecking action: {}", id);
                    
                    // Check if 'if' was converted to 'when'
                    if action.get("when").is_some() && action.get("if").is_none() {
                        println!("  ✓ if/then/else shorthand was expanded");
                    }
                    
                    // Check if bf.* was expanded
                    if let Some(uses) = action.get("uses").and_then(|v| v.as_str()) {
                        if !uses.starts_with("bf.") {
                            println!("  ✓ Standard library action expanded to: {}", uses);
                        }
                    }
                    
                    // Check if log shortcut was expanded
                    if action.get("ui").is_some() && action.get("log").is_none() {
                        println!("  ✓ log shortcut was expanded");
                    }
                }
            }
        }
    }
    
    // Check zones.json
    if let Ok(zones_json) = fs::read_to_string(bundle_dir.join("zones.json")) {
        let zones: Value = serde_json::from_str(&zones_json).unwrap();
        println!("\nZones.json:");
        
        if let Some(arr) = zones.as_array() {
            for zone in arr {
                if let Some(id) = zone.get("id").and_then(|v| v.as_str()) {
                    if id == "board" {
                        if zone.get("type").and_then(|v| v.as_str()) == Some("board") {
                            if zone.get("rows").is_some() && zone.get("cols").is_some() {
                                println!("✓ Grid shortcut was expanded for board zone!");
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Check phases.json
    if let Ok(phases_json) = fs::read_to_string(bundle_dir.join("phases.json")) {
        let phases: Value = serde_json::from_str(&phases_json).unwrap();
        println!("\nPhases.json (first 500 chars):");
        let phases_str = serde_json::to_string_pretty(&phases).unwrap();
        println!("{}", &phases_str.chars().take(500).collect::<String>());
    }
}
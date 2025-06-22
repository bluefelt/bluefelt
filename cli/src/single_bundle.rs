//! Single bundle output support - creates one minified JSON file

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::Path;

/// Bundle structure for single file output
#[derive(serde::Serialize)]
pub struct SingleBundle {
    pub game_id: String,
    pub version: String,
    pub metadata: JsonValue,
    pub entities: Option<JsonValue>,
    pub zones: Option<JsonValue>,
    pub actions: Option<JsonValue>,
    pub phases: Option<JsonValue>,
    pub hooks: Option<String>, // Base64 encoded WASM
}

/// Write a single minified bundle file
pub fn write_single_bundle(
    output_path: &Path,
    game_id: &str,
    version: &str,
    manifest: &JsonValue,
    entities: Option<JsonValue>,
    zones: Option<JsonValue>,
    actions: Option<JsonValue>,
    phases: Option<JsonValue>,
    hooks: Option<Vec<u8>>,
) -> Result<()> {
    let bundle = SingleBundle {
        game_id: game_id.to_string(),
        version: version.to_string(),
        metadata: manifest["metadata"].clone(),
        entities,
        zones,
        actions,
        phases,
        hooks: hooks.map(|h| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, h)),
    };
    
    // Serialize to minified JSON
    let json = serde_json::to_string(&bundle)
        .context("Failed to serialize bundle")?;
    
    // Write to file
    fs::write(output_path, json)
        .with_context(|| format!("Failed to write bundle to {}", output_path.display()))?;
    
    // Print size information
    let size = fs::metadata(output_path)?.len();
    println!("Bundle size: {} bytes ({:.1} KB)", size, size as f64 / 1024.0);
    
    Ok(())
}
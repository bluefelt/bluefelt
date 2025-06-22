use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs};
use crate::shorthand::expand_game_definitions;
use crate::validation::{validate_bundle, print_validation_errors, ValidationError};

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayersRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
    pub name: String,
    pub author: String,
    pub players: PlayersRange,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZoneGroup {
    pub id: String,
    pub title: String,
    pub zones: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(rename = "gameId")]
    pub game_id: String,
    pub version: String,
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    pub metadata: ManifestMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phases: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "zoneGroups")]
    pub zone_groups: Option<Vec<ZoneGroup>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub game_id: String,
    pub manifest: Manifest,
    pub entities: Value,
    pub zones: Value,
    pub actions: Value,
    pub phases: Value,
}

impl Default for Bundle {
    fn default() -> Self {
        Self {
            game_id: String::new(),
            manifest: Manifest {
                game_id: String::new(),
                version: String::new(),
                spec_version: "1.0".to_string(),
                metadata: ManifestMetadata {
                    name: String::new(),
                    author: String::new(),
                    description: String::new(),
                    players: PlayersRange { min: 2, max: 4 },
                },
                phases: None,
                setup: None,
                zone_groups: None,
            },
            entities: serde_json::json!({}),
            zones: serde_json::json!({}),
            actions: serde_json::json!([]),
            phases: serde_json::json!({}),
        }
    }
}

#[derive(Clone)]
pub struct BundleMap {
    bundles: HashMap<String, Bundle>,
}

impl BundleMap {
    pub fn load_dir(path: &str) -> anyhow::Result<Self> {
        let mut bundles = HashMap::new();
        let mut all_validation_errors: Vec<ValidationError> = Vec::new();
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let game_id = entry.file_name().to_string_lossy().to_string();
            // find latest version directory
            let mut versions: Vec<String> = fs::read_dir(entry.path())?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().ok().map(|f| f.is_dir()).unwrap_or(false))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            versions.sort();
            if let Some(ver) = versions.last() {
                let base = entry.path().join(ver);
                let manifest_path = base.join("manifest.json");
                if manifest_path.exists() {
                    let manifest_contents = fs::read_to_string(&manifest_path)?;
                    let manifest: Manifest = serde_json::from_str(&manifest_contents)
                        .map_err(|e| anyhow::anyhow!("Failed to parse manifest.json for game '{}': {}", game_id, e))?;

                    let entities_path = base.join("entities.json");
                    let zones_path = base.join("zones.json");
                    let actions_path = base.join("actions.json");
                    let phases_path = base.join("phases.json");

                    let mut entities: Value = if entities_path.exists() {
                        serde_json::from_str(&fs::read_to_string(&entities_path)?)
                            .map_err(|e| anyhow::anyhow!("Failed to parse entities.json for game '{}': {}", game_id, e))?
                    } else { Value::Null };
                    let mut zones: Value = if zones_path.exists() {
                        serde_json::from_str(&fs::read_to_string(&zones_path)?)
                            .map_err(|e| anyhow::anyhow!("Failed to parse zones.json for game '{}': {}", game_id, e))?
                    } else { Value::Null };
                    let mut actions: Value = if actions_path.exists() {
                        serde_json::from_str(&fs::read_to_string(&actions_path)?)
                            .map_err(|e| anyhow::anyhow!("Failed to parse actions.json for game '{}': {}", game_id, e))?
                    } else { Value::Null };
                    let phases: Value = if phases_path.exists() {
                        serde_json::from_str(&fs::read_to_string(&phases_path)?)
                            .map_err(|e| anyhow::anyhow!("Failed to parse phases.json for game '{}': {}", game_id, e))?
                    } else { Value::Null };
                    
                    // Expand shorthand syntax
                    expand_game_definitions(&mut entities, &mut zones, &mut actions, &manifest);
                    

                    println!("Loading game: {} v{}", game_id, ver);
                    let bundle = Bundle { game_id: game_id.clone(), manifest, entities, zones, actions, phases };
                    
                    // Validate the bundle
                    let validation_errors = validate_bundle(&bundle);
                    all_validation_errors.extend(validation_errors);
                    
                    bundles.insert(game_id.clone(), bundle);
                }
            }
        }
        
        println!("Loaded {} games total", bundles.len());
        
        // Print validation errors if any
        print_validation_errors(&all_validation_errors);
        
        Ok(Self { bundles })
    }

    pub fn get_latest(&self, game_id: &str) -> Option<Bundle> {
        self.bundles.get(game_id).cloned()
    }

    pub fn list_games(&self) -> Vec<String> {
        self.bundles.keys().cloned().collect()
    }
    
    /// Create empty BundleMap for testing
    pub fn new_empty() -> Self {
        Self {
            bundles: HashMap::new(),
        }
    }
    
    /// Insert bundle for testing
    pub fn insert_bundle(&mut self, game_id: String, bundle: Bundle) {
        self.bundles.insert(game_id, bundle);
    }
}

/// Load bundles from directory  
pub fn load_bundles_from_dir(dir: &str) -> anyhow::Result<BundleMap> {
    BundleMap::load_dir(dir)
}

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs};
use crate::shorthand::expand_game_definitions;

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
}

#[derive(Clone)]
pub struct Bundle {
    pub game_id: String,
    pub manifest: Manifest,
    pub entities: Value,
    pub zones: Value,
    pub actions: Value,
    pub phases: Value,
    pub hooks: Option<Vec<u8>>, // WebAssembly module bytes
}

#[derive(Clone)]
pub struct BundleMap {
    bundles: HashMap<String, Bundle>,
}

impl BundleMap {
    pub fn load_dir(path: &str) -> anyhow::Result<Self> {
        let mut bundles = HashMap::new();
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
                let manifest_path = base.join("manifest.yaml");
                if manifest_path.exists() {
                    let manifest_contents = fs::read_to_string(&manifest_path)?;
                    let manifest: Manifest = serde_yaml::from_str(&manifest_contents)
                        .map_err(|e| anyhow::anyhow!("Failed to parse manifest for game '{}': {}", game_id, e))?;

                    let entities_path = base.join("entities.yaml");
                    let zones_path = base.join("zones.yaml");
                    let actions_path = base.join("actions.yaml");
                    let phases_path = base.join("phases.yaml");

                    let mut entities: Value = if entities_path.exists() {
                        serde_yaml::from_str(&fs::read_to_string(&entities_path)?)
                            .map_err(|e| anyhow::anyhow!("Failed to parse entities.yaml for game '{}': {}", game_id, e))?
                    } else { Value::Null };
                    let mut zones: Value = if zones_path.exists() {
                        serde_yaml::from_str(&fs::read_to_string(&zones_path)?)
                            .map_err(|e| anyhow::anyhow!("Failed to parse zones.yaml for game '{}': {}", game_id, e))?
                    } else { Value::Null };
                    let mut actions: Value = if actions_path.exists() {
                        serde_yaml::from_str(&fs::read_to_string(&actions_path)?)
                            .map_err(|e| anyhow::anyhow!("Failed to parse actions.yaml for game '{}': {}", game_id, e))?
                    } else { Value::Null };
                    let phases: Value = if phases_path.exists() {
                        serde_yaml::from_str(&fs::read_to_string(&phases_path)?)
                            .map_err(|e| anyhow::anyhow!("Failed to parse phases.yaml for game '{}': {}", game_id, e))?
                    } else { Value::Null };
                    
                    // Expand shorthand syntax
                    expand_game_definitions(&mut entities, &mut zones, &mut actions, &manifest);
                    
                    // Load hooks if available
                    let hooks_path = base.join("hooks.wasm");
                    let hooks = if hooks_path.exists() {
                        Some(fs::read(&hooks_path)?)
                    } else {
                        None
                    };

                    println!("Loading game: {} v{}", game_id, ver);
                    bundles.insert(
                        game_id.clone(),
                        Bundle { game_id: game_id.clone(), manifest, entities, zones, actions, phases, hooks },
                    );
                }
            }
        }
        println!("Loaded {} games total", bundles.len());
        Ok(Self { bundles })
    }

    pub fn get_latest(&self, game_id: &str) -> Option<Bundle> {
        self.bundles.get(game_id).cloned()
    }

    pub fn list_games(&self) -> Vec<String> {
        self.bundles.keys().cloned().collect()
    }
}

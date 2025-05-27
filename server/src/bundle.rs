use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs};

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
}

#[derive(Clone)]
pub struct Bundle {
    pub game_id: String,
    pub manifest: Manifest,
    pub entities: Value,
    pub zones: Value,
    pub verbs: Value,
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
                    let manifest: Manifest = serde_yaml::from_str(&manifest_contents)?;

                    let entities_path = base.join("entities.yaml");
                    let zones_path = base.join("zones.yaml");
                    let verbs_path = base.join("verbs.yaml");

                    let entities: Value = if entities_path.exists() {
                        serde_yaml::from_str(&fs::read_to_string(&entities_path)?)?
                    } else { Value::Null };
                    let zones: Value = if zones_path.exists() {
                        serde_yaml::from_str(&fs::read_to_string(&zones_path)?)?
                    } else { Value::Null };
                    let verbs: Value = if verbs_path.exists() {
                        serde_yaml::from_str(&fs::read_to_string(&verbs_path)?)?
                    } else { Value::Null };

                    bundles.insert(
                        game_id.clone(),
                        Bundle { game_id: game_id.clone(), manifest, entities, zones, verbs },
                    );
                }
            }
        }
        Ok(Self { bundles })
    }

    pub fn get_latest(&self, game_id: &str) -> Option<Bundle> {
        self.bundles.get(game_id).cloned()
    }

    pub fn list_games(&self) -> Vec<String> {
        self.bundles.keys().cloned().collect()
    }
}

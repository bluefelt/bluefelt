//! Bundle data structures and utilities

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Complete game bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub manifest: BundleManifest,
    pub entities: Option<Value>,
    pub zones: Option<Value>,
    pub actions: Option<Value>,
    pub phases: Option<Value>,
}

/// Game manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleManifest {
    pub game_id: String,
    pub version: String,
    pub spec_version: String,
    pub metadata: GameMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_groups: Option<Vec<ZoneGroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// Game metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    pub name: String,
    pub description: String,
    pub author: String,
    pub players: PlayersConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Players configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlayersConfig {
    Fixed(u32),
    Range { min: u32, max: u32 },
}

/// Zone group for UI organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneGroup {
    pub id: String,
    pub title: String,
    pub zones: Vec<String>,
}
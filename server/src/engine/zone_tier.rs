use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The four-tier spatial organization system for zones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZoneTier {
    /// Player-owned entities, private information (cards in hand, personal resources)
    Hand,
    
    /// Direct interaction spaces, immediate gameplay (game board, play areas)
    Tactical,
    
    /// Big picture view, game state overview (score tracks, resource summaries)
    Strategic,
    
    /// Meta-information, UI chrome (game log, chat, settings)
    Ambient,
}

impl ZoneTier {
    /// Get the rendering priority for this tier (lower number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            ZoneTier::Tactical => 1,  // Most important, center stage
            ZoneTier::Hand => 2,      // Player's primary interaction
            ZoneTier::Strategic => 3,  // Important but secondary
            ZoneTier::Ambient => 4,    // Least important, can be hidden
        }
    }
    
    /// Get default visibility behavior for this tier
    pub fn default_visibility(&self) -> &'static str {
        match self {
            ZoneTier::Hand => "owner",      // Usually private to player
            ZoneTier::Tactical => "public",  // Shared game space
            ZoneTier::Strategic => "public", // Game state info
            ZoneTier::Ambient => "public",   // Meta information
        }
    }
    
    /// Check if this tier typically contains interactive elements
    pub fn is_interactive(&self) -> bool {
        match self {
            ZoneTier::Hand | ZoneTier::Tactical => true,
            ZoneTier::Strategic | ZoneTier::Ambient => false,
        }
    }
    
    /// Get default position hint for this tier
    pub fn default_position(&self) -> &'static str {
        match self {
            ZoneTier::Hand => "bottom",      // Near player
            ZoneTier::Tactical => "center",   // Center of attention  
            ZoneTier::Strategic => "top",     // Overview position
            ZoneTier::Ambient => "sidebar",   // Out of the way
        }
    }
}

/// Layout hints for responsive zone positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutHints {
    /// Preferred position (top, bottom, left, right, center, sidebar)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    
    /// Display style (grid, fan, stack, list, track, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    
    /// Maximum width as percentage or pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<String>,
    
    /// Maximum height as percentage or pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<String>,
    
    /// Aspect ratio constraint (e.g., "1:1", "16:9")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    
    /// Whether this zone can be collapsed/hidden
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapsible: Option<bool>,
    
    /// Flex grow factor for responsive layouts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_grow: Option<f32>,
    
    /// Z-index for layering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i32>,
}

impl Default for LayoutHints {
    fn default() -> Self {
        Self {
            position: None,
            display: None,
            max_width: None,
            max_height: None,
            aspect_ratio: None,
            collapsible: None,
            flex_grow: None,
            z_index: None,
        }
    }
}

/// Infer zone tier from zone properties if not explicitly set
pub fn infer_zone_tier(zone: &Value) -> ZoneTier {
    // Check if tier is explicitly set
    if let Some(tier_str) = zone["tier"].as_str() {
        if let Ok(tier) = serde_json::from_value(Value::String(tier_str.to_string())) {
            return tier;
        }
    }
    
    // Infer from zone properties
    let shape = zone["shape"].as_str().unwrap_or("");
    let visibility = zone["visibility"].as_str().unwrap_or("");
    let view_type = zone["viewType"].as_str().unwrap_or("");
    let zone_id = zone["id"].as_str().unwrap_or("");
    
    // Hand tier heuristics
    if visibility == "owner" || zone_id.contains("hand") {
        return ZoneTier::Hand;
    }
    
    // Ambient tier heuristics  
    if view_type == "log" || zone_id.contains("log") || zone_id.contains("chat") {
        return ZoneTier::Ambient;
    }
    
    // Strategic tier heuristics
    if shape == "view" || view_type == "strategic" || zone_id.contains("score") || zone_id.contains("track") {
        return ZoneTier::Strategic;
    }
    
    // Default to tactical for game boards and play areas
    ZoneTier::Tactical
}

/// Generate default layout hints based on zone tier and properties
pub fn generate_layout_hints(tier: ZoneTier, zone: &Value) -> LayoutHints {
    let mut hints = LayoutHints::default();
    
    // Set position based on tier
    hints.position = Some(tier.default_position().to_string());
    
    // Set display style based on shape
    if let Some(shape) = zone["shape"].as_str() {
        hints.display = Some(match shape {
            "grid" => "grid",
            "hex" => "hex-grid", 
            "list" => "list",
            "stack" | "deck" => "stack",
            "view" => "info-panel",
            _ => "default",
        }.to_string());
    }
    
    // Set tier-specific defaults
    match tier {
        ZoneTier::Hand => {
            hints.max_width = Some("80%".to_string());
            hints.collapsible = Some(false);
            hints.z_index = Some(10);
        }
        ZoneTier::Tactical => {
            hints.aspect_ratio = Some("1:1".to_string());
            hints.flex_grow = Some(1.0);
            hints.z_index = Some(5);
        }
        ZoneTier::Strategic => {
            hints.max_height = Some("20%".to_string());
            hints.collapsible = Some(true);
            hints.z_index = Some(3);
        }
        ZoneTier::Ambient => {
            hints.max_width = Some("300px".to_string());
            hints.collapsible = Some(true);
            hints.z_index = Some(1);
        }
    }
    
    hints
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_zone_tier_serialization() {
        assert_eq!(serde_json::to_string(&ZoneTier::Hand).unwrap(), "\"hand\"");
        assert_eq!(serde_json::to_string(&ZoneTier::Tactical).unwrap(), "\"tactical\"");
        assert_eq!(serde_json::to_string(&ZoneTier::Strategic).unwrap(), "\"strategic\"");
        assert_eq!(serde_json::to_string(&ZoneTier::Ambient).unwrap(), "\"ambient\"");
    }
    
    #[test]
    fn test_zone_tier_inference() {
        let hand_zone = json!({
            "id": "player_hand",
            "visibility": "owner"
        });
        assert_eq!(infer_zone_tier(&hand_zone), ZoneTier::Hand);
        
        let board_zone = json!({
            "id": "board",
            "shape": "grid"
        });
        assert_eq!(infer_zone_tier(&board_zone), ZoneTier::Tactical);
        
        let score_zone = json!({
            "id": "score_track",
            "shape": "view",
            "viewType": "strategic"
        });
        assert_eq!(infer_zone_tier(&score_zone), ZoneTier::Strategic);
        
        let log_zone = json!({
            "id": "game_log",
            "viewType": "log"
        });
        assert_eq!(infer_zone_tier(&log_zone), ZoneTier::Ambient);
    }
}
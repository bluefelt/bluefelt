/// Hex Grid Zone Authority Module
/// 
/// This module extends the zone authority system with hex grid specific features,
/// providing server authority for hex grid rendering and interaction.

use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::engine::hex::{HexCoord, HexLayout};

/// Enhanced hex zone render data with server authority
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HexZoneRenderData {
    /// Map of hex coordinates to cell data
    pub cells: HashMap<String, HexCellData>,
    /// Hex layout (flat or pointy)
    pub layout: HexLayout,
    /// Shape metadata
    pub shape_meta: HexShapeMeta,
    /// Optional hex metadata for enhanced rendering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hex_metadata: Option<HexGridMetadata>,
}

/// Individual hex cell data with server authority
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HexCellData {
    /// The entity in this hex (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<Value>,
    /// Whether this hex is clickable
    pub clickable: bool,
    /// Whether this hex is highlighted
    pub highlighted: bool,
    /// Interaction hint for this hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_hint: Option<String>,
    /// Visual properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terrain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edges: Option<HashMap<u8, EdgeData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertices: Option<HashMap<u8, VertexData>>,
}

/// Hex shape metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HexShapeMeta {
    /// Shape type (hexagonal, rectangular, custom)
    pub shape_type: HexShapeType,
    /// For hexagonal shape
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<u32>,
    /// For rectangular shape
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols: Option<u32>,
    /// For custom shape
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_hexes: Option<Vec<String>>,
}

/// Hex shape type
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum HexShapeType {
    Hexagonal,
    Rectangular,
    Custom,
}

/// Hex grid metadata for enhanced rendering
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HexGridMetadata {
    /// Coordinate display system
    pub coordinate_system: HexCoordinateSystem,
    /// Whether to show coordinates
    pub show_coordinates: bool,
    /// Hex size in pixels
    pub hex_size: u32,
    /// Spacing between hexes
    pub hex_spacing: u32,
    /// Visual style options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_style: Option<HexVisualStyle>,
}

/// Hex coordinate display system
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum HexCoordinateSystem {
    Axial,      // q,r
    Offset,     // col,row
    Cube,       // x,y,z
    Alphanumeric, // A1, B2, etc.
}

/// Hex visual style options
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HexVisualStyle {
    pub hex_style: String, // "filled", "outline", "textured"
    pub border_width: u32,
    pub border_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terrain_colors: Option<HashMap<String, String>>,
}

/// Edge data for Catan-style games
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EdgeData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    pub clickable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_type: Option<String>, // "road", "rail", etc.
}

/// Vertex data for Catan-style games
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VertexData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    pub clickable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure_type: Option<String>, // "settlement", "city", etc.
}

/// Compute enhanced hex zone metadata for a specific player
pub fn compute_hex_zone_metadata(
    zone_id: &str,
    zone_data: &Value,
    zone_def: &Value,
    player_id: &str,
    player_actions: &serde_json::Map<String, Value>,
) -> HexZoneRenderData {
    // Extract layout
    let layout_str = zone_def.get("shapeMeta")
        .and_then(|sm| sm.get("layout"))
        .and_then(|l| l.as_str())
        .unwrap_or("flat");
    
    let layout = match layout_str {
        "pointy" => HexLayout::Pointy,
        _ => HexLayout::Flat,
    };
    
    // Extract shape metadata
    let shape_meta = extract_hex_shape_meta(zone_def);
    
    // Generate hex cells based on shape
    let mut cells = generate_hex_cells(&shape_meta, zone_data);
    
    // Enhance cells with action map data
    enhance_hex_cells_with_actions(zone_id, &mut cells, player_actions);
    
    // Extract optional hex metadata
    let hex_metadata = extract_hex_metadata(zone_def);
    
    HexZoneRenderData {
        cells,
        layout,
        shape_meta,
        hex_metadata,
    }
}

/// Extract hex shape metadata from zone definition
fn extract_hex_shape_meta(zone_def: &Value) -> HexShapeMeta {
    let shape_meta = zone_def.get("shapeMeta");
    
    // Check for hexagonal shape (radius-based)
    if let Some(radius) = shape_meta.and_then(|sm| sm.get("radius")).and_then(|r| r.as_u64()) {
        return HexShapeMeta {
            shape_type: HexShapeType::Hexagonal,
            radius: Some(radius as u32),
            rows: None,
            cols: None,
            custom_hexes: None,
        };
    }
    
    // Check for rectangular shape
    let rows = shape_meta.and_then(|sm| sm.get("rows")).and_then(|r| r.as_u64());
    let cols = shape_meta.and_then(|sm| sm.get("cols")).and_then(|c| c.as_u64());
    
    if let (Some(rows), Some(cols)) = (rows, cols) {
        return HexShapeMeta {
            shape_type: HexShapeType::Rectangular,
            radius: None,
            rows: Some(rows as u32),
            cols: Some(cols as u32),
            custom_hexes: None,
        };
    }
    
    // Check for custom shape
    if let Some(hexes) = shape_meta
        .and_then(|sm| sm.get("hexes"))
        .and_then(|h| h.as_array())
    {
        let custom_hexes = hexes.iter()
            .filter_map(|h| h.as_str().map(|s| s.to_string()))
            .collect();
        
        return HexShapeMeta {
            shape_type: HexShapeType::Custom,
            radius: None,
            rows: None,
            cols: None,
            custom_hexes: Some(custom_hexes),
        };
    }
    
    // Default to small hexagonal grid
    HexShapeMeta {
        shape_type: HexShapeType::Hexagonal,
        radius: Some(3),
        rows: None,
        cols: None,
        custom_hexes: None,
    }
}

/// Generate hex cells based on shape metadata
fn generate_hex_cells(
    shape_meta: &HexShapeMeta,
    zone_data: &Value,
) -> HashMap<String, HexCellData> {
    let mut cells = HashMap::new();
    
    // Get existing cell data from zone_data
    let existing_cells = zone_data.get("cells")
        .and_then(|c| c.as_object())
        .cloned()
        .unwrap_or_default();
    
    match shape_meta.shape_type {
        HexShapeType::Hexagonal => {
            let radius = shape_meta.radius.unwrap_or(3) as i32;
            
            // Generate hexagonal grid
            for q in -radius..=radius {
                let r1 = (-radius).max(-q - radius);
                let r2 = radius.min(-q + radius);
                
                for r in r1..=r2 {
                    let coord = HexCoord::new(q, r);
                    let coord_str = coord.to_string();
                    
                    let entity = existing_cells.get(&coord_str).cloned();
                    
                    cells.insert(coord_str, HexCellData {
                        entity,
                        clickable: false,
                        highlighted: false,
                        interaction_hint: None,
                        terrain: None,
                        edges: None,
                        vertices: None,
                    });
                }
            }
        }
        
        HexShapeType::Rectangular => {
            let rows = shape_meta.rows.unwrap_or(5) as i32;
            let cols = shape_meta.cols.unwrap_or(5) as i32;
            
            // Generate rectangular grid (using offset coordinates)
            for row in 0..rows {
                for col in 0..cols {
                    let offset_coord = crate::engine::hex::OffsetCoord { col, row };
                    let hex_coord = HexCoord::from_offset(offset_coord, HexLayout::Flat);
                    let coord_str = hex_coord.to_string();
                    
                    let entity = existing_cells.get(&coord_str).cloned();
                    
                    cells.insert(coord_str, HexCellData {
                        entity,
                        clickable: false,
                        highlighted: false,
                        interaction_hint: None,
                        terrain: None,
                        edges: None,
                        vertices: None,
                    });
                }
            }
        }
        
        HexShapeType::Custom => {
            if let Some(custom_hexes) = &shape_meta.custom_hexes {
                for hex_str in custom_hexes {
                    if let Ok(coord) = HexCoord::from_string(hex_str) {
                        let coord_str = coord.to_string();
                        let entity = existing_cells.get(&coord_str).cloned();
                        
                        cells.insert(coord_str, HexCellData {
                            entity,
                            clickable: false,
                            highlighted: false,
                            interaction_hint: None,
                            terrain: None,
                            edges: None,
                            vertices: None,
                        });
                    }
                }
            }
        }
    }
    
    cells
}

/// Enhance hex cells with action map data
fn enhance_hex_cells_with_actions(
    zone_id: &str,
    cells: &mut HashMap<String, HexCellData>,
    player_actions: &serde_json::Map<String, Value>,
) {
    let zone_prefix = format!("/zones/{}/", zone_id);
    
    for (location, action) in player_actions {
        if location.starts_with(&zone_prefix) {
            // Extract hex coordinate from location
            if let Some(hex_part) = location.strip_prefix(&zone_prefix) {
                // Handle different hex action formats
                if hex_part.starts_with("hexes/") {
                    // Format: /zones/board/hexes/1,2
                    if let Some(coord_str) = hex_part.strip_prefix("hexes/") {
                        if let Some(cell) = cells.get_mut(coord_str) {
                            cell.clickable = true;
                            
                            // Extract interaction hint from action
                            if let Some(direction) = action.get("direction").and_then(|d| d.as_str()) {
                                cell.interaction_hint = Some(direction.to_string());
                            }
                        }
                    }
                } else if hex_part.starts_with("edges/") {
                    // Format: /zones/board/edges/1,2/3 (hex coord/edge number)
                    // TODO: Implement edge actions
                } else if hex_part.starts_with("vertices/") {
                    // Format: /zones/board/vertices/1,2/4 (hex coord/vertex number)
                    // TODO: Implement vertex actions
                }
            }
        }
    }
}

/// Extract hex metadata from zone definition
fn extract_hex_metadata(zone_def: &Value) -> Option<HexGridMetadata> {
    let ui = zone_def.get("ui")?;
    
    let coordinate_system = ui.get("coordinateSystem")
        .and_then(|cs| cs.as_str())
        .map(|s| match s {
            "offset" => HexCoordinateSystem::Offset,
            "cube" => HexCoordinateSystem::Cube,
            "alphanumeric" => HexCoordinateSystem::Alphanumeric,
            _ => HexCoordinateSystem::Axial,
        })
        .unwrap_or(HexCoordinateSystem::Axial);
    
    let show_coordinates = ui.get("showCoordinates")
        .and_then(|sc| sc.as_bool())
        .unwrap_or(false);
    
    let hex_size = ui.get("hexSize")
        .and_then(|hs| hs.as_u64())
        .unwrap_or(40) as u32;
    
    let hex_spacing = ui.get("hexSpacing")
        .and_then(|hs| hs.as_u64())
        .unwrap_or(2) as u32;
    
    let visual_style = extract_hex_visual_style(ui);
    
    Some(HexGridMetadata {
        coordinate_system,
        show_coordinates,
        hex_size,
        hex_spacing,
        visual_style,
    })
}

/// Extract hex visual style from UI configuration
fn extract_hex_visual_style(ui: &Value) -> Option<HexVisualStyle> {
    let hex_style = ui.get("hexStyle")
        .and_then(|hs| hs.as_str())
        .unwrap_or("filled")
        .to_string();
    
    let border_width = ui.get("borderWidth")
        .and_then(|bw| bw.as_u64())
        .unwrap_or(2) as u32;
    
    let border_color = ui.get("borderColor")
        .and_then(|bc| bc.as_str())
        .unwrap_or("#333")
        .to_string();
    
    let terrain_colors = ui.get("terrainColors")
        .and_then(|tc| tc.as_object())
        .map(|tc| {
            tc.iter()
                .filter_map(|(k, v)| {
                    v.as_str().map(|s| (k.clone(), s.to_string()))
                })
                .collect()
        });
    
    Some(HexVisualStyle {
        hex_style,
        border_width,
        border_color,
        terrain_colors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_hexagonal_shape_extraction() {
        let zone_def = json!({
            "shapeMeta": {
                "layout": "flat",
                "radius": 5
            }
        });
        
        let shape_meta = extract_hex_shape_meta(&zone_def);
        
        assert!(matches!(shape_meta.shape_type, HexShapeType::Hexagonal));
        assert_eq!(shape_meta.radius, Some(5));
    }
    
    #[test]
    fn test_rectangular_shape_extraction() {
        let zone_def = json!({
            "shapeMeta": {
                "layout": "pointy",
                "rows": 10,
                "cols": 8
            }
        });
        
        let shape_meta = extract_hex_shape_meta(&zone_def);
        
        assert!(matches!(shape_meta.shape_type, HexShapeType::Rectangular));
        assert_eq!(shape_meta.rows, Some(10));
        assert_eq!(shape_meta.cols, Some(8));
    }
    
    #[test]
    fn test_hex_cell_generation() {
        let shape_meta = HexShapeMeta {
            shape_type: HexShapeType::Hexagonal,
            radius: Some(2),
            rows: None,
            cols: None,
            custom_hexes: None,
        };
        
        let zone_data = json!({});
        let cells = generate_hex_cells(&shape_meta, &zone_data);
        
        // Should generate 19 hexes for radius 2
        // Center (1) + first ring (6) + second ring (12) = 19
        assert_eq!(cells.len(), 19);
        
        // Check that origin exists
        assert!(cells.contains_key("0,0"));
        
        // Check some first ring hexes
        assert!(cells.contains_key("1,0"));
        assert!(cells.contains_key("0,1"));
        assert!(cells.contains_key("-1,1"));
    }
    
    #[test]
    fn test_action_map_enhancement() {
        let zone_id = "hex_board";
        let mut cells = HashMap::new();
        cells.insert("1,0".to_string(), HexCellData {
            entity: None,
            clickable: false,
            highlighted: false,
            interaction_hint: None,
            terrain: None,
            edges: None,
            vertices: None,
        });
        
        let mut player_actions = serde_json::Map::new();
        player_actions.insert(
            "/zones/hex_board/hexes/1,0".to_string(),
            json!({
                "direction": "Place a settlement",
                "actionId": "place_settlement"
            })
        );
        
        enhance_hex_cells_with_actions(zone_id, &mut cells, &player_actions);
        
        let cell = cells.get("1,0").unwrap();
        assert!(cell.clickable);
        assert_eq!(cell.interaction_hint, Some("Place a settlement".to_string()));
    }
}
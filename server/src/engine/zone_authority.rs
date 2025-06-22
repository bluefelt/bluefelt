/// Zone Rendering Authority Module
/// 
/// This module implements server authority for zone rendering decisions,
/// eliminating the need for clients to classify or process zone data.
/// 
/// Philosophy: Server computes all rendering metadata, client renders exactly
/// what the server provides.

use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::bundle::Bundle;
use crate::engine::zone_tier::{ZoneTier, LayoutHints, infer_zone_tier, generate_layout_hints};

/// Zone render type with server-computed rendering data
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "renderType")]
pub enum ZoneRenderData {
    #[serde(rename = "card")]
    Card {
        cards: Vec<CardRenderData>,
        layout: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        show_count: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        show_top: Option<bool>,
    },
    #[serde(rename = "grid")]
    Grid {
        cells: Vec<Vec<Value>>,
        rows: u32,
        cols: u32,
        // Server authority for coordinate system and layout
        #[serde(skip_serializing_if = "Option::is_none")]
        coordinate_system: Option<GridCoordinateSystem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        perspective: Option<GridPerspective>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cell_metadata: Option<Vec<Vec<CellMetadata>>>,
        // Performance optimization: sparse representation for mostly empty grids
        #[serde(skip_serializing_if = "Option::is_none")]
        sparse_cells: Option<HashMap<String, Value>>, // "row,col" -> cell data
        #[serde(skip_serializing_if = "Option::is_none")]
        use_sparse: Option<bool>,
    },
    #[serde(rename = "choice")]
    Choice {
        items: Vec<ChoiceItem>,
        prompt: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        multi_select: Option<bool>,
    },
    #[serde(rename = "hex")]
    Hex {
        cells: HashMap<String, Value>,
        layout: String, // "flat" | "pointy"
        radius: u32,
    },
    #[serde(rename = "view")]
    View {
        view_type: String,
        data: crate::engine::view_zones::ViewZoneData,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<crate::engine::view_zones::ViewFormat>,
    },
}

/// Card rendering data with server-computed visibility and interaction hints
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CardRenderData {
    pub entity: String,
    pub visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_hint: Option<String>, // "playable", "selectable", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight: Option<String>, // "selected", "available", etc.
}

/// Choice item with server-computed label and availability
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChoiceItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Grid coordinate system metadata with server authority
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GridCoordinateSystem {
    pub origin: GridOrigin,       // Where (0,0) is positioned
    pub numbering: GridNumbering, // How coordinates are numbered
    pub display_coords: bool,     // Whether to show coordinates to players
}

/// Grid origin positioning
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GridOrigin {
    #[serde(rename = "top_left")]
    TopLeft,
    #[serde(rename = "bottom_left")]
    BottomLeft,
    #[serde(rename = "center")]
    Center,
}

/// Grid coordinate numbering system
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GridNumbering {
    #[serde(rename = "zero_based")]
    ZeroBased,    // 0,1,2...
    #[serde(rename = "one_based")]
    OneBased,     // 1,2,3...
    #[serde(rename = "algebraic")]
    Algebraic,    // a1,b2,c3... (chess-style)
}

/// Grid perspective and rotation for player-specific view
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GridPerspective {
    pub rotation_for_player: Option<String>, // Which player should see rotated view
    pub rotate_degrees: u32,                 // Degrees to rotate (0, 90, 180, 270)
    pub flip_horizontal: bool,               // Mirror horizontally
    pub flip_vertical: bool,                 // Mirror vertically
}

/// Per-cell metadata with server authority
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CellMetadata {
    pub clickable: bool,
    pub highlighted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_hint: Option<String>, // "place_piece", "move_here", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_style: Option<String>,     // "dark_square", "light_square", etc.
    pub coordinates: CellCoordinates,
}

/// Cell coordinate information 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CellCoordinates {
    pub logical_row: u32,    // Logical row (game logic)
    pub logical_col: u32,    // Logical column (game logic) 
    pub display_row: u32,    // Display row (after perspective transforms)
    pub display_col: u32,    // Display column (after perspective transforms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_label: Option<String>, // Human-readable coordinate (e.g., "a1", "3,4")
}

/// Enhanced zone metadata with server authority
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ZoneMetadata {
    pub id: String,
    pub name: String,
    pub resolved_name: String, // Template-resolved name for current player
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>, // Explicit ownership
    pub visibility: ZoneVisibility,
    pub layout_order: u32, // Server-computed ordering
    pub tier: ZoneTier, // Spatial hierarchy tier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_hints: Option<LayoutHints>, // Responsive layout hints
    #[serde(flatten)]
    pub render_data: ZoneRenderData,
    // Performance optimization: version tracking for client-side caching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>, // Increment on zone changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<u64>, // Hash of zone content for change detection
}

/// Zone visibility with server authority
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ZoneVisibility {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "owner")]
    Owner,
    #[serde(rename = "hidden")]
    Hidden,
}

/// Find matching zone definition, handling template patterns
fn find_matching_zone_def<'a>(
    zone_id: &str,
    bundle_zones_map: &'a std::collections::HashMap<String, &'a Value>
) -> Option<&'a Value> {
    // Try exact match first
    if let Some(def) = bundle_zones_map.get(zone_id) {
        return Some(*def);
    }
    
    // Try template matches
    for (template_id, def) in bundle_zones_map {
        if template_id.contains("{player}") {
            // Check if zone_id matches the template pattern
            // e.g., "hand_{player}" should match "hand_p1", "hand_p2", etc.
            let base = template_id.replace("{player}", "");
            if zone_id.starts_with(&base) && 
               (zone_id.ends_with("p1") || zone_id.ends_with("p2") || 
                zone_id.ends_with("p3") || zone_id.ends_with("p4")) {
                return Some(*def);
            }
        }
    }
    
    None
}

/// Compute zone metadata with server authority for a specific player
pub fn compute_zone_metadata_for_player(
    game_state: &Value,
    bundle: &Bundle,
    player_id: &str,
    player_names: &[String],
) -> Vec<ZoneMetadata> {
    let mut metadata = Vec::new();
    let zones = game_state.get("zones").and_then(|z| z.as_object());
    
    // Pre-compute action map for choice zone population
    let action_map = crate::lobby::action_map::compute_action_map(game_state, bundle);
    let player_actions = action_map.get(player_id)
        .and_then(|pm| pm.as_object())
        .cloned()
        .unwrap_or_default();
    
    if let Some(zones) = zones {
        // Get bundle zone definitions - handle both array and object formats
        let mut layout_order = 0u32;
        
        if let Some(bundle_zones_array) = bundle.zones.as_array() {
            // Handle array format - convert to lookup map
            let mut bundle_zones_map = std::collections::HashMap::new();
            
            for zone_def in bundle_zones_array {
                if let Some(zone_id) = zone_def.get("id").and_then(|id| id.as_str()) {
                    bundle_zones_map.insert(zone_id.to_string(), zone_def);
                }
            }
            
            println!("[DEBUG zone_authority] Bundle has {} zone definitions", bundle_zones_map.len());
            println!("[DEBUG zone_authority] Game state has {} zones", zones.len());
            for (zone_id, zone_data) in zones {
                // Find matching zone definition (handles templates)
                let zone_def = find_matching_zone_def(zone_id, &bundle_zones_map);
                
                if let Some(zone_def) = zone_def {
                    let metadata_entry = compute_single_zone_metadata(
                        zone_id,
                        zone_data,
                        zone_def,
                        player_id,
                        player_names,
                        layout_order,
                        &player_actions,
                        game_state,
                        bundle,
                    );
                    
                    // Only include zones that should be visible to this player
                    let should_show = should_show_zone_to_player(&metadata_entry, player_id);
                    println!("[DEBUG zone_authority] Zone: {}, Owner: {:?}, Visibility: {:?}, Player: {}, Should Show: {}", 
                        zone_id, metadata_entry.owner, metadata_entry.visibility, player_id, should_show);
                    if should_show {
                        metadata.push(metadata_entry);
                        layout_order += 1;
                    }
                }
            }
        } else if let Some(bundle_zones) = bundle.zones.as_object() {
            // Handle object format (legacy)
            for (zone_id, zone_data) in zones {
                if let Some(zone_def) = bundle_zones.get(zone_id) {
                    let metadata_entry = compute_single_zone_metadata(
                        zone_id,
                        zone_data,
                        zone_def,
                        player_id,
                        player_names,
                        layout_order,
                        &player_actions,
                        game_state,
                        bundle,
                    );
                    
                    // Only include zones that should be visible to this player
                    let should_show = should_show_zone_to_player(&metadata_entry, player_id);
                    println!("[DEBUG zone_authority] Zone: {}, Owner: {:?}, Visibility: {:?}, Player: {}, Should Show: {}", 
                        zone_id, metadata_entry.owner, metadata_entry.visibility, player_id, should_show);
                    if should_show {
                        metadata.push(metadata_entry);
                        layout_order += 1;
                    }
                }
            }
        }
    }
    
    // Sort by layout order (server determines optimal order for this player)
    metadata.sort_by_key(|m| m.layout_order);
    metadata
}

/// Compute metadata for a single zone with server authority
fn compute_single_zone_metadata(
    zone_id: &str,
    zone_data: &Value,
    zone_def: &Value,
    player_id: &str,
    player_names: &[String],
    layout_order: u32,
    player_actions: &serde_json::Map<String, Value>,
    game_state: &Value,
    bundle: &Bundle,
) -> ZoneMetadata {
    // Resolve zone name templates
    let raw_name = zone_def.get("name")
        .and_then(|n| n.as_str())
        .unwrap_or(zone_id);
    let resolved_name = resolve_zone_name_for_player(raw_name, player_id, player_names);
    
    // Determine explicit ownership
    let owner = determine_zone_owner(zone_id, zone_def, player_id, player_names);
    
    // Determine visibility
    let visibility = determine_zone_visibility(zone_def, &owner, player_id);
    
    // Classify and process zone data with server authority
    let mut render_data = classify_zone_render_data(zone_id, zone_data, zone_def, player_id, player_actions, game_state, player_names, bundle);
    
    // Enhance grid zones with action map information
    if let ZoneRenderData::Grid { ref mut cell_metadata, .. } = render_data {
        if let Some(ref mut metadata) = cell_metadata {
            enhance_grid_metadata_with_actions(zone_id, metadata, player_actions);
        }
    }
    
    // Compute content hash for change detection (performance optimization)
    let content_hash = compute_zone_content_hash(zone_data);
    
    // Infer zone tier if not explicitly set
    let tier = infer_zone_tier(zone_def);
    
    // Generate layout hints based on tier and zone properties
    let layout_hints = Some(generate_layout_hints(tier, zone_def));
    
    ZoneMetadata {
        id: zone_id.to_string(),
        name: raw_name.to_string(),
        resolved_name,
        owner,
        visibility,
        layout_order,
        tier,
        layout_hints,
        render_data,
        version: None, // Will be set by caller if using versioning
        content_hash: Some(content_hash),
    }
}

/// Classify zone data into appropriate render type with server authority
fn classify_zone_render_data(
    zone_id: &str,
    zone_data: &Value,
    zone_def: &Value,
    player_id: &str,
    player_actions: &serde_json::Map<String, Value>,
    game_state: &Value,
    player_names: &[String],
    bundle: &Bundle,
) -> ZoneRenderData {
    // Check explicit type/shape from zone definition
    let zone_type = zone_def.get("type").and_then(|t| t.as_str());
    let zone_shape = zone_def.get("shape").and_then(|s| s.as_str());
    
    match (zone_type, zone_shape) {
        (Some("choice"), _) | (_, Some("choice")) => {
            classify_choice_zone(zone_id, zone_data, zone_def, player_actions)
        }
        (Some("grid"), _) | (_, Some("grid")) => {
            classify_grid_zone(zone_data, zone_def)
        }
        (Some("hexgrid"), _) | (_, Some("hexgrid")) => {
            classify_hex_zone(zone_id, zone_data, zone_def, player_id, player_actions)
        }
        (Some("view"), _) | (_, Some("view")) => {
            classify_view_zone(zone_id, zone_data, zone_def, game_state, player_names, bundle)
        }
        (Some("card"), _) | (Some("stack"), _) | (Some("deck"), _) | 
        (Some("list"), _) | (Some("single"), _) |
        (_, Some("stack")) | (_, Some("deck")) | (_, Some("list")) | (_, Some("single")) => {
            classify_card_zone(zone_id, zone_data, zone_def, player_id, game_state, player_actions)
        }
        _ => {
            // Use data structure heuristics as fallback, but with server authority
            classify_by_data_structure(zone_id, zone_data, zone_def, player_id, player_actions, game_state, player_names, bundle)
        }
    }
}

/// Classify choice zone with server-populated items
fn classify_choice_zone(
    zone_id: &str, 
    zone_data: &Value, 
    zone_def: &Value,
    player_actions: &serde_json::Map<String, Value>,
) -> ZoneRenderData {
    // Server populates choice items from action map - this is the key server authority improvement!
    let items = if let Some(items_array) = zone_data.get("items").and_then(|i| i.as_array()) {
        // If zone already has items (from presentChoice verb), use them
        items_array.iter().filter_map(|item| {
            if let Some(item_obj) = item.as_object() {
                Some(ChoiceItem {
                    id: item_obj.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string(),
                    label: item_obj.get("label").and_then(|l| l.as_str()).unwrap_or("").to_string(),
                    enabled: item_obj.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true),
                    description: item_obj.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
                })
            } else {
                None
            }
        }).collect()
    } else {
        // Populate from action map - server authority over choice options!
        populate_choice_items_from_actions(zone_id, player_actions)
    };
    
    let prompt = zone_data.get("prompt")
        .and_then(|p| p.as_str())
        .or_else(|| zone_def.get("prompt").and_then(|p| p.as_str()))
        .or_else(|| {
            // Extract prompt from first action if available
            if let Some(first_action) = player_actions.values().next() {
                first_action.get("direction").and_then(|d| d.as_str())
            } else {
                None
            }
        })
        .unwrap_or("Make a choice")
        .to_string();
    
    ZoneRenderData::Choice {
        items,
        prompt,
        multi_select: zone_def.get("multiSelect").and_then(|m| m.as_bool()),
    }
}

/// Populate choice items from action map with enhanced server authority
fn populate_choice_items_from_actions(
    zone_id: &str,
    player_actions: &serde_json::Map<String, Value>,
) -> Vec<ChoiceItem> {
    let mut items = Vec::new();
    let zone_prefix = format!("/zones/{}/", zone_id);
    
    // Performance optimization: collect items in bulk to reduce allocations
    let mut items_to_add: Vec<(String, String, Option<String>)> = Vec::with_capacity(player_actions.len());
    
    for (location, action) in player_actions {
        if location.starts_with(&zone_prefix) {
            // Extract choice ID from location path
            let choice_path = location.strip_prefix(&zone_prefix).unwrap_or("");
            
            // Handle different path formats with enhanced server authority
            let (choice_id, choice_label, enhanced_description) = if choice_path.contains("/") {
                // Structured path like "ranks/2" or "players/p1"
                let parts: Vec<&str> = choice_path.split('/').collect();
                if parts.len() >= 2 {
                    let category = parts[0];
                    let value = parts[1];
                    
                    let (label, description) = match category {
                        "ranks" => {
                            // Enhanced rank labeling with server authority
                            let label = match value.to_uppercase().as_str() {
                                "A" => "Ace".to_string(),
                                "J" => "Jack".to_string(),
                                "Q" => "Queen".to_string(),
                                "K" => "King".to_string(),
                                "2" => "Two".to_string(),
                                "3" => "Three".to_string(),
                                "4" => "Four".to_string(),
                                "5" => "Five".to_string(),
                                "6" => "Six".to_string(),
                                "7" => "Seven".to_string(),
                                "8" => "Eight".to_string(),
                                "9" => "Nine".to_string(),
                                "10" => "Ten".to_string(),
                                _ => value.to_string(),
                            };
                            let desc = format!("Ask for all {} cards", label.to_lowercase());
                            (label, Some(desc))
                        }
                        "players" => {
                            // Enhanced player labeling
                            let label = if value.starts_with('p') {
                                format!("Player {}", &value[1..])
                            } else {
                                value.to_string()
                            };
                            let desc = format!("Ask {} for cards", label);
                            (label, Some(desc))
                        }
                        "suits" => {
                            // Enhanced suit labeling
                            let label = match value.to_lowercase().as_str() {
                                "h" | "hearts" => "Hearts".to_string(),
                                "d" | "diamonds" => "Diamonds".to_string(),
                                "c" | "clubs" => "Clubs".to_string(),
                                "s" | "spades" => "Spades".to_string(),
                                _ => value.to_string(),
                            };
                            (label, None)
                        }
                        _ => (value.to_string(), None),
                    };
                    
                    (value.to_string(), label, description)
                } else {
                    (choice_path.to_string(), choice_path.to_string(), None)
                }
            } else {
                // Simple path like "2" or "p1" - enhanced with context
                let label = match choice_path.to_uppercase().as_str() {
                    "A" => "Ace".to_string(),
                    "J" => "Jack".to_string(),
                    "Q" => "Queen".to_string(),
                    "K" => "King".to_string(),
                    _ => choice_path.to_string(),
                };
                (choice_path.to_string(), label, None)
            };
            
            // Check if action is enabled (server authority over availability)
            let enabled = action.get("enabled")
                .and_then(|e| e.as_bool())
                .unwrap_or(true);
            
            // Get description from action or use enhanced description
            let description = action.get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string())
                .or(enhanced_description);
            
            items.push(ChoiceItem {
                id: choice_id,
                label: choice_label,
                enabled,
                description,
            });
        }
    }
    
    // Enhanced sorting with server authority over presentation
    items.sort_by(|a, b| {
        // Priority order: numbers first, then face cards, then special
        let a_priority = get_choice_sort_priority(&a.id);
        let b_priority = get_choice_sort_priority(&b.id);
        
        match a_priority.cmp(&b_priority) {
            std::cmp::Ordering::Equal => {
                // Within same priority, sort numerically or alphabetically
                match (a.id.parse::<u32>(), b.id.parse::<u32>()) {
                    (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
                    _ => a.id.cmp(&b.id),
                }
            }
            other => other,
        }
    });
    
    items
}

/// Get sorting priority for choice items with server authority
fn get_choice_sort_priority(choice_id: &str) -> u32 {
    match choice_id.to_uppercase().as_str() {
        // Numbers come first (1-10)
        "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" => 1,
        // Face cards next
        "J" => 2,
        "Q" => 3, 
        "K" => 4,
        "A" => 5, // Ace last in most games
        // Players
        _ if choice_id.starts_with('p') => 10,
        // Everything else
        _ => 100,
    }
}

/// Classify grid zone with server authority including coordinate system and perspective
fn classify_grid_zone(zone_data: &Value, zone_def: &Value) -> ZoneRenderData {
    let (cells_vec, rows, cols) = if let Some(cells) = zone_data.get("cells").and_then(|c| c.as_array()) {
        let rows = cells.len() as u32;
        let cols = cells.first()
            .and_then(|row| row.as_array())
            .map(|row| row.len() as u32)
            .unwrap_or(0);
        
        let cells_vec = cells.iter().map(|row| {
            row.as_array().unwrap_or(&vec![]).clone()
        }).collect();
        
        (cells_vec, rows, cols)
    } else if zone_data.is_array() {
        // Legacy format support
        let cells_array = zone_data.as_array().unwrap();
        let rows = cells_array.len() as u32;
        let cols = cells_array.first()
            .and_then(|row| row.as_array())
            .map(|row| row.len() as u32)
            .unwrap_or(0);
        
        let cells_vec = cells_array.iter().map(|row| {
            row.as_array().unwrap_or(&vec![]).clone()
        }).collect();
        
        (cells_vec, rows, cols)
    } else {
        // Default empty grid
        (vec![], 0, 0)
    };
    
    // Server authority for coordinate system
    let coordinate_system = extract_coordinate_system(zone_def);
    
    // Server authority for perspective/rotation
    let perspective = extract_grid_perspective(zone_def);
    
    // Generate per-cell metadata with server authority
    let cell_metadata = if rows > 0 && cols > 0 {
        Some(generate_cell_metadata(
            rows, 
            cols, 
            &coordinate_system, 
            &perspective
        ))
    } else {
        None
    };
    
    // Performance optimization: use sparse representation for large, mostly empty grids
    let (sparse_cells, use_sparse) = if should_use_sparse_grid(rows, cols, &cells_vec) {
        let sparse = convert_to_sparse_grid(&cells_vec);
        (Some(sparse), Some(true))
    } else {
        (None, None)
    };
    
    ZoneRenderData::Grid {
        cells: if use_sparse.unwrap_or(false) { vec![] } else { cells_vec },
        rows,
        cols,
        coordinate_system,
        perspective,
        cell_metadata,
        sparse_cells,
        use_sparse,
    }
}

/// Extract coordinate system configuration from zone definition
fn extract_coordinate_system(zone_def: &Value) -> Option<GridCoordinateSystem> {
    let ui = zone_def.get("ui")?;
    
    // Default coordinate system settings
    let origin = match ui.get("origin").and_then(|o| o.as_str()).unwrap_or("top_left") {
        "bottom_left" => GridOrigin::BottomLeft,
        "center" => GridOrigin::Center,
        _ => GridOrigin::TopLeft,
    };
    
    let numbering = match ui.get("numbering").and_then(|n| n.as_str()).unwrap_or("zero_based") {
        "one_based" => GridNumbering::OneBased,
        "algebraic" => GridNumbering::Algebraic,
        _ => GridNumbering::ZeroBased,
    };
    
    let display_coords = ui.get("displayCoords")
        .and_then(|d| d.as_bool())
        .unwrap_or(false);
    
    Some(GridCoordinateSystem {
        origin,
        numbering,
        display_coords,
    })
}

/// Extract grid perspective and rotation configuration
fn extract_grid_perspective(zone_def: &Value) -> Option<GridPerspective> {
    let ui = zone_def.get("ui")?;
    
    let rotation_for_player = ui.get("rotateForPlayer")
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());
    
    let rotate_degrees = ui.get("rotateDegrees")
        .and_then(|r| r.as_u64())
        .unwrap_or(0) as u32;
    
    let flip_horizontal = ui.get("flipHorizontal")
        .and_then(|f| f.as_bool())
        .unwrap_or(false);
    
    let flip_vertical = ui.get("flipVertical")
        .and_then(|f| f.as_bool())
        .unwrap_or(false);
    
    Some(GridPerspective {
        rotation_for_player,
        rotate_degrees,
        flip_horizontal,
        flip_vertical,
    })
}

/// Generate cell metadata with server authority including coordinates and styling
fn generate_cell_metadata(
    rows: u32,
    cols: u32,
    coordinate_system: &Option<GridCoordinateSystem>,
    perspective: &Option<GridPerspective>,
) -> Vec<Vec<CellMetadata>> {
    let mut metadata = Vec::new();
    
    for row in 0..rows {
        let mut row_metadata = Vec::new();
        
        for col in 0..cols {
            // Calculate display coordinates based on perspective transforms
            let (display_row, display_col) = apply_perspective_transform(
                row, col, rows, cols, perspective
            );
            
            // Generate display label based on coordinate system
            let display_label = generate_coordinate_label(
                row, col, coordinate_system
            );
            
            // Determine visual style (e.g., checkerboard pattern)
            let visual_style = if (row + col) % 2 == 1 {
                Some("dark_square".to_string())
            } else {
                Some("light_square".to_string())
            };
            
            let cell_meta = CellMetadata {
                clickable: false, // Will be updated by action map processing later
                highlighted: false, // Will be updated by selection state later
                interaction_hint: None, // Will be updated by action map processing
                visual_style,
                coordinates: CellCoordinates {
                    logical_row: row,
                    logical_col: col,
                    display_row,
                    display_col,
                    display_label,
                },
            };
            
            row_metadata.push(cell_meta);
        }
        
        metadata.push(row_metadata);
    }
    
    metadata
}

/// Apply perspective transforms to coordinates
fn apply_perspective_transform(
    logical_row: u32,
    logical_col: u32,
    total_rows: u32,
    total_cols: u32,
    perspective: &Option<GridPerspective>,
) -> (u32, u32) {
    if let Some(p) = perspective {
        let mut row = logical_row;
        let mut col = logical_col;
        
        // Apply rotation
        match p.rotate_degrees {
            90 => {
                let temp = row;
                row = col;
                col = total_rows - 1 - temp;
            }
            180 => {
                row = total_rows - 1 - row;
                col = total_cols - 1 - col;
            }
            270 => {
                let temp = row;
                row = total_cols - 1 - col;
                col = temp;
            }
            _ => {} // 0 degrees or invalid - no rotation
        }
        
        // Apply flips
        if p.flip_horizontal {
            col = total_cols - 1 - col;
        }
        if p.flip_vertical {
            row = total_rows - 1 - row;
        }
        
        (row, col)
    } else {
        (logical_row, logical_col)
    }
}

/// Generate human-readable coordinate label
fn generate_coordinate_label(
    row: u32,
    col: u32,
    coordinate_system: &Option<GridCoordinateSystem>,
) -> Option<String> {
    if let Some(cs) = coordinate_system {
        if cs.display_coords {
            match cs.numbering {
                GridNumbering::ZeroBased => Some(format!("{},{}", row, col)),
                GridNumbering::OneBased => Some(format!("{},{}", row + 1, col + 1)),
                GridNumbering::Algebraic => {
                    // Chess-style: a1, b2, etc.
                    let col_letter = ((b'a' + col as u8) as char).to_string();
                    let row_number = match cs.origin {
                        GridOrigin::BottomLeft => row + 1,
                        _ => row + 1, // Simplify for now
                    };
                    Some(format!("{}{}", col_letter, row_number))
                }
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// Enhance grid metadata with action map information (server authority)
fn enhance_grid_metadata_with_actions(
    zone_id: &str,
    cell_metadata: &mut [Vec<CellMetadata>],
    player_actions: &serde_json::Map<String, Value>,
) {
    // Check for column-based actions (like Connect 4 gravity-based games)
    let zone_prefix = format!("/zones/{}/", zone_id);
    let column_prefix = format!("/zones/{}/columns/", zone_id);
    let cell_prefix = format!("/zones/{}/cells/", zone_id);
    
    for (location, action) in player_actions {
        if location.starts_with(&cell_prefix) {
            // Parse cell location like "/zones/board/cells/2/1"
            if let Some(coords) = parse_cell_location(location, &cell_prefix) {
                let (row, col) = coords;
                if let Some(cell_meta) = cell_metadata
                    .get_mut(row)
                    .and_then(|row_meta| row_meta.get_mut(col))
                {
                    // Mark cell as clickable
                    cell_meta.clickable = true;
                    
                    // Set interaction hint based on action type
                    cell_meta.interaction_hint = determine_interaction_hint(action);
                }
            }
        } else if location.starts_with(&column_prefix) {
            // Handle column actions (for games like Connect 4)
            if let Some(col) = parse_column_location(location, &column_prefix) {
                // Mark entire column as having column action available
                for row_meta in cell_metadata.iter_mut() {
                    if let Some(cell_meta) = row_meta.get_mut(col) {
                        // Don't override cell-specific clickability, but note column action exists
                        if !cell_meta.clickable {
                            cell_meta.interaction_hint = Some("column_drop".to_string());
                        }
                    }
                }
            }
        } else if location.starts_with(&zone_prefix) {
            // Handle zone-level actions
            // For now, we don't mark individual cells for zone actions
        }
    }
}

/// Parse cell location from action map path
fn parse_cell_location(location: &str, cell_prefix: &str) -> Option<(usize, usize)> {
    if let Some(coords_part) = location.strip_prefix(cell_prefix) {
        let parts: Vec<&str> = coords_part.split('/').collect();
        if parts.len() >= 2 {
            if let (Ok(row), Ok(col)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                return Some((row, col));
            }
        }
    }
    None
}

/// Parse column location from action map path  
fn parse_column_location(location: &str, column_prefix: &str) -> Option<usize> {
    if let Some(col_part) = location.strip_prefix(column_prefix) {
        if let Ok(col) = col_part.parse::<usize>() {
            return Some(col);
        }
    }
    None
}

/// Determine interaction hint based on action type
fn determine_interaction_hint(action: &Value) -> Option<String> {
    if let Some(action_name) = action.get("action").and_then(|a| a.as_str()) {
        match action_name {
            "placePiece" | "place" => Some("place_piece".to_string()),
            "movePiece" | "move" => Some("move_piece".to_string()),
            "selectPiece" | "select" => Some("select_piece".to_string()),
            "dropColumn" | "dropPiece" => Some("drop_column".to_string()),
            _ => Some("clickable".to_string()),
        }
    } else {
        Some("clickable".to_string())
    }
}

/// Compute a hash of zone content for change detection (performance optimization)
fn compute_zone_content_hash(zone_data: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    
    // Hash the JSON string representation
    // This is a simple approach that works for our use case
    // A more sophisticated approach would hash specific fields
    if let Ok(json_str) = serde_json::to_string(zone_data) {
        json_str.hash(&mut hasher);
    }
    
    hasher.finish()
}

/// Determine if a grid should use sparse representation (performance optimization)
fn should_use_sparse_grid(rows: u32, cols: u32, cells: &[Vec<Value>]) -> bool {
    // Use sparse representation for grids larger than 10x10 with less than 30% occupancy
    if rows * cols < 100 {
        return false;
    }
    
    let mut non_empty_count = 0;
    let total_cells = rows * cols;
    
    for row in cells {
        for cell in row {
            if !cell.is_null() && cell != &json!({}) {
                non_empty_count += 1;
            }
        }
    }
    
    // Use sparse if less than 30% occupied
    (non_empty_count as f32) / (total_cells as f32) < 0.3
}

/// Convert grid to sparse representation (performance optimization)
fn convert_to_sparse_grid(cells: &[Vec<Value>]) -> HashMap<String, Value> {
    let mut sparse = HashMap::new();
    
    for (row_idx, row) in cells.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            if !cell.is_null() && cell != &json!({}) {
                let key = format!("{},{}", row_idx, col_idx);
                sparse.insert(key, cell.clone());
            }
        }
    }
    
    sparse
}

/// Classify view zone with computed data
fn classify_view_zone(
    zone_id: &str,
    _zone_data: &Value,
    zone_def: &Value,
    game_state: &Value,
    player_names: &[String],
    bundle: &Bundle,
) -> ZoneRenderData {
    let view_data = crate::engine::view_zones::compute_view_zone_data(
        zone_id,
        zone_def,
        game_state,
        player_names,
        bundle,
    );
    
    ZoneRenderData::View {
        view_type: view_data.view_type,
        data: view_data.data,
        format: view_data.format,
    }
}

/// Classify hex zone with server authority
fn classify_hex_zone(
    zone_id: &str,
    zone_data: &Value, 
    zone_def: &Value,
    player_id: &str,
    player_actions: &serde_json::Map<String, Value>,
) -> ZoneRenderData {
    // Use the enhanced hex zone authority module
    let hex_render_data = crate::engine::hex_zone_authority::compute_hex_zone_metadata(
        zone_id,
        zone_data,
        zone_def,
        player_id,
        player_actions,
    );
    
    // Convert to simplified format for now (can be enhanced later)
    let cells: HashMap<String, Value> = hex_render_data.cells.into_iter()
        .map(|(coord, cell_data)| {
            let mut cell_value = json!({});
            if let Some(entity) = cell_data.entity {
                cell_value = entity;
            }
            (coord, cell_value)
        })
        .collect();
    
    let layout = match hex_render_data.layout {
        crate::engine::hex::HexLayout::Flat => "flat",
        crate::engine::hex::HexLayout::Pointy => "pointy",
    }.to_string();
    
    let radius = hex_render_data.shape_meta.radius.unwrap_or(3);
    
    ZoneRenderData::Hex {
        cells,
        layout,
        radius,
    }
}

/// Classify card zone with server-computed visibility and enhanced metadata
fn classify_card_zone(
    zone_id: &str,
    zone_data: &Value,
    zone_def: &Value,
    player_id: &str,
    game_state: &Value,
    action_map: &serde_json::Map<String, Value>,
) -> ZoneRenderData {
    let cards = extract_cards_with_visibility(zone_data, zone_def, player_id, game_state, action_map);
    
    let layout = zone_def.get("ui")
        .and_then(|ui| ui.get("layout"))
        .and_then(|l| l.as_str())
        .unwrap_or(if zone_id.contains("hand") { "fan" } else { "spread" })
        .to_string();
    
    let show_count = zone_def.get("ui")
        .and_then(|ui| ui.get("showCount"))
        .and_then(|sc| sc.as_bool());
    
    let show_top = zone_def.get("ui")
        .and_then(|ui| ui.get("showTop"))
        .and_then(|st| st.as_bool());
    
    ZoneRenderData::Card {
        cards,
        layout,
        show_count,
        show_top,
    }
}

/// Classify by data structure as fallback
fn classify_by_data_structure(
    zone_id: &str,
    zone_data: &Value,
    zone_def: &Value,
    player_id: &str,
    player_actions: &serde_json::Map<String, Value>,
    game_state: &Value,
    player_names: &[String],
    bundle: &Bundle,
) -> ZoneRenderData {
    if zone_data.is_array() {
        let array = zone_data.as_array().unwrap();
        if array.is_empty() {
            // Empty array - default to card
            return classify_card_zone(zone_id, zone_data, zone_def, player_id, game_state, player_actions);
        }
        
        if array.first().unwrap().is_array() {
            // 2D array - grid
            classify_grid_zone(zone_data, zone_def)
        } else {
            // 1D array - card zone
            classify_card_zone(zone_id, zone_data, zone_def, player_id, game_state, player_actions)
        }
    } else if zone_data.is_object() {
        if zone_data.get("cells").is_some() {
            if zone_data.get("type").and_then(|t| t.as_str()) == Some("hexgrid") {
                classify_hex_zone(zone_id, zone_data, zone_def, player_id, player_actions)
            } else {
                classify_grid_zone(zone_data, zone_def)
            }
        } else if zone_data.get("items").is_some() {
            classify_choice_zone(zone_id, zone_data, zone_def, player_actions)
        } else {
            // Default to card
            classify_card_zone(zone_id, zone_data, zone_def, player_id, game_state, player_actions)
        }
    } else {
        // Default fallback
        classify_card_zone(zone_id, zone_data, zone_def, player_id, game_state, player_actions)
    }
}

/// Extract cards with server-computed visibility and enhanced metadata
fn extract_cards_with_visibility(
    zone_data: &Value,
    zone_def: &Value,
    player_id: &str,
    game_state: &Value,
    action_map: &serde_json::Map<String, Value>,
) -> Vec<CardRenderData> {
    let mut cards = Vec::new();
    
    // Extract card array from various formats
    let card_array = if let Some(items) = zone_data.get("items").and_then(|i| i.as_array()) {
        items
    } else if let Some(cards_arr) = zone_data.as_array() {
        cards_arr
    } else {
        return cards; // No cards
    };
    
    // Determine visibility rules from zone definition
    let visibility_rule = zone_def.get("visibility")
        .and_then(|v| v.as_str())
        .unwrap_or("all");
    
    // Check zone ownership for enhanced visibility computation
    let zone_owner = zone_def.get("owner")
        .and_then(|o| o.as_str())
        .unwrap_or("");
    
    for (index, card) in card_array.iter().enumerate() {
        if card.is_null() {
            continue; // Skip null/empty cards
        }
        
        let entity = if let Some(entity_str) = card.as_str() {
            entity_str.to_string()
        } else if let Some(entity_str) = card.get("entity").and_then(|e| e.as_str()) {
            entity_str.to_string()
        } else {
            continue; // Skip invalid cards
        };
        
        // Server computes visibility based on game rules and ownership
        let visible = compute_enhanced_card_visibility(
            visibility_rule, 
            index, 
            card_array.len(), 
            player_id,
            zone_owner,
            &entity
        );
        
        // Compute interaction hints based on entity and player context
        let interaction_hint = compute_card_interaction_hint(&entity, player_id, zone_owner, action_map);
        
        // Compute highlight status (selected, available, etc.)
        let highlight = compute_card_highlight_status(&entity, player_id, game_state, index, zone_data);
        
        cards.push(CardRenderData {
            entity,
            visible,
            interaction_hint,
            highlight,
        });
    }
    
    cards
}

/// Compute enhanced card visibility with server authority
fn compute_enhanced_card_visibility(
    visibility_rule: &str,
    card_index: usize,
    total_cards: usize,
    player_id: &str,
    zone_owner: &str,
    entity: &str,
) -> bool {
    match visibility_rule {
        "all" => true,
        "none" => false,
        "owner" => {
            // Enhanced ownership check - player must own the zone
            if zone_owner.is_empty() {
                true // Default to visible if no owner specified
            } else {
                // Check if player owns this zone
                zone_owner == player_id || 
                zone_owner == format!("p{}", extract_player_number(player_id))
            }
        },
        "top" => card_index == total_cards - 1,
        "faceDown" => {
            // Cards are present but not visible (show card backs)
            false
        },
        "topOnly" => {
            // Only top card is visible in stack
            card_index == total_cards - 1
        },
        _ => true, // Default visible
    }
}

/// Compute card interaction hints based on entity and context with action map integration
fn compute_card_interaction_hint(
    entity: &str,
    player_id: &str,
    zone_owner: &str,
    action_map: &serde_json::Map<String, Value>,
) -> Option<String> {
    // Check if this card has any actions available in the action map
    let empty_map = serde_json::Map::new();
    let player_actions = action_map.get(player_id)
        .and_then(|pm| pm.as_object())
        .unwrap_or(&empty_map);
    
    // Look for actions on this specific entity or zone
    let has_action = player_actions.iter().any(|(location, _action)| {
        location.contains(entity) || 
        location.contains(&format!("_{}", entity)) ||
        location.contains(&format!("/{}", entity))
    });
    
    if has_action {
        // Determine card ownership from entity naming patterns
        if entity.contains(&format!("_{}", player_id)) || 
           entity.contains(&format!("_p{}", extract_player_number(player_id))) {
            Some("owned_actionable".to_string())
        } else if zone_owner == player_id || 
                  zone_owner == format!("p{}", extract_player_number(player_id)) {
            Some("playable".to_string())
        } else {
            Some("actionable".to_string())
        }
    } else {
        // No actions available - provide context hint
        if entity.contains(&format!("_{}", player_id)) || 
           entity.contains(&format!("_p{}", extract_player_number(player_id))) {
            Some("owned".to_string())
        } else if zone_owner == player_id || 
                  zone_owner == format!("p{}", extract_player_number(player_id)) {
            Some("in_hand".to_string())
        } else if entity.starts_with("card_") {
            Some("visible".to_string())
        } else {
            None
        }
    }
}

/// Compute card highlight status with full game state integration
fn compute_card_highlight_status(
    entity: &str,
    player_id: &str,
    game_state: &Value,
    card_index: usize,
    zone_data: &Value,
) -> Option<String> {
    // Check for explicit entity highlight in game state
    if entity.contains("_selected") {
        return Some("selected".to_string());
    } else if entity.contains("_available") {
        return Some("available".to_string());
    }
    
    // Check selection state for this player
    let selection = game_state.get("selection")
        .or_else(|| game_state.get("game").and_then(|g| g.get("selection")));
    
    if let Some(selection_obj) = selection.and_then(|s| s.as_object()) {
        if let Some(player_selection) = selection_obj.get(player_id) {
            // Check if this card is selected
            if let Some(selected_entity) = player_selection.get("entity").and_then(|e| e.as_str()) {
                if selected_entity == entity {
                    return Some("selected".to_string());
                }
            }
            
            // Check by location pattern if entity-based selection not found
            if let Some(selected_location) = player_selection.get("location").and_then(|l| l.as_str()) {
                // Try to match location pattern with card index
                if selected_location.contains(&card_index.to_string()) {
                    return Some("selected".to_string());
                }
            }
        }
    }
    
    // Check for recently played or moved cards (game log context)
    if let Some(game_log) = game_state.get("gameLog").and_then(|log| log.as_array()) {
        if let Some(last_action) = game_log.last() {
            if let Some(action_text) = last_action.get("message").and_then(|m| m.as_str()) {
                if action_text.contains(entity) && action_text.contains(player_id) {
                    return Some("recently_played".to_string());
                }
            }
        }
    }
    
    // Check for special card states based on game context
    if let Some(meta) = game_state.get("meta") {
        // Check for last played card
        if let Some(last_played) = meta.get("lastPlayedCard").and_then(|lpc| lpc.as_str()) {
            if last_played == entity {
                return Some("last_played".to_string());
            }
        }
        
        // Check for trump suit or special cards
        if let Some(trump) = meta.get("trumpSuit").and_then(|ts| ts.as_str()) {
            if entity.contains(trump) {
                return Some("trump".to_string());
            }
        }
    }
    
    None
}

/// Extract player number from player ID (e.g., "p1" -> 1, "alice" -> 1 if first player)
fn extract_player_number(player_id: &str) -> usize {
    if player_id.starts_with('p') && player_id.len() > 1 {
        player_id[1..].parse().unwrap_or(1)
    } else {
        1 // Default for named players - would need player list to determine actual number
    }
}

/// Legacy function kept for compatibility
fn compute_card_visibility(
    visibility_rule: &str,
    card_index: usize,
    total_cards: usize,
    player_id: &str,
) -> bool {
    compute_enhanced_card_visibility(visibility_rule, card_index, total_cards, player_id, "", "")
}

/// Resolve zone name templates for current player
fn resolve_zone_name_for_player(
    name: &str,
    player_id: &str,
    player_names: &[String],
) -> String {
    let mut resolved = name.to_string();
    
    // Replace {p1}, {p2}, etc. with actual player names
    for (index, player_name) in player_names.iter().enumerate() {
        let placeholder = format!("{{p{}}}", index + 1);
        resolved = resolved.replace(&placeholder, player_name);
    }
    
    // Replace {player} with current player
    resolved = resolved.replace("{player}", player_id);
    
    resolved
}

/// Determine explicit zone ownership with enhanced pattern matching
fn determine_zone_owner(
    zone_id: &str,
    zone_def: &Value,
    player_id: &str,
    player_names: &[String],
) -> Option<String> {
    // Check explicit owner in definition
    if let Some(owner) = zone_def.get("owner").and_then(|o| o.as_str()) {
        return Some(owner.to_string());
    }
    
    // Enhanced pattern matching for zone ownership
    
    // 1. Check for _p{number} patterns (e.g., hand_p1, deck_p2)
    for (index, _) in player_names.iter().enumerate() {
        let player_id_formal = format!("p{}", index + 1);
        let patterns = vec![
            format!("_p{}", index + 1),
            format!("p{}_", index + 1),
            format!("_{}_", player_id_formal),
            format!("_{}$", player_id_formal), // End of string
        ];
        
        for pattern in patterns {
            if zone_id.contains(&pattern.replace("$", "")) {
                return Some(player_id_formal.clone());
            }
        }
    }
    
    // 2. Check for {player} templates resolved with actual names
    for (index, player_name) in player_names.iter().enumerate() {
        let patterns = vec![
            format!("_{}_", player_name),
            format!("_{}$", player_name),
            format!("{}_", player_name),
        ];
        
        for pattern in patterns {
            if zone_id.contains(&pattern.replace("$", "")) {
                return Some(format!("p{}", index + 1));
            }
        }
    }
    
    // 3. Check for common ownership patterns
    if zone_id.contains("hand") || zone_id.contains("deck") || 
       zone_id.contains("pile") || zone_id.contains("collection") {
        // These are typically player-owned zones
        // Try to extract player from the zone name
        for (index, _) in player_names.iter().enumerate() {
            let player_id_formal = format!("p{}", index + 1);
            if zone_id.contains(&player_id_formal) {
                return Some(player_id_formal);
            }
        }
    }
    
    // 4. Special zones that are typically shared/neutral
    if zone_id.contains("board") || zone_id.contains("table") || 
       zone_id.contains("discard") || zone_id.contains("draw") ||
       zone_id.contains("common") || zone_id.contains("shared") {
        return None; // Explicitly no owner (shared)
    }
    
    None
}

/// Determine zone visibility for current player
fn determine_zone_visibility(
    zone_def: &Value,
    owner: &Option<String>,
    player_id: &str,
) -> ZoneVisibility {
    let visibility_str = zone_def.get("visibility")
        .and_then(|v| v.as_str())
        .unwrap_or("all");
    
    match visibility_str {
        "hidden" => ZoneVisibility::Hidden,
        "owner" => ZoneVisibility::Owner,
        "public" => ZoneVisibility::All,
        "all" => ZoneVisibility::All,
        "count" => ZoneVisibility::All, // count means show zone but only card count
        _ => ZoneVisibility::All,
    }
}

/// Check if zone should be shown to player
fn should_show_zone_to_player(metadata: &ZoneMetadata, player_id: &str) -> bool {
    match metadata.visibility {
        ZoneVisibility::All => true,
        ZoneVisibility::Hidden => false,
        ZoneVisibility::Owner => {
            metadata.owner.as_ref().map_or(false, |owner| {
                // Check direct match
                owner == player_id ||
                // Check if owner is p1, p2 etc and player_id matches
                (owner.starts_with('p') && owner[1..].parse::<usize>().is_ok() && 
                 owner == &format!("p{}", extract_player_number(player_id)))
            })
        }
    }
}
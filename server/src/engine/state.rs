use crate::bundle::Bundle;
use crate::engine::enhanced_phases::initialize_enhanced_phases;
use serde_json::{json, Value, Map};
use rand::seq::SliceRandom;
use rand::RngCore;

const DEFAULT_GRID_SIZE: u64 = 3;
const INITIAL_PLAYER: &str = "p1";
const INITIAL_TICK: u64 = 0;
const INITIAL_TURN: u64 = 0;

pub fn load_initial_state(bundle: &Bundle) -> Value {
    load_initial_state_with_player_count(bundle, None)
}

pub fn load_initial_state_with_player_count(bundle: &Bundle, actual_player_count: Option<u32>) -> Value {
    // Use thread-local RNG for backward compatibility
    let mut rng = rand::rng();
    load_initial_state_with_rng(bundle, actual_player_count, &mut rng)
}

pub fn load_initial_state_with_rng<R: RngCore>(bundle: &Bundle, actual_player_count: Option<u32>, rng: &mut R) -> Value {
    let player_count = actual_player_count.unwrap_or(bundle.manifest.metadata.players.max);
    let players = create_players(player_count);
    let zones = create_zones_with_rng(&players, &bundle.zones, rng);
    let phase_states = initialize_phase_states(&bundle.phases);

    let initial_state = json!({
        "zones": zones,
        "players": players,
        "tick": INITIAL_TICK,
        "turn": INITIAL_TURN,
        "currentPlayer": INITIAL_PLAYER,
        "gameStatus": {
            "state": "playing",
            "winner": null,
            "tie": false
        },
        "phases": phase_states,
        "selection": {}  // Initialize empty selection object for temporary game state
    });
    
    initial_state
}

pub fn load_initial_state_with_player_names<R: RngCore>(bundle: &Bundle, player_names: &[String], rng: &mut R) -> Value {
    let player_count = player_names.len() as u32;
    let players = create_players_with_names(player_names);
    let zones = create_zones_with_rng(&players, &bundle.zones, rng);
    let phase_states = initialize_phase_states(&bundle.phases);

    let initial_state = json!({
        "zones": zones,
        "players": players,
        "tick": INITIAL_TICK,
        "turn": INITIAL_TURN,
        "currentPlayer": INITIAL_PLAYER,
        "gameStatus": {
            "state": "playing",
            "winner": null,
            "tie": false
        },
        "phases": phase_states,
        "selection": {}  // Initialize empty selection object for temporary game state
    });
    
    initial_state
}

fn create_players(player_count: u32) -> Vec<Value> {
    (1..=player_count)
        .map(|i| json!({"id": format!("p{}", i)}))
        .collect()
}

fn create_players_with_names(player_names: &[String]) -> Vec<Value> {
    player_names.iter()
        .enumerate()
        .map(|(i, name)| json!({
            "id": format!("p{}", i + 1),
            "name": name
        }))
        .collect()
}

fn create_zones(players: &[Value], zones_def: &Value) -> Map<String, Value> {
    // Use thread-local RNG for backward compatibility
    let mut rng = rand::rng();
    create_zones_with_rng(players, zones_def, &mut rng)
}

fn create_zones_with_rng<R: RngCore>(players: &[Value], zones_def: &Value, rng: &mut R) -> Map<String, Value> {
    let mut zones = Map::new();
    
    if let Some(arr) = zones_def.as_array() {
        for zone in arr {
            process_zone_definition_with_rng(zone, players, &mut zones, rng);
        }
    }
    
    zones
}

fn process_zone_definition(zone: &Value, players: &[Value], zones: &mut Map<String, Value>) {
    // Use thread-local RNG for backward compatibility
    let mut rng = rand::rng();
    process_zone_definition_with_rng(zone, players, zones, &mut rng);
}

fn process_zone_definition_with_rng<R: RngCore>(zone: &Value, players: &[Value], zones: &mut Map<String, Value>, rng: &mut R) {
    let id = zone["id"].as_str().unwrap_or("");
    let zone_type = zone["type"].as_str()
        .or_else(|| zone["shape"].as_str()) // Support old 'shape' field
        .unwrap_or("");
    let contents = zone.get("contents")
        .cloned()
        .unwrap_or_else(|| Value::String("empty".to_string()));
    
    let per_player = id.contains("{player}");
    
    // Check if contents contain {player} templates
    let contents_have_player = check_contents_for_player_template(&contents);
    
    if per_player {
        // Zone ID has {player}, expand for each player
        let ids = expand_player_zones(id, players);
        for pid in ids {
            println!("  Creating zone: {}", pid);
            create_single_zone_with_rng(&pid, zone, zone_type, &contents, true, zones, rng);
        }
    } else if contents_have_player {
        // Zone ID doesn't have {player}, but contents do
        // Create single zone with expanded contents for all players
        println!("  Creating zone: {}", id);
        let expanded_contents = expand_contents_for_all_players(&contents, players);
        create_single_zone_with_rng(id, zone, zone_type, &expanded_contents, false, zones, rng);
    } else {
        // No player templates, create normally
        println!("  Creating zone: {}", id);
        create_single_zone_with_rng(id, zone, zone_type, &contents, false, zones, rng);
    }
}

fn expand_player_zones(id_template: &str, players: &[Value]) -> Vec<String> {
    players.iter()
        .map(|p| id_template.replace("{player}", p["id"].as_str().unwrap()))
        .collect()
}

fn check_contents_for_player_template(contents: &Value) -> bool {
    match contents {
        Value::String(s) => s.contains("{player}"),
        Value::Array(arr) => arr.iter().any(|item| {
            if let Some(s) = item.as_str() {
                s.contains("{player}")
            } else {
                false
            }
        }),
        Value::Object(obj) => {
            if let Some(entity) = obj.get("entity").and_then(|e| e.as_str()) {
                entity.contains("{player}")
            } else {
                false
            }
        },
        _ => false
    }
}

fn expand_contents_for_all_players(contents: &Value, players: &[Value]) -> Value {
    if let Some(arr) = contents.as_array() {
        // Expand array contents
        let mut expanded = Vec::new();
        for item in arr {
            if let Some(entity_template) = item.as_str() {
                if entity_template.contains("{player}") {
                    // Add one for each player
                    for player in players {
                        let player_id = player["id"].as_str().unwrap();
                        expanded.push(json!(entity_template.replace("{player}", player_id)));
                    }
                } else {
                    expanded.push(item.clone());
                }
            } else {
                expanded.push(item.clone());
            }
        }
        Value::Array(expanded)
    } else {
        contents.clone()
    }
}

fn create_single_zone(
    pid: &str,
    zone: &Value,
    zone_type: &str,
    contents: &Value,
    per_player: bool,
    zones: &mut Map<String, Value>,
) {
    // Use thread-local RNG for backward compatibility
    let mut rng = rand::rng();
    create_single_zone_with_rng(pid, zone, zone_type, contents, per_player, zones, &mut rng);
}

fn create_single_zone_with_rng<R: RngCore>(
    pid: &str,
    zone: &Value,
    zone_type: &str,
    contents: &Value,
    per_player: bool,
    zones: &mut Map<String, Value>,
    rng: &mut R,
) {
    let mut content_spec = contents.clone();
    
    if per_player {
        content_spec = resolve_player_specific_content(pid, contents, &mut content_spec);
    }

    let mut value = match zone_type {
        "grid" => init_grid(zone, &content_spec),
        "hexgrid" => init_hexgrid(zone, &content_spec),
        "list" | "deck" | "stack" => init_list(&content_spec),
        "single" => init_single(&content_spec),
        "choice" => init_choice(zone),
        _ => Value::Null,
    };
    
    if zone_type == "deck" || zone_type == "stack" {
        apply_deck_shuffling_with_rng(zone, &mut value, pid, rng);
    }
    
    zones.insert(pid.to_string(), value);
}

fn resolve_player_specific_content(
    pid: &str,
    contents: &Value,
    content_spec: &mut Value,
) -> Value {
    let player_id = pid.split('_').last().unwrap_or("");
    
    if let Some(map) = contents.as_object() {
        if let Some(specific) = map.get(player_id) {
            *content_spec = specific.clone();
        }
    }
    
    // Replace {player} placeholders in the content_spec
    if let Some(obj) = content_spec.as_object_mut() {
        if let Some(entity) = obj.get_mut("entity") {
            if let Some(entity_str) = entity.as_str() {
                *entity = Value::String(entity_str.replace("{player}", player_id));
            }
        }
    }
    
    content_spec.clone()
}

fn apply_deck_shuffling(zone: &Value, value: &mut Value, pid: &str) {
    // Use thread-local RNG for backward compatibility
    let mut rng = rand::rng();
    apply_deck_shuffling_with_rng(zone, value, pid, &mut rng);
}

fn apply_deck_shuffling_with_rng<R: RngCore>(zone: &Value, value: &mut Value, pid: &str, rng: &mut R) {
    // Check for shuffle in both deckProps and shapeMeta (for compatibility with different game definitions)
    let should_shuffle = zone.get("deckProps")
        .and_then(|dp| dp.get("shuffle"))
        .and_then(|s| s.as_bool())
        .unwrap_or(false)
        || zone.get("shapeMeta")
            .and_then(|sm| sm.get("shuffle"))
            .and_then(|s| s.as_bool())
            .unwrap_or(false);
    
    if should_shuffle {
        if let Some(items) = value.get_mut("items").and_then(|i| i.as_array_mut()) {
            items.shuffle(rng);
            println!("  Shuffled deck for zone: {}", pid);
        }
    }
}

fn init_grid(zone: &Value, contents: &Value) -> Value {
    // Support multiple paths for grid dimensions
    let rows = zone["rows"].as_u64()
        .or_else(|| zone["gridProps"]["rows"].as_u64())
        .or_else(|| zone["shapeMeta"]["rows"].as_u64())
        .unwrap_or(DEFAULT_GRID_SIZE) as usize;
    let cols = zone["cols"].as_u64()
        .or_else(|| zone["gridProps"]["cols"].as_u64())
        .or_else(|| zone["shapeMeta"]["cols"].as_u64())
        .unwrap_or(DEFAULT_GRID_SIZE) as usize;
    
    let grid = create_grid_cells(rows, cols, contents);
    
    json!({
        "type": "grid",
        "cells": grid
    })
}

fn create_grid_cells(rows: usize, cols: usize, contents: &Value) -> Vec<Value> {
    (0..rows).map(|_| {
        let row: Vec<Value> = (0..cols).map(|_| {
            if contents.as_str() == Some("empty") {
                Value::Null
            } else {
                contents.clone()
            }
        }).collect();
        Value::Array(row)
    }).collect()
}

fn init_hexgrid(zone: &Value, contents: &Value) -> Value {
    use crate::engine::hex::{HexCoord, HexLayout};
    use std::collections::HashMap;
    
    // Get hex grid configuration from shapeMeta
    let shape_meta = zone.get("shapeMeta").unwrap_or(&Value::Null);
    
    // Determine layout type
    let layout = shape_meta.get("layout")
        .and_then(|l| l.as_str())
        .map(|s| match s {
            "pointy" => HexLayout::Pointy,
            _ => HexLayout::Flat,
        })
        .unwrap_or(HexLayout::Flat);
    
    // Get hex grid size - support multiple configuration styles
    let hex_cells = if let Some(size) = shape_meta.get("size").and_then(|s| s.as_u64()) {
        // Hexagonal grid with radius
        create_hexagonal_cells(size as i32, contents)
    } else if let (Some(rows), Some(cols)) = (
        shape_meta.get("rows").and_then(|r| r.as_u64()),
        shape_meta.get("cols").and_then(|c| c.as_u64())
    ) {
        // Rectangular hex grid
        create_rectangular_hex_cells(rows as i32, cols as i32, contents, layout)
    } else {
        // Default small hex grid
        create_hexagonal_cells(1, contents)
    };
    
    json!({
        "type": "hexgrid",
        "layout": match layout {
            HexLayout::Flat => "flat",
            HexLayout::Pointy => "pointy",
        },
        "cells": hex_cells,
        "shapeMeta": shape_meta
    })
}

fn create_hexagonal_cells(radius: i32, contents: &Value) -> Value {
    use crate::engine::hex::HexCoord;
    use std::collections::HashMap;
    
    let mut cells = HashMap::new();
    
    // Generate hexes in a hexagonal pattern
    for q in -radius..=radius {
        let r1 = std::cmp::max(-radius, -q - radius);
        let r2 = std::cmp::min(radius, -q + radius);
        for r in r1..=r2 {
            let coord = HexCoord::new(q, r);
            let cell_value = if contents.as_str() == Some("empty") {
                Value::Null
            } else {
                contents.clone()
            };
            cells.insert(format!("{},{}", q, r), cell_value);
        }
    }
    
    json!(cells)
}

fn create_rectangular_hex_cells(rows: i32, cols: i32, contents: &Value, layout: crate::engine::hex::HexLayout) -> Value {
    use crate::engine::hex::{HexCoord, OffsetCoord};
    use std::collections::HashMap;
    
    let mut cells = HashMap::new();
    
    // Generate hexes in rectangular arrangement using offset coordinates
    for row in 0..rows {
        for col in 0..cols {
            let offset = OffsetCoord { col, row };
            let hex = HexCoord::from_offset(offset, layout);
            
            let cell_value = if contents.as_str() == Some("empty") {
                Value::Null
            } else {
                contents.clone()
            };
            cells.insert(format!("{},{}", hex.q, hex.r), cell_value);
        }
    }
    
    json!(cells)
}

fn init_list(contents: &Value) -> Value {
    let items = create_list_items(contents);
    
    json!({
        "type": "list",
        "items": items
    })
}

fn init_single(contents: &Value) -> Value {
    json!({
        "type": "single",
        "contents": if contents.is_null() || contents.as_str() == Some("empty") {
            Value::Null
        } else {
            contents.clone()
        }
    })
}

fn init_choice(zone: &Value) -> Value {
    // Initialize choice zones with empty items array and optional prompt
    let prompt = zone.get("prompt")
        .and_then(|p| p.as_str())
        .unwrap_or("Make a choice");
    
    json!({
        "type": "choice",
        "items": [],
        "prompt": prompt
    })
}

fn create_list_items(contents: &Value) -> Vec<Value> {
    if contents.as_str() == Some("empty") {
        Vec::new()
    } else if let Some(entity_id) = contents.get("entity").and_then(|e| e.as_str()) {
        let count = contents.get("count").and_then(|c| c.as_u64()).unwrap_or(1);
        (0..count).map(|_| json!({"entity": entity_id})).collect()
    } else if let Some(arr) = contents.as_array() {
        // Handle array of entity objects or strings
        arr.iter().map(|item| {
            if let Some(entity_str) = item.as_str() {
                // Convert string to entity object
                json!({"entity": entity_str})
            } else if item.get("entity").is_some() {
                // Already an entity object, use as-is
                item.clone()
            } else {
                // Other object type, use as-is
                item.clone()
            }
        }).collect()
    } else if let Some(contents_str) = contents.as_str() {
        // Handle single string entity reference
        vec![json!({"entity": contents_str})]
    } else {
        vec![contents.clone()]
    }
}

fn initialize_phase_states(phases_def: &Value) -> Value {
    // Check if we should use enhanced phase system
    let use_enhanced = should_use_enhanced_phases(phases_def);
    
    if use_enhanced {
        // Use enhanced phase system with enter/exit actions
        initialize_enhanced_phases(phases_def)
    } else {
        // Use legacy simple phase states
        initialize_legacy_phase_states(phases_def)
    }
}

fn should_use_enhanced_phases(phases_def: &Value) -> bool {
    // Check if any phase has enterActions or exitActions
    if let Some(phase_sets) = phases_def.as_array() {
        for phase_set in phase_sets {
            if let Some(phases) = phase_set["phases"].as_array() {
                for phase in phases {
                    if phase.get("enterActions").is_some() || phase.get("exitActions").is_some() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn initialize_legacy_phase_states(phases_def: &Value) -> Value {
    let mut phase_states = json!({});
    
    if let Some(phase_sets) = phases_def.as_array() {
        for phase_set in phase_sets {
            initialize_phase_set(&mut phase_states, phase_set);
        }
    }
    
    phase_states
}

fn initialize_phase_set(phase_states: &mut Value, phase_set: &Value) {
    if let Some(set_id) = phase_set["id"].as_str() {
        if let Some(phases) = phase_set["phases"].as_array() {
            let initial = find_initial_phase(phases);
            phase_states[set_id] = json!(initial);
        }
    }
}

fn find_initial_phase(phases: &[Value]) -> &str {
    phases.iter()
        .find(|p| p["initial"].as_bool().unwrap_or(false))
        .or_else(|| phases.first())
        .and_then(|p| p["id"].as_str())
        .unwrap_or("null")
}
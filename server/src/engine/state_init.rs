//! Simplified state initialization for the new architecture

use crate::bundle::Bundle;
use serde_json::{json, Value, Map};
use rand::seq::SliceRandom;
use rand::RngCore;

const INITIAL_PLAYER: &str = "p1";

/// Initialize game state with simplified structure
pub fn load_initial_state<R: RngCore>(
    bundle: &Bundle,
    player_names: &[String],
    rng: &mut R,
) -> Value {
    let players = create_players(player_names);
    let zones = create_zones(&players, &bundle.zones, rng);
    let phases = initialize_phases(&bundle.phases);
    
    json!({
        "players": players,
        "currentPlayer": INITIAL_PLAYER,
        "zones": zones,
        "phases": phases,
        "gameStatus": "playing",  // Simplified: "playing" | "won:p1" | "tie" | "abandoned"
        "selection": {}  // For multi-step actions
    })
}

/// Create player objects with simplified structure
fn create_players(player_names: &[String]) -> Vec<Value> {
    player_names.iter().enumerate().map(|(i, name)| {
        json!({
            "id": format!("p{}", i + 1),
            "name": name,  // Lobby member ID
            "ui": {
                "isActive": i == 0,  // First player is active
            }
        })
    }).collect()
}

/// Create zones without redundant type specifications
fn create_zones<R: RngCore>(
    players: &[Value],
    zones_def: &Value,
    rng: &mut R,
) -> Map<String, Value> {
    let mut zones = Map::new();
    
    if let Some(arr) = zones_def.as_array() {
        for zone in arr {
            process_zone(zone, players, &mut zones, rng);
        }
    }
    
    zones
}

/// Process a single zone definition
fn process_zone<R: RngCore>(
    zone: &Value,
    players: &[Value],
    zones: &mut Map<String, Value>,
    rng: &mut R,
) {
    let id = zone["id"].as_str().unwrap_or("");
    let zone_type = zone["shape"].as_str().unwrap_or("");
    let contents = zone.get("contents").cloned().unwrap_or(json!("empty"));
    
    // Handle player-specific zones
    if id.contains("{player}") {
        for (i, _) in players.iter().enumerate() {
            let player_id = format!("p{}", i + 1);
            let zone_id = id.replace("{player}", &player_id);
            let zone_data = create_zone_data(zone_type, &contents, zone, rng);
            zones.insert(zone_id, zone_data);
        }
    } else {
        let zone_data = create_zone_data(zone_type, &contents, zone, rng);
        zones.insert(id.to_string(), zone_data);
    }
}

/// Create zone data without type field
fn create_zone_data<R: RngCore>(
    zone_type: &str,
    contents: &Value,
    zone_def: &Value,
    rng: &mut R,
) -> Value {
    match zone_type {
        "grid" => {
            let empty_meta = json!({});
            let shape_meta = zone_def.get("shapeMeta").unwrap_or(&empty_meta);
            let rows = shape_meta["rows"].as_u64().unwrap_or(3) as usize;
            let cols = shape_meta["cols"].as_u64().unwrap_or(3) as usize;
            
            let cells = create_grid_cells(rows, cols, contents);
            json!({ "cells": cells })
        }
        
        "list" | "stack" => {
            let mut items = create_list_items(contents);
            
            // Check for shuffling
            if zone_def.get("shapeMeta")
                .and_then(|sm| sm.get("shuffle"))
                .and_then(|s| s.as_bool())
                .unwrap_or(false) {
                items.shuffle(rng);
            }
            
            json!({ "items": items })
        }
        
        "choice" => {
            json!({
                "items": [],
                "prompt": zone_def.get("prompt")
                    .and_then(|p| p.as_str())
                    .unwrap_or("Make a choice")
            })
        }
        
        "hexgrid" => {
            // Simplified hex grid (details omitted for brevity)
            json!({ "cells": {} })
        }
        
        _ => json!({})
    }
}

/// Create grid cells without redundant data
fn create_grid_cells(rows: usize, cols: usize, contents: &Value) -> Vec<Vec<Value>> {
    (0..rows).map(|_| {
        (0..cols).map(|_| {
            if contents.as_str() == Some("empty") {
                json!(null)
            } else {
                contents.clone()
            }
        }).collect()
    }).collect()
}

/// Create list items with simplified structure
fn create_list_items(contents: &Value) -> Vec<Value> {
    match contents {
        Value::String(s) if s == "empty" => Vec::new(),
        Value::String(s) if s == "standardDeck" => create_standard_deck(),
        Value::Array(arr) => arr.clone(),
        Value::Object(obj) if obj.contains_key("entity") => {
            let count = obj.get("count").and_then(|c| c.as_u64()).unwrap_or(1);
            (0..count).map(|_| Value::Object(obj.clone())).collect()
        }
        _ => vec![contents.clone()]
    }
}

/// Create a standard deck of cards
fn create_standard_deck() -> Vec<Value> {
    let suits = ["hearts", "diamonds", "clubs", "spades"];
    let ranks = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
    
    let mut cards = Vec::new();
    for suit in &suits {
        for rank in &ranks {
            cards.push(json!({
                "entity": format!("card_{}_{}", rank, suit.chars().next().unwrap().to_uppercase())
            }));
        }
    }
    
    cards
}

/// Initialize phases with simplified structure
fn initialize_phases(phases_def: &Value) -> Map<String, Value> {
    let mut phases = Map::new();
    
    if let Some(phase_sets) = phases_def.as_array() {
        for phase_set in phase_sets {
            if let Some(set_id) = phase_set["id"].as_str() {
                if let Some(phase_list) = phase_set["phases"].as_array() {
                    let initial = phase_list.iter()
                        .find(|p| p["initial"].as_bool().unwrap_or(false))
                        .or_else(|| phase_list.first())
                        .and_then(|p| p["id"].as_str())
                        .unwrap_or("null");
                    
                    phases.insert(set_id.to_string(), json!(initial));
                }
            }
        }
    }
    
    phases
}
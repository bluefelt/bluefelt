use crate::bundle::Bundle;
use serde_json::{json, Value, Map};
use rand::seq::SliceRandom;
use rand::rng;

const DEFAULT_GRID_SIZE: u64 = 3;
const INITIAL_PLAYER: &str = "p1";
const INITIAL_TICK: u64 = 0;
const INITIAL_TURN: u64 = 0;

pub fn load_initial_state(bundle: &Bundle) -> Value {
    let player_count = bundle.manifest.metadata.players.max;
    let players = create_players(player_count);
    let zones = create_zones(&players, &bundle.zones);
    let phase_states = initialize_phase_states(&bundle.phases);

    json!({
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
        "phases": phase_states
    })
}

fn create_players(player_count: u32) -> Vec<Value> {
    (1..=player_count)
        .map(|i| json!({"id": format!("p{}", i)}))
        .collect()
}

fn create_zones(players: &[Value], zones_def: &Value) -> Map<String, Value> {
    let mut zones = Map::new();
    
    if let Some(arr) = zones_def.as_array() {
        for zone in arr {
            process_zone_definition(zone, players, &mut zones);
        }
    }
    
    zones
}

fn process_zone_definition(zone: &Value, players: &[Value], zones: &mut Map<String, Value>) {
    let id = zone["id"].as_str().unwrap_or("");
    let zone_type = zone["type"].as_str()
        .or_else(|| zone["shape"].as_str()) // Support old 'shape' field
        .unwrap_or("");
    let contents = zone.get("contents")
        .cloned()
        .unwrap_or_else(|| Value::String("empty".to_string()));
    
    let per_player = id.contains("{player}");
    let ids = if per_player {
        expand_player_zones(id, players)
    } else {
        vec![id.to_string()]
    };

    for pid in ids {
        println!("  Creating zone: {}", pid);
        create_single_zone(&pid, zone, zone_type, &contents, per_player, zones);
    }
}

fn expand_player_zones(id_template: &str, players: &[Value]) -> Vec<String> {
    players.iter()
        .map(|p| id_template.replace("{player}", p["id"].as_str().unwrap()))
        .collect()
}

fn create_single_zone(
    pid: &str,
    zone: &Value,
    zone_type: &str,
    contents: &Value,
    per_player: bool,
    zones: &mut Map<String, Value>,
) {
    let mut content_spec = contents.clone();
    
    if per_player {
        content_spec = resolve_player_specific_content(pid, contents, &mut content_spec);
    }

    let mut value = match zone_type {
        "grid" => init_grid(zone, &content_spec),
        "list" | "deck" => init_list(&content_spec),
        _ => Value::Null,
    };
    
    if zone_type == "deck" {
        apply_deck_shuffling(zone, &mut value, pid);
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
    if let Some(deck_props) = zone.get("deckProps") {
        if deck_props.get("shuffle").and_then(|s| s.as_bool()).unwrap_or(false) {
            if let Some(items) = value.get_mut("items").and_then(|i| i.as_array_mut()) {
                let mut rng = rng();
                items.shuffle(&mut rng);
                println!("  Shuffled deck for zone: {}", pid);
            }
        }
    }
}

fn init_grid(zone: &Value, contents: &Value) -> Value {
    let rows = zone["rows"].as_u64().unwrap_or(DEFAULT_GRID_SIZE) as usize;
    let cols = zone["cols"].as_u64().unwrap_or(DEFAULT_GRID_SIZE) as usize;
    
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

fn init_list(contents: &Value) -> Value {
    let items = create_list_items(contents);
    
    json!({
        "type": "list",
        "items": items
    })
}

fn create_list_items(contents: &Value) -> Vec<Value> {
    if contents.as_str() == Some("empty") {
        Vec::new()
    } else if let Some(entity_id) = contents.get("entity").and_then(|e| e.as_str()) {
        let count = contents.get("count").and_then(|c| c.as_u64()).unwrap_or(1);
        (0..count).map(|_| json!({"entity": entity_id})).collect()
    } else if let Some(arr) = contents.as_array() {
        arr.clone()
    } else {
        vec![contents.clone()]
    }
}

fn initialize_phase_states(phases_def: &Value) -> Value {
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
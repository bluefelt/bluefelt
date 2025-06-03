use serde_json::{json, Value};
use crate::bundle::Bundle;
use crate::engine::path::{get_zone_mut, get_cell_value, set_cell_value};
use crate::engine::grid::apply_check_for_win;

pub fn apply_verb(
    state: &mut Value,
    verb: &str,
    args: &Value,
    bundle: &Bundle,
) -> Result<Vec<Value>, String> {
    match verb {
        "draw" => apply_draw(state, args),
        "moveEntity" => apply_move_entity(state, args),
        "place" => apply_place(state, args),
        "nextTurn" => apply_next_turn(state, args, bundle),
        "setPhase" => apply_set_phase(state, args),
        "grid.lineOfMarks" => apply_check_for_win(state, args),
        _ => Err(format!("Unknown verb: {}", verb)),
    }
}

fn apply_draw(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;
    let count = args["count"].as_u64().unwrap_or(1) as usize;

    let mut patches = Vec::new();

    for _ in 0..count {
        draw_single_item(state, from_path, to_path, &mut patches)?;
    }

    Ok(patches)
}

fn draw_single_item(
    state: &mut Value,
    from_path: &str,
    to_path: &str,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    // Get the source zone
    let from_zone = get_zone_mut(state, from_path)?;
    let items = from_zone["items"].as_array_mut()
        .ok_or("Source zone is not a list/deck")?;
    
    if items.is_empty() {
        return Err("Cannot draw from empty deck".to_string());
    }

    // Remove item from source
    let item = items.remove(0);
    patches.push(json!({
        "op": "remove",
        "path": format!("{}/items/0", from_path)
    }));

    // Add to destination
    let to_zone = get_zone_mut(state, to_path)?;
    let to_items = to_zone["items"].as_array_mut()
        .ok_or("Destination zone is not a list")?;
    
    let insert_index = to_items.len();
    to_items.push(item.clone());
    patches.push(json!({
        "op": "add",
        "path": format!("{}/items/{}", to_path, insert_index),
        "value": item
    }));

    Ok(())
}

fn apply_move_entity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;

    let mut patches = Vec::new();

    // Handle grid to grid moves
    if from_path.contains("/cells/") && to_path.contains("/cells/") {
        move_between_grid_cells(state, from_path, to_path, &mut patches)?;
    }

    Ok(patches)
}

fn move_between_grid_cells(
    state: &mut Value,
    from_path: &str,
    to_path: &str,
    patches: &mut Vec<Value>,
) -> Result<(), String> {
    let from_value = get_cell_value(state, from_path)?;
    
    // Remove from source
    set_cell_value(state, from_path, Value::Null)?;
    patches.push(json!({
        "op": "replace",
        "path": from_path,
        "value": null
    }));

    // Add to destination
    set_cell_value(state, to_path, from_value.clone())?;
    patches.push(json!({
        "op": "replace",
        "path": to_path,
        "value": from_value
    }));

    Ok(())
}

fn apply_place(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let location = args["location"].as_str().ok_or("Missing 'location' path")?;
    let entity = args["entity"].as_str().ok_or("Missing 'entity' id")?;
    
    let entity_value = json!({"entity": entity});
    set_cell_value(state, location, entity_value.clone())?;
    
    Ok(vec![json!({
        "op": "replace", 
        "path": format!("/game{}", location),
        "value": entity_value
    })])
}

fn apply_next_turn(state: &mut Value, _args: &Value, bundle: &Bundle) -> Result<Vec<Value>, String> {
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    
    // Increment tick
    let current_tick = state_obj["tick"].as_u64().unwrap_or(0);
    let new_tick = current_tick + 1;
    state_obj.insert("tick".to_string(), json!(new_tick));
    
    // Advance turn
    let player_count = bundle.manifest.metadata.players.max;
    let current_turn = state_obj["turn"].as_u64().unwrap_or(0);
    let next_turn = (current_turn + 1) % player_count as u64;
    let next_player = format!("p{}", next_turn + 1);
    
    state_obj.insert("turn".to_string(), json!(next_turn));
    state_obj.insert("currentPlayer".to_string(), json!(next_player.clone()));
    
    Ok(vec![
        json!({
            "op": "replace",
            "path": "/game/tick",
            "value": new_tick
        }),
        json!({
            "op": "replace",
            "path": "/game/turn",
            "value": next_turn
        }),
        json!({
            "op": "replace",
            "path": "/game/currentPlayer",
            "value": next_player
        })
    ])
}

fn apply_set_phase(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let phase_set = args["phaseSet"].as_str().ok_or("Missing 'phaseSet'")?;
    let phase = args["phase"].as_str().ok_or("Missing 'phase'")?;
    
    let state_obj = state.as_object_mut().ok_or("State is not an object")?;
    let phases = state_obj.get_mut("phases")
        .and_then(|p| p.as_object_mut())
        .ok_or("Missing phases in state")?;
    
    phases.insert(phase_set.to_string(), json!(phase));
    
    Ok(vec![json!({
        "op": "replace",
        "path": format!("/game/phases/{}", phase_set),
        "value": phase
    })])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::bundle::{Bundle, Manifest, ManifestMetadata, PlayersRange};

    #[allow(dead_code)]
    fn create_test_bundle() -> Bundle {
        Bundle {
            game_id: "test".to_string(),
            manifest: Manifest {
                game_id: "test".to_string(),
                version: "1.0".to_string(),
                spec_version: "0.1".to_string(),
                metadata: ManifestMetadata {
                    name: "Test Game".to_string(),
                    author: "Test Author".to_string(),
                    players: PlayersRange { min: 2, max: 2 },
                    description: "Test game".to_string(),
                },
                phases: None,
                setup: None,
                zone_groups: None,
            },
            entities: Value::Null,
            zones: Value::Null,
            actions: Value::Null,
            phases: Value::Null,
            _hooks: None,
        }
    }

    #[test]
    fn test_apply_draw_basic() {
        let mut state = json!({
            "zones": {
                "deck": {
                    "type": "list",
                    "items": [
                        {"entity": "card1"},
                        {"entity": "card2"}
                    ]
                },
                "hand": {
                    "type": "list", 
                    "items": []
                }
            }
        });

        let args = json!({
            "from": "/zones/deck",
            "to": "/zones/hand",
            "count": 1
        });

        let result = apply_draw(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 2);
        
        // Check that card was moved
        let deck_items = state["zones"]["deck"]["items"].as_array().unwrap();
        let hand_items = state["zones"]["hand"]["items"].as_array().unwrap();
        
        assert_eq!(deck_items.len(), 1);
        assert_eq!(hand_items.len(), 1);
        assert_eq!(hand_items[0]["entity"], "card1");
    }

    #[test]
    fn test_apply_draw_empty_deck() {
        let mut state = json!({
            "zones": {
                "deck": {
                    "type": "list",
                    "items": []
                },
                "hand": {
                    "type": "list",
                    "items": []
                }
            }
        });

        let args = json!({
            "from": "/zones/deck",
            "to": "/zones/hand",
            "count": 1
        });

        let result = apply_draw(&mut state, &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty deck"));
    }

    #[test]
    fn test_apply_place() {
        let mut state = json!({
            "zones": {
                "board": {
                    "type": "grid",
                    "cells": [
                        [null, null, null],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            }
        });

        let args = json!({
            "location": "/zones/board/cells/0/0",
            "entity": "x_token"
        });

        let result = apply_place(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 1);
        
        // Check that entity was placed
        let cell_value = &state["zones"]["board"]["cells"][0][0];
        assert_eq!(cell_value["entity"], "x_token");
    }

    #[test]
    fn test_apply_move_entity() {
        let mut state = json!({
            "zones": {
                "board": {
                    "type": "grid",
                    "cells": [
                        [{"entity": "piece"}, null, null],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            }
        });

        let args = json!({
            "from": "/zones/board/cells/0/0",
            "to": "/zones/board/cells/1/1"
        });

        let result = apply_move_entity(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 2);
        
        // Check that entity was moved
        assert!(state["zones"]["board"]["cells"][0][0].is_null());
        assert_eq!(state["zones"]["board"]["cells"][1][1]["entity"], "piece");
    }
}
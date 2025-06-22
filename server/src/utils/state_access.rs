use serde_json::Value;

/// Get the current phase from state, handling both old and enhanced phase systems
pub fn get_current_phase(state: &Value) -> Option<&str> {
    state.get("phases")
        .and_then(|phases| {
            // When both exist, prefer the direct phase value (legacy) as it's more recent
            phases.get("game")
                // Fallback to enhanced phase system
                .or_else(|| phases.get("current")
                    .and_then(|current| current.get("game")))
        })
        .and_then(|phase| phase.as_str())
}

/// Get zones from state, handling nested game state structure
pub fn get_zones(state: &Value) -> Option<&Value> {
    // First try direct access
    state.get("zones")
        // Then try under game
        .or_else(|| state.get("game").and_then(|g| g.get("zones")))
}

/// Get game status from state, handling different locations
pub fn get_game_status(state: &Value) -> Option<&Value> {
    // Check at root level first
    state.get("gameStatus")
        // Then check under game
        .or_else(|| state.get("game").and_then(|g| g.get("gameStatus")))
}

/// Check if the game has ended
pub fn is_game_ended(state: &Value) -> bool {
    get_game_status(state)
        .and_then(|status| status.get("state"))
        .and_then(|s| s.as_str())
        .map(|s| s == "ended")
        .unwrap_or(false)
}

/// Get the current player
pub fn get_current_player(state: &Value) -> Option<&str> {
    state.get("currentPlayer")
        .or_else(|| state.get("game").and_then(|g| g.get("currentPlayer")))
        .and_then(|p| p.as_str())
}

/// Get players array
pub fn get_players(state: &Value) -> Option<&Vec<Value>> {
    state.get("players")
        .or_else(|| state.get("game").and_then(|g| g.get("players")))
        .and_then(|p| p.as_array())
}

/// Get selection for a player
pub fn get_player_selection<'a>(state: &'a Value, player_id: &str) -> Option<&'a Value> {
    state.get("selection")
        .or_else(|| state.get("game").and_then(|g| g.get("selection")))
        .and_then(|s| s.get(player_id))
}

/// Get a zone by ID
pub fn get_zone_by_id<'a>(state: &'a Value, zone_id: &str) -> Option<&'a Value> {
    get_zones(state)
        .and_then(|zones| zones.get(zone_id))
}

/// Get entity definitions
pub fn get_entity_definitions(state: &Value) -> Option<&Vec<Value>> {
    state.get("entityDefinitions")
        .or_else(|| state.get("game").and_then(|g| g.get("entityDefinitions")))
        .and_then(|e| e.as_array())
}

/// Get a specific phase set
pub fn get_phase_set<'a>(state: &'a Value, phase_set: &str) -> Option<&'a str> {
    state.get("phases")
        .and_then(|phases| phases.get(phase_set))
        .and_then(|phase| phase.as_str())
}

/// Get all active phases (for enhanced phase system)
pub fn get_all_active_phases(state: &Value) -> Option<&Value> {
    state.get("phases")
        .and_then(|phases| phases.get("current"))
}

/// Get tick/turn information
pub fn get_tick(state: &Value) -> Option<u64> {
    state.get("tick")
        .or_else(|| state.get("game").and_then(|g| g.get("tick")))
        .and_then(|t| t.as_u64())
}

pub fn get_turn(state: &Value) -> Option<u64> {
    state.get("turn")
        .or_else(|| state.get("game").and_then(|g| g.get("turn")))
        .and_then(|t| t.as_u64())
}

/// Get value at a path, handling both direct and game-nested access
pub fn get_value_at_path<'a>(state: &'a Value, path: &str) -> Option<&'a Value> {
    // If path starts with /game/, try direct access first
    if path.starts_with("/game/") {
        let direct_path = &path[6..]; // Remove "/game/" prefix
        if let Some(value) = get_value_at_path_direct(state, direct_path) {
            return Some(value);
        }
    }
    
    // Try the full path
    get_value_at_path_direct(state, path)
        // If not found and doesn't start with /game/, try under game
        .or_else(|| {
            if !path.starts_with("/game/") {
                state.get("game")
                    .and_then(|game| get_value_at_path_direct(game, path))
            } else {
                None
            }
        })
}

/// Direct path access helper
fn get_value_at_path_direct<'a>(state: &'a Value, path: &str) -> Option<&'a Value> {
    let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in path_parts {
        if let Ok(index) = part.parse::<usize>() {
            // Array index
            current = current.as_array()?.get(index)?;
        } else {
            // Object key
            current = current.get(part)?;
        }
    }
    
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_current_phase() {
        // Test legacy phase system
        let state = json!({
            "phases": {
                "game": "playing"
            }
        });
        assert_eq!(get_current_phase(&state), Some("playing"));

        // Test enhanced phase system
        let state = json!({
            "phases": {
                "current": {
                    "game": "movement"
                }
            }
        });
        assert_eq!(get_current_phase(&state), Some("movement"));

        // Test both present (legacy takes precedence)
        let state = json!({
            "phases": {
                "game": "playing",
                "current": {
                    "game": "movement"
                }
            }
        });
        assert_eq!(get_current_phase(&state), Some("playing"));
    }

    #[test]
    fn test_get_game_status() {
        // Test root level
        let state = json!({
            "gameStatus": {
                "state": "ended",
                "winner": "p1"
            }
        });
        assert!(get_game_status(&state).is_some());
        assert!(is_game_ended(&state));

        // Test under game
        let state = json!({
            "game": {
                "gameStatus": {
                    "state": "playing"
                }
            }
        });
        assert!(!is_game_ended(&state));
    }

    #[test]
    fn test_get_value_at_path() {
        let state = json!({
            "zones": {
                "board": {
                    "cells": [
                        [null, {"entity": "mark_p1"}, null],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            },
            "game": {
                "currentPlayer": "p2"
            }
        });

        // Direct path
        assert_eq!(
            get_value_at_path(&state, "/zones/board/cells/0/1"),
            state.get("zones").unwrap().get("board").unwrap().get("cells").unwrap().get(0).unwrap().get(1)
        );

        // Nested under game
        assert_eq!(
            get_value_at_path(&state, "/game/currentPlayer"),
            Some(&json!("p2"))
        );
    }
}
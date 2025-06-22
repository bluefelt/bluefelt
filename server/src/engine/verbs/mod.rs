//! Verb implementations for game state manipulation

// Import all verb modules
pub mod cards;
pub mod choices;
pub mod game_logic;
pub mod math;
pub mod movement;
pub mod multi_step;
pub mod phase_management;
pub mod queries;
pub mod selection;
pub mod state_management;
pub mod hex_win_detection;

// New simplified verbs
pub mod set_property;
pub mod transfer_entity;
pub mod shuffle;
pub mod deal;
pub mod set_phase;
pub mod clear_selection;
pub mod increment_property;
pub mod decrement_property;
pub mod append_to_log;
pub mod end_game;

use serde_json::{json, Value};
use crate::bundle::Bundle;

/// Apply a verb to the game state
pub fn apply_verb(
    state: &mut Value,
    verb: &str,
    args: &Value,
    bundle: &Bundle,
) -> Result<Vec<Value>, String> {
    println!("[DEBUG apply_verb] verb={}, args={:?}", verb, args);
    
    match verb {
        // New simplified verbs
        "setProperty" => set_property::execute(args, state),
        "transferEntity" => transfer_entity::execute(args, bundle, state),
        "shuffle" => shuffle::execute(args, state),
        "deal" => deal::execute(args, state),
        "setPhase" => set_phase::execute(args, state),
        "clearSelection" => clear_selection::execute(state),
        "incrementProperty" => increment_property::execute(args, state),
        "decrementProperty" => decrement_property::execute(args, state),
        "appendToLog" => append_to_log::execute(args, state),
        "endGame" => end_game::execute(args, state),
        
        // Legacy verbs - still supported for backward compatibility
        "setState" => state_management::apply_set_state(state, args),
        "queryEntities" => queries::apply_query_entities(state, args),
        "moveEntity" => movement::apply_move_entity(state, args),
        "place" => movement::apply_place(state, args),
        "placeWithGravity" => movement::apply_place_with_gravity(state, args),
        "presentChoice" => choices::apply_present_choice(state, args),
        "selectEntity" => selection::apply_select_entity(state, args),
        "calculateWinner" => game_logic::apply_calculate_winner(state, args),
        "nextTurn" => state_management::apply_next_turn(state, args, bundle),
        
        // Card-specific verbs
        "shuffleCards" => cards::apply_shuffle(state, args),
        "dealToAllPlayers" => cards::apply_deal_to_all_players(state, args, bundle),
        "draw" => cards::apply_draw(state, args),
        "drawCard" => cards::apply_draw(state, args),
        "drawWithReshuffle" => cards::apply_draw_with_reshuffle(state, args),
        "transferMatching" => cards::apply_transfer_matching(state, args),
        "formPairs" => cards::apply_form_pairs(state, args),
        "validateMeld" => cards::apply_validate_meld(state, args),
        "matchCard" => cards::apply_match_card(state, args),
        "removePairs" => cards::apply_remove_pairs(state, args),
        "compareCardsEqual" => cards::apply_compare_cards_equal(state, args),
        "compareCardsGreater" => cards::apply_compare_cards_greater(state, args),
        "formMelds" => cards::apply_form_melds(state, args, bundle),
        
        // Game logic verbs
        "conditionalAction" => game_logic::apply_conditional_action(state, args, bundle, "system"),
        
        // Math operations
        "calculate" => math::apply_calculate(state, args),
        
        // Phase management
        "transitionPhase" => phase_management::transition_phase(state, args, bundle),
        
        // Hex-specific verbs
        "checkHexWin" => hex_win_detection::apply_check_hex_win(state, args),
        "checkHexLine" => hex_win_detection::apply_check_hex_line(state, args),
        "countHexGroups" => hex_win_detection::apply_count_hex_groups(state, args),
        
        _ => Err(format!("Unknown verb: {}", verb)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::Bundle;
    use serde_json::json;
    
    #[test]
    fn test_apply_verb_set_property() {
        let mut state = json!({
            "currentPlayer": "p1",
            "gameStatus": "playing"
        });
        
        let bundle = Bundle {
            game_id: "test".to_string(),
            manifest: crate::bundle::Manifest {
                game_id: "test".to_string(),
                version: "1.0".to_string(),
                spec_version: "1.0".to_string(),
                metadata: crate::bundle::ManifestMetadata {
                    name: "Test".to_string(),
                    author: "Test".to_string(),
                    description: "Test".to_string(),
                    players: crate::bundle::PlayersRange { min: 2, max: 2 },
                },
                phases: None,
                setup: None,
                zone_groups: None,
            },
            entities: Value::Null,
            zones: Value::Null,
            actions: Value::Null,
            phases: Value::Null,
        };
        let args = json!({
            "path": "/gameStatus",
            "value": "won:p1"
        });
        
        let result = apply_verb(&mut state, "setProperty", &args, &bundle);
        assert!(result.is_ok());
        assert_eq!(state["gameStatus"], "won:p1");
    }
}
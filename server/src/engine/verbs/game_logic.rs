use serde_json::{json, Value};
use crate::bundle::Bundle;
use crate::engine::path::{get_zone_mut, get_cell_value};

pub fn apply_calculate_winner(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let mut max_pairs = 0;
    let mut winner = None;
    let mut tie = false;
    
    // Check each player's pairs
    for player_num in 1..=4 {
        let player_id = format!("p{}", player_num);
        let pairs_path = format!("/zones/pairs_{}", player_id);
        
        if let Ok(pairs_zone) = get_zone_mut(state, &pairs_path) {
            if let Some(pairs_items) = pairs_zone["items"].as_array() {
                let pairs_count = pairs_items.len();
                if pairs_count > max_pairs {
                    max_pairs = pairs_count;
                    winner = Some(player_id);
                    tie = false;
                } else if pairs_count == max_pairs && max_pairs > 0 {
                    tie = true;
                }
            }
        }
    }
    
    // Set game status
    let game_status = if tie {
        json!({
            "state": "ended",
            "winner": null,
            "tie": true
        })
    } else if let Some(winner_id) = winner {
        json!({
            "state": "ended", 
            "winner": winner_id,
            "tie": false
        })
    } else {
        json!({
            "state": "active",
            "winner": null,
            "tie": false
        })
    };
    
    // Update state
    if let Some(state_obj) = state.as_object_mut() {
        state_obj.insert("gameStatus".to_string(), game_status.clone());
    }
    
    Ok(vec![json!({
        "op": "replace",
        "path": "/game/gameStatus",
        "value": game_status
    })])
}

pub fn apply_conditional_action(state: &mut Value, args: &Value, bundle: &Bundle, current_actor: &str) -> Result<Vec<Value>, String> {
    let condition = args.get("condition").ok_or("Missing 'condition'")?;
    
    // Support both formats: ifTrue/ifFalse and then/else
    let if_true = args.get("ifTrue").and_then(|t| t.as_array())
        .or_else(|| args.get("then").and_then(|t| t.as_array()));
    let if_false = args.get("ifFalse").and_then(|f| f.as_array())
        .or_else(|| args.get("else").and_then(|e| e.as_array()));
    
    // Evaluate the condition
    let condition_result = if let Some(conditions) = condition.as_array() {
        // Evaluate all conditions (AND logic)
        conditions.iter().all(|cond| {
            match crate::conditions::evaluate_condition(cond, state, args, current_actor) {
                Ok(result) => result,
                Err(e) => {
                    println!("Error evaluating condition: {}", e);
                    false
                }
            }
        })
    } else {
        // Single condition
        match crate::conditions::evaluate_condition(condition, state, args, current_actor) {
            Ok(result) => result,
            Err(e) => {
                println!("Error evaluating condition: {}", e);
                false
            }
        }
    };
    
    let actions_to_execute = if condition_result {
        println!("[DEBUG conditionalAction] Condition TRUE, executing ifTrue actions");
        if_true
    } else {
        println!("[DEBUG conditionalAction] Condition FALSE, executing ifFalse actions");
        if_false
    };
    
    let mut all_patches = Vec::new();
    
    if let Some(actions) = actions_to_execute {
        for action in actions {
            if let Some(action_name) = action.get("action").and_then(|a| a.as_str()) {
                let empty_args = json!({});
                let action_args = action.get("with").unwrap_or(&empty_args);
                
                // Find the action in the bundle and execute it
                if let Some(actions_array) = bundle.actions.as_array() {
                    if let Some(action_def) = actions_array.iter().find(|a| a["id"].as_str() == Some(action_name)) {
                        if let Some(verb) = action_def["uses"].as_str() {
                            // Merge action args with definition's "with" params
                            let mut final_args = action_def.get("with").cloned().unwrap_or(json!({}));
                            if let (Some(final_obj), Some(args_obj)) = (final_args.as_object_mut(), action_args.as_object()) {
                                for (k, v) in args_obj {
                                    final_obj.insert(k.clone(), v.clone());
                                }
                            }
                            
                            // Replace templates in the args
                            final_args = crate::engine::patches::replace_template_vars(&final_args, state);
                            final_args = crate::engine::patches::replace_actor_template(&final_args, current_actor);
                            
                            // Execute the verb
                            println!("[DEBUG conditionalAction] Executing action {} with verb {}", action_name, verb);
                            match crate::engine::verbs::apply_verb(state, verb, &final_args, bundle) {
                                Ok(mut patches) => {
                                    println!("[DEBUG conditionalAction] Action {} produced {} patches", action_name, patches.len());
                                    all_patches.append(&mut patches)
                                },
                                Err(e) => return Err(format!("Failed to execute conditional action '{}': {}", action_name, e)),
                            }
                        }
                    } else {
                        // If it's not an action ID, treat it as a verb name directly
                        match crate::engine::verbs::apply_verb(state, action_name, action_args, bundle) {
                            Ok(mut patches) => all_patches.append(&mut patches),
                            Err(e) => return Err(format!("Failed to execute conditional action '{}': {}", action_name, e)),
                        }
                    }
                } else {
                    return Err("Bundle missing actions array".to_string());
                }
            }
        }
    }
    
    Ok(all_patches)
}

pub fn apply_make_bid(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let player = args["player"].as_str().ok_or("Missing 'player'")?;
    let bid = args["bid"].as_u64().ok_or("Missing 'bid'")? as i32;
    let bid_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("tricks");
    
    let mut patches = Vec::new();
    
    // Store bid in state
    let bid_path = format!("/game/bids/{}", player);
    patches.push(json!({
        "op": "replace",
        "path": bid_path,
        "value": {
            "amount": bid,
            "type": bid_type
        }
    }));
    
    // Check for hook rule if specified
    if let Some(true) = args.get("enforceHook").and_then(|v| v.as_bool()) {
        // Calculate total bids
        let bids = state["game"]["bids"].as_object();
        let total_bids: i32 = bids.map(|b| {
            b.values()
                .filter_map(|v| v["amount"].as_i64())
                .sum::<i64>() as i32
        }).unwrap_or(0);
        
        let available_tricks = args.get("availableTricks")
            .and_then(|v| v.as_i64())
            .unwrap_or(13) as i32;
        
        if total_bids == available_tricks {
            return Err("Total bids cannot equal available tricks (hook rule)".to_string());
        }
    }
    
    Ok(patches)
}

pub fn apply_play_to_trick(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let player = args["player"].as_str().ok_or("Missing 'player'")?;
    let card_path = args["card"].as_str().ok_or("Missing 'card' path")?;
    let trick_path = args.get("trickPath")
        .and_then(|v| v.as_str())
        .unwrap_or("/game/currentTrick");
    
    let mut patches = Vec::new();
    
    // Get the card
    let card = get_cell_value(state, card_path)?.clone();
    
    // Remove from player's hand
    patches.push(json!({
        "op": "remove",
        "path": card_path
    }));
    
    // Add to current trick
    let trick_card_path = format!("{}/{}", trick_path, player);
    patches.push(json!({
        "op": "replace",
        "path": trick_card_path,
        "value": card
    }));
    
    Ok(patches)
}

pub fn apply_resolve_trick(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let trick_path = args.get("trickPath")
        .and_then(|v| v.as_str())
        .unwrap_or("/game/currentTrick");
    let trump_suit = args.get("trumpSuit").and_then(|v| v.as_str());
    let led_suit_path = args.get("ledSuit").and_then(|v| v.as_str());
    
    let mut patches = Vec::new();
    
    // Get current trick
    let trick = state.pointer(trick_path)
        .ok_or("Trick not found")?
        .as_object()
        .ok_or("Trick is not an object")?;
    
    // Determine led suit
    let led_suit = if let Some(path) = led_suit_path {
        state.pointer(path).and_then(|v| v.as_str())
    } else {
        // Get from first card played
        trick.values().next()
            .and_then(|card| card["suit"].as_str())
    };
    
    // Find winning card
    let mut winner = None;
    let mut highest_value = -1;
    
    for (player, card) in trick.iter() {
        let suit = card["suit"].as_str().unwrap_or("");
        let rank_value = get_card_trick_value(card, suit, led_suit, trump_suit);
        
        if rank_value > highest_value {
            highest_value = rank_value;
            winner = Some(player.clone());
        }
    }
    
    if let Some(winner) = winner {
        // Store trick winner
        patches.push(json!({
            "op": "replace",
            "path": "/game/lastTrickWinner",
            "value": winner
        }));
        
        // Clear current trick
        patches.push(json!({
            "op": "replace",
            "path": trick_path,
            "value": {}
        }));
        
        // Increment tricks won
        let won_path = format!("/game/tricksWon/{}", winner);
        let current_won = state.pointer(&won_path)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        patches.push(json!({
            "op": "replace",
            "path": won_path,
            "value": current_won + 1
        }));
    }
    
    Ok(patches)
}

fn get_card_trick_value(card: &Value, suit: &str, led_suit: Option<&str>, trump_suit: Option<&str>) -> i32 {
    let base_value = match card["rank"].as_str().unwrap_or("") {
        "2" => 2,
        "3" => 3,
        "4" => 4,
        "5" => 5,
        "6" => 6,
        "7" => 7,
        "8" => 8,
        "9" => 9,
        "10" => 10,
        "J" => 11,
        "Q" => 12,
        "K" => 13,
        "A" => 14,
        _ => 0,
    };
    
    // Trump suit beats all
    if let Some(trump) = trump_suit {
        if suit == trump {
            return base_value + 100;
        }
    }
    
    // Must follow suit to win (unless trump)
    if let Some(led) = led_suit {
        if suit == led {
            return base_value;
        }
    }
    
    // Off suit, can't win
    0
}
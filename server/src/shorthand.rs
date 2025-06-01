use serde_json::{json, Value};
use crate::bundle::Manifest;

/// Expands shorthand syntax in game definitions
pub fn expand_game_definitions(
    entities: &mut Value,
    zones: &mut Value,
    actions: &mut Value,
    manifest: &Manifest,
) {
    expand_entities(entities);
    expand_zones(zones);
    expand_actions(actions, manifest.metadata.players.max);
    
    // Debug: Print expanded counts
    if let Value::Array(entities_array) = entities {
        println!("  Expanded to {} entities", entities_array.len());
    }
}

/// Expands entity shortcuts like standardDeck
fn expand_entities(entities: &mut Value) {
    if let Value::Array(entities_array) = entities {
        let mut expanded = Vec::new();
        
        for entity in entities_array.iter() {
            if let Some(entity_type) = entity.get("type").and_then(|t| t.as_str()) {
                if entity_type == "standardDeck" {
                    // Generate standard 52-card deck
                    expanded.extend(generate_standard_deck());
                } else {
                    expanded.push(entity.clone());
                }
            } else {
                expanded.push(entity.clone());
            }
        }
        
        *entities = Value::Array(expanded);
    }
}

/// Generates a standard 52-card deck
fn generate_standard_deck() -> Vec<Value> {
    let suits = ["hearts", "diamonds", "clubs", "spades"];
    let ranks = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
    let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
    
    let mut cards = Vec::new();
    
    for suit in &suits {
        for (rank, value) in ranks.iter().zip(values.iter()) {
            let card_id = format!("card_{}_{}", suit, rank.to_lowercase());
            let card = json!({
                "id": card_id,
                "type": "card",
                "props": {
                    "suit": suit,
                    "rank": rank,
                    "value": value
                },
                "ui": {
                    "cardType": "playing_card"
                }
            });
            cards.push(card);
        }
    }
    
    cards
}

/// Expands zone shortcuts like contents: standardDeck
fn expand_zones(zones: &mut Value) {
    if let Value::Array(zones_array) = zones {
        for zone in zones_array.iter_mut() {
            if let Some(zone_id) = zone.get("id").and_then(|id| id.as_str()) {
                println!("  Processing zone: {}", zone_id);
            }
            if let Some(Value::String(contents)) = zone.get("contents") {
                if contents == "standardDeck" {
                    println!("  Expanding standardDeck for zone");
                    // Replace with list of all card IDs
                    let mut card_ids = Vec::new();
                    let suits = ["hearts", "diamonds", "clubs", "spades"];
                    let ranks = ["a", "2", "3", "4", "5", "6", "7", "8", "9", "10", "j", "q", "k"];
                    
                    for suit in &suits {
                        for rank in &ranks {
                            card_ids.push(Value::String(format!("card_{}_{}", suit, rank)));
                        }
                    }
                    
                    println!("  Generated {} card IDs", card_ids.len());
                    zone["contents"] = Value::Array(card_ids);
                }
            }
        }
    }
}

/// Expands action shortcuts like deal and reveal builtins
fn expand_actions(actions: &mut Value, max_players: u32) {
    if let Value::Array(actions_array) = actions {
        println!("  Processing {} actions", actions_array.len());
        let mut expanded = Vec::new();
        
        for action in actions_array.iter() {
            if let Some(action_id) = action.get("id").and_then(|id| id.as_str()) {
                println!("  Processing action: {}", action_id);
            }
            if let Some(uses) = action.get("uses").and_then(|b| b.as_str()) {
                println!("  Action uses: {}", uses);
                match uses {
                    "cards.deal" => {
                        println!("  Expanding cards.deal action");
                        expanded.extend(expand_deal_action(action, max_players));
                    },
                    "cards.reveal" => {
                        println!("  Expanding cards.reveal action");
                        expanded.push(expand_reveal_action(action));
                    },
                    _ => expanded.push(action.clone()),
                }
            } else {
                expanded.push(action.clone());
            }
        }
        
        println!("  Expanded to {} actions", expanded.len());
        *actions = Value::Array(expanded);
    }
}

/// Expands deal builtin into individual moveEntity actions
fn expand_deal_action(action: &Value, max_players: u32) -> Vec<Value> {
    let with = action.get("with").unwrap_or(&Value::Null);
    let count = with.get("count").and_then(|c| c.as_u64()).unwrap_or(1) as usize;
    let to_param = with.get("to").and_then(|t| t.as_str()).unwrap_or("");
    let from_zone = with.get("from").and_then(|f| f.as_str()).unwrap_or("drawPile");
    
    let mut expanded_actions = Vec::new();
    
    if to_param == "eachPlayer" {
        // Create a trigger action that starts the dealing
        let base_id = action.get("id").and_then(|id| id.as_str()).unwrap_or("deal");
        let mut triggers = Vec::new();
        
        // Use the actual player count from the manifest
        for player_num in 1..=max_players {
            for card_num in 1..=count {
                let deal_id = format!("{}_p{}_{}", base_id, player_num, card_num);
                triggers.push(Value::String(deal_id.clone()));
                
                let deal_action = json!({
                    "id": deal_id,
                    "uses": "entity.move",
                    "auto": true,
                    "phase": action.get("phase").cloned().unwrap_or(json!("setup")),
                    "with": {
                        "source": from_zone,
                        "target": format!("hand_p{}", player_num),
                        "count": 1
                    }
                });
                
                expanded_actions.push(deal_action);
            }
        }
        
        // Add original action's then to the last deal action
        if let Some(original_then) = action.get("then") {
            if let Some(last_action) = expanded_actions.last_mut() {
                last_action["then"] = original_then.clone();
            }
        }
        
        // Create the trigger action
        let mut trigger_action = action.clone();
        if let Value::Object(ref mut obj) = trigger_action {
            obj.remove("uses");
            obj.remove("with");
            obj["then"] = Value::Array(triggers.into_iter().map(|t| json!({"action": t})).collect());
        }
        
        // Add the trigger action first
        expanded_actions.insert(0, trigger_action);
    } else {
        // Single target dealing - just convert to moveEntity
        let mut expanded = action.clone();
        if let Value::Object(ref mut obj) = expanded {
            obj["uses"] = json!("entity.move");
        }
        expanded_actions.push(expanded);
    }
    
    expanded_actions
}

/// Expands reveal builtin into moveEntity
fn expand_reveal_action(action: &Value) -> Value {
    let mut expanded = action.clone();
    if let Value::Object(ref mut obj) = expanded {
        obj["uses"] = json!("entity.move");
    }
    expanded
}
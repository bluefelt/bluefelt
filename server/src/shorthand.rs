use serde_json::{json, Value};
use crate::bundle::Manifest;

/// Expands shorthand syntax in game definitions
pub fn expand_game_definitions(
    entities: &mut Value,
    zones: &mut Value,
    actions: &mut Value,
    manifest: &Manifest,
) {
    expand_entities(entities, manifest.metadata.players.max);
    expand_zones(zones);
    expand_actions(actions, manifest.metadata.players.max);
    
    // Debug: Print expanded counts
    if let Value::Array(entities_array) = entities {
        println!("  Expanded to {} entities", entities_array.len());
    }
}

/// Expands entity shortcuts like standardDeck and {player} patterns
fn expand_entities(entities: &mut Value, max_players: u32) {
    if let Value::Array(entities_array) = entities {
        let mut expanded = Vec::new();
        
        for entity in entities_array.iter() {
            // Check if entity id contains {player}
            if let Some(entity_id) = entity.get("id").and_then(|id| id.as_str()) {
                if entity_id.contains("{player}") {
                    // Expand for each player
                    for player_num in 1..=max_players {
                        let mut player_entity = entity.clone();
                        if let Some(obj) = player_entity.as_object_mut() {
                            // Replace {player} in id
                            let player_id = format!("p{}", player_num);
                            let new_id = entity_id.replace("{player}", &player_id);
                            obj.insert("id".to_string(), json!(new_id));
                            
                            // Replace {player} in name if it exists
                            if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                                let new_name = name.replace("{player}", &format!("Player {}", player_num));
                                obj.insert("name".to_string(), json!(new_name));
                            }
                            
                            // Replace {player} in nested objects (like ui.tokenType)
                            replace_player_in_object(obj, &player_id);
                        }
                        expanded.push(player_entity);
                    }
                } else if let Some(entity_type) = entity.get("type").and_then(|t| t.as_str()) {
                    if entity_type == "standardDeck" {
                        // Generate standard 52-card deck
                        expanded.extend(generate_standard_deck());
                    } else if entity_type == "deck" {
                        // Check if this is a playing card deck
                        if let Some(options) = entity.get("options") {
                            if options.get("cardType").and_then(|ct| ct.as_str()) == Some("cardType.playingCard") {
                                // Generate standard 52-card deck
                                expanded.extend(generate_standard_deck());
                            } else {
                                expanded.push(entity.clone());
                            }
                        } else {
                            expanded.push(entity.clone());
                        }
                    } else {
                        expanded.push(entity.clone());
                    }
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

/// Recursively replace {player} placeholder in nested objects
fn replace_player_in_object(obj: &mut serde_json::Map<String, Value>, player_id: &str) {
    for (_, value) in obj.iter_mut() {
        match value {
            Value::String(s) => {
                if s.contains("{player}") {
                    *s = s.replace("{player}", player_id);
                }
            }
            Value::Object(nested_obj) => {
                replace_player_in_object(nested_obj, player_id);
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    if let Value::Object(nested_obj) = item {
                        replace_player_in_object(nested_obj, player_id);
                    } else if let Value::String(s) = item {
                        if s.contains("{player}") {
                            *s = s.replace("{player}", player_id);
                        }
                    }
                }
            }
            _ => {}
        }
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
                // Handle both with and without presets. prefix
                let core_uses = if uses.starts_with("presets.") {
                    &uses[8..]
                } else {
                    uses
                };
                
                match core_uses {
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
                    "uses": "presets.entity.move",
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
                if let Value::Object(ref mut obj) = last_action {
                    obj.insert("then".to_string(), original_then.clone());
                }
            }
        }
        
        // Create the trigger action
        let mut trigger_action = action.clone();
        if let Value::Object(ref mut obj) = trigger_action {
            obj.remove("uses");
            obj.remove("with");
            obj.insert("auto".to_string(), json!(true)); // Mark as auto so it can be called from phases
            obj.insert("then".to_string(), Value::Array(triggers.into_iter().map(|t| json!({"action": t})).collect()));
        }
        
        // Add the trigger action first
        expanded_actions.insert(0, trigger_action);
    } else {
        // Single target dealing - just convert to moveEntity
        let mut expanded = action.clone();
        if let Value::Object(ref mut obj) = expanded {
            obj["uses"] = json!("presets.entity.move");
        }
        expanded_actions.push(expanded);
    }
    
    expanded_actions
}

/// Expands reveal builtin into moveEntity
fn expand_reveal_action(action: &Value) -> Value {
    let mut expanded = action.clone();
    if let Value::Object(ref mut obj) = expanded {
        obj["uses"] = json!("presets.entity.move");
    }
    expanded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{Manifest, ManifestMetadata, PlayersRange};

    fn create_test_manifest(max_players: u32) -> Manifest {
        Manifest {
            game_id: "test-game".to_string(),
            version: "1.0".to_string(),
            spec_version: "1.0".to_string(),
            metadata: ManifestMetadata {
                name: "Test Game".to_string(),
                author: "Test Author".to_string(),
                players: PlayersRange {
                    min: 2,
                    max: max_players,
                },
                description: "Test description".to_string(),
            },
            phases: None,
            setup: None,
            zone_groups: None,
        }
    }

    #[test]
    fn test_expand_entities_player_replacement() {
        let mut entities = json!([
            {
                "id": "token_{player}",
                "name": "Token for {player}",
                "type": "token"
            }
        ]);

        expand_entities(&mut entities, 2);

        let expanded = entities.as_array().unwrap();
        assert_eq!(expanded.len(), 2);
        
        assert_eq!(expanded[0]["id"], "token_p1");
        assert_eq!(expanded[0]["name"], "Token for Player 1");
        
        assert_eq!(expanded[1]["id"], "token_p2");
        assert_eq!(expanded[1]["name"], "Token for Player 2");
    }

    #[test]
    fn test_expand_entities_standard_deck() {
        let mut entities = json!([
            {
                "id": "deck",
                "type": "standardDeck"
            }
        ]);

        expand_entities(&mut entities, 2);

        let expanded = entities.as_array().unwrap();
        assert_eq!(expanded.len(), 52); // Standard deck has 52 cards
        
        // Check first card
        assert_eq!(expanded[0]["id"], "card_hearts_a");
        assert_eq!(expanded[0]["props"]["suit"], "hearts");
        assert_eq!(expanded[0]["props"]["rank"], "A");
        assert_eq!(expanded[0]["props"]["value"], 1);
        
        // Check last card
        assert_eq!(expanded[51]["id"], "card_spades_k");
        assert_eq!(expanded[51]["props"]["suit"], "spades");
        assert_eq!(expanded[51]["props"]["rank"], "K");
        assert_eq!(expanded[51]["props"]["value"], 13);
    }

    #[test]
    fn test_expand_zones_standard_deck() {
        let mut zones = json!([
            {
                "id": "draw_pile",
                "type": "deck",
                "contents": "standardDeck"
            }
        ]);

        expand_zones(&mut zones);

        let zone = &zones[0];
        let contents = zone["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 52);
        assert_eq!(contents[0], "card_hearts_a");
        assert_eq!(contents[51], "card_spades_k");
    }

    #[test]
    fn test_replace_player_in_object_nested() {
        let mut obj = serde_json::Map::new();
        obj.insert("ui".to_string(), json!({
            "tokenType": "token_{player}",
            "color": "{player}_color"
        }));

        replace_player_in_object(&mut obj, "p1");

        assert_eq!(obj["ui"]["tokenType"], "token_p1");
        assert_eq!(obj["ui"]["color"], "p1_color");
    }

    #[test]
    fn test_expand_deal_action_each_player() {
        let action = json!({
            "id": "deal_cards",
            "uses": "presets.cards.deal",
            "phase": "setup",
            "with": {
                "count": 2,
                "to": "eachPlayer",
                "from": "deck"
            }
        });

        let expanded = expand_deal_action(&action, 2);
        
        // Should have 1 trigger action + 4 individual deal actions (2 players * 2 cards)
        assert_eq!(expanded.len(), 5);
        
        // First action should be the trigger
        assert!(expanded[0]["auto"].as_bool().unwrap_or(false));
        
        // Check individual deal actions
        assert_eq!(expanded[1]["id"], "deal_cards_p1_1");
        assert_eq!(expanded[1]["uses"], "presets.entity.move");
        assert_eq!(expanded[1]["with"]["target"], "hand_p1");
        
        assert_eq!(expanded[3]["id"], "deal_cards_p2_1");
        assert_eq!(expanded[3]["with"]["target"], "hand_p2");
    }

    #[test]
    fn test_expand_reveal_action() {
        let action = json!({
            "id": "reveal_card",
            "uses": "presets.cards.reveal",
            "with": {
                "from": "deck",
                "to": "revealed"
            }
        });

        let expanded = expand_reveal_action(&action);
        assert_eq!(expanded["uses"], "presets.entity.move");
        assert_eq!(expanded["id"], "reveal_card");
    }

    #[test]
    fn test_expand_game_definitions_integration() {
        let manifest = create_test_manifest(2);
        
        let mut entities = json!([
            {
                "id": "token_{player}",
                "type": "token"
            },
            {
                "id": "deck",
                "type": "standardDeck"
            }
        ]);
        
        let mut zones = json!([
            {
                "id": "board",
                "type": "grid",
                "contents": "empty"
            }
        ]);
        
        let mut actions = json!([
            {
                "id": "deal",
                "uses": "cards.deal",
                "with": {
                    "count": 1,
                    "to": "eachPlayer"
                }
            }
        ]);

        expand_game_definitions(&mut entities, &mut zones, &mut actions, &manifest);

        // Should have 2 tokens + 52 cards = 54 entities
        assert_eq!(entities.as_array().unwrap().len(), 54);
        
        // Actions should be expanded from deal builtin
        let expanded_actions = actions.as_array().unwrap();
        assert!(expanded_actions.len() > 1); // Should have multiple actions from deal expansion
    }
}
use serde_json::{json, Value};
use crate::engine::path::{get_zone_mut, get_cell_value};
use crate::bundle::Bundle;
use chrono;
use rand::seq::SliceRandom;

pub fn apply_draw(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;
    let count = args["count"].as_u64().unwrap_or(1) as usize;

    let mut patches = Vec::new();

    for _ in 0..count {
        draw_single_item(state, from_path, to_path, &mut patches)?;
    }

    Ok(patches)
}

/// Deal cards to all players in the game
pub fn apply_deal_to_all_players(state: &mut Value, args: &Value, _bundle: &Bundle) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_zone_pattern = args["toZonePattern"].as_str()
        .or_else(|| args["to"].as_str())
        .ok_or("Missing 'toZonePattern' or 'to' parameter")?;
    let count = args["count"].as_u64().unwrap_or(1) as usize;
    let mode = args["mode"].as_str().unwrap_or("fixed");
    
    // Get the list of player IDs from state
    let player_ids: Vec<String> = state["players"].as_array()
        .ok_or("No players array in state")?
        .iter()
        .filter_map(|p| p["id"].as_str().map(|s| s.to_string()))
        .collect();
    
    if player_ids.is_empty() {
        return Err("No players in game".to_string());
    }
    
    let mut patches = Vec::new();
    let player_count = player_ids.len();
    
    match mode {
        "fixed" => {
            // Original behavior: deal exact count to each player
            for player_id in player_ids {
                let player_zone_path = to_zone_pattern.replace("{player}", &player_id);
                
                for _ in 0..count {
                    match draw_single_item(state, from_path, &player_zone_path, &mut patches) {
                        Ok(_) => {},
                        Err(e) => {
                            if e.contains("empty deck") {
                                break;
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
            }
            
            // Only add game log if UI structure exists (game has started)
            if state.get("ui").is_some() {
                let log_message = format!("Dealt {} cards to {} players", count, player_count);
                patches.push(json!({
                    "op": "add",
                    "path": "/ui/gameLog/-",
                    "value": {
                        "auto": true,
                        "message": log_message,
                        "timestamp": chrono::Utc::now().format("%H:%M").to_string()
                    }
                }));
            }
        },
        
        "dealAll" => {
            // Deal cards in rotation until deck is empty
            let mut total_dealt = 0;
            let mut cards_per_player: Vec<usize> = vec![0; player_count];
            let mut dealing = true;
            
            while dealing {
                // One round of dealing
                for (idx, player_id) in player_ids.iter().enumerate() {
                    let player_zone_path = to_zone_pattern.replace("{player}", player_id);
                    
                    // Deal 'count' cards to this player
                    let mut dealt_this_round = 0;
                    for _ in 0..count {
                        match draw_single_item(state, from_path, &player_zone_path, &mut patches) {
                            Ok(_) => {
                                dealt_this_round += 1;
                                cards_per_player[idx] += 1;
                                total_dealt += 1;
                            },
                            Err(e) => {
                                if e.contains("empty deck") {
                                    dealing = false;
                                    break;
                                } else {
                                    return Err(e);
                                }
                            }
                        }
                    }
                    
                    if !dealing {
                        break;
                    }
                }
            }
            
            // Create a detailed log message
            let card_distribution: Vec<String> = player_ids.iter()
                .zip(cards_per_player.iter())
                .map(|(id, count)| format!("{}: {}", id, count))
                .collect();
            
            let log_message = format!(
                "Dealt {} cards total ({})",
                total_dealt,
                card_distribution.join(", ")
            );
            
            // Only add game log if UI structure exists (game has started)
            if state.get("ui").is_some() {
                patches.push(json!({
                    "op": "add",
                    "path": "/ui/gameLog/-",
                    "value": {
                        "auto": true,
                        "message": log_message,
                        "timestamp": chrono::Utc::now().format("%H:%M").to_string()
                    }
                }));
            }
        },
        
        _ => return Err(format!("Unknown dealing mode: {}", mode))
    }
    
    Ok(patches)
}

pub fn draw_single_item(
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

pub fn apply_draw_with_reshuffle(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;
    let count = args["count"].as_u64().unwrap_or(1) as usize;
    let reshuffle_from = args.get("reshuffleFrom").and_then(|v| v.as_str());

    let mut patches = Vec::new();
    
    for i in 0..count {
        // Check if deck is empty and reshuffle is enabled
        let from_zone = get_zone_mut(state, from_path)?;
        let items = from_zone["items"].as_array().ok_or("Source is not a deck")?;
        
        if items.is_empty() && reshuffle_from.is_some() {
            // Reshuffle discard pile back into deck
            let discard_path = reshuffle_from.unwrap();
            reshuffle_deck(state, discard_path, from_path, &mut patches)?;
        }
        
        // Now try to draw
        match draw_single_item(state, from_path, to_path, &mut patches) {
            Ok(_) => {},
            Err(e) => {
                if i > 0 {
                    // Partial success - drew some cards
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }
    
    Ok(patches)
}

fn reshuffle_deck(state: &mut Value, from_path: &str, to_path: &str, patches: &mut Vec<Value>) -> Result<(), String> {
    let from_zone = get_zone_mut(state, from_path)?;
    let items = from_zone["items"].as_array_mut()
        .ok_or("Discard pile is not a list")?;
    
    if items.is_empty() {
        return Err("Cannot reshuffle empty discard pile".to_string());
    }
    
    // Move all cards from discard to deck
    let cards: Vec<Value> = items.drain(..).collect();
    patches.push(json!({
        "op": "replace",
        "path": format!("{}/items", from_path),
        "value": []
    }));
    
    // Shuffle the cards (would need RNG from state)
    // For now, just move them as-is
    let to_zone = get_zone_mut(state, to_path)?;
    let to_items = to_zone["items"].as_array_mut()
        .ok_or("Deck is not a list")?;
    
    for (i, card) in cards.into_iter().enumerate() {
        to_items.push(card.clone());
        patches.push(json!({
            "op": "add",
            "path": format!("{}/items/{}", to_path, i),
            "value": card
        }));
    }
    
    Ok(())
}

pub fn apply_shuffle(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args["zone"].as_str().ok_or("Missing 'zone' path")?;
    
    let mut patches = Vec::new();
    
    // Get the zone and its items
    let zone = get_zone_mut(state, zone_path)?;
    let items = zone["items"].as_array_mut()
        .ok_or("Zone is not a list")?;
    
    if items.is_empty() {
        return Ok(patches); // Nothing to shuffle
    }
    
    // Shuffle the items using thread RNG
    let mut rng = rand::rng();
    items.shuffle(&mut rng);
    
    // Create a patch to replace the entire items array
    patches.push(json!({
        "op": "replace",
        "path": format!("{}/items", zone_path),
        "value": items.clone()
    }));
    
    Ok(patches)
}

pub fn apply_transfer_matching(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;
    let property = args["property"].as_str().ok_or("Missing 'property'")?;
    let value = args["value"].as_str().ok_or("Missing 'value'")?;
    
    let mut patches = Vec::new();
    
    // Get source zone
    let from_zone = get_zone_mut(state, from_path)?;
    let from_items = from_zone["items"].as_array_mut()
        .ok_or("Source zone is not a list")?;
    
    // Find and remove matching entities
    let mut matching_entities = Vec::new();
    let mut i = 0;
    while i < from_items.len() {
        let mut matches = false;
        if let Some(entity_id) = from_items[i].get("entity").and_then(|e| e.as_str()) {
            // Check if entity matches the property value
            if entity_id.starts_with("card_") {
                let parts: Vec<&str> = entity_id.split('_').collect();
                if parts.len() >= 3 {
                    let entity_value = match property {
                        "rank" => parts[2],  // rank is the third part: card_suit_rank
                        "suit" => parts[1],  // suit is the second part: card_suit_rank
                        _ => "",
                    };
                    matches = entity_value == value;
                }
            }
        }
        
        if matches {
            matching_entities.push(from_items.remove(i));
        } else {
            i += 1;
        }
    }
    
    // Create patch for source zone
    patches.push(json!({
        "op": "replace",
        "path": format!("/game{}/items", from_path),
        "value": from_items
    }));
    
    // Add entities to target zone
    let to_zone = get_zone_mut(state, to_path)?;
    let to_items = to_zone["items"].as_array_mut()
        .ok_or("Target zone is not a list")?;
    
    for entity in matching_entities {
        to_items.push(entity);
    }
    
    // Create patch for target zone
    patches.push(json!({
        "op": "replace",
        "path": format!("/game{}/items", to_path),
        "value": to_items
    }));
    
    Ok(patches)
}

pub fn apply_form_pairs(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let player = args["player"].as_str().ok_or("Missing 'player'")?;
    
    let hand_path = format!("/zones/hand_{}", player);
    let pairs_path = format!("/zones/pairs_{}", player);
    
    let mut patches = Vec::new();
    
    // Get player's hand
    let hand_zone = get_zone_mut(state, &hand_path)?;
    let hand_items = hand_zone["items"].as_array_mut()
        .ok_or("Hand zone is not a list")?;
    
    // Count cards by rank
    let mut rank_counts: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    for (index, item) in hand_items.iter().enumerate() {
        if let Some(entity_id) = item.get("entity").and_then(|e| e.as_str()) {
            if entity_id.starts_with("card_") {
                let parts: Vec<&str> = entity_id.split('_').collect();
                if parts.len() >= 3 {
                    let rank = parts[2].to_string();  // rank is the third part: card_suit_rank
                    rank_counts.entry(rank).or_insert_with(Vec::new).push(index);
                }
            }
        }
    }
    
    // Find ranks with 2+ cards (pairs)
    let mut pairs_formed = false;
    let mut cards_to_remove = Vec::new();
    let mut pairs_to_add = Vec::new();
    let mut formed_ranks = Vec::new();
    
    for (rank, indices) in rank_counts {
        if indices.len() >= 2 {
            // Found a pair (2 or more cards of same rank)
            // Take only the first 2 cards to form a pair
            pairs_formed = true;
            formed_ranks.push(rank.clone());
            for i in 0..2 {
                let index = indices[i];
                cards_to_remove.push(index);
                pairs_to_add.push(hand_items[index].clone());
            }
        }
    }
    
    if pairs_formed {
        // Remove cards from hand (in reverse order to maintain indices)
        cards_to_remove.sort_by(|a, b| b.cmp(a));
        for index in cards_to_remove {
            hand_items.remove(index);
        }
        
        // Update hand
        patches.push(json!({
            "op": "replace",
            "path": format!("/game{}/items", hand_path),
            "value": hand_items
        }));
        
        // Add to pairs zone
        let pairs_zone = get_zone_mut(state, &pairs_path)?;
        let pairs_items = pairs_zone["items"].as_array_mut()
            .ok_or("Pairs zone is not a list")?;
        
        for card in pairs_to_add {
            pairs_items.push(card);
        }
        
        patches.push(json!({
            "op": "replace", 
            "path": format!("/game{}/items", pairs_path),
            "value": pairs_items
        }));
        
        // Generate log messages for each pair formed
        for rank in formed_ranks {
            let log_message = format!("{} forms a pair of {}s", player, rank);
            patches.push(json!({
                "op": "add",
                "path": "/ui/gameLog/-",
                "value": {
                    "auto": true,
                    "message": log_message,
                    "timestamp": chrono::Utc::now().format("%H:%M").to_string()
                }
            }));
        }
    }
    
    Ok(patches)
}

pub fn apply_validate_meld(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let cards = args["cards"].as_array().ok_or("Missing 'cards' array")?;
    let meld_type = args.get("type").and_then(|v| v.as_str());
    let result_path = args["resultPath"].as_str().ok_or("Missing 'resultPath'")?;
    
    let is_valid = match meld_type {
        Some("set") => validate_set(cards),
        Some("run") => validate_run(cards),
        _ => validate_set(cards) || validate_run(cards), // Either type
    };
    
    Ok(vec![json!({
        "op": "replace",
        "path": result_path,
        "value": is_valid
    })])
}

fn validate_set(cards: &[Value]) -> bool {
    if cards.len() < 3 {
        return false;
    }
    
    // All cards must have same rank
    let first_rank = cards[0]["rank"].as_str();
    if first_rank.is_none() {
        return false;
    }
    
    cards.iter().all(|card| {
        card["rank"].as_str() == first_rank
    })
}

fn validate_run(cards: &[Value]) -> bool {
    if cards.len() < 3 {
        return false;
    }
    
    // All cards must be same suit and consecutive ranks
    let first_suit = cards[0]["suit"].as_str();
    if first_suit.is_none() {
        return false;
    }
    
    // Check all same suit
    if !cards.iter().all(|card| card["suit"].as_str() == first_suit) {
        return false;
    }
    
    // Get ranks and sort
    let mut ranks: Vec<i32> = cards.iter()
        .filter_map(|card| {
            let rank = card["rank"].as_str()?;
            match rank {
                "A" => Some(1),  // Can also be 14 in some games
                "J" => Some(11),
                "Q" => Some(12),
                "K" => Some(13),
                num => num.parse().ok(),
            }
        })
        .collect();
    
    if ranks.len() != cards.len() {
        return false;
    }
    
    ranks.sort();
    
    // Check consecutive
    for i in 1..ranks.len() {
        if ranks[i] != ranks[i-1] + 1 {
            // Check for Ace-high run (Q-K-A)
            if i == ranks.len() - 1 && ranks[0] == 1 && ranks[i-1] == 13 {
                continue;
            }
            return false;
        }
    }
    
    true
}

pub fn apply_match_card(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let card_path = args["card"].as_str().ok_or("Missing 'card' path")?;
    let match_rank = args.get("matchRank").and_then(|v| v.as_str());
    let match_suit = args.get("matchSuit").and_then(|v| v.as_str());
    let result_path = args["resultPath"].as_str().ok_or("Missing 'resultPath'")?;
    
    // Get the card
    let card = crate::engine::path::get_cell_value(state, card_path)?;
    
    let mut matches = true;
    
    if let Some(rank) = match_rank {
        matches = matches && card["rank"].as_str() == Some(rank);
    }
    
    if let Some(suit) = match_suit {
        matches = matches && card["suit"].as_str() == Some(suit);
    }
    
    Ok(vec![json!({
        "op": "replace",
        "path": result_path,
        "value": matches
    })])
}

pub fn apply_remove_pairs(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let hand_path = args["hand"].as_str().ok_or("Missing 'hand' path")?;
    let pairs_path = args.get("pairsZone").and_then(|v| v.as_str());
    
    let hand_zone = get_zone_mut(state, hand_path)?;
    let items = hand_zone["items"].as_array_mut()
        .ok_or("Hand is not a list")?;
    
    let mut patches = Vec::new();
    let mut pairs_found = Vec::new();
    
    // Find all pairs by rank
    let mut rank_counts: std::collections::HashMap<String, Vec<(usize, Value)>> = std::collections::HashMap::new();
    
    for (i, card) in items.iter().enumerate() {
        if let Some(rank) = card["rank"].as_str() {
            rank_counts.entry(rank.to_string())
                .or_insert(Vec::new())
                .push((i, card.clone()));
        }
    }
    
    // Collect pairs
    for (rank, cards) in rank_counts {
        if cards.len() >= 2 {
            // Take pairs (2 at a time)
            for chunk in cards.chunks(2) {
                if chunk.len() == 2 {
                    pairs_found.push((chunk[0].0, chunk[1].0));
                }
            }
        }
    }
    
    // Remove pairs from hand (in reverse order to maintain indices)
    let mut indices_to_remove: Vec<usize> = pairs_found.iter()
        .flat_map(|(a, b)| vec![*a, *b])
        .collect();
    indices_to_remove.sort_by(|a, b| b.cmp(a));
    
    for idx in indices_to_remove {
        items.remove(idx);
        patches.push(json!({
            "op": "remove",
            "path": format!("{}/items/{}", hand_path, idx)
        }));
    }
    
    // If pairs zone specified, add pairs there
    if let Some(zone_path) = pairs_path {
        let pairs_zone = get_zone_mut(state, zone_path)?;
        let pairs_items = pairs_zone["items"].as_array_mut()
            .ok_or("Pairs zone is not a list")?;
        
        for (i, _pair) in pairs_found.iter().enumerate() {
            pairs_items.push(json!({"id": format!("pair_{}", i)}));
            patches.push(json!({
                "op": "add", 
                "path": format!("{}/items/{}", zone_path, i),
                "value": {"id": format!("pair_{}", i)}
            }));
        }
    }
    
    Ok(patches)
}

/// Transfer a specific entity by ID from one zone to another
pub fn apply_transfer_entity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let from_path = args["from"].as_str().ok_or("Missing 'from' path")?;
    let to_path = args["to"].as_str().ok_or("Missing 'to' path")?;
    let entity_id = args["entity"].as_str().ok_or("Missing 'entity' ID")?;
    
    let mut patches = Vec::new();
    
    // Get source zone
    let from_zone = get_zone_mut(state, from_path)?;
    let from_items = from_zone["items"].as_array_mut()
        .ok_or("Source zone is not a list")?;
    
    // Find and remove the specific entity
    let mut found_entity = None;
    let mut found_index = None;
    
    for (i, item) in from_items.iter().enumerate() {
        if let Some(item_entity) = item.get("entity").and_then(|e| e.as_str()) {
            if item_entity == entity_id {
                found_entity = Some(item.clone());
                found_index = Some(i);
                break;
            }
        }
    }
    
    if let (Some(entity), Some(index)) = (found_entity, found_index) {
        // Remove from source
        from_items.remove(index);
        patches.push(json!({
            "op": "remove",
            "path": format!("{}/items/{}", from_path, index)
        }));
        
        // Add to destination
        let to_zone = get_zone_mut(state, to_path)?;
        let to_items = to_zone["items"].as_array_mut()
            .ok_or("Target zone is not a list")?;
        
        let insert_index = to_items.len();
        to_items.push(entity.clone());
        patches.push(json!({
            "op": "add",
            "path": format!("{}/items/{}", to_path, insert_index),
            "value": entity
        }));
        
        Ok(patches)
    } else {
        Err(format!("Entity '{}' not found in source zone", entity_id))
    }
}

/// Compare two cards for equality
pub fn apply_compare_cards_equal(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let card1_path = args["card1"].as_str().ok_or("Missing 'card1' path")?;
    let card2_path = args["card2"].as_str().ok_or("Missing 'card2' path")?;
    
    let card1 = get_cell_value(state, card1_path)?;
    let card2 = get_cell_value(state, card2_path)?;
    
    let value1 = get_card_value(&card1)?;
    let value2 = get_card_value(&card2)?;
    
    let are_equal = value1 == value2;
    
    // Store result in state for conditional actions to use
    let mut patches = vec![json!({
        "op": "replace",
        "path": "/game/lastComparison",
        "value": {
            "equal": are_equal,
            "card1Value": value1,
            "card2Value": value2
        }
    })];
    
    Ok(patches)
}

/// Compare two cards to see if first is greater
pub fn apply_compare_cards_greater(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let card1_path = args["card1"].as_str().ok_or("Missing 'card1' path")?;
    let card2_path = args["card2"].as_str().ok_or("Missing 'card2' path")?;
    
    let card1 = get_cell_value(state, card1_path)?;
    let card2 = get_cell_value(state, card2_path)?;
    
    let value1 = get_card_value(&card1)?;
    let value2 = get_card_value(&card2)?;
    
    let is_greater = value1 > value2;
    
    // Store result in state for conditional actions to use
    let mut patches = vec![json!({
        "op": "replace",
        "path": "/game/lastComparison", 
        "value": {
            "greater": is_greater,
            "card1Value": value1,
            "card2Value": value2
        }
    })];
    
    Ok(patches)
}

/// Form melds from selected cards (multi-step action)
pub fn apply_form_melds(state: &mut Value, args: &Value, _bundle: &Bundle) -> Result<Vec<Value>, String> {
    // This is a multi-step action that should be handled by the multi-step system
    // For now, we'll implement basic meld validation and formation
    
    let from_zone = args["fromZone"].as_str().ok_or("Missing 'fromZone'")?;
    let to_zone = args["toZone"].as_str().ok_or("Missing 'toZone'")?;
    let selected_cards = args["selectedCards"].as_array()
        .ok_or("Missing 'selectedCards' array")?;
    let meld_type = args["meldType"].as_str();
    
    let mut patches = Vec::new();
    
    // Validate the meld
    let is_valid = match meld_type {
        Some("set") => validate_set(selected_cards),
        Some("run") => validate_run(selected_cards),
        _ => validate_set(selected_cards) || validate_run(selected_cards),
    };
    
    if !is_valid {
        return Err("Selected cards do not form a valid meld".to_string());
    }
    
    // Get the from zone
    let from_zone_state = get_zone_mut(state, from_zone)?;
    let from_items = from_zone_state["items"].as_array_mut()
        .ok_or("From zone is not a list")?;
    
    // Remove selected cards from hand
    let mut cards_to_move = Vec::new();
    for selected in selected_cards {
        if let Some(index) = selected.as_u64() {
            let idx = index as usize;
            if idx < from_items.len() {
                cards_to_move.push((idx, from_items[idx].clone()));
            }
        }
    }
    
    // Sort indices in reverse order to remove correctly
    cards_to_move.sort_by(|a, b| b.0.cmp(&a.0));
    
    // Remove cards from hand
    for (idx, _) in &cards_to_move {
        from_items.remove(*idx);
        patches.push(json!({
            "op": "remove",
            "path": format!("{}/items/{}", from_zone, idx)
        }));
    }
    
    // Add cards to meld zone
    let to_zone_state = get_zone_mut(state, to_zone)?;
    let to_items = to_zone_state["items"].as_array_mut()
        .ok_or("To zone is not a list")?;
    
    // Create a meld group
    let meld_group = json!({
        "type": meld_type.unwrap_or("unknown"),
        "cards": cards_to_move.iter().map(|(_, card)| card).collect::<Vec<_>>()
    });
    
    to_items.push(meld_group.clone());
    patches.push(json!({
        "op": "add",
        "path": format!("{}/items/{}", to_zone, to_items.len() - 1),
        "value": meld_group
    }));
    
    // Log the meld formation
    let meld_desc = match meld_type {
        Some("set") => format!("set of {}", cards_to_move.len()),
        Some("run") => format!("run of {}", cards_to_move.len()),
        _ => format!("meld of {}", cards_to_move.len())
    };
    
    patches.push(json!({
        "op": "add",
        "path": "/ui/gameLog/-",
        "value": {
            "message": format!("Player formed a {}", meld_desc),
            "timestamp": chrono::Utc::now().format("%H:%M").to_string()
        }
    }));
    
    Ok(patches)
}

/// Extract card value from a card entity
fn get_card_value(card: &Value) -> Result<i64, String> {
    // Try different value fields
    if let Some(value) = card.get("value").and_then(|v| v.as_i64()) {
        return Ok(value);
    }
    
    if let Some(value) = card.get("pointValue").and_then(|v| v.as_i64()) {
        return Ok(value);
    }
    
    // Try to parse rank for standard playing cards
    if let Some(rank) = card.get("rank").and_then(|r| r.as_str()) {
        return match rank {
            "A" => Ok(1),
            "2" => Ok(2),
            "3" => Ok(3),
            "4" => Ok(4),
            "5" => Ok(5),
            "6" => Ok(6),
            "7" => Ok(7),
            "8" => Ok(8),
            "9" => Ok(9),
            "10" => Ok(10),
            "J" => Ok(11),
            "Q" => Ok(12),
            "K" => Ok(13),
            _ => Err(format!("Unknown rank: {}", rank))
        };
    }
    
    Err("Card has no comparable value".to_string())
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
        }
    }

    #[test]
    fn test_apply_draw_basic() {
        let mut state = json!({
            "game": {},
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
            "game": {},
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
    fn test_apply_compare_cards_equal_with_ranks() {
        let mut state = json!({
            "game": {},
            "zones": {
                "battle_p1": {
                    "items": [{"rank": "K", "suit": "hearts"}]
                },
                "battle_p2": {
                    "items": [{"rank": "K", "suit": "spades"}]
                }
            }
        });

        let args = json!({
            "card1": "/zones/battle_p1/items/0",
            "card2": "/zones/battle_p2/items/0"
        });

        let result = apply_compare_cards_equal(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 1);
        
        // Check the patch content directly
        let patch = &patches[0];
        assert_eq!(patch["path"], "/game/lastComparison");
        let comparison = patch["value"].as_object().unwrap();
        assert_eq!(comparison["equal"], true);
        assert_eq!(comparison["card1Value"], 13);
        assert_eq!(comparison["card2Value"], 13);
    }

    #[test]
    fn test_apply_compare_cards_equal_with_values() {
        let mut state = json!({
            "game": {},
            "zones": {
                "battle_p1": {
                    "items": [{"value": 7}]
                },
                "battle_p2": {
                    "items": [{"value": 10}]
                }
            }
        });

        let args = json!({
            "card1": "/zones/battle_p1/items/0",
            "card2": "/zones/battle_p2/items/0"
        });

        let result = apply_compare_cards_equal(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 1);
        
        // Check the patch content directly
        let patch = &patches[0];
        assert_eq!(patch["path"], "/game/lastComparison");
        let comparison = patch["value"].as_object().unwrap();
        assert_eq!(comparison["equal"], false);
        assert_eq!(comparison["card1Value"], 7);
        assert_eq!(comparison["card2Value"], 10);
    }

    #[test]
    fn test_apply_compare_cards_greater() {
        let mut state = json!({
            "game": {},
            "zones": {
                "battle_p1": {
                    "items": [{"rank": "A", "suit": "hearts"}]
                },
                "battle_p2": {
                    "items": [{"rank": "K", "suit": "spades"}]
                }
            }
        });

        let args = json!({
            "card1": "/zones/battle_p1/items/0",
            "card2": "/zones/battle_p2/items/0"
        });

        let result = apply_compare_cards_greater(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 1);
        
        // Check the patch content directly
        let patch = &patches[0];
        assert_eq!(patch["path"], "/game/lastComparison");
        let comparison = patch["value"].as_object().unwrap();
        assert_eq!(comparison["greater"], false); // Ace (1) is not greater than King (13)
        assert_eq!(comparison["card1Value"], 1);
        assert_eq!(comparison["card2Value"], 13);
    }

    #[test]
    fn test_apply_compare_cards_greater_true_case() {
        let mut state = json!({
            "game": {},
            "zones": {
                "battle_p1": {
                    "items": [{"rank": "Q", "suit": "hearts"}]
                },
                "battle_p2": {
                    "items": [{"rank": "5", "suit": "spades"}]
                }
            }
        });

        let args = json!({
            "card1": "/zones/battle_p1/items/0",
            "card2": "/zones/battle_p2/items/0"
        });

        let result = apply_compare_cards_greater(&mut state, &args);
        assert!(result.is_ok());
        
        let patches = result.unwrap();
        assert_eq!(patches.len(), 1);
        
        // Check the patch content directly
        let patch = &patches[0];
        assert_eq!(patch["path"], "/game/lastComparison");
        let comparison = patch["value"].as_object().unwrap();
        assert_eq!(comparison["greater"], true); // Queen (12) is greater than 5
        assert_eq!(comparison["card1Value"], 12);
        assert_eq!(comparison["card2Value"], 5);
    }

    #[test]
    fn test_get_card_value_with_point_value() {
        let card = json!({"pointValue": 25});
        let result = get_card_value(&card);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 25);
    }

    #[test]
    fn test_get_card_value_invalid_rank() {
        let card = json!({"rank": "X"});
        let result = get_card_value(&card);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown rank: X"));
    }

    #[test]
    fn test_get_card_value_no_value() {
        let card = json!({"suit": "hearts"});
        let result = get_card_value(&card);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Card has no comparable value"));
    }
}
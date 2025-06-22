//! Deal verb - deals cards from one zone to multiple zones

use serde_json::{json, Value};

pub fn execute(args: &Value, state: &mut Value) -> Result<Vec<Value>, String> {
    let from_zone = args["from"].as_str()
        .ok_or("deal requires 'from' zone")?;
    let to_zones = args["to"].as_array()
        .ok_or("deal requires 'to' array of zones")?;
    let count = args["count"].as_u64()
        .ok_or("deal requires 'count' number")? as usize;
    
    let mut patches = Vec::new();
    
    // Collect cards to deal
    let mut cards_to_deal = Vec::new();
    {
        let source_items = state["zones"][from_zone]["items"].as_array_mut()
            .ok_or_else(|| format!("Zone {} has no items", from_zone))?;
        
        let total_to_deal = count * to_zones.len();
        for _ in 0..total_to_deal.min(source_items.len()) {
            cards_to_deal.push(source_items.remove(0));
        }
    }
    
    // Generate patches for removed cards
    for (idx, card) in cards_to_deal.iter().enumerate() {
        patches.push(json!({
            "op": "remove", 
            "path": format!("/zones/{}/items/0", from_zone),
            "value": card
        }));
    }
    
    // Deal cards round-robin to target zones
    let mut card_iter = cards_to_deal.into_iter();
    for _ in 0..count {
        for to_zone_val in to_zones.iter() {
            let to_zone = to_zone_val.as_str()
                .ok_or("to zones must be strings")?;
            
            if let Some(card) = card_iter.next() {
                // Add to destination zone
                if let Some(dest_items) = state["zones"][to_zone]["items"].as_array_mut() {
                    dest_items.push(card.clone());
                    
                    patches.push(json!({
                        "op": "add",
                        "path": format!("/zones/{}/items/-", to_zone),
                        "value": card
                    }));
                } else {
                    return Err(format!("Zone {} has no items array", to_zone));
                }
            } else {
                break;
            }
        }
    }
    
    Ok(patches)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deal_cards() {
        let mut state = json!({
            "zones": {
                "deck": {
                    "items": ["card1", "card2", "card3", "card4", "card5", "card6"]
                },
                "p1_hand": {
                    "items": []
                },
                "p2_hand": {
                    "items": []
                }
            }
        });
        
        let args = json!({
            "from": "deck",
            "to": ["p1_hand", "p2_hand"],
            "count": 2
        });
        
        let patches = execute(&args, &mut state).unwrap();
        
        // Should have dealt 2 cards to each player
        assert_eq!(state["zones"]["p1_hand"]["items"].as_array().unwrap().len(), 2);
        assert_eq!(state["zones"]["p2_hand"]["items"].as_array().unwrap().len(), 2);
        assert_eq!(state["zones"]["deck"]["items"].as_array().unwrap().len(), 2);
    }
}
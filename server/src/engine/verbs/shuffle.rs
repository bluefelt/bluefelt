//! Shuffle verb - shuffles items in a zone

use serde_json::{json, Value};
use rand::seq::SliceRandom;
use rand::Rng;

pub fn execute(args: &Value, state: &mut Value) -> Result<Vec<Value>, String> {
    let zone_id = args["zone"].as_str()
        .ok_or("shuffle requires 'zone' string")?;
    
    let zone = state["zones"].get_mut(zone_id)
        .ok_or_else(|| format!("Zone not found: {}", zone_id))?;
    
    if let Some(items) = zone.get_mut("items").and_then(|i| i.as_array_mut()) {
        let mut rng = rand::rng();
        items.shuffle(&mut rng);
        
        // Generate patch with new order
        let patch = json!({
            "op": "replace",
            "path": format!("/zones/{}/items", zone_id),
            "value": items.clone()
        });
        
        Ok(vec![patch])
    } else {
        Err(format!("Zone {} does not have items to shuffle", zone_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shuffle_deck() {
        let mut state = json!({
            "zones": {
                "deck": {
                    "items": ["card1", "card2", "card3", "card4"]
                }
            }
        });
        
        let args = json!({
            "zone": "deck"
        });
        
        let original_items = state["zones"]["deck"]["items"].clone();
        let patches = execute(&args, &mut state).unwrap();
        
        assert_eq!(patches.len(), 1);
        // Items should still contain same elements (just reordered)
        let new_items = &state["zones"]["deck"]["items"];
        assert_eq!(new_items.as_array().unwrap().len(), 4);
    }
}
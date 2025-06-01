use serde_json::{json, Value};

fn main() {
    // Test zone with deck shape
    let zone = json!({
        "id": "drawPile",
        "shape": "deck",
        "contents": ["card_hearts_a", "card_hearts_2"]
    });
    
    // Test what the init_list function would produce
    let contents = &zone["contents"];
    let result = if let Some(arr) = contents.as_array() {
        json!({"items": arr.clone()})
    } else {
        json!({"items": []})
    };
    
    println!("Zone: {}", serde_json::to_string_pretty(&zone).unwrap());
    println!("Initialized as: {}", serde_json::to_string_pretty(&result).unwrap());
    
    // Test with standardDeck contents after expansion
    let expanded_contents = json!([
        "card_hearts_a", "card_hearts_2", "card_hearts_3", "card_hearts_4",
        "card_hearts_5", "card_hearts_6", "card_hearts_7", "card_hearts_8",
        "card_hearts_9", "card_hearts_10", "card_hearts_j", "card_hearts_q", "card_hearts_k"
        // ... and so on for all 52 cards
    ]);
    
    let deck_result = json!({"items": expanded_contents.as_array().unwrap().clone()});
    println!("\nStandard deck would initialize with {} cards", expanded_contents.as_array().unwrap().len());
}
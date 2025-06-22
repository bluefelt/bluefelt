//! Clear selection verb - resets selection state

use serde_json::{json, Value};

pub fn execute(state: &mut Value) -> Result<Vec<Value>, String> {
    // Generate patch
    let patch = json!({
        "op": "replace",
        "path": "/selection",
        "value": {}
    });
    
    // Clear selection
    state["selection"] = json!({});
    
    Ok(vec![patch])
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clear_selection() {
        let mut state = json!({
            "selection": {
                "selectedPiece": "p1_piece",
                "targetZone": "board"
            }
        });
        
        let patches = execute(&mut state).unwrap();
        assert_eq!(patches.len(), 1);
        assert_eq!(state["selection"], json!({}));
    }
}
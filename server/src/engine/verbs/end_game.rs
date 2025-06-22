//! End game verb - uses simplified gameStatus format

use serde_json::{json, Value};

pub fn execute(args: &Value, state: &mut Value) -> Result<Vec<Value>, String> {
    // Determine the game ending state
    let game_status = if let Some(winner) = args.get("winner").and_then(|w| w.as_str()) {
        format!("won:{}", winner)
    } else if args.get("tie").and_then(|t| t.as_bool()).unwrap_or(false) {
        "tie".to_string()
    } else if args.get("abandoned").and_then(|a| a.as_bool()).unwrap_or(false) {
        "abandoned".to_string()
    } else {
        "ended".to_string()
    };
    
    // Generate patch
    let patch = json!({
        "op": "replace",
        "path": "/gameStatus",
        "value": game_status
    });
    
    // Update state
    state["gameStatus"] = json!(game_status);
    
    Ok(vec![patch])
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_end_game_with_winner() {
        let mut state = json!({
            "gameStatus": "playing"
        });
        
        let args = json!({
            "winner": "p2"
        });
        
        let patches = execute(&args, &mut state).unwrap();
        assert_eq!(patches.len(), 1);
        assert_eq!(state["gameStatus"], "won:p2");
    }
    
    #[test]
    fn test_end_game_tie() {
        let mut state = json!({
            "gameStatus": "playing"
        });
        
        let args = json!({
            "tie": true
        });
        
        let patches = execute(&args, &mut state).unwrap();
        assert_eq!(patches.len(), 1);
        assert_eq!(state["gameStatus"], "tie");
    }
    
    #[test]
    fn test_end_game_abandoned() {
        let mut state = json!({
            "gameStatus": "playing"
        });
        
        let args = json!({
            "abandoned": true
        });
        
        let patches = execute(&args, &mut state).unwrap();
        assert_eq!(patches.len(), 1);
        assert_eq!(state["gameStatus"], "abandoned");
    }
}
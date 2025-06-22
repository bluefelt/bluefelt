//! Set property verb - handles simplified state structure

use serde_json::{json, Value};

pub fn execute(args: &Value, state: &mut Value) -> Result<Vec<Value>, String> {
    let path = args["path"].as_str()
        .ok_or("setProperty requires 'path' string")?;
    let value = args.get("value")
        .ok_or("setProperty requires 'value'")?;
    
    // Generate patch first (before mutation)
    let patch = json!({
        "op": "replace",
        "path": path,
        "value": value
    });
    
    // Apply to state
    set_value_at_path(state, path, value.clone())?;
    
    Ok(vec![patch])
}

/// Set a value at a JSON pointer path
fn set_value_at_path(state: &mut Value, path: &str, value: Value) -> Result<(), String> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    
    let mut current = state;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let Some(obj) = current.as_object_mut() {
                obj.insert(part.to_string(), value);
                return Ok(());
            } else {
                return Err(format!("Cannot set property on non-object at path: {}", path));
            }
        } else {
            // Navigate deeper
            current = current.get_mut(part)
                .ok_or_else(|| format!("Path not found: {}", path))?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_set_game_status() {
        let mut state = json!({
            "gameStatus": "playing",
            "currentPlayer": "p1"
        });
        
        let args = json!({
            "path": "/gameStatus",
            "value": "won:p1"
        });
        
        let patches = execute(&args, &mut state).unwrap();
        assert_eq!(patches.len(), 1);
        assert_eq!(state["gameStatus"], "won:p1");
    }
    
    #[test]
    fn test_set_nested_property() {
        let mut state = json!({
            "zones": {
                "deck": {
                    "items": []
                }
            }
        });
        
        let args = json!({
            "path": "/zones/deck/items",
            "value": ["card1", "card2"]
        });
        
        let patches = execute(&args, &mut state).unwrap();
        assert_eq!(patches.len(), 1);
        assert_eq!(state["zones"]["deck"]["items"], json!(["card1", "card2"]));
    }
}
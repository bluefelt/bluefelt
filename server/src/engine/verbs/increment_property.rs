//! Increment property verb

use serde_json::{json, Value};

pub fn execute(args: &Value, state: &mut Value) -> Result<Vec<Value>, String> {
    let path = args["path"].as_str()
        .ok_or("incrementProperty requires 'path' string")?;
    let amount = args.get("amount")
        .and_then(|a| a.as_i64())
        .unwrap_or(1);
    
    // Get current value
    let current = get_value_at_path(state, path)?;
    let current_num = current.as_i64()
        .ok_or_else(|| format!("Value at {} is not a number", path))?;
    
    let new_value = current_num + amount;
    
    // Generate patch
    let patch = json!({
        "op": "replace",
        "path": path,
        "value": new_value
    });
    
    // Update state
    set_value_at_path(state, path, json!(new_value))?;
    
    Ok(vec![patch])
}

fn get_value_at_path<'a>(state: &'a Value, path: &str) -> Result<&'a Value, String> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    
    let mut current = state;
    for part in parts {
        current = current.get(part)
            .ok_or_else(|| format!("Path not found: {}", path))?;
    }
    
    Ok(current)
}

fn set_value_at_path(state: &mut Value, path: &str, value: Value) -> Result<(), String> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    
    let mut current = state;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Some(obj) = current.as_object_mut() {
                obj.insert(part.to_string(), value);
                return Ok(());
            }
        } else {
            current = current.get_mut(part)
                .ok_or_else(|| format!("Path not found: {}", path))?;
        }
    }
    
    Ok(())
}
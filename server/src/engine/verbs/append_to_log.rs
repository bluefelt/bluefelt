//! Append to log verb - adds entries to game log

use serde_json::{json, Value};

pub fn execute(args: &Value, state: &mut Value) -> Result<Vec<Value>, String> {
    let message = args["message"].as_str()
        .ok_or("appendToLog requires 'message' string")?;
    let log_type = args.get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("info");
    
    // Ensure gameLog exists
    if state.get("gameLog").is_none() {
        state["gameLog"] = json!([]);
    }
    
    let log_entry = json!({
        "message": message,
        "type": log_type,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    // Generate patch
    let patch = json!({
        "op": "add",
        "path": "/gameLog/-",
        "value": log_entry.clone()
    });
    
    // Add to state
    state["gameLog"].as_array_mut()
        .unwrap()
        .push(log_entry);
    
    Ok(vec![patch])
}
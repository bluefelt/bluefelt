//! Set phase verb - works with simplified phase structure

use serde_json::{json, Value};

pub fn execute(args: &Value, state: &mut Value) -> Result<Vec<Value>, String> {
    let phase_set = args["phaseSet"].as_str()
        .ok_or("setPhase requires 'phaseSet' string")?;
    let phase_id = args["phase"].as_str()
        .ok_or("setPhase requires 'phase' string")?;
    
    // Check if using enhanced phase system (has 'current' object)
    let is_enhanced = state["phases"].get("current").is_some();
    
    let patch = if is_enhanced {
        // Enhanced phase system - update current phases
        state["phases"]["current"][phase_set] = json!(phase_id);
        json!({
            "op": "replace",
            "path": format!("/phases/current/{}", phase_set),
            "value": phase_id
        })
    } else {
        // Simple phase system - update directly
        state["phases"][phase_set] = json!(phase_id);
        json!({
            "op": "replace",
            "path": format!("/phases/{}", phase_set),
            "value": phase_id
        })
    };
    
    Ok(vec![patch])
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_set_phase() {
        let mut state = json!({
            "phases": {
                "game": "placement",
                "turn": "draw"
            }
        });
        
        let args = json!({
            "phaseSet": "game",
            "phase": "movement"
        });
        
        let patches = execute(&args, &mut state).unwrap();
        assert_eq!(patches.len(), 1);
        assert_eq!(state["phases"]["game"], "movement");
    }
}
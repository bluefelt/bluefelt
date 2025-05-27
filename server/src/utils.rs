use axum::{response::Json, http::StatusCode};
use serde_json::{json, Value};

/// Create a standard error response
pub fn error_response(message: &str) -> Json<Value> {
    Json(json!({ "error": message }))
}

/// Create a standard error response with status code
pub fn error_response_with_status(message: &str, status: StatusCode) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": message })))
}

/// Create a standard success response
pub fn success_response(data: Value) -> Json<Value> {
    Json(data)
}

/// Helper trait for broadcasting messages to WebSocket connections
pub trait BroadcastExt {
    async fn broadcast_json(&self, message: &Value);
}

/// Helper to safely navigate JSON paths
pub trait JsonNavigator {
    fn get_path(&self, path: &str) -> Option<&Value>;
    fn get_path_mut(&mut self, path: &str) -> Option<&mut Value>;
}

impl JsonNavigator for Value {
    fn get_path(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = self;
        
        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                Value::Array(arr) => {
                    let index = part.parse::<usize>().ok()?;
                    current = arr.get(index)?;
                }
                _ => return None,
            }
        }
        
        Some(current)
    }
    
    fn get_path_mut(&mut self, path: &str) -> Option<&mut Value> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = self;
        
        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get_mut(part)?;
                }
                Value::Array(arr) => {
                    let index = part.parse::<usize>().ok()?;
                    current = arr.get_mut(index)?;
                }
                _ => return None,
            }
        }
        
        Some(current)
    }
}

/// Helper to build consistent lobby info JSON
pub fn build_lobby_info(
    id: &str,
    game_id: &str,
    players: Vec<String>,
    started: bool,
) -> Value {
    json!({
        "id": id,
        "game_id": game_id,
        "players": players,
        "started": started,
    })
}

/// Helper to prefix JSON patch paths
pub fn prefix_patch_paths(patch: &mut Value, prefix: &str) {
    if let Some(patches) = patch.as_array_mut() {
        for p in patches {
            if let Some(path) = p.get_mut("path").and_then(|v| v.as_str()) {
                *p.get_mut("path").unwrap() = json!(format!("{}{}", prefix, path));
            }
        }
    }
}
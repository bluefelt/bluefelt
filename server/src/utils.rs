use axum::response::Json;
use serde_json::{json, Value};

/// Create a standard error response
pub fn error_response(message: &str) -> Json<Value> {
    Json(json!({ "error": message }))
}

// NOTE: The following functions were removed as they were unused:
// - error_response_with_status
// - success_response  
// - BroadcastExt trait
// - JsonNavigator trait
// - build_lobby_info
// - prefix_patch_paths
// If these are needed in the future, they can be found in git history.
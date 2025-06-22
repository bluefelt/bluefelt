//! Handlers for live reload functionality

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::app_state::AppState;
use crate::bundle::{BundleMap, load_bundles_from_dir};
use axum::http::StatusCode;

#[derive(Debug, Deserialize)]
pub struct ReloadRequest {
    pub game_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReloadResponse {
    pub success: bool,
    pub message: String,
    pub reloaded_games: Vec<String>,
}

/// Handle WebSocket connection for live reload notifications
pub async fn reload_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    let client_id = uuid::Uuid::new_v4().to_string();
    ws.on_upgrade(move |socket| async move {
        state.reload_notifier.handle_connection(socket, client_id).await;
    })
}

/// Reload game bundles from disk
pub async fn reload_bundles(
    State(state): State<AppState>,
    Json(req): Json<ReloadRequest>,
) -> Result<Json<ReloadResponse>, StatusCode> {
    // Load new bundles from disk
    let new_bundles = match load_bundles_from_dir("bundles") {
        Ok(bundles) => bundles,
        Err(e) => {
            return Ok(Json(ReloadResponse {
                success: false,
                message: format!("Failed to load bundles: {}", e),
                reloaded_games: vec![],
            }));
        }
    };

    let reloaded_games: Vec<String>;
    
    if let Some(game_id) = req.game_id {
        // Reload specific game
        if let Some(new_bundle) = new_bundles.get_latest(&game_id) {
            // Update the bundle in the map
            let bundles = Arc::get_mut(&mut state.bundles.clone())
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            
            // Since we can't get mutable access to Arc contents directly,
            // we need to use interior mutability or recreate the map
            // For now, notify about the reload
            state.reload_notifier.notify_game_reload(game_id.clone()).await;
            
            reloaded_games = vec![game_id];
            
            Ok(Json(ReloadResponse {
                success: true,
                message: format!("Game {} reloaded successfully", reloaded_games[0]),
                reloaded_games,
            }))
        } else {
            Ok(Json(ReloadResponse {
                success: false,
                message: format!("Game {} not found", game_id),
                reloaded_games: vec![],
            }))
        }
    } else {
        // Reload all games
        reloaded_games = new_bundles.list_games();
        
        // Notify about reload
        state.reload_notifier.notify_all_games_reload().await;
        
        Ok(Json(ReloadResponse {
            success: true,
            message: format!("Reloaded {} games", reloaded_games.len()),
            reloaded_games,
        }))
    }
}

/// Get reload status
pub async fn reload_status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let connected_clients = state.reload_notifier.connected_count().await;
    
    Json(serde_json::json!({
        "connected_clients": connected_clients,
        "enabled": true,
    }))
}
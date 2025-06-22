//! WebSocket handlers for lobby connections

use axum::{
    extract::{ws::WebSocketUpgrade, Query, State, Path},
    response::Response,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    app_state::AppState,
    lobby::lobby_impl::Lobby,
    message_format::{MessageFormat, UpdateFormat},
};

#[derive(Deserialize)]
pub struct WsQuery {
    player: String,
    #[serde(default)]
    join: Option<bool>,
    #[serde(default = "default_message_format")]
    message_format: MessageFormat,
    #[serde(default = "default_update_format")]
    update_format: UpdateFormat,
    #[serde(default)]
    reconnection_token: Option<String>,
}

fn default_message_format() -> MessageFormat {
    MessageFormat::Standard
}

fn default_update_format() -> UpdateFormat {
    UpdateFormat::Patch
}

/// Handle WebSocket connection upgrade
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(lobby_id): Path<String>,
    Query(params): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    // Get existing lobby
    let lobby = match state.lobbies.get(&lobby_id) {
        Some(existing) => Arc::clone(&existing),
        None => {
            // Return error response if lobby doesn't exist
            return Response::builder()
                .status(404)
                .body("Lobby not found".into())
                .unwrap();
        }
    };
    
    // Check if this is a reconnection attempt
    let is_reconnection = if let Some(token) = &params.reconnection_token {
        if let Some(info) = state.connection_manager.validate_reconnection_token(token) {
            // Valid reconnection token
            if info.username != params.player || info.lobby_id != lobby_id {
                return Response::builder()
                    .status(401)
                    .body("Invalid reconnection token".into())
                    .unwrap();
            }
            true
        } else {
            return Response::builder()
                .status(401)
                .body("Invalid or expired reconnection token".into())
                .unwrap();
        }
    } else {
        false
    };
    
    // Check connection limits using ConnectionPool (skip for reconnections)
    if !is_reconnection {
        if let Err(_) = state.connection_manager.pool.try_acquire(&params.player) {
            return Response::builder()
                .status(429)
                .body("Too many connections for this user".into())
                .unwrap();
        }
    }
    
    // Check lobby capacity
    {
        let members = lobby.state.members.read();
        if members.len() >= 50 {  // Max connections per lobby
            return Response::builder()
                .status(429)
                .body("Lobby at maximum capacity".into())
                .unwrap();
        }
    }
    
    // Log connection attempt
    println!("[WS] Connection attempt from {} to lobby {}", params.player, lobby_id);
    
    // Upgrade to WebSocket
    let connection_manager = state.connection_manager.clone();
    let should_join = params.join.unwrap_or(false) || is_reconnection;
    ws.on_upgrade(move |socket| {
        crate::lobby::websocket::handle_websocket(
            socket,
            lobby,
            params.player,
            should_join,
            params.message_format,
            params.update_format,
            connection_manager,
        )
    })
}
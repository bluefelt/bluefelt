use axum::{
    extract::{Path, WebSocketUpgrade},
    response::IntoResponse,
};
use axum::extract::ws::Message;
use std::sync::Arc;
use tokio::sync::broadcast;
use futures_util::{SinkExt, StreamExt};

use crate::lobby::{LobbyMap, current_lobbies_json};

pub async fn lobbies_ws_handler(
    ws: WebSocketUpgrade,
    lobby_updates: broadcast::Sender<Message>,
    lobbies: Arc<LobbyMap>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (mut tx, mut rx) = socket.split();

        // send initial lobby list
        let list = current_lobbies_json(&lobbies);
        let _ = tx.send(Message::Text(list.to_string())).await;

        let mut updates = lobby_updates.subscribe();
        tokio::spawn(async move {
            while let Ok(msg) = updates.recv().await {
                if tx.send(msg.clone()).await.is_err() {
                    break;
                }
            }
        });

        // ignore incoming messages
        while let Some(Ok(_)) = rx.next().await {}
    })
}

pub async fn ws_handler(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    lobbies: Arc<LobbyMap>,
) -> impl IntoResponse {
    // Extract player_id from query params, default to a random ID if missing
    let player_id = params.get("player_id").cloned().unwrap_or_else(|| {
        format!("guest_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap())
    });
    let join = params.get("join").map(|v| v != "0" && v != "false").unwrap_or(true);
    let since = params.get("since").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    
    // Message format options for clients with limited JSON support
    let format = params.get("format").cloned().unwrap_or_else(|| "standard".to_string());
    let updates = params.get("updates").cloned().unwrap_or_else(|| "patch".to_string());
    
    println!("[Socket] Connection request from player {} for lobby {}", player_id, id);
    
    let Some(lobby_ref) = lobbies.get(&id) else {
        println!("[Socket] ERROR: Attempt to join non-existent lobby: {}", id);
        return ws.on_upgrade(|mut sock| async move {
            let _ = sock.send(Message::Text(serde_json::json!({
                "type": "error",
                "message": "Lobby does not exist"
            }).to_string())).await;
            // Close the socket immediately after sending the error with policy violation code
            let _ = sock.send(Message::Close(Some(axum::extract::ws::CloseFrame {
                code: 1008, // Policy violation - lobby doesn't exist
                reason: std::borrow::Cow::from("Lobby does not exist"),
            }))).await;
            let _ = sock.close().await;
        });
    };
    
    // Clone the lobby Arc to avoid holding the DashMap entry
    let lobby = lobby_ref.clone();                 
    
    // Debug existing lobby state
    println!("[Socket] Current lobby state - ID: {}, Players: {:?}, Started: {}", 
        lobby.id, 
        lobby.player_list(),
        lobby.is_started()
    );
    
    ws.on_upgrade(move |sock| async move {
        println!("[Socket] WebSocket connections successful for player {} in lobby {} (format: {}, updates: {})", player_id, id, format, updates);
        lobby.accept_client(sock, player_id, join, since, format, updates).await;
    })
}
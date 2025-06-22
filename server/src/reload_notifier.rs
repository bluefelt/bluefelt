//! Live reload notification system for development

use axum::extract::ws::{WebSocket, Message};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use std::collections::HashSet;

/// Manages live reload notifications to connected clients
#[derive(Clone)]
pub struct ReloadNotifier {
    tx: broadcast::Sender<ReloadEvent>,
    connected_clients: Arc<RwLock<HashSet<String>>>,
}

#[derive(Debug, Clone)]
pub enum ReloadEvent {
    GameReloaded { game_id: String },
    AllGamesReloaded,
}

impl ReloadNotifier {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            connected_clients: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Notify all connected clients that a game has been reloaded
    pub async fn notify_game_reload(&self, game_id: String) {
        let _ = self.tx.send(ReloadEvent::GameReloaded { game_id });
    }

    /// Notify all connected clients that all games have been reloaded
    pub async fn notify_all_games_reload(&self) {
        let _ = self.tx.send(ReloadEvent::AllGamesReloaded);
    }

    /// Handle a new reload WebSocket connection
    pub async fn handle_connection(&self, socket: WebSocket, client_id: String) {
        let (mut sender, mut receiver) = socket.split();
        let mut rx = self.tx.subscribe();
        
        // Add to connected clients
        {
            let mut clients = self.connected_clients.write().await;
            clients.insert(client_id.clone());
        }

        // Send initial connection confirmation
        let _ = sender.send(Message::Text(
            serde_json::json!({
                "type": "connected",
                "message": "Live reload connected"
            }).to_string()
        )).await;

        // Spawn task to forward reload events
        let reload_notifier = self.clone();
        let client_id_clone = client_id.clone();
        let mut send_task = tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let msg = match event {
                    ReloadEvent::GameReloaded { game_id } => {
                        serde_json::json!({
                            "type": "gameReloaded",
                            "gameId": game_id,
                            "message": format!("Game {} reloaded", game_id)
                        })
                    }
                    ReloadEvent::AllGamesReloaded => {
                        serde_json::json!({
                            "type": "allGamesReloaded",
                            "message": "All games reloaded"
                        })
                    }
                };
                
                if sender.send(Message::Text(msg.to_string())).await.is_err() {
                    break;
                }
            }
        });

        // Handle incoming messages (mainly for ping/pong)
        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(txt) => {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                            if json["type"] == "ping" {
                                // Respond to ping (keep-alive)
                                continue;
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        }

        // Remove from connected clients
        {
            let mut clients = self.connected_clients.write().await;
            clients.remove(&client_id);
        }
    }

    /// Get count of connected reload clients
    pub async fn connected_count(&self) -> usize {
        self.connected_clients.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reload_notifier_creation() {
        let notifier = ReloadNotifier::new();
        assert_eq!(notifier.connected_count().await, 0);
    }

    #[tokio::test]
    async fn test_reload_events() {
        let notifier = ReloadNotifier::new();
        let mut rx = notifier.tx.subscribe();
        
        // Send game reload event
        notifier.notify_game_reload("tic-tac-toe".to_string()).await;
        
        // Verify event received
        if let Ok(ReloadEvent::GameReloaded { game_id }) = rx.recv().await {
            assert_eq!(game_id, "tic-tac-toe");
        } else {
            panic!("Expected GameReloaded event");
        }
        
        // Send all games reload event
        notifier.notify_all_games_reload().await;
        
        // Verify event received
        if let Ok(ReloadEvent::AllGamesReloaded) = rx.recv().await {
            // Success
        } else {
            panic!("Expected AllGamesReloaded event");
        }
    }
}
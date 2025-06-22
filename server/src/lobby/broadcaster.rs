//! Lobby list broadcaster - handles serialized broadcasting of lobby list updates

use super::{LobbyMap, current_lobbies_json};
use axum::extract::ws::Message;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Request to broadcast the lobby list
#[derive(Debug)]
pub struct BroadcastRequest {
    pub lobby_id: String,
    pub requested_at: Instant,
}

/// Handles serialized broadcasting of lobby list updates to avoid contention
pub struct LobbyListBroadcaster {
    lobbies: Arc<LobbyMap>,
    lobby_updates: broadcast::Sender<Message>,
    request_tx: mpsc::Sender<BroadcastRequest>,
    request_rx: Option<mpsc::Receiver<BroadcastRequest>>,
}

impl LobbyListBroadcaster {
    /// Create a new broadcaster
    pub fn new(lobbies: Arc<LobbyMap>, lobby_updates: broadcast::Sender<Message>) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<BroadcastRequest>(100);
        
        Self {
            lobbies,
            lobby_updates,
            request_tx,
            request_rx: Some(request_rx),
        }
    }
    
    /// Get a sender for broadcast requests
    pub fn get_sender(&self) -> mpsc::Sender<BroadcastRequest> {
        self.request_tx.clone()
    }
    
    /// Request a lobby list broadcast
    pub async fn request_broadcast(&self, lobby_id: String) {
        let request = BroadcastRequest {
            lobby_id,
            requested_at: Instant::now(),
        };
        
        // Non-blocking send, ignore if channel is full
        let _ = self.request_tx.try_send(request);
    }
    
    /// Start the broadcaster task
    pub fn start(mut self) -> JoinHandle<()> {
        let request_rx = self.request_rx.take().expect("Broadcaster already started");
        
        tokio::spawn(async move {
            self.run(request_rx).await;
        })
    }
    
    /// Main loop for processing broadcast requests
    async fn run(&self, mut request_rx: mpsc::Receiver<BroadcastRequest>) {
        const MIN_BROADCAST_INTERVAL: Duration = Duration::from_millis(100);
        const BATCH_TIMEOUT: Duration = Duration::from_millis(50);
        
        let mut last_broadcast = Instant::now() - MIN_BROADCAST_INTERVAL;
        let mut pending_lobbies = std::collections::HashSet::new();
        
        loop {
            // Collect requests for a short period or until we get at least one
            let deadline = Instant::now() + BATCH_TIMEOUT;
            
            // Collect all pending requests
            loop {
                match tokio::time::timeout_at(deadline.into(), request_rx.recv()).await {
                    Ok(Some(request)) => {
                        pending_lobbies.insert(request.lobby_id);
                    }
                    Ok(None) => {
                        // Channel closed, exit
                        println!("[Broadcaster] Request channel closed, exiting");
                        return;
                    }
                    Err(_) => {
                        // Timeout reached, process what we have
                        break;
                    }
                }
                
                // If we have pending requests and haven't collected any more in a while, process them
                if !pending_lobbies.is_empty() && request_rx.is_empty() {
                    break;
                }
            }
            
            // Process pending broadcasts if we have any
            if !pending_lobbies.is_empty() {
                let now = Instant::now();
                
                // Enforce minimum interval between broadcasts
                if now.duration_since(last_broadcast) < MIN_BROADCAST_INTERVAL {
                    let sleep_duration = MIN_BROADCAST_INTERVAL - now.duration_since(last_broadcast);
                    tokio::time::sleep(sleep_duration).await;
                }
                
                // Perform the broadcast
                self.broadcast_lobby_list().await;
                
                last_broadcast = Instant::now();
                pending_lobbies.clear();
                
                println!("[Broadcaster] Broadcasted lobby list update");
            }
        }
    }
    
    /// Perform the actual broadcast
    async fn broadcast_lobby_list(&self) {
        // Compute lobby list without holding locks during network I/O
        let list = current_lobbies_json(&self.lobbies);
        
        // Send to all subscribers
        match self.lobby_updates.send(Message::Text(list.to_string())) {
            Ok(count) => {
                println!("[Broadcaster] Sent lobby list to {} subscribers", count);
            }
            Err(_) => {
                // No subscribers, this is fine
            }
        }
    }
}
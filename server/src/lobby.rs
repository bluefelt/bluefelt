//! lobby.rs – minimal in-memory lobby with broadcast fan-out
//! Supports: welcome snapshot → JSON verb → diff broadcast

use crate::{bundle::Bundle, engine};
use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex as TokioMutex};
use chrono;

pub type LobbyMap = DashMap<String, Arc<Lobby>>;

/* --------------------------------------------------------------------------
   constructor helper
   ----------------------------------------------------------------------- */
pub fn new_lobby(id: String, bundle: Bundle) -> Arc<Lobby> {
    Arc::new(Lobby::new(id, bundle))
}

/* --------------------------------------------------------------------------
   Lobby struct
   ----------------------------------------------------------------------- */
pub struct Lobby {
    pub id: String,
    pub bundle: Bundle,

    /// authoritative mutable state (JSON object)
    state: Mutex<serde_json::Value>,

    /// broadcast channel for diff events
    tx: broadcast::Sender<Message>,
    
    /// Track connected players
    players: Mutex<Vec<String>>,
    
    /// Game has started flag
    game_started: Mutex<bool>,
}

impl Lobby {
    pub fn new(id: String, bundle: Bundle) -> Self {
        let initial = engine::load_initial_state(&bundle);
        let (tx, _) = broadcast::channel(64);
        Self {
            id,
            bundle,
            state: Mutex::new(initial),
            tx,
            players: Mutex::new(Vec::new()),
            game_started: Mutex::new(false),
        }
    }

    pub fn players(&self) -> usize {
        // Return the actual player count instead of subscribers
        let players = self.players.lock();
        players.len()
    }
    
    pub fn player_list(&self) -> Vec<String> {
        // Return a copy of the player list
        let players = self.players.lock();
        players.clone()
    }

    pub fn add_player(&self, player_id: String) -> bool {
        let mut players = self.players.lock();
        
        // If this is the same player reconnecting, allow it
        if players.contains(&player_id) {
            println!("[Socket] Player {} is reconnecting to the lobby", player_id);
            return true;
        }
        
        // Check if we already have 2 players (max for tic-tac-toe)
        if players.len() < 2 {
            println!("[Socket] Adding new player {} to the lobby", player_id);
            players.push(player_id);
            // If we now have 2 players, start the game
            if players.len() == 2 {
                *self.game_started.lock() = true;
                println!("[Socket] Two players joined, starting the game!");
            }
            return true;
        }
        
        println!("Could not add player {} - lobby is full", player_id);
        false
    }

    /// Optional method to remove a player - normally not needed as disconnections are handled implicitly
    pub fn remove_player(&self, player_id: &str) -> bool {
        let mut players = self.players.lock();
        let before_len = players.len();
        players.retain(|id| id != player_id);
        
        if players.len() < before_len {
            println!("[Socket] Player {} removed from lobby", player_id);
            return true;
        }
        
        println!("[Socket] ERROR: Player {} was not in the lobby and could not be removed", player_id);
        false
    }

    /// Check if the game has started
    pub fn is_started(&self) -> bool {
        *self.game_started.lock()
    }

    /// Accept a new WebSocket client, drive send/recv loops.
    pub async fn accept_client(self: Arc<Self>, socket: WebSocket) {
        // --- split socket ---------------------------------------------------
        let (sink_raw, mut stream) = socket.split();
        let sink = Arc::new(TokioMutex::new(sink_raw)); // make clonable
        
        // --- 1️⃣ send welcome message regardless of game state ------------------------------------
        let is_game_started = *self.game_started.lock();
        let player_id = self.players.lock().last().cloned().unwrap_or_else(|| "unknown".to_string());
        
        println!("[Socket] WebSocket client connected for player: {}", player_id);
        
        // Send information about lobby state first
        {
            let mut locked = sink.lock().await;
            
            if is_game_started {
                // Game has started, send the full game state
                let snapshot = {
                    let guard = self.state.lock();
                    guard.clone()
                };
                let welcome = serde_json::json!({
                    "type": "welcome",
                    "bundleMeta": {
                        "cards": {},
                        "verbs": {
                            "place": {
                                "ui": {
                                    "prompt": "Place your mark",
                                    "paramPrompts": {
                                        "row": "Row (0-2)",
                                        "col": "Column (0-2)"
                                    }
                                }
                            }
                        }
                    },
                    "initialState": snapshot
                });
                
                println!("[Socket] Sending welcome message to player: {}", player_id);
                if let Err(e) = locked.send(Message::Text(welcome.to_string())).await {
                    println!("[Socket] ERROR: Error sending welcome message: {}", e);
                    return;
                }
                
                // Send legal moves for the initial game state
                let legal_moves = serde_json::json!({
                    "type": "legalMoves",
                    "verbs": [
                        {
                            "verb": "place",
                            "params": {
                                "row": "u8",
                                "col": "u8"
                            }
                        }
                    ]
                });
                println!("[Socket] Sending legal moves to player: {}", player_id);
                if let Err(e) = locked.send(Message::Text(legal_moves.to_string())).await {
                    println!("[Socket] ERROR: Error sending legal moves: {}", e);
                    return;
                }
            } else {
                // Game not started yet, send waiting message
                let waiting_msg = serde_json::json!({
                    "type": "info",
                    "message": "Waiting for another player to join..."
                });
                println!("[Socket] Sending waiting message to player: {}", player_id);
                if let Err(e) = locked.send(Message::Text(waiting_msg.to_string())).await {
                    println!("[Socket] ERROR: Error sending waiting message: {}", e);
                    return;
                }
            }
        }

        /* spawn task to forward broadcast events */
        let forward_handle;
        {
            let mut rx = self.tx.subscribe();
            let sink_clone = sink.clone();
            let player_id_clone = player_id.clone();
            let self_clone = self.clone();
            
            // Set up forward task
            forward_handle = tokio::spawn(async move {
                // Also watch for game start
                let mut last_game_started = is_game_started;
                
                loop {
                    // Check if game started state changed
                    let curr_game_started = *self_clone.game_started.lock();
                    if !last_game_started && curr_game_started {
                        // Game just started, send welcome message with game state
                        let snapshot = {
                            let guard = self_clone.state.lock();
                            guard.clone()
                        };
                        let welcome = serde_json::json!({
                            "type": "welcome",
                            "bundleMeta": {
                                "cards": {},
                                "verbs": {
                                    "place": {
                                        "ui": {
                                            "prompt": "Place your mark",
                                            "paramPrompts": {
                                                "row": "Row (0-2)",
                                                "col": "Column (0-2)"
                                            }
                                        }
                                    }
                                }
                            },
                            "initialState": snapshot
                        });
                        
                        // Use a different approach to avoid borrow checker issues
                        let sink_for_welcome = sink_clone.clone();
                        let lock_attempt = tokio::time::timeout(
                            tokio::time::Duration::from_millis(500), 
                            sink_for_welcome.lock()
                        ).await;
                        
                        match lock_attempt {
                            Ok(mut locked) => {
                                println!("[Socket] Game started! Sending welcome message to player: {}", player_id_clone);
                                if let Err(e) = locked.send(Message::Text(welcome.to_string())).await {
                                    println!("[Socket] ERROR: Error sending welcome message on game start: {}", e);
                                    return;
                                }
                                
                                // Send legal moves
                                let legal_moves = serde_json::json!({
                                    "type": "legalMoves",
                                    "verbs": [
                                        {
                                            "verb": "place",
                                            "params": {
                                                "row": "u8",
                                                "col": "u8"
                                            }
                                        }
                                    ]
                                });
                                println!("[Socket] Sending legal moves to player: {}", player_id_clone);
                                if let Err(e) = locked.send(Message::Text(legal_moves.to_string())).await {
                                    println!("[Socket] ERROR: Error sending legal moves on game start: {}", e);
                                    return;
                                }
                                
                                last_game_started = curr_game_started;
                            },
                            Err(_) => {
                                println!("[Socket] ERROR: Timeout acquiring lock for welcome message to player: {}", player_id_clone);
                                return;
                            }
                        }
                    }
                    
                    // Wait for messages or timeout
                    tokio::select! {
                        Ok(msg) = rx.recv() => {
                            // Use a different approach to avoid borrow checker issues
                            let sink_for_broadcast = sink_clone.clone();
                            
                            // Do the timeout separately
                            let lock_attempt = tokio::time::timeout(
                                tokio::time::Duration::from_millis(500),
                                sink_for_broadcast.lock()
                            ).await;
                            
                            // Handle the result
                            match lock_attempt {
                                Ok(mut locked) => {
                                    if let Err(e) = locked.send(msg).await {
                                        println!("[Socket] ERROR: Error forwarding message to client: {}", e);
                                        return;
                                    }
                                },
                                Err(_) => {
                                    println!("[Socket] ERROR: Timeout acquiring lock for broadcast");
                                    return;
                                }
                            }
                        },
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                            // Periodic check for game start
                        }
                    }
                }
            });
        }

        // Send periodic pings to keep the connection alive
        let sink_for_ping = sink.clone();
        let player_id_for_ping = player_id.clone();
        let ping_handle = tokio::spawn(async move {
            // Use a shorter interval to prevent timeouts
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
            
            // Track failed attempts
            let mut consecutive_failures = 0;
            
            loop {
                interval.tick().await;
                
                // Generate a ping message with current timestamp
                let ping_msg = serde_json::json!({
                    "type": "ping",
                    "timestamp": chrono::Utc::now().timestamp()
                });
                
                // Restructure to fix borrowing issue
                let sink_for_ping_temp = sink_for_ping.clone();
                
                // Do the timeout separately to avoid borrow checker issues
                let lock_attempt = tokio::time::timeout(
                    tokio::time::Duration::from_millis(500),
                    sink_for_ping_temp.lock()
                ).await;
                
                match lock_attempt {
                    Ok(mut locked) => {
                        // Reset failure count on successful lock
                        consecutive_failures = 0;
                        
                        // Try to send ping frame
                        if let Err(e) = locked.send(Message::Ping(vec![1, 2, 3])).await {
                            println!("[Socket] ERROR: Error sending WebSocket ping to player {}: {}", player_id_for_ping, e);
                            consecutive_failures += 1;
                            
                            if consecutive_failures >= 3 {
                                println!("[Socket] Too many consecutive ping failures for player {}, stopping ping service", player_id_for_ping);
                                break;
                            }
                            
                            continue;
                        }
                        
                        // Try to send application ping
                        if let Err(e) = locked.send(Message::Text(ping_msg.to_string())).await {
                            println!("[Socket] ERROR: Error sending application ping to player {}: {}", player_id_for_ping, e);
                            consecutive_failures += 1;
                            
                            if consecutive_failures >= 3 {
                                println!("[Socket] ERROR: Too many consecutive ping failures for player {}, stopping ping service", player_id_for_ping);
                                break;
                            }
                        }
                    },
                    Err(_) => {
                        // Timeout acquiring lock - count as failure
                        consecutive_failures += 1;
                        println!("[Socket] Timeout acquiring lock for ping to player {}", player_id_for_ping);
                        
                        if consecutive_failures >= 3 {
                            println!("[Socket] Too many consecutive ping failures for player {}, stopping ping service", player_id_for_ping);
                            break;
                        }
                    }
                }
            }
        });

        // --- 3️⃣ read loop: handle verbs from this client -------------------
        while let Some(result) = stream.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    // First try to parse the JSON
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(json) => {
                            // Process the verb only if game has started
                            if *self.game_started.lock() {
                                // Handle the json command
                                if json["verb"] == "place" {
                                    println!("[Socket] Received place command from player {}: {}", player_id, text);
                                    let diff = engine::apply_verb(
                                        &self.bundle,
                                        &mut self.state.lock(),
                                        &json,
                                    );
    
                                    let event = serde_json::json!({
                                        "type": "event",
                                        "t": 1,
                                        "verb": json["verb"],
                                        "diff": diff
                                    });
                                    if let Err(e) = self.tx.send(Message::Text(event.to_string())) {
                                        println!("[Socket] ERROR: Error broadcasting event: {}", e);
                                    }
                                } else if json["verb"].is_string() {
                                    println!("[Socket] ERROR: Received unsupported verb '{}' from player {}", json["verb"], player_id);
                                } 
                            } else {
                                println!("[Socket] ERROR: Received command from player {} but game hasn't started yet", player_id);
                            }
                        },
                        Err(e) => {
                            // Handle non-JSON text messages safely
                            if text.contains("\"type\":\"pong\"") {
                                // This is a pong response, just ignore silently
                            } else {
                                println!("[Socket] ERROR: Received invalid JSON from player {}: {}", player_id, e);
                            }
                        }
                    }
                },
                Ok(Message::Ping(bytes)) => {
                    match sink.lock().await.send(Message::Pong(bytes)).await {
                        Ok(_) => {},
                        Err(e) => {
                            println!("[Socket] ERROR: Error sending pong: {}", e);
                            // Don't break here - continue trying to read messages
                        }
                    }
                },
                Ok(Message::Pong(_)) => {
                    // Received a pong, client is still alive
                    // Update last ping time if we want to track client aliveness
                },
                Ok(Message::Close(frame)) => {
                    println!("[Socket] WebSocket closed by client: {:?}", frame);
                    break;
                },
                Ok(Message::Binary(_)) => {
                    println!("[Socket] ERROR: Received binary message, which is not supported");
                },
                Err(e) => {
                    // Check if it's a normal disconnection or a real error
                    if e.to_string().contains("connection reset") || 
                       e.to_string().contains("broken pipe") ||
                       e.to_string().contains("closed") {
                        // Normal disconnection
                        println!("[Socket] WebSocket disconnected normally: {}", e);
                    } else {
                        // Actual error
                        println!("[Socket] ERROR: WebSocket error: {}", e);
                    }
                    break;
                }
            }
        }

        // --- disconnect -----------------------------------------------------
        println!("[Socket] WebSocket connection for player {} disconnected", player_id);
        forward_handle.abort();
        ping_handle.abort();
        
        // For a clean shutdown, just log that we're disconnecting
        // Don't attempt to send close frames manually - this often causes errors
        // WebSockets will be cleaned up automatically
        println!("[Socket] Connection cleanup complete for player {}", player_id);
    }
}

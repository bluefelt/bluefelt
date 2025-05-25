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
pub fn new_lobby(
    id: String,
    bundle: Bundle,
    lobbies: Arc<LobbyMap>,
    lobby_updates: broadcast::Sender<Message>,
) -> Arc<Lobby> {
    Arc::new(Lobby::new(id, bundle, lobbies, lobby_updates))
}

pub fn current_lobbies_json(lobbies: &LobbyMap) -> serde_json::Value {
    let list = lobbies
        .iter()
        .map(|l| {
            let lobby = l.value();
            serde_json::json!({
                "id": l.key(),
                "game_id": lobby.bundle.game_id,
                "name": format!("{} - Lobby {}", lobby.bundle.game_id, &l.key()[0..6]),
                "players": lobby.player_list(),
                "started": lobby.is_started()
            })
        })
        .collect::<Vec<_>>();
    serde_json::Value::Array(list)
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

    /// Incrementing tick for diff frames
    tick: Mutex<u64>,

    /// Stored diff history
    history: Mutex<Vec<serde_json::Value>>,

    /// Sender for lobby list updates
    lobby_updates: broadcast::Sender<Message>,

    /// Reference to lobby map for updates
    lobbies: Arc<LobbyMap>,
}

impl Lobby {
    pub fn new(
        id: String,
        bundle: Bundle,
        lobbies: Arc<LobbyMap>,
        lobby_updates: broadcast::Sender<Message>,
    ) -> Self {
        let initial = engine::load_initial_state(&bundle);
        let (tx, _) = broadcast::channel(64);
        Self {
            id,
            bundle,
            state: Mutex::new(initial),
            tx,
            players: Mutex::new(Vec::new()),
            game_started: Mutex::new(false),
            tick: Mutex::new(0),
            history: Mutex::new(Vec::new()),
            lobby_updates,
            lobbies,
        }
    }

    fn broadcast_lobby_list(&self) {
        let list = current_lobbies_json(&self.lobbies);
        let _ = self.lobby_updates.send(Message::Text(list.to_string()));
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

    /// Map a player's username to their actor ID ("p1" or "p2")
    fn actor_for_player(&self, username: &str) -> Option<String> {
        let players = self.players.lock();
        players
            .iter()
            .position(|p| p == username)
            .map(|idx| format!("p{}", idx + 1))
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
            drop(players);
            self.broadcast_lobby_list();
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
            drop(players);
            self.broadcast_lobby_list();
            return true;
        }
        
        println!("[Socket] ERROR: Player {} was not in the lobby and could not be removed", player_id);
        false
    }

    /// Compute possible verbs for each player based on current state
    fn possible_verbs(state: &serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();

        let turn_player = state["turn"].as_str().unwrap_or("");

        if let Some(players) = state["players"].as_array() {
            if let Some(board) = state["zones"]["board"].as_array() {
                for player in players {
                    if let Some(id) = player["id"].as_str() {
                        let mut verbs = Vec::new();
                        if id == turn_player {
                            let mark = player["mark"].as_str().unwrap_or("");
                            for (r, row) in board.iter().enumerate() {
                                if let Some(cells) = row.as_array() {
                                    for (c, cell) in cells.iter().enumerate() {
                                        if cell.is_null() {
                                            verbs.push(serde_json::json!({
                                                "verb": "place",
                                                "args": {
                                                    "entityId": mark,
                                                    "row": r,
                                                    "col": c
                                                }
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                        map.insert(id.to_string(), serde_json::Value::Array(verbs));
                    }
                }
            }
        }

        map
    }

    /// Check if the game has started
    pub fn is_started(&self) -> bool {
        *self.game_started.lock()
    }

pub fn start_game(&self) {
    *self.game_started.lock() = true;
    self.broadcast_lobby_list();
}

/// Accept a new WebSocket client, drive send/recv loops.
    pub async fn accept_client(self: Arc<Self>, socket: WebSocket, player_id: String, join: bool, since: u64) {
        let (tx, mut rx) = socket.split();
        let tx = Arc::new(TokioMutex::new(tx));
        if join {
            self.add_player(player_id.clone());
        }

        if self.is_started() {
            let snapshot = { let g = self.state.lock(); g.clone() };
            let possible = Lobby::possible_verbs(&snapshot);
            let actor = self
                .actor_for_player(&player_id)
                .unwrap_or_else(|| "spectator".to_string());
            let welcome = serde_json::json!({
                "type":"welcome",
                "you": actor,
                "started": self.is_started(),
                "state": snapshot,
                "meta": {"possibleVerbs": possible}
            });
            let mut lock = tx.lock().await;
            let _ = lock.send(Message::Text(welcome.to_string())).await;
        } else {
            let waiting = serde_json::json!({"type":"info","message":"Waiting for another player..."});
            let mut lock = tx.lock().await;
            let _ = lock.send(Message::Text(waiting.to_string())).await;
        }

        let history_frames: Vec<_> = {
            let history = self.history.lock();
            history
                .iter()
                .filter(|f| f["tick"].as_u64().unwrap_or(0) > since)
                .cloned()
                .collect()
        };
        for frame in history_frames {
            let mut lock = tx.lock().await;
            if lock
                .send(Message::Text(frame.to_string()))
                .await
                .is_err()
            {
                break;
            }
        }

        let mut bcast = self.tx.subscribe();
        let tx_forward = tx.clone();
        let forward = tokio::spawn(async move {
            while let Ok(msg) = bcast.recv().await {
                if tx_forward
                    .lock()
                    .await
                    .send(msg)
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        while let Some(Ok(message)) = rx.next().await {
            if let Message::Text(text) = message {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json["action"] == "join" {
                        if self.add_player(player_id.clone()) { self.broadcast_lobby_list(); }
                    } else if json["action"] == "leave" {
                        if self.remove_player(&player_id) { self.broadcast_lobby_list(); }
                    } else if json["action"] == "start_game" {
                        if !self.is_started() && self.players() >= 2 { self.start_game(); }
                    } else if json["verb"] == "place" && self.is_started() {
                        let diff = engine::apply_verb(&self.bundle, &mut self.state.lock(), &json);
                        let tick = { let mut t = self.tick.lock(); *t += 1; *t };
                        let mut patch_ops = Vec::new();
                        if let Some(arr) = diff.as_array() {
                            for op in arr {
                                if let Some(path) = op.get("path").and_then(|p| p.as_str()) {
                                    let mut op_obj = op.clone();
                                    op_obj["path"] = serde_json::Value::String(format!("/state{}", path));
                                    patch_ops.push(op_obj);
                                } else {
                                    patch_ops.push(op.clone());
                                }
                            }
                        }
                        let current_state = { let g = self.state.lock(); g.clone() };
                        let possible = Lobby::possible_verbs(&current_state);
                        for (pid, verbs) in possible {
                            patch_ops.push(serde_json::json!({"op":"replace","path":format!("/meta/possibleVerbs/{}",pid),"value":verbs}));
                        }
                        let frame = serde_json::json!({"type":"diff","tick":tick,"patch":patch_ops});
                        self.history.lock().push(frame.clone());
                        let _ = self.tx.send(Message::Text(frame.to_string()));
                    }
                }
            }
        }

        forward.abort();
    }
}

//! lobby.rs - simple lobby handling WebSocket players and diff broadcasts

use crate::{bundle::Bundle, engine};
use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, Mutex as TokioMutex};

pub type LobbyMap = DashMap<String, Arc<Lobby>>;

pub fn new_lobby(id: String, bundle: Bundle) -> Arc<Lobby> {
    Arc::new(Lobby::new(id, bundle))
}

pub struct Lobby {
    pub id: String,
    pub bundle: Bundle,
    state: Mutex<serde_json::Value>,
    tx: broadcast::Sender<Message>,
    players: Mutex<Vec<String>>,            // connected player ids
    roles: Mutex<HashMap<String, String>>,  // player_id -> p1/p2
    tick: Mutex<u64>,
    history: Mutex<Vec<serde_json::Value>>, // stored diff frames
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
            roles: Mutex::new(HashMap::new()),
            tick: Mutex::new(0),
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn player_list(&self) -> Vec<String> {
        self.players.lock().clone()
    }

    pub fn is_started(&self) -> bool {
        self.players.lock().len() == 2
    }

    pub fn add_player(&self, player_id: String) -> Option<String> {
        let mut players = self.players.lock();
        if let Some(role) = self.roles.lock().get(&player_id).cloned() {
            if !players.contains(&player_id) {
                players.push(player_id);
            }
            return Some(role);
        }
        if players.len() >= 2 {
            return None;
        }
        players.push(player_id.clone());
        let role = if players.len() == 1 { "p1" } else { "p2" }.to_string();
        self.roles.lock().insert(player_id, role.clone());
        if self.is_started() {
            let patch = serde_json::json!([
                {
                    "op": "replace",
                    "path": "/meta/possibleVerbs",
                    "value": self.possible_verbs()
                }
            ]);
            self.broadcast_diff(patch);
        }
        Some(role)
    }

    fn possible_verbs(&self) -> serde_json::Value {
        let state = self.state.lock();
        let turn = state["turn"].as_str().unwrap_or("");
        let players = state["players"].as_array().cloned().unwrap_or_default();
        let mut map = serde_json::Map::new();
        for p in players {
            let pid = p["id"].as_str().unwrap_or("");
            if pid == turn {
                let mark = p["mark"].as_str().unwrap_or("");
                let mut verbs = Vec::new();
                if let Some(board) = state["zones"]["board"].as_array() {
                    for (r, row) in board.iter().enumerate() {
                        if let Some(cells) = row.as_array() {
                            for (c, cell) in cells.iter().enumerate() {
                                if cell.is_null() {
                                    verbs.push(serde_json::json!({
                                        "verb": "place",
                                        "args": {"entityId": mark, "row": r as u64, "col": c as u64}
                                    }));
                                }
                            }
                        }
                    }
                }
                map.insert(pid.to_string(), serde_json::Value::Array(verbs));
            } else {
                map.insert(pid.to_string(), serde_json::Value::Array(vec![]));
            }
        }
        serde_json::Value::Object(map)
    }

    fn broadcast_diff(&self, patch: serde_json::Value) {
        let mut tick_guard = self.tick.lock();
        *tick_guard += 1;
        let tick = *tick_guard;
        drop(tick_guard);
        let frame = serde_json::json!({
            "type": "diff",
            "tick": tick,
            "patch": patch
        });
        self.history.lock().push(frame.clone());
        let _ = self.tx.send(Message::Text(frame.to_string()));
    }

    pub async fn accept_client(self: Arc<Self>, socket: WebSocket, player_id: String) {
        let role = {
            let roles = self.roles.lock();
            roles.get(&player_id).cloned().unwrap_or_else(|| "p1".to_string())
        };

        let (sink_raw, mut stream) = socket.split();
        let sink = Arc::new(TokioMutex::new(sink_raw));

        // Send welcome message
        {
            let snapshot = { self.state.lock().clone() };
            let meta = serde_json::json!({
                "possibleVerbs": self.possible_verbs()
            });
            let welcome = serde_json::json!({
                "type": "welcome",
                "you": role,
                "state": snapshot,
                "meta": meta
            });
            let mut locked = sink.lock().await;
            let _ = locked.send(Message::Text(welcome.to_string())).await;
        }

        // Forward broadcast diffs to this client
        let mut rx = self.tx.subscribe();
        let sink_clone = sink.clone();
        let forward = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if sink_clone.lock().await.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Read loop
        while let Some(Ok(Message::Text(text))) = stream.next().await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if json.get("verb").is_some() {
                    let mut state_guard = self.state.lock();
                    let patch = engine::apply_verb(&self.bundle, &mut *state_guard, &json);
                    drop(state_guard);
                    self.broadcast_diff(patch);
                } else if json["type"] == "getDiffs" {
                    if let Some(from) = json["from"].as_u64() {
                        let frames: Vec<_> = {
                            let history = self.history.lock();
                            history
                                .iter()
                                .filter(|f| f["tick"].as_u64().unwrap_or(0) >= from)
                                .cloned()
                                .collect()
                        };
                        for frame in frames {
                            let mut locked = sink.lock().await;
                            let _ = locked.send(Message::Text(frame.to_string())).await;
                        }
                    }
                }
            }
        }

        forward.abort();
        println!("[Socket] Client {} disconnected", player_id);
    }
}


use crate::{bundle::Bundle, engine};
use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub type LobbyMap = DashMap<String, Arc<Lobby>>;

#[allow(dead_code)]
pub fn new_lobby(id: String, bundle: Bundle) -> Arc<Lobby> {
    Arc::new(Lobby::new(id, bundle))
}

pub struct Lobby {
    pub id: String,
    pub bundle: Bundle,
    state: Mutex<serde_json::Value>,
    tx: broadcast::Sender<Message>,
}

impl Lobby {
    pub fn new(id: String, bundle: Bundle) -> Self {
        let initial = engine::load_initial_state(&bundle);
        let (tx, _) = broadcast::channel(32);
        Self { id, bundle, state: Mutex::new(initial), tx }
    }
    pub fn players(&self) -> usize { self.tx.receiver_count() }

    pub async fn accept_client(self: Arc<Self>, socket: WebSocket) {
        let (mut ws_tx, mut ws_rx) = socket.split();
        let mut rx = self.tx.subscribe();

        /* send welcome snapshot */
        let snap = {
            let s = self.state.lock().await;
            s.clone()
        };
        let welcome = serde_json::json!({
            "type":"welcome",
            "bundleMeta":{},
            "initialState": snap
        });
        let _ = ws_tx.send(Message::Text(welcome.to_string())).await;

        /* spawn task to forward events */
        let forward = {
            let ws_tx = Arc::new(tokio::sync::Mutex::new(ws_tx));
            let ws_tx_copy = ws_tx.clone();
            tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    let mut guard = ws_tx_copy.lock().await;
                    let _ = guard.send(msg).await;
                }
            })
        };

        /* read loop */
        while let Some(Ok(Message::Text(t))) = ws_rx.next().await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&t) {
                if json["verb"] == "place" {
                    let mut state_guard = self.state.lock().await;
                    let diff = engine::apply_verb(&self.bundle, &mut *state_guard, &json);
                    let event = serde_json::json!({
                        "type":"event",
                        "t":0,
                        "actor":"p1",
                        "verb": json["verb"],
                        "diff": diff
                    });
                    let _ = self.tx.send(Message::Text(event.to_string()));
                }
            }
        }
        forward.abort();
    }
}

pub type State = serde_json::Value;

pub fn load_initial_state(_bundle: &Bundle) -> serde_json::Value {
    // Return a properly structured initial state for a tic-tac-toe game
    serde_json::json!({
        "zones": {
            "board": [
                [null, null, null],
                [null, null, null],
                [null, null, null]
            ]
        },
        "players": [
            { "id": "p1", "mark": "mark_x" },
            { "id": "p2", "mark": "mark_o" }
        ],
        "turn": "p1"
    })
}

pub fn apply_verb(_bundle: &Bundle, state: &mut serde_json::Value, json: &serde_json::Value) -> serde_json::Value {
    // Check if we have a "place" verb
    if json["verb"] == "place" {
        // Extract parameters
        if let (Some(row), Some(col)) = (json["args"]["row"].as_u64(), json["args"]["col"].as_u64()) {
            // Convert to usize
            let row = row as usize;
            let col = col as usize;
            
            // Make sure they're within bounds (0-2)
            if row <= 2 && col <= 2 {
                // Get the current turn player - extract to a String to break the borrow
                let current_player = state["turn"].as_str().unwrap_or("p1").to_string();
                let mark = if current_player == "p1" { "mark_x" } else { "mark_o" };
                
                // Calculate the next player now
                let next_player = if current_player == "p1" { "p2" } else { "p1" };
                
                // Update the board
                if let Some(board) = state["zones"]["board"].as_array_mut() {
                    if let Some(row_arr) = board.get_mut(row) {
                        if let Some(cell) = row_arr.as_array_mut() {
                            if cell[col].is_null() {
                                cell[col] = serde_json::json!(mark);
                                
                                // Update the turn
                                state["turn"] = serde_json::json!(next_player);
                                
                                // Return the diff for the cell update
                                return serde_json::json!([
                                    {
                                        "op": "replace",
                                        "path": format!("/zones/board/{}/{}", row, col),
                                        "value": mark
                                    },
                                    {
                                        "op": "replace",
                                        "path": "/turn",
                                        "value": next_player
                                    }
                                ]);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Default empty diff if no valid move was made
    serde_json::json!([])
}

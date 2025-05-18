use axum::{
    routing::{get, post},
    extract::{Path, State},
    response::IntoResponse,
    Json, Router,
};
use axum::extract::ws::{Message, WebSocketUpgrade};
use bluefelt_core::{Lobby, LobbyMap};
use uuid::Uuid;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use serde::Serialize;

#[tokio::main]
async fn main() {
    let lobbies = Arc::new(LobbyMap::new());
    
    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:5173".parse().unwrap()])
        .allow_methods(Any)
        .allow_headers(Any);
    
    let app = Router::new()
        .route("/lobbies", post(create_lobby))
        .route("/lobbies", get(list_lobbies))
        .route("/games", get(list_games))
        .route("/lobbies/:id/ws", get(ws_handler))
        .layer(cors)
        .with_state(lobbies);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Running on http://127.0.0.1:8000");
    
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_lobby(
    State(lobbies): State<Arc<LobbyMap>>,
    Json(payload): Json<serde_json::Value>,          // { gameId: "love-letter" }
) -> impl IntoResponse {
    let game_id = payload["gameId"].as_str().unwrap_or("unknown");
    let lobby = Lobby { id: Uuid::new_v4(), game_id: game_id.to_string() };
    lobbies.insert(lobby.id, lobby.clone());
    Json(lobby)
}

async fn list_lobbies(
    State(lobbies): State<Arc<LobbyMap>>,
) -> impl IntoResponse {
    let vec: Vec<Lobby> = lobbies.iter().map(|e| e.value().clone()).collect();
    Json(vec)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<Uuid>,
    State(_lobbies): State<Arc<LobbyMap>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        while let Some(Ok(msg)) = socket.recv().await {
            match msg {
                Message::Text(_) | Message::Binary(_) => {
                    let _ = socket.send(msg).await;
                }
                _ => {}
            }
        }
        println!("WS for lobby {id} closed");
    })
}

#[derive(Serialize)]
struct GameInfo {
    id: String,
    name: String,
}

fn parse_manifest(path: &std::path::Path) -> Option<GameInfo> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut id = None;
    let mut name = None;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("gameId:") {
            id = Some(t.trim_start_matches("gameId:").trim().to_string());
        }
        if t.contains("metadata") && t.contains("name:") {
            if let Some(idx) = t.find("name:") {
                let mut val = t[idx + 5..].trim();
                if val.starts_with('{') {
                    val = val.trim_start_matches('{').trim();
                }
                if val.starts_with('"') {
                    if let Some(end) = val[1..].find('"') {
                        name = Some(val[1..1 + end].to_string());
                    }
                } else {
                    name = Some(val.to_string());
                }
            }
        }
    }
    match (id, name) {
        (Some(i), Some(n)) => Some(GameInfo { id: i, name: n }),
        (Some(i), None) => Some(GameInfo { id: i.clone(), name: i }),
        _ => None,
    }
}

async fn list_games() -> impl IntoResponse {
    let mut vec = Vec::new();
    let base: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../games");
    if let Ok(games) = std::fs::read_dir(base) {
        for g in games.flatten() {
            if let Ok(versions) = std::fs::read_dir(g.path()) {
                for v in versions.flatten() {
                    let m = v.path().join("manifest.yaml");
                    if m.exists() {
                        if let Some(info) = parse_manifest(&m) {
                            vec.push(info);
                        }
                        break;
                    }
                }
            }
        }
    }
    Json(vec)
}

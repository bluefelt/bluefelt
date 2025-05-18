use axum::{
    routing::{post, get},
    extract::{State, Path},
    response::IntoResponse,
    Json, Router,
};
use axum::extract::ws::{Message, WebSocketUpgrade};
use bluefelt_core::{Lobby, LobbyMap};
use uuid::Uuid;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};

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

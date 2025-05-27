use axum::{
    extract::{Path, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use uuid::Uuid;
use tower_http::cors::{CorsLayer, Any};
use axum::extract::ws::Message;
use std::sync::Arc;
use tokio::sync::broadcast;
use futures_util::{SinkExt, StreamExt};

mod bundle;
mod engine;
mod lobby;
mod utils;

use bundle::BundleMap;
use crate::lobby::{LobbyMap, new_lobby, current_lobbies_json};
use crate::utils::error_response;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Locate the games directory relative to the crate location so `cargo run`
    // works from any path
    let games_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("games");
    let bundles = BundleMap::load_dir(games_dir.to_str().unwrap())?;
    
    // Wrap the DashMap in an Arc to ensure proper sharing between requests
    let lobbies = Arc::new(LobbyMap::default());
    let (lobby_updates_tx, _) = broadcast::channel::<Message>(16);
    
    // Clone for each route handler
    let bundles_for_games = bundles.clone();
    let bundles_for_lobbies = bundles.clone();
    let bundles_for_manifest = bundles.clone();
    let lobbies_for_lobbies_route = lobbies.clone();
    let lobbies_for_ws = lobbies.clone();
    let lobbies_for_create = lobbies.clone();
    let lobbies_for_ws_list = lobbies.clone();
    let lobby_updates_for_create = lobby_updates_tx.clone();
    let lobby_updates_for_ws_list = lobby_updates_tx.clone();

    // Improved CORS configuration for WebSocket support
    let cors = CorsLayer::new()
        .allow_origin(Any)       // Allow any origin for development
        .allow_methods(Any)      // Allow all methods 
        .allow_headers(Any)      // Allow all headers
        .allow_credentials(false) // Must be false when using wildcard headers
        .expose_headers(Any);    // Use Any to expose all headers

    let app = Router::new()
        .route("/games", get(move || list_games(bundles_for_games.clone())))
        .route("/games/:id", get(move |path| get_game(path, bundles_for_manifest.clone())))
        .route("/lobbies", post(
            move |req| create_lobby(
                req,
                bundles_for_lobbies.clone(),
                lobbies_for_create.clone(),
                lobby_updates_for_create.clone(),
            )
        ).get(
            move || list_lobbies(lobbies_for_lobbies_route.clone())
        ))
        .route("/lobbies/:id", get(move |path| get_lobby(path, lobbies.clone())))
        .route("/lobbies/ws", get(
            move |ws| lobbies_ws_handler(ws, lobby_updates_for_ws_list.clone(), lobbies_for_ws_list.clone())
        ))
        .route("/lobbies/:id/ws", get(
            move |path, ws, query| ws_handler(path, ws, query, lobbies_for_ws.clone())
        ))
        // Apply the CORS middleware
        .layer(cors);

    let addr: SocketAddr = "0.0.0.0:8000".parse()?;
    println!("Server started on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

/* ---------- REST ---------- */

async fn create_lobby(
    Json(req): Json<serde_json::Value>,
    bundles: BundleMap,
    lobbies: Arc<LobbyMap>,
    lobby_updates: broadcast::Sender<Message>,
) -> impl IntoResponse {
    let game_id = req["game_id"].as_str().unwrap_or("tic-tac-toe");
    let bundle = match bundles.get_latest(game_id) {
        Some(b) => b,
        None => {
            return error_response(&format!("Unknown game: {}", game_id));
        }
    };
    
    let id = Uuid::new_v4().to_string();
    println!("[HTTP] Creating new lobby: {} for game: {}", id, game_id);
    
    lobbies.insert(id.clone(), new_lobby(id.clone(), bundle, lobbies.clone(), lobby_updates.clone()));

    // broadcast updated lobby list
    let list = current_lobbies_json(&lobbies);
    let _ = lobby_updates.send(Message::Text(list.to_string()));

    Json(serde_json::json!({ "id": id, "game_id": game_id }))
}

async fn list_lobbies(
    lobbies: Arc<LobbyMap>,
) -> impl IntoResponse {
    let json = current_lobbies_json(&lobbies);
    Json(json)
}

async fn list_games(
    bundles: BundleMap,
) -> impl IntoResponse {
    let games = bundles.list_games();
    let game_list = games.iter().map(|game_id| {
        let name = if let Some(bundle) = bundles.get_latest(game_id) {
            bundle.manifest.metadata.name.clone()
        } else {
            game_id.clone()
        };
        serde_json::json!({
            "id": game_id,
            "name": name,
        })
    }).collect::<Vec<_>>();
    
    Json(game_list)
}

async fn get_game(
    Path(id): Path<String>,
    bundles: BundleMap,
) -> impl IntoResponse {
    if let Some(bundle) = bundles.get_latest(&id) {
        Json(serde_json::to_value(&bundle.manifest).unwrap())
    } else {
        error_response("Unknown game")
    }
}

async fn get_lobby(
    Path(id): Path<String>,
    lobbies: Arc<LobbyMap>,
) -> impl IntoResponse {
    if let Some(lobby_ref) = lobbies.get(&id) {
        let lobby = lobby_ref.value();
        Json(serde_json::json!({
            "id": lobby.id,
            "game_id": lobby.bundle.game_id,
            "players": lobby.player_list(),
            "started": lobby.is_started(),
            "manifest": lobby.bundle.manifest
        }))
    } else {
        error_response("Lobby not found")
    }
}


async fn lobbies_ws_handler(
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

/* ---------- WS ---------- */

async fn ws_handler(
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
    
    println!("[Socket] Connection request from player {} for lobby {}", player_id, id);
    
    let Some(lobby_ref) = lobbies.get(&id) else {
        println!("[Socket] ERROR: Attempt to join non-existent lobby: {}", id);
        return ws.on_upgrade(|mut sock| async move {
            let _ = sock.send(Message::Text(serde_json::json!({
                "type": "error",
                "message": "Lobby does not exist"
            }).to_string())).await;
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
        println!("[Socket] WebSocket connections successful for player {} in lobby {}", player_id, id);
        lobby.accept_client(sock, player_id, join, since).await;
    })
}

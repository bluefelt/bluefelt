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

mod bundle;
mod engine;
mod lobby;

use bundle::BundleMap;
use crate::lobby::{LobbyMap, new_lobby};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bundles = BundleMap::load_dir("./games")?;
    
    // Wrap the DashMap in an Arc to ensure proper sharing between requests
    let lobbies = Arc::new(LobbyMap::default());
    
    // Clone for each route handler
    let bundles_for_games = bundles.clone();
    let bundles_for_lobbies = bundles.clone();
    let lobbies_for_lobbies_route = lobbies.clone();
    let lobbies_for_ws = lobbies.clone();

    // Improved CORS configuration for WebSocket support
    let cors = CorsLayer::new()
        .allow_origin(Any)       // Allow any origin for development
        .allow_methods(Any)      // Allow all methods 
        .allow_headers(Any)      // Allow all headers
        .allow_credentials(false) // Must be false when using wildcard headers
        .expose_headers(Any);    // Use Any to expose all headers

    let app = Router::new()
        .route("/games", get(move || list_games(bundles_for_games.clone())))
        .route("/lobbies", post(
            move |req| create_lobby(req, bundles_for_lobbies.clone(), lobbies.clone())
        ).get(
            move || list_lobbies(lobbies_for_lobbies_route.clone())
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
) -> impl IntoResponse {
    let game_id = req["gameId"].as_str().unwrap_or("tic-tac-toe");
    let bundle = match bundles.get_latest(game_id) {
        Some(b) => b,
        None => {
            return Json(serde_json::json!({ 
                "error": format!("Unknown game: {}", game_id) 
            }));
        }
    };
    
    let id = Uuid::new_v4().to_string();
    println!("[HTTP] Creating new lobby: {} for game: {}", id, game_id);
    
    lobbies.insert(id.clone(), new_lobby(id.clone(), bundle));
    
    Json(serde_json::json!({ "id": id, "game_id": game_id }))
}

async fn list_lobbies(
    lobbies: Arc<LobbyMap>,
) -> impl IntoResponse {
    
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
    
    Json(list)
}

async fn list_games(
    bundles: BundleMap,
) -> impl IntoResponse {
    let games = bundles.list_games();
    let game_list = games.iter().map(|game_id| {
        serde_json::json!({
            "id": game_id,
            "name": game_id, // You might want to include more metadata
        })
    }).collect::<Vec<_>>();
    
    Json(game_list)
}

/* ---------- WS ---------- */

async fn ws_handler(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    // Access query parameters to get player_id
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    lobbies: Arc<LobbyMap>,
) -> impl IntoResponse {
    // Extract player_id from query params, default to a random ID if missing
    let player_id = params.get("player_id").cloned().unwrap_or_else(|| {
        format!("guest_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap())
    });
    
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
    
    // Add player to the lobby
    let added = lobby.add_player(player_id.clone());
    
    println!("[Socket] Player {} attempted to join lobby {}. Added: {}", player_id, id, added);
    
    if !added {
        // Player couldn't be added (lobby full)
        println!("[Socket] ERROR: Could not add player {} to lobby {}: lobby is full", player_id, id);
        return ws.on_upgrade(|mut sock| async move {
            let _ = sock.send(Message::Text(serde_json::json!({
                "type": "error",
                "message": "Could not join lobby - it may be full"
            }).to_string())).await;
        });
    }
    
    ws.on_upgrade(move |sock| async move {
        println!("[Socket] WebSocket connections successful for player {} in lobby {}", player_id, id);
        lobby.accept_client(sock).await;
    })
}

use axum::{extract::Path, response::IntoResponse, Json};
use std::sync::Arc;
use tokio::sync::broadcast;
use axum::extract::ws::Message;
use uuid::Uuid;

use crate::bundle::BundleMap;
use crate::lobby::{LobbyMap, new_lobby, new_lobby_with_seed, current_lobbies_json};
use crate::utils::error_response;

/// Generate a 10-character lobby ID using UUID
fn generate_lobby_id() -> String {
    Uuid::new_v4()
        .to_string()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(10)
        .collect()
}

pub async fn create_lobby(
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
    
    let id = generate_lobby_id();
    println!("[HTTP] Creating new lobby: {} for game: {}", id, game_id);
    
    // Check if a seed was provided in the request
    let lobby = if let Some(seed_str) = req["seed"].as_str() {
        // Parse hex string to bytes
        if seed_str.len() != 64 {
            return error_response("Seed must be 64 hex characters (32 bytes)");
        }
        
        let mut seed = [0u8; 32];
        for (i, chunk) in seed_str.as_bytes().chunks(2).enumerate() {
            if i >= 32 {
                return error_response("Seed too long");
            }
            let hex_str = std::str::from_utf8(chunk).unwrap_or("");
            match u8::from_str_radix(hex_str, 16) {
                Ok(byte) => seed[i] = byte,
                Err(_) => return error_response("Invalid hex character in seed"),
            }
        }
        
        println!("[HTTP] Using provided seed: {}", seed_str);
        new_lobby_with_seed(id.clone(), bundle, lobbies.clone(), lobby_updates.clone(), seed)
    } else {
        new_lobby(id.clone(), bundle, lobbies.clone(), lobby_updates.clone())
    };
    
    lobbies.insert(id.clone(), lobby);

    // broadcast updated lobby list
    let list = current_lobbies_json(&lobbies);
    let _ = lobby_updates.send(Message::Text(list.to_string()));

    Json(serde_json::json!({ "id": id, "game_id": game_id }))
}

pub async fn list_lobbies(
    lobbies: Arc<LobbyMap>,
) -> impl IntoResponse {
    let json = current_lobbies_json(&lobbies);
    Json(json)
}

pub async fn list_games(
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

pub async fn get_game(
    Path(id): Path<String>,
    bundles: BundleMap,
) -> impl IntoResponse {
    if let Some(bundle) = bundles.get_latest(&id) {
        Json(serde_json::to_value(&bundle.manifest).unwrap())
    } else {
        error_response("Unknown game")
    }
}

pub async fn get_lobby(
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
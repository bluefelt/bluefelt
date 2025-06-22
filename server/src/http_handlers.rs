//! HTTP handlers for lobby/game architecture

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::lobby::lobby_impl::Lobby;
use crate::message_format::{MessageFormat, UpdateFormat};
use crate::app_state::AppState;
use axum::http::StatusCode;

#[derive(Debug, Deserialize)]
pub struct CreateLobbyRequest {
    pub name: Option<String>,
    pub max_members: Option<usize>,
    pub private_room: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CreateLobbyResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// Create a new lobby (without a game)
pub async fn create_lobby(
    State(state): State<AppState>,
    Json(req): Json<CreateLobbyRequest>,
) -> Json<CreateLobbyResponse> {
    let name = req.name.unwrap_or_else(|| "Untitled Lobby".to_string());
    
    // Create the lobby
    let lobby = Lobby::new(
        name.clone(),
        state.bundles.clone(),
        state.lobbies.clone(),
        None, // No seed for now
    );
    
    let lobby_id = lobby.state.id.clone();
    let created_at = chrono::DateTime::<chrono::Utc>::from(lobby.state.created_at)
        .to_rfc3339();
    
    // Add to lobby map
    state.lobbies.insert(lobby_id.clone(), lobby);
    
    Json(CreateLobbyResponse {
        id: lobby_id,
        name,
        created_at,
    })
}

/// List all lobbies
pub async fn list_lobbies(
    State(state): State<AppState>,
) -> Json<Vec<Value>> {
    println!("[HTTP] list_lobbies called");
    
    let mut lobby_list = Vec::new();
    
    // Process lobbies one at a time with minimal locking
    for entry in state.lobbies.iter() {
        let lobby_id = entry.key();
        let lobby = entry.value();
        
        // Skip archived lobbies
        if lobby.state.is_archived() {
            continue;
        }
        
        // Basic info with minimal locking
        let name = lobby.state.name.lock().clone();
        let owner = lobby.state.owner.lock().clone();
        let mut lobby_info = json!({
            "id": lobby_id.clone(),
            "name": name,
            "owner": owner,
            "created_at": chrono::DateTime::<chrono::Utc>::from(lobby.state.created_at).to_rfc3339(),
        });
        
        // Try to get member count
        if let Some(members) = lobby.state.members.try_read() {
            lobby_info["member_count"] = json!(members.len());
            lobby_info["members"] = json!(members.iter().map(|m| json!({
                "username": m.id.clone(),
                "connected": m.connected
            })).collect::<Vec<_>>());
        } else {
            lobby_info["member_count"] = json!(0);
            lobby_info["members"] = json!([]);
        }
        
        // Try to get table count
        if let Some(tables) = lobby.state.tables.try_lock() {
            lobby_info["table_count"] = json!(tables.len());
            // Just basic table info, no nested locks
            lobby_info["tables"] = json!(tables.iter().map(|(id, _)| json!({
                "id": id
            })).collect::<Vec<_>>());
        } else {
            lobby_info["table_count"] = json!(0);
            lobby_info["tables"] = json!([]);
        }
        
        lobby_list.push(lobby_info);
    }
    
    println!("[HTTP] list_lobbies returning {} lobbies", lobby_list.len());
    Json(lobby_list)
}

/// Get specific lobby info
pub async fn get_lobby(
    Path(lobby_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let lobby = state.lobbies
        .get(&lobby_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let name = lobby.state.name.lock().clone();
    let owner = lobby.state.owner.lock().clone();
    let archived = lobby.state.is_archived();
    
    let response = json!({
        "id": lobby.state.id,
        "name": name,
        "owner": owner,
        "archived": archived,
        "members": lobby.state.members.read().clone(),
        "games": lobby.get_games_summary(),
        "completedGames": [], // TODO: Track completed tables
        "settings": lobby.state.settings,
    });
    
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    pub player: String,
    #[serde(default = "default_message_format")]
    pub message_format: MessageFormat,
    #[serde(default = "default_update_format")]
    pub update_format: UpdateFormat,
}

fn default_message_format() -> MessageFormat {
    MessageFormat::Simple
}

fn default_update_format() -> UpdateFormat {
    UpdateFormat::Full
}

/// WebSocket endpoint for lobby
pub async fn lobby_websocket(
    ws: WebSocketUpgrade,
    Path(lobby_id): Path<String>,
    Query(query): Query<WebSocketQuery>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let lobby = state.lobbies
        .get(&lobby_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let lobby = Arc::clone(&lobby);
    let username = query.player;
    let message_format = query.message_format;
    let update_format = query.update_format;
    let connection_manager = state.connection_manager.clone();
    
    Ok(ws.on_upgrade(move |socket| async move {
        crate::lobby::websocket::handle_websocket(
            socket,
            lobby,
            username,
            true, // Auto-join for legacy endpoint
            message_format,
            update_format,
            connection_manager,
        ).await;
    }))
}

/// Create a game within a lobby
pub async fn create_game_in_lobby(
    Path(lobby_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let lobby = state.lobbies
        .get(&lobby_id)
        .ok_or((StatusCode::NOT_FOUND, "Lobby not found".to_string()))?;
    
    let game_type = req["gameType"].as_str()
        .ok_or((StatusCode::BAD_REQUEST, "gameType is required".to_string()))?;
    
    let creator = req["creator"].as_str()
        .ok_or((StatusCode::BAD_REQUEST, "creator is required".to_string()))?;
    
    match lobby.create_game(game_type, creator) {
        Ok(game_id) => Ok(Json(json!({
            "gameId": game_id,
            "gameType": game_type,
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

/// Delete a lobby
pub async fn delete_lobby(
    Path(lobby_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    state.lobbies.remove(&lobby_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(json!({
        "message": "Lobby deleted successfully",
        "lobbyId": lobby_id,
    })))
}

/// List available games
pub async fn list_games(
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let games = state.bundles.list_games();
    Ok(Json(json!(games)).into_response())
}

// ===== NEW TABLE ENDPOINTS =====

#[derive(Debug, Deserialize)]
pub struct CreateTableRequest {
    pub bundle_id: String,
    pub owner: String,
    pub name: Option<String>,
    pub min_players: Option<u32>,
    pub max_players: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct CreateTableResponse {
    pub table_id: String,
    pub bundle_id: String,
    pub seats: Vec<Option<Value>>,
}

/// Create a new table within a lobby
pub async fn create_table(
    Path(lobby_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<CreateTableRequest>,
) -> Result<Json<CreateTableResponse>, (StatusCode, String)> {
    let lobby = state.lobbies
        .get(&lobby_id)
        .ok_or((StatusCode::NOT_FOUND, "Lobby not found".to_string()))?;
    
    // Get bundle to determine player count
    let bundle = state.bundles
        .get_latest(&req.bundle_id)
        .ok_or((StatusCode::BAD_REQUEST, format!("Unknown game: {}", req.bundle_id)))?;
    
    let min_players = req.min_players.unwrap_or(bundle.manifest.metadata.players.min);
    let max_players = req.max_players.unwrap_or(bundle.manifest.metadata.players.max);
    
    // Create table using the new method
    match lobby.state.create_table(
        req.bundle_id.clone(),
        Arc::new(bundle),
        req.owner,
    ) {
        Ok(table_id) => {
            // Get the table to return seat info
            if let Some(table) = lobby.state.get_table(&table_id) {
                let seats = table.seats.read();
                Ok(Json(CreateTableResponse {
                    table_id,
                    bundle_id: req.bundle_id,
                    seats: seats.iter().map(|s| s.as_ref().map(|o| match o {
                        crate::lobby::table_instance::SeatOccupant::Player(player_id) => json!({
                            "player_id": player_id,
                            "username": player_id, // TODO: Map to actual username
                        }),
                        crate::lobby::table_instance::SeatOccupant::Reserved => json!({
                            "player_id": null,
                            "username": "Reserved",
                        }),
                    })).collect(),
                }))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve created table".to_string()))
            }
        },
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

/// List tables in a lobby
pub async fn list_tables(
    Path(lobby_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Value>>, StatusCode> {
    let lobby = state.lobbies
        .get(&lobby_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let tables = lobby.state.tables.lock();
    let table_list: Vec<Value> = tables.iter().map(|(id, table)| {
        let seats = table.seats.read();
        let status = table.status.read();
        json!({
            "id": id,
            "bundle_id": table.bundle_id,
            "owner": table.owner,
            "status": format!("{:?}", *status),
            "seats": seats.iter().map(|s| s.as_ref().map(|o| match o {
                crate::lobby::table_instance::SeatOccupant::Player(player_id) => json!({
                    "player_id": player_id,
                    "username": player_id, // TODO: Map to actual username
                }),
                crate::lobby::table_instance::SeatOccupant::Reserved => json!({
                    "player_id": null,
                    "username": "Reserved",
                }),
            })).collect::<Vec<_>>(),
            "min_players": table.min_players,
            "max_players": table.max_players,
        })
    }).collect();
    
    Ok(Json(table_list))
}

#[derive(Debug, Deserialize)]
pub struct ClaimSeatRequest {
    pub player_id: String,
    pub username: String,
}

/// Claim a seat at a table
pub async fn claim_seat(
    Path((lobby_id, table_id, seat_index)): Path<(String, String, usize)>,
    State(state): State<AppState>,
    Json(req): Json<ClaimSeatRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let lobby = state.lobbies
        .get(&lobby_id)
        .ok_or((StatusCode::NOT_FOUND, "Lobby not found".to_string()))?;
    
    let table = lobby.state.get_table(&table_id)
        .ok_or((StatusCode::NOT_FOUND, "Table not found".to_string()))?;
    
    let player_id = req.player_id.clone();
    let username = req.username.clone();
    
    match table.claim_seat(seat_index, req.player_id, req.username) {
        Ok(()) => {
            // Update member's active tables
            if let Err(e) = lobby.state.add_member_to_table(&table_id, &player_id) {
                eprintln!("Failed to update member's active tables: {}", e);
            }
            
            Ok(Json(json!({
                "success": true,
                "seat_index": seat_index,
            })))
        },
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

/// Release a seat at a table
pub async fn release_seat(
    Path((lobby_id, table_id, seat_index)): Path<(String, String, usize)>,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let lobby = state.lobbies
        .get(&lobby_id)
        .ok_or((StatusCode::NOT_FOUND, "Lobby not found".to_string()))?;
    
    let table = lobby.state.get_table(&table_id)
        .ok_or((StatusCode::NOT_FOUND, "Table not found".to_string()))?;
    
    // Get player ID before releasing seat
    let player_id = {
        let seats = table.seats.read();
        seats.get(seat_index)
            .and_then(|s| s.as_ref())
            .and_then(|o| match o {
                crate::lobby::table_instance::SeatOccupant::Player(id) => Some(id.clone()),
                _ => None,
            })
    };
    
    match table.release_seat(seat_index) {
        Ok(()) => {
            // Update member's active tables if we had a player
            if let Some(pid) = player_id {
                lobby.state.leave_table(&table_id, &pid);
            }
            
            Ok(Json(json!({
                "success": true,
                "seat_index": seat_index,
            })))
        },
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

#[derive(Debug, Deserialize)]
pub struct SetReadyRequest {
    pub player_id: String,
    pub ready: bool,
}

/// Set ready state for a player
pub async fn set_ready_state(
    Path((lobby_id, table_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(req): Json<SetReadyRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let lobby = state.lobbies
        .get(&lobby_id)
        .ok_or((StatusCode::NOT_FOUND, "Lobby not found".to_string()))?;
    
    let table = lobby.state.get_table(&table_id)
        .ok_or((StatusCode::NOT_FOUND, "Table not found".to_string()))?;
    
    match table.set_ready_state(&req.player_id, req.ready) {
        Ok(countdown_started) => Ok(Json(json!({
            "success": true,
            "countdown_started": countdown_started,
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

/// Delete a table
pub async fn delete_table(
    Path((lobby_id, table_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let lobby = state.lobbies
        .get(&lobby_id)
        .ok_or((StatusCode::NOT_FOUND, "Lobby not found".to_string()))?;
    
    // Get seated players before removing table
    let seated_players = if let Some(table) = lobby.state.get_table(&table_id) {
        table.get_seated_players()
    } else {
        return Err((StatusCode::NOT_FOUND, "Table not found".to_string()));
    };
    
    // Remove table
    lobby.state.tables.lock().remove(&table_id);
    
    // Update members' active tables
    for player_id in seated_players {
        lobby.state.leave_table(&table_id, &player_id);
    }
    
    // Clean up chat
    lobby.state.chat.clear_table_chat(&table_id);
    
    Ok(Json(json!({
        "success": true,
        "table_id": table_id,
    })))
}
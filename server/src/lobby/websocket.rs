//! WebSocket handler for lobby/game architecture

use super::lobby_impl::{Lobby, ClientInfo};
use super::lock_helpers::GameSnapshot;
use super::table_instance::{SeatOccupant, TableStatus};
use super::action_map::compute_action_map;
use super::connection_manager::ConnectionManager;
use crate::message_format::{MessageFormat, UpdateFormat};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt, stream::SplitSink, stream::SplitStream};
use serde_json::{json, Value};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

/// Handle WebSocket connection for a lobby member
pub async fn handle_websocket(
    mut socket: WebSocket,
    lobby: Arc<Lobby>,
    username: String,
    should_join: bool,
    message_format: MessageFormat,
    update_format: UpdateFormat,
    connection_manager: Arc<ConnectionManager>,
) {
    // Subscribe to lobby broadcasts
    let mut rx = lobby.state.tx.subscribe();
    
    let client_info = ClientInfo {
        message_format,
        update_format,
        username: username.clone(),
    };
    
    // Check if user is already a member
    let is_already_member = {
        let members = lobby.state.members.read();
        members.iter().any(|m| m.id == username)
    };
    
    // Send initial state based on join status or existing membership
    if should_join || is_already_member {
        // Join the lobby if not already a member
        if !is_already_member {
            if let Err(e) = lobby.join_lobby(username.clone(), client_info) {
                let _ = socket.send(Message::Text(json!({
                    "type": "error",
                    "message": e
                }).to_string())).await;
                return;
            }
            // Register connection with manager
            connection_manager.register_connection(
                username.clone(),
                lobby.state.id.clone(),
                Arc::clone(&lobby),
                true,
            );
        } else {
            // Update connection status for existing member
            lobby.state.set_member_connected(&username, true);
            // Register reconnection
            connection_manager.register_connection(
                username.clone(),
                lobby.state.id.clone(),
                Arc::clone(&lobby),
                true,
            );
        }
        
        // Send joined message
        let welcome_msg = create_lobby_joined_message(&lobby, &username);
        if let Err(_) = socket.send(Message::Text(welcome_msg.to_string())).await {
            if !is_already_member {
                lobby.leave_lobby(&username);
            }
            return;
        }
    } else {
        // Just viewing - send lobby view message
        let view_msg = create_lobby_view_message(&lobby);
        if let Err(_) = socket.send(Message::Text(view_msg.to_string())).await {
            return;
        }
        // Register observer connection
        connection_manager.register_connection(
            username.clone(),
            lobby.state.id.clone(),
            Arc::clone(&lobby),
            false,
        );
    }
    
    // Create channels for bidirectional communication
    // Use bounded channel to prevent memory exhaustion
    let (tx, mut socket_rx) = tokio::sync::mpsc::channel(100);
    
    // Split the WebSocket for concurrent read/write
    let (ws_sender, ws_receiver) = socket.split();
    
    // Task to receive from socket and process
    let lobby_clone = lobby.clone();
    let username_clone = username.clone();
    let is_member = Arc::new(std::sync::atomic::AtomicBool::new(should_join || is_already_member));
    let is_member_handler = Arc::clone(&is_member);
    let tx_clone = tx.clone();
    let connection_manager_clone = Arc::clone(&connection_manager);
    let mut ws_receiver = ws_receiver;
    let recv_task = tokio::spawn(async move {
        println!("[WS] Receive task started for {}", username_clone);
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        handle_client_message(&lobby_clone, &username_clone, json, &is_member_handler, &connection_manager_clone).await;
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Notify send task that we received a pong
                    let _ = tx_clone.send(Message::Text("__PONG__".to_string())).await;
                    connection_manager_clone.update_activity(&username_clone);
                }
                Ok(Message::Close(_)) => {
                    println!("[WS] Client {} sent close message", username_clone);
                    break;
                }
                Err(e) => {
                    println!("[WS] Receive error for {}: {:?}", username_clone, e);
                    break;
                }
                _ => {}
            }
        }
        println!("[WS] Receive task completed for {}", username_clone);
        // Signal the send task to stop by dropping the channel
        drop(tx_clone);
    });
    
    // Task to send broadcasts to socket
    let username_for_filter = username.clone();
    let lobby_for_filter = Arc::clone(&lobby);
    let connection_manager_send = Arc::clone(&connection_manager);
    let mut ws_sender = ws_sender;
    let send_task = tokio::spawn(async move {
        println!("[WS] Send task started for {}", username_for_filter);
        let mut heartbeat = interval(Duration::from_secs(30)); // Send ping every 30 seconds
        let mut last_pong = std::time::Instant::now();
        
        loop {
            tokio::select! {
                // Heartbeat/ping mechanism
                _ = heartbeat.tick() => {
                    // Check if we've received a pong recently
                    if last_pong.elapsed() > Duration::from_secs(60) {
                        println!("[WS] No pong received from {} in 60s, closing connection", username_for_filter);
                        break;
                    }
                    
                    // Send ping
                    match ws_sender.send(Message::Ping(vec![])).await {
                        Ok(_) => {
                            // Update activity
                            connection_manager_send.update_activity(&username_for_filter);
                        }
                        Err(e) => {
                            println!("[WS] Failed to send ping to {} ({}), closing", username_for_filter, e);
                            break;
                        }
                    }
                }
                // Lobby broadcasts
                msg = rx.recv() => {
                    match msg {
                        Ok(text) => {
                            // Filter messages for this user
                            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                if should_send_to_user(&json, &username_for_filter, &lobby_for_filter) {
                                    // Send message with proper error handling
                                    match ws_sender.send(Message::Text(text)).await {
                                        Ok(_) => {},
                                        Err(e) => {
                                            println!("[WS] Failed to send to {} ({}), closing connection", username_for_filter, e);
                                            break;
                                        }
                                    }
                                } else {
                                    let msg_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
                                    let to_field = json.get("to").and_then(|t| t.as_str()).unwrap_or("broadcast");
                                    println!("[WS] FILTERED OUT {} message to {} (user is {})", msg_type, to_field, username_for_filter);
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    }
                }
                // Direct messages to this client
                Some(msg) = socket_rx.recv() => {
                    // Check if this is a pong notification
                    if let Message::Text(text) = &msg {
                        if text == "__PONG__" {
                            last_pong = std::time::Instant::now();
                            continue;
                        }
                    }
                    
                    match ws_sender.send(msg).await {
                        Ok(_) => {},
                        Err(e) => {
                            println!("[WS] Failed to send direct message to {} ({}), closing", username_for_filter, e);
                            break;
                        }
                    }
                }
                else => {
                    // Channel closed, exit gracefully
                    println!("[WS] Direct message channel closed for {}", username_for_filter);
                    break;
                }
            }
        }
    });
    
    // Wait for both tasks to complete
    let recv_result = recv_task.await;
    let send_result = send_task.await;
    
    // Log task completion
    match recv_result {
        Ok(_) => println!("[WS] Receive task completed successfully for {}", username),
        Err(e) => println!("[WS] Receive task error for {}: {:?}", username, e),
    }
    match send_result {
        Ok(_) => println!("[WS] Send task completed successfully for {}", username),
        Err(e) => println!("[WS] Send task error for {}: {:?}", username, e),
    }
    
    // Perform cleanup only once
    println!("[WS] Performing cleanup for {}", username);
    
    // Generate reconnection token if this is a member
    let reconnection_token = if is_member.load(std::sync::atomic::Ordering::Relaxed) {
        Some(connection_manager.generate_reconnection_token(&username, &lobby.state.id, true))
    } else {
        None
    };
    
    // The WebSocket will be closed when both halves are dropped from the tasks
    println!("[WS] WebSocket tasks completed for {} (reconnection token: {})", 
        username, 
        reconnection_token.is_some()
    );
    
    // Notify connection manager about disconnection
    connection_manager.disconnect(&username, &lobby.state.id);
    
    if is_member.load(std::sync::atomic::Ordering::Relaxed) {
        lobby.disconnect_member(&username);
    }
    
    // Send lobby state update after disconnection
    lobby.broadcast_lobby_state();
    
    // If we have a reconnection token, broadcast it to the user
    if let Some(token) = reconnection_token {
        // Send through the broadcast channel as a direct message
        let disconnect_msg = json!({
            "type": "reconnectionToken",
            "to": username.clone(),
            "token": token,
            "message": "You can use this token to reconnect within 5 minutes"
        });
        let _ = lobby.state.tx.send(disconnect_msg.to_string());
    }
    
    println!("[WS] Connection fully closed for {}", username);
}

/// Handle incoming message from client
async fn handle_client_message(
    lobby: &Arc<Lobby>,
    username: &str,
    msg: Value,
    is_member: &Arc<std::sync::atomic::AtomicBool>,
    connection_manager: &Arc<ConnectionManager>,
) {
    let action = msg.get("action").and_then(|a| a.as_str()).unwrap_or("");
    
    match action {
        "joinLobby" => {
            // Handle joining the lobby
            if !is_member.load(std::sync::atomic::Ordering::Relaxed) {
                let client_info = ClientInfo {
                    message_format: MessageFormat::Standard,
                    update_format: UpdateFormat::Patch,
                    username: username.to_string(),
                };
                
                match lobby.join_lobby(username.to_string(), client_info) {
                    Ok(_) => {
                        println!("[WebSocket] {} joined lobby", username);
                        
                        // Mark as member
                        is_member.store(true, std::sync::atomic::Ordering::Relaxed);
                        
                        // Register connection with manager
                        connection_manager.register_connection(
                            username.to_string(),
                            lobby.state.id.clone(),
                            Arc::clone(lobby),
                            true,
                        );
                        
                        // Send joined confirmation directly to this client
                        let welcome_msg = create_lobby_joined_message(lobby, username);
                        let _ = lobby.state.tx.send(json!({
                            "type": "lobbyJoined",
                            "to": username,
                            "lobby": welcome_msg.get("lobby").unwrap()
                        }).to_string());
                        
                        // Broadcast member update to all
                        let members_update = json!({
                            "type": "memberJoined",
                            "member": username,
                            "members": lobby.state.members.read().iter().map(|m| {
                                json!({
                                    "username": m.name,
                                    "connected": m.connected,
                                })
                            }).collect::<Vec<_>>(),
                        });
                        let _ = lobby.state.tx.send(members_update.to_string());
                    }
                    Err(e) => {
                        let error_msg = json!({
                            "type": "error",
                            "message": e,
                            "to": username
                        });
                        let _ = lobby.state.tx.send(error_msg.to_string());
                    }
                }
            } else {
                // Already a member
                let error_msg = json!({
                    "type": "error",
                    "message": "Already a member of this lobby",
                    "to": username
                });
                let _ = lobby.state.tx.send(error_msg.to_string());
            }
        }
        "createGame" => {
            let game_type = msg.get("gameType")
                .and_then(|g| g.as_str())
                .unwrap_or("tic-tac-toe");
                
            match lobby.create_game(game_type, username) {
                Ok(game_id) => {
                    // Game created, broadcast will notify all clients
                }
                Err(e) => {
                    send_error(lobby, username, &e);
                }
            }
        }
        
        "joinGame" => {
            let game_id = match msg.get("gameId").and_then(|g| g.as_str()) {
                Some(id) => id,
                None => {
                    send_error(lobby, username, "Missing gameId");
                    return;
                }
            };
            
            if let Err(e) = lobby.join_game(game_id, username) {
                send_error(lobby, username, &e);
            }
        }
        
        "startGame" => {
            let game_id = match msg.get("gameId").and_then(|g| g.as_str()) {
                Some(id) => id,
                None => {
                    println!("[WS] startGame: Missing gameId");
                    send_error(lobby, username, "Missing gameId");
                    return;
                }
            };
            
            println!("[WS] User {} starting game {}", username, game_id);
            if let Err(e) = lobby.start_game(game_id) {
                println!("[WS] Error starting game: {}", e);
                send_error(lobby, username, &e);
            } else {
                println!("[WS] Game {} started successfully", game_id);
            }
        }
        
        "gameAction" => {
            let game_id = match msg.get("gameId").and_then(|g| g.as_str()) {
                Some(id) => id,
                None => {
                    send_error(lobby, username, "Missing gameId");
                    return;
                }
            };
            
            let action = msg.get("data").cloned().unwrap_or(json!({}));
            
            // Use the new v2 action processing pipeline
            // First, we need to find the player's slot
            let table = match lobby.state.get_table(game_id) {
                Some(t) => t,
                None => {
                    send_error(lobby, username, "Table not found");
                    return;
                }
            };
            
            let seats = table.seats.read();
            let mut slot = None;
            for (i, seat) in seats.iter().enumerate() {
                if let Some(crate::lobby::table_instance::SeatOccupant::Player(id)) = seat {
                    if id == username {
                        slot = Some(format!("p{}", i + 1));
                        break;
                    }
                }
            }
            drop(seats);
            
            let slot = match slot {
                Some(s) => s,
                None => {
                    send_error(lobby, username, "Player not seated at table");
                    return;
                }
            };
            
            // Process the action using v2 pipeline
            match lobby.process_game_action_v2(game_id, &slot, &action) {
                Ok(patches) => {
                    // Broadcast game update
                    broadcast_game_update(lobby, game_id, patches);
                }
                Err(e) => {
                    send_error(lobby, username, &e);
                }
            }
        }
        
        "leaveGame" => {
            // Legacy code - games are now managed through tables
            // TODO: Remove this once all clients are updated
            /*
            let game_id = match msg.get("gameId").and_then(|g| g.as_str()) {
                Some(id) => id,
                None => return,
            };
            
            lobby.state.leave_game(game_id, username);
            lobby.broadcast_lobby_state();
            */
        }
        
        // ===== NEW TABLE ACTIONS =====
        
        "createTable" => {
            // Check if user is a member
            if !is_member.load(std::sync::atomic::Ordering::Relaxed) {
                let error_msg = json!({
                    "type": "error",
                    "message": "Must join lobby to create tables",
                    "to": username
                });
                let _ = lobby.state.tx.send(error_msg.to_string());
                return;
            }
            let bundle_id = match msg.get("bundleId").and_then(|b| b.as_str()) {
                Some(id) => id,
                None => {
                    send_error(lobby, username, "Missing bundleId");
                    return;
                }
            };
            
            let min_players = msg.get("minPlayers").and_then(|m| m.as_u64()).map(|m| m as u32);
            let max_players = msg.get("maxPlayers").and_then(|m| m.as_u64()).map(|m| m as u32);
            
            // Get bundle to determine player count  
            let bundle = match lobby.bundles.get_latest(bundle_id) {
                Some(b) => b,
                None => {
                    send_error(lobby, username, &format!("Unknown game: {}", bundle_id));
                    return;
                }
            };
            
            match lobby.state.create_table(
                bundle_id.to_string(),
                Arc::new(bundle),
                username.to_string(),
            ) {
                Ok(table_id) => {
                    broadcast_table_created(lobby, &table_id);
                }
                Err(e) => {
                    send_error(lobby, username, &e);
                }
            }
        }
        
        "joinTable" => {
            // Check if user is a member
            if !is_member.load(std::sync::atomic::Ordering::Relaxed) {
                let error_msg = json!({
                    "type": "error",
                    "message": "Must join lobby to join tables",
                    "to": username
                });
                let _ = lobby.state.tx.send(error_msg.to_string());
                return;
            }
            
            // Check if user is already seated at another table
            if lobby.state.is_member_seated_anywhere(username) {
                send_error(lobby, username, "Already seated at another table. Leave your current table first.");
                return;
            }
            
            let table_id = match msg.get("tableId").and_then(|t| t.as_str()) {
                Some(id) => id,
                None => {
                    send_error(lobby, username, "Missing tableId");
                    return;
                }
            };
            
            if let Some(table) = lobby.state.get_table(table_id) {
                // Check table capacity before attempting to join
                let seated_count = table.seated_count();
                if seated_count >= table.max_players as usize {
                    send_error(lobby, username, "Table is full");
                    return;
                }
                
                // Use atomic seat assignment
                use crate::lobby::seat_manager::SeatManager;
                match SeatManager::atomic_claim_any_seat(
                    &table,
                    username.to_string(),
                    username.to_string()
                ) {
                    Ok(seat_index) => {
                        lobby.state.add_member_to_table(table_id, username).ok();
                        broadcast_table_updated(lobby, table_id);
                        
                        // Send confirmation with assigned seat
                        let confirm_msg = json!({
                            "type": "tableJoined",
                            "tableId": table_id,
                            "seatIndex": seat_index,
                            "to": username
                        });
                        let _ = lobby.state.tx.send(confirm_msg.to_string());
                    }
                    Err(e) => {
                        send_error(lobby, username, &e);
                    }
                }
            } else {
                send_error(lobby, username, "Table not found");
            }
        }
        
        "claimSeat" => {
            // Check if user is a member
            if !is_member.load(std::sync::atomic::Ordering::Relaxed) {
                let error_msg = json!({
                    "type": "error",
                    "message": "Must join lobby to claim seats",
                    "to": username
                });
                let _ = lobby.state.tx.send(error_msg.to_string());
                return;
            }
            let table_id = match msg.get("tableId").and_then(|t| t.as_str()) {
                Some(id) => id,
                None => {
                    send_error(lobby, username, "Missing tableId");
                    return;
                }
            };
            
            let seat_index = match msg.get("seatIndex").and_then(|s| s.as_u64()) {
                Some(idx) => idx as usize,
                None => {
                    send_error(lobby, username, "Missing seatIndex");
                    return;
                }
            };
            
            if let Some(table) = lobby.state.get_table(table_id) {
                match table.claim_seat(seat_index, username.to_string(), username.to_string()) {
                    Ok(()) => {
                        lobby.state.add_member_to_table(table_id, username).ok();
                        broadcast_table_updated(lobby, table_id);
                    }
                    Err(e) => {
                        send_error(lobby, username, &e);
                    }
                }
            } else {
                send_error(lobby, username, "Table not found");
            }
        }
        
        "releaseSeat" => {
            let table_id = match msg.get("tableId").and_then(|t| t.as_str()) {
                Some(id) => id,
                None => {
                    send_error(lobby, username, "Missing tableId");
                    return;
                }
            };
            
            let seat_index = match msg.get("seatIndex").and_then(|s| s.as_u64()) {
                Some(idx) => idx as usize,
                None => {
                    send_error(lobby, username, "Missing seatIndex");
                    return;
                }
            };
            
            if let Some(table) = lobby.state.get_table(table_id) {
                match table.release_seat(seat_index) {
                    Ok(()) => {
                        lobby.state.remove_member_from_table(table_id, username);
                        broadcast_table_updated(lobby, table_id);
                    }
                    Err(e) => {
                        send_error(lobby, username, &e);
                    }
                }
            } else {
                send_error(lobby, username, "Table not found");
            }
        }
        
        "setReady" => {
            // Check if user is a member
            if !is_member.load(std::sync::atomic::Ordering::Relaxed) {
                let error_msg = json!({
                    "type": "error",
                    "message": "Must join lobby to set ready state",
                    "to": username
                });
                let _ = lobby.state.tx.send(error_msg.to_string());
                return;
            }
            let table_id = match msg.get("tableId").and_then(|t| t.as_str()) {
                Some(id) => id,
                None => {
                    send_error(lobby, username, "Missing tableId");
                    return;
                }
            };
            
            let ready = msg.get("ready").and_then(|r| r.as_bool()).unwrap_or(true);
            
            if let Some(table) = lobby.state.get_table(table_id) {
                match table.set_ready_state(username, ready) {
                    Ok(countdown_started) => {
                        broadcast_table_updated(lobby, table_id);
                        if countdown_started {
                            // Actually start the countdown
                            use crate::lobby::countdown_manager::CountdownManager;
                            CountdownManager::start_countdown(&table, lobby, 3);
                            broadcast_countdown_started(lobby, table_id);
                        }
                    }
                    Err(e) => {
                        send_error(lobby, username, &e);
                    }
                }
            } else {
                send_error(lobby, username, "Table not found");
            }
        }
        
        "leaveLobby" => {
            // Actually leave the lobby (remove membership)
            if is_member.load(std::sync::atomic::Ordering::Relaxed) {
                lobby.leave_lobby(username);
                is_member.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
        "renameLobby" => {
            let new_name = msg.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");
                
            if new_name.is_empty() {
                let error_msg = json!({
                    "type": "error",
                    "message": "Lobby name cannot be empty",
                    "to": username
                });
                let _ = lobby.state.tx.send(error_msg.to_string());
            } else {
                match lobby.rename_lobby(username, new_name.to_string()) {
                    Ok(_) => {
                        println!("[WebSocket] {} renamed lobby to '{}'", username, new_name);
                        // broadcast_lobby_state will handle notifying all clients
                    }
                    Err(e) => {
                        let error_msg = json!({
                            "type": "error",
                            "message": e,
                            "to": username
                        });
                        let _ = lobby.state.tx.send(error_msg.to_string());
                    }
                }
            }
        }
        
        "sendChatMessage" => {
            // Check if user is a member
            if !is_member.load(std::sync::atomic::Ordering::Relaxed) {
                let error_msg = json!({
                    "type": "error",
                    "message": "Must join lobby to send messages",
                    "to": username
                });
                let _ = lobby.state.tx.send(error_msg.to_string());
                return;
            }
            let message = match msg.get("message").and_then(|m| m.as_str()) {
                Some(msg) => msg,
                None => {
                    send_error(lobby, username, "Missing message");
                    return;
                }
            };
            
            let scope = msg.get("scope").and_then(|s| s.as_str()).unwrap_or("lobby");
            
            match scope {
                "lobby" => {
                    lobby.state.chat.add_lobby_message(username.to_string(), message.to_string());
                    broadcast_chat_message(lobby, "lobby", None, username, message);
                }
                "table" => {
                    if let Some(table_id) = msg.get("tableId").and_then(|t| t.as_str()) {
                        lobby.state.chat.add_table_message(table_id, username.to_string(), message.to_string());
                        broadcast_chat_message(lobby, "table", Some(table_id), username, message);
                    } else {
                        send_error(lobby, username, "Missing tableId for table chat");
                    }
                }
                _ => {
                    send_error(lobby, username, "Invalid chat scope");
                }
            }
        }
        
        "requestGameState" => {
            // Check if user is a member
            if !is_member.load(std::sync::atomic::Ordering::Relaxed) {
                let error_msg = json!({
                    "type": "error",
                    "message": "Must join lobby to request game state",
                    "to": username
                });
                let _ = lobby.state.tx.send(error_msg.to_string());
                return;
            }
            
            let table_id = match msg.get("tableId").and_then(|t| t.as_str()) {
                Some(id) => id,
                None => {
                    send_error(lobby, username, "Missing tableId");
                    return;
                }
            };
            
            // Check if table exists and is in Playing state
            if let Some(table) = lobby.state.get_table(table_id) {
                let is_playing = {
                    let status = table.status.read();
                    *status == TableStatus::Playing
                };
                
                if is_playing {
                    // The table is playing, send the game state to this specific player
                    lobby.send_game_state_to_player(table_id, username);
                } else {
                    send_error(lobby, username, "Table is not in playing state");
                }
            } else {
                send_error(lobby, username, "Table not found");
            }
        }
        
        _ => {
            // Unknown action
        }
    }
}

/// Check if a broadcast message should be sent to a specific user
fn should_send_to_user(msg: &Value, username: &str, lobby: &Arc<Lobby>) -> bool {
    let msg_type = msg.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
    
    // Always send lobby state updates
    if msg_type == "lobbyState" {
        println!("[WS] Sending lobbyState to {}", username);
        return true;
    }
    
    // Direct messages check recipient first
    if let Some(to) = msg.get("to").and_then(|t| t.as_str()) {
        let should_send = to == username;
        println!("[WS] Message type {} targeted to {}, should send to {}: {}", 
                 msg_type, to, username, should_send);
        return should_send;
    }
    
    // Game messages - check if user is involved
    if let Some(game_instance_id) = msg.get("gameInstanceId").and_then(|g| g.as_str()) {
        if let Some(table) = lobby.state.get_table(game_instance_id) {
            // Check if user is seated at the table
            if table.is_player_seated(username) {
                println!("[WS] {} is player in table {}, sending {}", username, game_instance_id, msg_type);
                return true;
            }
            
            // Check if user is a spectator
            let spectators = table.spectators.read();
            if spectators.contains(&username.to_string()) {
                println!("[WS] {} is spectator in table {}, sending {}", username, game_instance_id, msg_type);
                return true;
            }
        }
        println!("[WS] {} is not involved in game {}, NOT sending {}", username, game_instance_id, msg_type);
        return false;
    }
    
    // Broadcast messages without 'to' field go to all connected users in lobby
    println!("[WS] Broadcasting {} to all users including {}", msg_type, username);
    true
}

/// Send error message to specific user
fn send_error(lobby: &Arc<Lobby>, username: &str, error: &str) {
    let msg = json!({
        "type": "error",
        "to": username,
        "message": error,
    });
    
    let _ = lobby.state.tx.send(msg.to_string());
}

/// Broadcast table/game update to players and observers
/// DEPRECATED: Use table-specific update broadcast instead
#[allow(dead_code)]
fn broadcast_game_update(lobby: &Arc<Lobby>, table_id: &str, patches: Vec<Value>) {
    let table = match lobby.state.get_table(table_id) {
        Some(t) => t,
        None => return,
    };
    
    // Get table state and seated players
    let state = table.game_state.read().clone();
    let tick = *table.tick.lock();
    let seated_players = table.get_seated_players();
    
    // Optimize patches for large game states
    let optimized_patches = if patches.len() > 10 {
        // Convert Value patches to json_patch format
        let mut patch_operations = Vec::new();
        for patch in &patches {
            if let Ok(op) = serde_json::from_value::<json_patch::PatchOperation>(patch.clone()) {
                patch_operations.push(op);
            }
        }
        
        if !patch_operations.is_empty() {
            let optimizer = crate::engine::patch_optimizer::PatchOptimizer::default();
            let json_patch = json_patch::Patch(patch_operations);
            let optimized = optimizer.optimize(json_patch);
            
            // Convert back to Value array
            optimized.0.into_iter()
                .map(|op| serde_json::to_value(op).unwrap_or(Value::Null))
                .collect()
        } else {
            patches
        }
    } else {
        patches
    };
    
    // Generate entity UI for each player without holding locks
    let mut player_entity_ui = HashMap::new();
    for player_id in seated_players.iter() {
        if let Ok(entity_ui) = crate::engine::entity_ui::enhance_entities_with_ui(&state, &table.bundle, player_id) {
            player_entity_ui.insert(player_id.clone(), entity_ui);
        }
    }
    
    // Compute UI data including action map
    let action_map = compute_action_map(&state, &table.bundle);
    
    // Get entity definitions for UI
    let entities = if let Some(entities) = table.bundle.entities.as_array() {
        entities.clone()
    } else {
        vec![]
    };
    
    // Create diff message with entity UI and action map
    let msg = json!({
        "type": "gameUpdate",
        "gameInstanceId": table_id,
        "tick": tick,
        "patches": optimized_patches,
        "entityUI": player_entity_ui,
        "ui": {
            "actionMap": action_map,
            "entities": entities,
        }
    });
    
    let _ = lobby.state.tx.send(msg.to_string());
}

// ===== NEW TABLE BROADCAST FUNCTIONS =====

/// Broadcast table created event
fn broadcast_table_created(lobby: &Arc<Lobby>, table_id: &str) {
    if let Some(table) = lobby.state.get_table(table_id) {
        let seats = table.seats.read();
        let ready_states = table.ready_states.read();
        let status = table.status.read();
        
        let msg = json!({
            "type": "tableCreated",
            "table": {
                "id": table_id,
                "bundleId": table.bundle_id,
                "owner": table.owner,
                "status": format!("{:?}", *status),
                "seats": seats.iter().map(|s| s.as_ref().map(|o| match o {
                    crate::lobby::table_instance::SeatOccupant::Player(player_id) => json!({
                        "playerId": player_id,
                        "username": player_id, // TODO: Map to actual username
                    }),
                    crate::lobby::table_instance::SeatOccupant::Reserved => json!({
                        "playerId": null,
                        "username": "Reserved",
                    }),
                })).collect::<Vec<_>>(),
                "readyStates": ready_states.clone(),
                "minPlayers": table.min_players,
                "maxPlayers": table.max_players,
            }
        });
        
        let _ = lobby.state.tx.send(msg.to_string());
    }
}

/// Broadcast table updated event  
fn broadcast_table_updated(lobby: &Arc<Lobby>, table_id: &str) {
    if let Some(table) = lobby.state.get_table(table_id) {
        let seats = table.seats.read();
        let ready_states = table.ready_states.read();
        let status = table.status.read();
        let countdown = table.countdown_ends_at.read();
        
        let msg = json!({
            "type": "tableUpdated",
            "tableId": table_id,
            "seats": seats.iter().map(|s| match s {
                Some(SeatOccupant::Player(id)) => Some(json!({
                    "playerId": id,
                    "username": id, // Using ID as username for now
                })),
                Some(SeatOccupant::Reserved) => Some(json!({
                    "reserved": true
                })),
                None => None,
            }).collect::<Vec<_>>(),
            "readyStates": ready_states.clone(),
            "status": format!("{:?}", *status),
            "countdownEndsAt": countdown.map(|t| {
                let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap();
                duration.as_secs()
            }),
        });
        
        let _ = lobby.state.tx.send(msg.to_string());
    }
}

/// Broadcast countdown started event
fn broadcast_countdown_started(lobby: &Arc<Lobby>, table_id: &str) {
    if let Some(table) = lobby.state.get_table(table_id) {
        let countdown = table.countdown_ends_at.read();
        
        if let Some(ends_at) = *countdown {
            let duration = ends_at.duration_since(std::time::UNIX_EPOCH).unwrap();
            let ends_at_secs = duration.as_secs();
            
            let msg = json!({
                "type": "countdownStarted",
                "tableId": table_id,
                "endsAt": ends_at_secs
            });
            
            let _ = lobby.state.tx.send(msg.to_string());
        }
    }
}

/// Broadcast chat message
fn broadcast_chat_message(lobby: &Arc<Lobby>, scope: &str, table_id: Option<&str>, username: &str, message: &str) {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    let timestamp = duration.as_secs();
    
    let msg = json!({
        "type": "chatMessage",
        "scope": scope,
        "tableId": table_id,
        "sender": username,
        "message": message,
        "timestamp": timestamp
    });
    
    let _ = lobby.state.tx.send(msg.to_string());
}

/// Create lobby joined message (for members)
fn create_lobby_joined_message(lobby: &Arc<Lobby>, username: &str) -> Value {
    let members = lobby.state.members.read();
    let tables = lobby.state.tables.lock();
    let name = lobby.state.name.lock().clone();
    let owner = lobby.state.owner.lock().clone();
    let archived = lobby.state.is_archived();
    
    json!({
        "type": "lobbyJoined",
        "lobby": {
            "id": lobby.state.id,
            "name": name,
            "owner": owner,
            "archived": archived,
            "inviteCode": lobby.state.invite_code,
            "myId": username,
            "members": members.iter().map(|m| {
                json!({
                    "username": m.name,
                    "connected": m.connected,
                })
            }).collect::<Vec<_>>(),
            "tables": tables.values().map(|t| {
                let status = t.status.read();
                let seats = t.seats.read();
                let ready_states = t.ready_states.read();
                json!({
                    "id": t.id,
                    "bundleId": t.bundle_id,
                    "owner": t.owner,
                    "status": format!("{:?}", *status),
                    "seats": seats.iter().map(|s| match s {
                        Some(SeatOccupant::Player(id)) => json!({"playerId": id, "username": id}),
                        Some(SeatOccupant::Reserved) => json!({"reserved": true}),
                        None => Value::Null,
                    }).collect::<Vec<_>>(),
                    "readyStates": ready_states.clone(),
                    "minPlayers": t.min_players,
                    "maxPlayers": t.max_players,
                    "countdownEndsAt": t.countdown_ends_at.read().map(|ends_at| {
                        let duration = ends_at.duration_since(std::time::UNIX_EPOCH).unwrap();
                        duration.as_secs()
                    }),
                })
            }).collect::<Vec<_>>(),
            "recentChat": lobby.state.chat.get_lobby_messages(50),
        }
    })
}

/// Create lobby view message (for non-members)
fn create_lobby_view_message(lobby: &Arc<Lobby>) -> Value {
    let members = lobby.state.members.read();
    let tables = lobby.state.tables.lock();
    let name = lobby.state.name.lock().clone();
    let owner = lobby.state.owner.lock().clone();
    let archived = lobby.state.is_archived();
    
    json!({
        "type": "lobbyView",
        "lobby": {
            "id": lobby.state.id,
            "name": name,
            "owner": owner,
            "archived": archived,
            "inviteCode": lobby.state.invite_code,
            "members": members.iter().map(|m| {
                json!({
                    "username": m.name,
                    "connected": m.connected,
                })
            }).collect::<Vec<_>>(),
            "tables": tables.values().map(|t| {
                let status = t.status.read();
                let seats = t.seats.read();
                let ready_states = t.ready_states.read();
                json!({
                    "id": t.id,
                    "bundleId": t.bundle_id,
                    "owner": t.owner,
                    "status": format!("{:?}", *status),
                    "seats": seats.iter().map(|s| match s {
                        Some(SeatOccupant::Player(id)) => json!({"playerId": id, "username": id}),
                        Some(SeatOccupant::Reserved) => json!({"reserved": true}),
                        None => Value::Null,
                    }).collect::<Vec<_>>(),
                    "readyStates": ready_states.clone(),
                    "minPlayers": t.min_players,
                    "maxPlayers": t.max_players,
                    "countdownEndsAt": t.countdown_ends_at.read().map(|ends_at| {
                        let duration = ends_at.duration_since(std::time::UNIX_EPOCH).unwrap();
                        duration.as_secs()
                    }),
                })
            }).collect::<Vec<_>>(),
            "recentChat": lobby.state.chat.get_lobby_messages(50),
        }
    })
}
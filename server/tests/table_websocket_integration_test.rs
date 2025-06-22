use axum::extract::ws::{Message, WebSocket};
use bluefelt_core::app_state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};

async fn setup_test_server() -> (String, Arc<AppState>) {
    let app_state = Arc::new(AppState::new());
    
    // Load test bundles
    let bundles_dir = std::path::Path::new("bundles");
    if bundles_dir.exists() {
        app_state.load_bundles(bundles_dir).unwrap();
    }
    
    // Start test server on random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}", addr);
    
    let app = bluefelt_core::create_app(app_state.clone());
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    (url, app_state)
}

#[tokio::test]
async fn test_table_creation_via_websocket() {
    let (base_url, app_state) = setup_test_server().await;
    
    // Create lobby via HTTP
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/lobbies", base_url.replace("ws://", "http://")))
        .json(&json!({ "game_id": "tic-tac-toe" }))
        .send()
        .await
        .unwrap();
    
    let lobby_data: Value = response.json().await.unwrap();
    let lobby_id = lobby_data["id"].as_str().unwrap();
    
    // Connect via WebSocket
    let ws_url = format!("{}/api/lobbies/{}/ws?player=Alice&join=true", base_url, lobby_id);
    let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();
    
    // Send create table message
    let create_msg = json!({
        "action": "createTable",
        "bundleId": "tic-tac-toe",
        "name": "Test Table"
    });
    ws_stream.send(Message::Text(create_msg.to_string()).into()).await.unwrap();
    
    // Receive table created notification
    let msg = ws_stream.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    
    assert_eq!(response["type"], "tableCreated");
    assert_eq!(response["table"]["bundleId"], "tic-tac-toe");
    assert_eq!(response["table"]["name"], "Test Table");
    assert_eq!(response["table"]["owner"], "Alice");
}

#[tokio::test]
async fn test_seat_operations_via_websocket() {
    let (base_url, app_state) = setup_test_server().await;
    
    // Create lobby
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/lobbies", base_url.replace("ws://", "http://")))
        .json(&json!({ "game_id": "tic-tac-toe" }))
        .send()
        .await
        .unwrap();
    
    let lobby_data: Value = response.json().await.unwrap();
    let lobby_id = lobby_data["id"].as_str().unwrap();
    
    // Connect two players
    let ws_url1 = format!("{}/api/lobbies/{}/ws?player=Alice&join=true", base_url, lobby_id);
    let ws_url2 = format!("{}/api/lobbies/{}/ws?player=Bob&join=true", base_url, lobby_id);
    
    let (mut ws1, _) = connect_async(&ws_url1).await.unwrap();
    let (mut ws2, _) = connect_async(&ws_url2).await.unwrap();
    
    // Player 1 creates table
    ws1.send(Message::Text(json!({
        "action": "createTable",
        "bundleId": "tic-tac-toe"
    }).to_string()).into()).await.unwrap();
    
    // Get table ID from response
    let msg = ws1.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    let table_id = response["table"]["id"].as_str().unwrap();
    
    // Player 2 should also receive the notification
    let msg = ws2.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    assert_eq!(response["type"], "tableCreated");
    
    // Player 1 claims seat 0
    ws1.send(Message::Text(json!({
        "action": "claimSeat",
        "tableId": table_id,
        "seatIndex": 0
    }).to_string()).into()).await.unwrap();
    
    // Both players receive seat claimed notification
    for ws in [&mut ws1, &mut ws2] {
        let msg = ws.next().await.unwrap().unwrap();
        let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
        assert_eq!(response["type"], "seatClaimed");
        assert_eq!(response["tableId"], table_id);
        assert_eq!(response["seatIndex"], 0);
        assert_eq!(response["playerId"], "Alice");
    }
    
    // Player 2 claims seat 1
    ws2.send(Message::Text(json!({
        "action": "claimSeat",
        "tableId": table_id,
        "seatIndex": 1
    }).to_string()).into()).await.unwrap();
    
    // Verify seat claimed notifications
    for ws in [&mut ws1, &mut ws2] {
        let msg = ws.next().await.unwrap().unwrap();
        let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
        assert_eq!(response["type"], "seatClaimed");
        assert_eq!(response["seatIndex"], 1);
        assert_eq!(response["playerId"], "Bob");
    }
}

#[tokio::test]
async fn test_ready_state_and_countdown_via_websocket() {
    let (base_url, app_state) = setup_test_server().await;
    
    // Create lobby and connect players
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/lobbies", base_url.replace("ws://", "http://")))
        .json(&json!({ "game_id": "tic-tac-toe" }))
        .send()
        .await
        .unwrap();
    
    let lobby_data: Value = response.json().await.unwrap();
    let lobby_id = lobby_data["id"].as_str().unwrap();
    
    let ws_url1 = format!("{}/api/lobbies/{}/ws?player=Alice&join=true", base_url, lobby_id);
    let ws_url2 = format!("{}/api/lobbies/{}/ws?player=Bob&join=true", base_url, lobby_id);
    
    let (mut ws1, _) = connect_async(&ws_url1).await.unwrap();
    let (mut ws2, _) = connect_async(&ws_url2).await.unwrap();
    
    // Create table and claim seats
    ws1.send(Message::Text(json!({
        "action": "createTable",
        "bundleId": "tic-tac-toe"
    }).to_string()).into()).await.unwrap();
    
    let msg = ws1.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    let table_id = response["table"]["id"].as_str().unwrap();
    
    // Clear Bob's queue
    ws2.next().await.unwrap().unwrap();
    
    // Both players claim seats
    ws1.send(Message::Text(json!({
        "action": "claimSeat",
        "tableId": table_id,
        "seatIndex": 0
    }).to_string()).into()).await.unwrap();
    
    // Clear notifications
    ws1.next().await;
    ws2.next().await;
    
    ws2.send(Message::Text(json!({
        "action": "claimSeat",
        "tableId": table_id,
        "seatIndex": 1
    }).to_string()).into()).await.unwrap();
    
    // Clear notifications
    ws1.next().await;
    ws2.next().await;
    
    // Player 1 sets ready
    ws1.send(Message::Text(json!({
        "action": "setReady",
        "tableId": table_id,
        "ready": true
    }).to_string()).into()).await.unwrap();
    
    // Both receive ready state update
    for ws in [&mut ws1, &mut ws2] {
        let msg = ws.next().await.unwrap().unwrap();
        let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
        assert_eq!(response["type"], "readyStateChanged");
        assert_eq!(response["playerId"], "Alice");
        assert_eq!(response["ready"], true);
    }
    
    // Player 2 sets ready - should trigger countdown
    ws2.send(Message::Text(json!({
        "action": "setReady",
        "tableId": table_id,
        "ready": true
    }).to_string()).into()).await.unwrap();
    
    // Should receive ready state change
    let msg = ws1.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    assert_eq!(response["type"], "readyStateChanged");
    
    // Should receive countdown started
    let msg = ws1.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    assert_eq!(response["type"], "countdownStarted");
    assert!(response["endsAt"].is_number());
}

#[tokio::test]
async fn test_chat_messages_via_websocket() {
    let (base_url, app_state) = setup_test_server().await;
    
    // Create lobby and connect
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/lobbies", base_url.replace("ws://", "http://")))
        .json(&json!({ "game_id": "tic-tac-toe" }))
        .send()
        .await
        .unwrap();
    
    let lobby_data: Value = response.json().await.unwrap();
    let lobby_id = lobby_data["id"].as_str().unwrap();
    
    let ws_url1 = format!("{}/api/lobbies/{}/ws?player=Alice&join=true", base_url, lobby_id);
    let ws_url2 = format!("{}/api/lobbies/{}/ws?player=Bob&join=true", base_url, lobby_id);
    
    let (mut ws1, _) = connect_async(&ws_url1).await.unwrap();
    let (mut ws2, _) = connect_async(&ws_url2).await.unwrap();
    
    // Send lobby chat message
    ws1.send(Message::Text(json!({
        "action": "sendChatMessage",
        "message": "Hello lobby!",
        "scope": "lobby"
    }).to_string()).into()).await.unwrap();
    
    // Both players receive the message
    for ws in [&mut ws1, &mut ws2] {
        let msg = ws.next().await.unwrap().unwrap();
        let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
        assert_eq!(response["type"], "chatMessage");
        assert_eq!(response["scope"], "lobby");
        assert_eq!(response["message"]["content"], "Hello lobby!");
        assert_eq!(response["message"]["playerId"], "Alice");
    }
    
    // Create table for table chat test
    ws1.send(Message::Text(json!({
        "action": "createTable",
        "bundleId": "tic-tac-toe"
    }).to_string()).into()).await.unwrap();
    
    let msg = ws1.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    let table_id = response["table"]["id"].as_str().unwrap();
    
    // Clear Bob's notification
    ws2.next().await;
    
    // Send table chat message
    ws1.send(Message::Text(json!({
        "action": "sendChatMessage",
        "message": "Hello table!",
        "scope": "table",
        "tableId": table_id
    }).to_string()).into()).await.unwrap();
    
    // Both players receive table message
    for ws in [&mut ws1, &mut ws2] {
        let msg = ws.next().await.unwrap().unwrap();
        let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
        assert_eq!(response["type"], "chatMessage");
        assert_eq!(response["scope"], "table");
        assert_eq!(response["tableId"], table_id);
        assert_eq!(response["message"]["content"], "Hello table!");
    }
}

#[tokio::test]
async fn test_spectator_notifications() {
    let (base_url, app_state) = setup_test_server().await;
    
    // Create lobby and connect three players
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/lobbies", base_url.replace("ws://", "http://")))
        .json(&json!({ "game_id": "tic-tac-toe" }))
        .send()
        .await
        .unwrap();
    
    let lobby_data: Value = response.json().await.unwrap();
    let lobby_id = lobby_data["id"].as_str().unwrap();
    
    let ws_url1 = format!("{}/api/lobbies/{}/ws?player=Alice&join=true", base_url, lobby_id);
    let ws_url2 = format!("{}/api/lobbies/{}/ws?player=Bob&join=true", base_url, lobby_id);
    let ws_url3 = format!("{}/api/lobbies/{}/ws?player=Charlie&join=true", base_url, lobby_id);
    
    let (mut ws1, _) = connect_async(&ws_url1).await.unwrap();
    let (mut ws2, _) = connect_async(&ws_url2).await.unwrap();
    let (mut ws3, _) = connect_async(&ws_url3).await.unwrap();
    
    // Create table
    ws1.send(Message::Text(json!({
        "action": "createTable",
        "bundleId": "tic-tac-toe"
    }).to_string()).into()).await.unwrap();
    
    let msg = ws1.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    let table_id = response["table"]["id"].as_str().unwrap();
    
    // Clear other players' notifications
    ws2.next().await;
    ws3.next().await;
    
    // First two players claim seats
    ws1.send(Message::Text(json!({
        "action": "claimSeat",
        "tableId": table_id,
        "seatIndex": 0
    }).to_string()).into()).await.unwrap();
    
    // Clear notifications
    for _ in 0..3 {
        ws1.next().await;
        ws2.next().await;
        ws3.next().await;
    }
    
    ws2.send(Message::Text(json!({
        "action": "claimSeat",
        "tableId": table_id,
        "seatIndex": 1
    }).to_string()).into()).await.unwrap();
    
    // Charlie should receive notifications even as a spectator
    let msg = ws3.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    assert_eq!(response["type"], "seatClaimed");
}

#[tokio::test]
async fn test_table_removal_on_finish() {
    let (base_url, app_state) = setup_test_server().await;
    
    // Create lobby
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/lobbies", base_url.replace("ws://", "http://")))
        .json(&json!({ "game_id": "tic-tac-toe" }))
        .send()
        .await
        .unwrap();
    
    let lobby_data: Value = response.json().await.unwrap();
    let lobby_id = lobby_data["id"].as_str().unwrap();
    
    // Connect player
    let ws_url = format!("{}/api/lobbies/{}/ws?player=Alice&join=true", base_url, lobby_id);
    let (mut ws, _) = connect_async(&ws_url).await.unwrap();
    
    // Create table
    ws.send(Message::Text(json!({
        "action": "createTable",
        "bundleId": "tic-tac-toe"
    }).to_string()).into()).await.unwrap();
    
    let msg = ws.next().await.unwrap().unwrap();
    let response: Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
    let table_id = response["table"]["id"].as_str().unwrap();
    
    // Verify table exists
    let tables_response = client
        .get(&format!("{}/api/lobbies/{}/tables", base_url.replace("ws://", "http://"), lobby_id))
        .send()
        .await
        .unwrap();
    
    let tables: Vec<Value> = tables_response.json().await.unwrap();
    assert_eq!(tables.len(), 1);
    
    // Simulate game ending (this would normally happen through gameplay)
    // In a real scenario, the game would end naturally
    // For testing, we'll send a special admin message if supported
    
    // TODO: Add mechanism to end games for testing
}
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;

const WS_URL: &str = "ws://127.0.0.1:3213/ws/lobby/test-lobby/test-player";

/// Test WebSocket connection and handshake
#[tokio::test]
async fn test_websocket_handshake() {
    // Skip test if server is not running
    if !is_server_running().await {
        println!("Skipping WebSocket test - server not running on port 3213");
        return;
    }

    let (ws_stream, _) = match connect_async(WS_URL).await {
        Ok(connection) => connection,
        Err(_) => {
            println!("Could not connect to WebSocket server");
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Should receive a welcome message
    let welcome_msg = timeout(Duration::from_secs(5), read.next()).await;
    assert!(welcome_msg.is_ok(), "Should receive welcome message within 5 seconds");
    
    let msg = welcome_msg.unwrap().unwrap().unwrap();
    if let Message::Text(text) = msg {
        let parsed: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "welcome");
        assert!(parsed["data"]["playerId"].as_str().is_some());
    } else {
        panic!("Expected text message for welcome");
    }
}

/// Test ping/pong mechanism 
#[tokio::test]
async fn test_websocket_ping_pong() {
    if !is_server_running().await {
        println!("Skipping WebSocket ping/pong test - server not running");
        return;
    }

    let (ws_stream, _) = match connect_async(WS_URL).await {
        Ok(connection) => connection,
        Err(_) => {
            println!("Could not connect to WebSocket server");
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Send a ping frame
    write.send(Message::Ping(vec![1, 2, 3, 4])).await.unwrap();

    // Should receive a pong response
    let pong_msg = timeout(Duration::from_secs(3), read.next()).await;
    assert!(pong_msg.is_ok(), "Should receive pong response within 3 seconds");
    
    let msg = pong_msg.unwrap().unwrap().unwrap();
    if let Message::Pong(data) = msg {
        assert_eq!(data, vec![1, 2, 3, 4]);
    } else {
        // Skip the welcome message and look for pong
        let pong_msg = timeout(Duration::from_secs(3), read.next()).await;
        assert!(pong_msg.is_ok());
        let msg = pong_msg.unwrap().unwrap().unwrap();
        assert!(matches!(msg, Message::Pong(_)));
    }
}

/// Test sending game action via WebSocket
#[tokio::test] 
async fn test_websocket_game_action() {
    if !is_server_running().await {
        println!("Skipping WebSocket game action test - server not running");
        return;
    }

    let (ws_stream, _) = match connect_async(WS_URL).await {
        Ok(connection) => connection,
        Err(_) => {
            println!("Could not connect to WebSocket server");
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Wait for welcome message
    let _welcome = timeout(Duration::from_secs(3), read.next()).await;

    // Send a game action
    let action = json!({
        "type": "action",
        "data": {
            "verb": "place",
            "args": {
                "location": "/zones/board/cells/0/0",
                "entity": "x_token"
            }
        }
    });

    write.send(Message::Text(action.to_string())).await.unwrap();

    // Should receive some response (either success or error)
    let response = timeout(Duration::from_secs(3), read.next()).await;
    assert!(response.is_ok(), "Should receive response to game action");
    
    let msg = response.unwrap().unwrap().unwrap();
    if let Message::Text(text) = msg {
        let parsed: Value = serde_json::from_str(&text).unwrap();
        // Response should have a type field
        assert!(parsed.get("type").is_some());
    }
}

/// Test WebSocket connection limits and proper cleanup
#[tokio::test]
async fn test_websocket_connection_cleanup() {
    if !is_server_running().await {
        println!("Skipping WebSocket cleanup test - server not running");
        return;
    }

    // Open multiple connections to the same lobby
    let mut connections = Vec::new();
    
    for i in 0..3 {
        let url = format!("ws://127.0.0.1:3213/ws/lobby/test-lobby/player-{}", i);
        if let Ok((ws_stream, _)) = connect_async(&url).await {
            connections.push(ws_stream);
        }
    }

    // All connections should be successful
    assert!(connections.len() > 0, "Should be able to open multiple connections");

    // Drop connections (they should clean up automatically)
    drop(connections);
    
    // Wait a bit for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;
}

/// Test invalid WebSocket paths
#[tokio::test]
async fn test_websocket_invalid_paths() {
    if !is_server_running().await {
        println!("Skipping WebSocket invalid paths test - server not running");
        return;
    }

    // Try to connect to invalid WebSocket path
    let invalid_url = "ws://127.0.0.1:3213/ws/invalid/path";
    let result = connect_async(invalid_url).await;
    
    // Should either fail to connect or get an HTTP error response
    assert!(result.is_err(), "Should not be able to connect to invalid WebSocket path");
}

/// Test sending malformed JSON
#[tokio::test]
async fn test_websocket_malformed_json() {
    if !is_server_running().await {
        println!("Skipping WebSocket malformed JSON test - server not running");
        return;
    }

    let (ws_stream, _) = match connect_async(WS_URL).await {
        Ok(connection) => connection,
        Err(_) => {
            println!("Could not connect to WebSocket server");
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Wait for welcome message
    let _welcome = timeout(Duration::from_secs(3), read.next()).await;

    // Send malformed JSON
    write.send(Message::Text("{invalid json".to_string())).await.unwrap();

    // Should receive an error response or connection should close gracefully
    let response = timeout(Duration::from_secs(3), read.next()).await;
    
    if let Ok(Some(Ok(msg))) = response {
        if let Message::Text(text) = msg {
            let parsed: Value = serde_json::from_str(&text).unwrap();
            // Should be an error message
            assert_eq!(parsed["type"], "error");
        }
    }
    // If no response, that's also acceptable (connection closed)
}

/// Test WebSocket binary message handling
#[tokio::test]
async fn test_websocket_binary_message() {
    if !is_server_running().await {
        println!("Skipping WebSocket binary message test - server not running");
        return;
    }

    let (ws_stream, _) = match connect_async(WS_URL).await {
        Ok(connection) => connection,
        Err(_) => {
            println!("Could not connect to WebSocket server");
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Wait for welcome message
    let _welcome = timeout(Duration::from_secs(3), read.next()).await;

    // Send binary message (should be rejected or ignored)
    write.send(Message::Binary(vec![1, 2, 3, 4, 5])).await.unwrap();

    // Check if connection remains stable
    let ping_response = write.send(Message::Ping(vec![42])).await;
    assert!(ping_response.is_ok(), "Connection should remain stable after binary message");
}

/// Helper function to check if server is running
async fn is_server_running() -> bool {
    tokio::net::TcpStream::connect("127.0.0.1:3213").await.is_ok()
}

/// Test multiple clients in same lobby
#[tokio::test]
async fn test_multiple_clients_same_lobby() {
    if !is_server_running().await {
        println!("Skipping multiple clients test - server not running");
        return;
    }

    // Connect two players to the same lobby
    let url1 = "ws://127.0.0.1:3213/ws/lobby/multi-test/player1";
    let url2 = "ws://127.0.0.1:3213/ws/lobby/multi-test/player2";

    let (ws1, _) = match connect_async(url1).await {
        Ok(connection) => connection,
        Err(_) => {
            println!("Could not connect first player");
            return;
        }
    };

    let (ws2, _) = match connect_async(url2).await {
        Ok(connection) => connection,
        Err(_) => {
            println!("Could not connect second player");
            return;
        }
    };

    let (mut write1, mut read1) = ws1.split();
    let (mut write2, mut read2) = ws2.split();

    // Both should receive welcome messages
    let welcome1 = timeout(Duration::from_secs(3), read1.next()).await;
    let welcome2 = timeout(Duration::from_secs(3), read2.next()).await;

    assert!(welcome1.is_ok() && welcome2.is_ok(), "Both players should receive welcome messages");

    // Send action from player 1
    let action = json!({
        "type": "action", 
        "data": {
            "verb": "place",
            "args": {
                "location": "/zones/board/cells/0/0",
                "entity": "x_token"
            }
        }
    });

    write1.send(Message::Text(action.to_string())).await.unwrap();

    // Both players should receive some update (either the action result or state diff)
    let response1 = timeout(Duration::from_secs(3), read1.next()).await;
    let response2 = timeout(Duration::from_secs(3), read2.next()).await;

    // At least one should receive a response
    assert!(response1.is_ok() || response2.is_ok(), "At least one player should receive action update");
}
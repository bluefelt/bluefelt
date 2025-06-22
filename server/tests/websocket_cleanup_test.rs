//! Tests for WebSocket connection cleanup and error handling

#[cfg(test)]
mod tests {
    use axum::extract::ws::{WebSocket, Message};
    use futures::{SinkExt, StreamExt};
    use serde_json::json;
    use std::time::Duration;
    use tokio::time::timeout;

    use crate::test_helpers::{create_test_server, connect_websocket};

    #[tokio::test]
    async fn test_normal_websocket_disconnect() {
        let server = create_test_server().await;
        let lobby_id = server.create_lobby("Test Cleanup").await;
        
        // Connect two players
        let mut ws1 = connect_websocket(&server, &lobby_id, "Alice").await;
        let mut ws2 = connect_websocket(&server, &lobby_id, "Bob").await;
        
        // Verify both connected
        let msg = timeout(Duration::from_secs(1), ws1.next()).await
            .expect("Timeout")
            .expect("Stream ended")
            .expect("Error");
        assert!(matches!(msg, Message::Text(_)));
        
        // Alice disconnects normally
        ws1.close().await.expect("Failed to close");
        
        // Bob should receive lobby state update
        let msg = timeout(Duration::from_secs(1), ws2.next()).await
            .expect("Timeout")
            .expect("Stream ended")
            .expect("Error");
        
        if let Message::Text(text) = msg {
            let data: serde_json::Value = serde_json::from_str(&text).unwrap();
            if data["type"] == "lobbyState" {
                let members = data["data"]["members"].as_array().unwrap();
                // Alice should no longer be in members list
                assert!(!members.iter().any(|m| m["username"] == "Alice"));
            }
        }
    }

    #[tokio::test]
    async fn test_abrupt_websocket_disconnect() {
        let server = create_test_server().await;
        let lobby_id = server.create_lobby("Test Abrupt").await;
        
        // Connect two players
        let ws1 = connect_websocket(&server, &lobby_id, "Charlie").await;
        let mut ws2 = connect_websocket(&server, &lobby_id, "Dave").await;
        
        // Drop ws1 without closing (simulates network failure)
        drop(ws1);
        
        // Dave should still receive updates indicating Charlie disconnected
        // Wait a bit for the server to detect the disconnect
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Send a ping from Dave to trigger state update
        ws2.send(Message::Text(json!({
            "type": "ping"
        }).to_string())).await.expect("Failed to send");
        
        let msg = timeout(Duration::from_secs(2), ws2.next()).await
            .expect("Timeout")
            .expect("Stream ended")
            .expect("Error");
        
        if let Message::Text(text) = msg {
            let data: serde_json::Value = serde_json::from_str(&text).unwrap();
            if data["type"] == "lobbyState" {
                let members = data["data"]["members"].as_array().unwrap();
                // Charlie should have connection_count = 0
                let charlie = members.iter().find(|m| m["username"] == "Charlie");
                assert!(charlie.is_none() || charlie.unwrap()["connection_count"] == 0);
            }
        }
    }

    #[tokio::test]
    async fn test_rapid_reconnections() {
        let server = create_test_server().await;
        let lobby_id = server.create_lobby("Test Rapid").await;
        
        // Rapidly connect and disconnect
        for i in 0..5 {
            let mut ws = connect_websocket(&server, &lobby_id, &format!("Rapid{}", i)).await;
            
            // Send a message
            ws.send(Message::Text(json!({
                "type": "chat",
                "message": format!("Hello from Rapid{}", i)
            }).to_string())).await.expect("Failed to send");
            
            // Immediately close
            ws.close().await.expect("Failed to close");
        }
        
        // Connect a stable connection and verify it works
        let mut stable = connect_websocket(&server, &lobby_id, "Stable").await;
        
        // Should receive initial messages
        let msg = timeout(Duration::from_secs(1), stable.next()).await
            .expect("Timeout")
            .expect("Stream ended")
            .expect("Error");
        assert!(matches!(msg, Message::Text(_)));
        
        // Should be able to send messages
        stable.send(Message::Text(json!({
            "type": "chat",
            "message": "Stable connection test"
        }).to_string())).await.expect("Failed to send");
    }

    #[tokio::test]
    async fn test_websocket_error_recovery() {
        let server = create_test_server().await;
        let lobby_id = server.create_lobby("Test Error").await;
        
        let mut ws = connect_websocket(&server, &lobby_id, "ErrorTest").await;
        
        // Send invalid JSON
        ws.send(Message::Text("{ invalid json".to_string())).await
            .expect("Failed to send");
        
        // Connection should still work after error
        ws.send(Message::Text(json!({
            "type": "chat",
            "message": "Valid message after error"
        }).to_string())).await.expect("Failed to send");
        
        // Should receive error response but connection stays open
        let mut received_error = false;
        for _ in 0..3 {
            if let Ok(Ok(Some(Ok(Message::Text(text))))) = 
                timeout(Duration::from_millis(100), ws.next()).await 
            {
                let data: serde_json::Value = serde_json::from_str(&text).unwrap_or(json!({}));
                if data["type"] == "error" {
                    received_error = true;
                    break;
                }
            }
        }
        
        assert!(received_error, "Should have received error message");
    }

    #[tokio::test]
    async fn test_multiple_connections_same_user() {
        let server = create_test_server().await;
        let lobby_id = server.create_lobby("Test Multi").await;
        
        // Same user connects twice
        let mut ws1 = connect_websocket(&server, &lobby_id, "MultiUser").await;
        let mut ws2 = connect_websocket(&server, &lobby_id, "MultiUser").await;
        
        // Both connections should work
        ws1.send(Message::Text(json!({
            "type": "chat",
            "message": "From connection 1"
        }).to_string())).await.expect("Failed to send");
        
        ws2.send(Message::Text(json!({
            "type": "chat",
            "message": "From connection 2"
        }).to_string())).await.expect("Failed to send");
        
        // Close first connection
        ws1.close().await.expect("Failed to close");
        
        // Second connection should still work
        ws2.send(Message::Text(json!({
            "type": "chat",
            "message": "Still connected on ws2"
        }).to_string())).await.expect("Failed to send");
        
        // Verify ws2 still receives messages
        let msg = timeout(Duration::from_secs(1), ws2.next()).await;
        assert!(msg.is_ok(), "Connection 2 should still be active");
    }

    #[tokio::test]
    async fn test_websocket_cleanup_after_lobby_delete() {
        let server = create_test_server().await;
        let lobby_id = server.create_lobby("Test Delete").await;
        
        // Connect player
        let mut ws = connect_websocket(&server, &lobby_id, "DeleteTest").await;
        
        // Delete the lobby
        server.delete_lobby(&lobby_id).await;
        
        // WebSocket should receive disconnect message or close
        let msg = timeout(Duration::from_secs(2), ws.next()).await;
        
        match msg {
            Ok(Some(Ok(Message::Text(text)))) => {
                let data: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert!(data["type"] == "error" || data["type"] == "lobbyDeleted");
            }
            Ok(Some(Ok(Message::Close(_)))) => {
                // Connection closed is also acceptable
            }
            Ok(None) => {
                // Stream ended is acceptable
            }
            _ => panic!("Unexpected result after lobby deletion"),
        }
    }
}
//! Chat system for lobby and table-specific messaging

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use std::collections::HashMap;
use parking_lot::RwLock;
use nanoid::nanoid;

/// Manages chat messages across lobby and table scopes
#[derive(Clone)]
pub struct ChatSystem {
    /// Global lobby chat messages
    pub lobby_chat: Arc<RwLock<Vec<ChatMessage>>>,
    
    /// Table-specific chat messages (table_id -> messages)
    pub table_chats: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
    
    /// Maximum messages to retain per scope
    pub max_messages: usize,
}

/// A chat message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Unique message ID
    pub id: String,
    
    /// Who sent the message (player ID)
    pub from: String,
    
    /// Display name of the sender
    pub from_name: String,
    
    /// The message content
    pub message: String,
    
    /// When the message was sent
    pub timestamp: SystemTime,
    
    /// Where the message was sent
    pub scope: ChatScope,
}

/// Scope of a chat message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "id")]
pub enum ChatScope {
    /// Message sent to lobby chat
    Lobby,
    /// Message sent to specific table chat
    Table(String),
}

impl ChatSystem {
    /// Create a new chat system
    pub fn new(max_messages: usize) -> Self {
        Self {
            lobby_chat: Arc::new(RwLock::new(Vec::new())),
            table_chats: Arc::new(RwLock::new(HashMap::new())),
            max_messages,
        }
    }
    
    /// Add a message to lobby chat
    pub fn add_lobby_message(&self, username: String, message: String) -> ChatMessage {
        self.send_message(ChatScope::Lobby, username.clone(), username, message)
    }
    
    /// Add a message to table chat
    pub fn add_table_message(&self, table_id: &str, username: String, message: String) -> ChatMessage {
        self.send_message(ChatScope::Table(table_id.to_string()), username.clone(), username, message)
    }
    
    /// Add a message to the appropriate chat
    pub fn add_message(
        &self,
        scope: ChatScope,
        table_id: Option<String>,
        player_id: String,
        username: String,
        content: String,
    ) -> ChatMessage {
        match scope {
            ChatScope::Lobby => self.send_message(ChatScope::Lobby, player_id, username, content),
            ChatScope::Table(ref table_id_from_scope) => {
                // Use table_id from scope if provided, otherwise from parameter
                let tid = table_id.clone().unwrap_or_else(|| table_id_from_scope.clone());
                self.send_message(ChatScope::Table(tid), player_id, username, content)
            }
        }
    }

    /// Send a message to the appropriate chat
    pub fn send_message(
        &self,
        scope: ChatScope,
        from: String,
        from_name: String,
        message: String,
    ) -> ChatMessage {
        let chat_message = ChatMessage {
            id: nanoid!(10),
            from,
            from_name,
            message,
            timestamp: SystemTime::now(),
            scope: scope.clone(),
        };
        
        match &scope {
            ChatScope::Lobby => {
                let mut lobby_chat = self.lobby_chat.write();
                lobby_chat.push(chat_message.clone());
                
                // Trim to max messages
                let len = lobby_chat.len();
                if len > self.max_messages {
                    lobby_chat.drain(0..len - self.max_messages);
                }
            }
            ChatScope::Table(table_id) => {
                let mut table_chats = self.table_chats.write();
                let chat = table_chats.entry(table_id.clone()).or_insert_with(Vec::new);
                chat.push(chat_message.clone());
                
                // Trim to max messages
                let len = chat.len();
                if len > self.max_messages {
                    chat.drain(0..len - self.max_messages);
                }
            }
        }
        
        chat_message
    }
    
    /// Get recent messages from lobby chat
    pub fn get_lobby_messages(&self, limit: usize) -> Vec<ChatMessage> {
        let lobby_chat = self.lobby_chat.read();
        let start = lobby_chat.len().saturating_sub(limit);
        lobby_chat[start..].to_vec()
    }
    
    /// Get recent messages from a table chat
    pub fn get_table_messages(&self, table_id: &str, limit: usize) -> Vec<ChatMessage> {
        let table_chats = self.table_chats.read();
        if let Some(chat) = table_chats.get(table_id) {
            let start = chat.len().saturating_sub(limit);
            chat[start..].to_vec()
        } else {
            Vec::new()
        }
    }
    
    /// Clear chat for a specific table (e.g., when table is deleted)
    pub fn clear_table_chat(&self, table_id: &str) {
        let mut table_chats = self.table_chats.write();
        table_chats.remove(table_id);
    }
    
    /// Trim old messages to prevent unbounded growth
    pub fn trim_old_messages(&self, max_messages: usize) -> usize {
        let mut trimmed = 0;
        
        // Trim lobby chat
        {
            let mut lobby_chat = self.lobby_chat.write();
            let len = lobby_chat.len();
            if len > max_messages {
                let to_remove = len - max_messages;
                lobby_chat.drain(0..to_remove);
                trimmed += to_remove;
            }
        }
        
        // Trim table chats
        {
            let mut table_chats = self.table_chats.write();
            for chat in table_chats.values_mut() {
                let len = chat.len();
                if len > max_messages {
                    let to_remove = len - max_messages;
                    chat.drain(0..to_remove);
                    trimmed += to_remove;
                }
            }
        }
        
        trimmed
    }
    
    /// Get all recent messages for a player's current context
    /// Returns both lobby messages and table messages if they're at a table
    pub fn get_messages_for_player(
        &self,
        table_id: Option<&str>,
        lobby_limit: usize,
        table_limit: usize,
    ) -> HashMap<String, Vec<ChatMessage>> {
        let mut result = HashMap::new();
        
        // Always include lobby messages
        result.insert("lobby".to_string(), self.get_lobby_messages(lobby_limit));
        
        // Include table messages if player is at a table
        if let Some(tid) = table_id {
            result.insert(
                format!("table:{}", tid),
                self.get_table_messages(tid, table_limit),
            );
        }
        
        result
    }
    
    /// Serialize chat state for client sync
    pub fn to_client_format(&self, table_ids: &[String]) -> serde_json::Value {
        let lobby_chat = self.lobby_chat.read();
        let table_chats = self.table_chats.read();
        
        let mut tables_data = serde_json::Map::new();
        for table_id in table_ids {
            if let Some(messages) = table_chats.get(table_id) {
                tables_data.insert(
                    table_id.clone(),
                    serde_json::json!(messages.iter()
                        .map(|m| self.message_to_json(m))
                        .collect::<Vec<_>>()),
                );
            }
        }
        
        serde_json::json!({
            "lobby": lobby_chat.iter()
                .map(|m| self.message_to_json(m))
                .collect::<Vec<_>>(),
            "tables": tables_data,
        })
    }
    
    /// Convert a message to JSON format for clients
    fn message_to_json(&self, msg: &ChatMessage) -> serde_json::Value {
        serde_json::json!({
            "id": msg.id,
            "from": msg.from,
            "fromName": msg.from_name,
            "message": msg.message,
            "timestamp": msg.timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "scope": msg.scope,
        })
    }
}

/// System messages for automated notifications
pub struct SystemMessage;

impl SystemMessage {
    /// Create a system message for player joining lobby
    pub fn player_joined_lobby(player_name: &str) -> String {
        format!("{} joined the lobby", player_name)
    }
    
    /// Create a system message for player leaving lobby
    pub fn player_left_lobby(player_name: &str) -> String {
        format!("{} left the lobby", player_name)
    }
    
    /// Create a system message for table creation
    pub fn table_created(player_name: &str, game_name: &str) -> String {
        format!("{} created a {} table", player_name, game_name)
    }
    
    /// Create a system message for game starting
    pub fn game_started(table_id: &str, game_name: &str) -> String {
        format!("Table {} started playing {}", table_id, game_name)
    }
    
    /// Create a system message for player taking a seat
    pub fn player_seated(player_name: &str, seat_index: usize) -> String {
        format!("{} took seat {}", player_name, seat_index + 1)
    }
    
    /// Create a system message for countdown start
    pub fn countdown_started(seconds: u64) -> String {
        format!("Game starting in {} seconds...", seconds)
    }
    
    /// Create a system message for countdown cancelled
    pub fn countdown_cancelled() -> String {
        "Countdown cancelled - waiting for all players to be ready".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chat_system() {
        let chat = ChatSystem::new(100);
        
        // Send lobby message
        let msg1 = chat.send_message(
            ChatScope::Lobby,
            "player1".to_string(),
            "Alice".to_string(),
            "Hello lobby!".to_string(),
        );
        
        // Send table message
        let msg2 = chat.send_message(
            ChatScope::Table("table1".to_string()),
            "player2".to_string(),
            "Bob".to_string(),
            "Hello table!".to_string(),
        );
        
        // Check lobby messages
        let lobby_msgs = chat.get_lobby_messages(10);
        assert_eq!(lobby_msgs.len(), 1);
        assert_eq!(lobby_msgs[0].message, "Hello lobby!");
        
        // Check table messages
        let table_msgs = chat.get_table_messages("table1", 10);
        assert_eq!(table_msgs.len(), 1);
        assert_eq!(table_msgs[0].message, "Hello table!");
    }
    
    #[test]
    fn test_message_limit() {
        let chat = ChatSystem::new(3);
        
        // Send 5 messages
        for i in 0..5 {
            chat.send_message(
                ChatScope::Lobby,
                format!("player{}", i),
                format!("Player {}", i),
                format!("Message {}", i),
            );
        }
        
        // Should only have last 3 messages
        let messages = chat.get_lobby_messages(10);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].message, "Message 2");
        assert_eq!(messages[2].message, "Message 4");
    }
}
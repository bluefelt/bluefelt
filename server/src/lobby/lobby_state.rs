//! Lobby state management separate from game state

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::{Mutex, RwLock};
use tokio::sync::broadcast;
use nanoid::nanoid;
use super::chat::ChatSystem;

/// Represents a member of a lobby
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyMember {
    /// Unique ID for this member (username)
    pub id: String,
    
    /// Display name
    pub name: String,
    
    /// When they joined the lobby
    pub joined_at: std::time::SystemTime,
    
    /// Current connection status
    pub connected: bool,
    
    /// Tables they're currently seated at
    pub active_tables: Vec<String>,
}

/// Persistent lobby state that spans multiple games
#[derive(Clone)]
pub struct LobbyState {
    /// Unique lobby ID
    pub id: String,
    
    /// Lobby name/title
    pub name: Arc<Mutex<String>>,
    
    /// Lobby owner/admin (username of the first player to join)
    pub owner: Arc<Mutex<Option<String>>>,
    
    /// Members in the lobby (read-heavy, so using RwLock)
    pub members: Arc<RwLock<Vec<LobbyMember>>>,
    
    /// Active tables in this lobby
    pub tables: Arc<Mutex<std::collections::HashMap<String, Arc<super::table_instance::TableInstance>>>>,
    
    /// Chat system for lobby and table chats
    pub chat: Arc<ChatSystem>,
    
    /// Invite code for joining the lobby
    pub invite_code: String,
    
    /// Broadcast channel for lobby-wide messages
    pub tx: broadcast::Sender<String>,
    
    /// When the lobby was created
    pub created_at: std::time::SystemTime,
    
    /// Lobby settings
    pub settings: LobbySettings,
    
    /// Whether the lobby is archived (no new joins allowed)
    pub archived: Arc<Mutex<bool>>,
}

/// Settings for a lobby
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbySettings {
    /// Maximum number of members allowed
    pub max_members: usize,
    
    /// Whether observers can join games in progress
    pub allow_observers: bool,
    
    /// Maximum number of concurrent tables
    pub max_concurrent_tables: usize,
    
    /// Whether to auto-close when empty
    pub auto_close: bool,
}

impl Default for LobbySettings {
    fn default() -> Self {
        Self {
            max_members: 20,
            allow_observers: true,
            max_concurrent_tables: 10,
            auto_close: true,
        }
    }
}

// Note: Completed tables are now tracked within each TableInstance
// and can be queried directly from the tables collection

impl LobbyState {
    /// Create a new lobby
    pub fn new(name: String) -> Self {
        // Increase buffer size to handle more messages without blocking
        let (tx, _) = broadcast::channel(1000);
        
        Self {
            id: nanoid!(10),
            name: Arc::new(Mutex::new(name)),
            owner: Arc::new(Mutex::new(None)),
            members: Arc::new(RwLock::new(Vec::new())),
            tables: Arc::new(Mutex::new(std::collections::HashMap::new())),
            chat: Arc::new(ChatSystem::new(200)), // Keep 200 messages per scope
            invite_code: nanoid!(8), // 8-character invite code
            tx,
            created_at: std::time::SystemTime::now(),
            settings: LobbySettings::default(),
            archived: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Add a member to the lobby
    pub fn add_member(&self, username: String) -> Result<(), String> {
        let mut members = self.members.write();
        
        // Check if already exists
        if members.iter().any(|m| m.id == username) {
            return Err("Member already in lobby".to_string());
        }
        
        // Check max members
        if members.len() >= self.settings.max_members {
            return Err("Lobby is full".to_string());
        }
        
        members.push(LobbyMember {
            id: username.clone(),
            name: username,
            joined_at: std::time::SystemTime::now(),
            connected: true,
            active_tables: Vec::new(),
        });
        
        Ok(())
    }
    
    /// Remove a member from the lobby
    pub fn remove_member(&self, username: &str) {
        let mut members = self.members.write();
        members.retain(|m| m.id != username);
    }
    
    /// Rename the lobby
    pub fn rename(&self, new_name: String) {
        let mut name = self.name.lock();
        *name = new_name;
    }
    
    /// Transfer ownership to the earliest joined member (excluding current owner)
    pub fn transfer_ownership(&self) -> Option<String> {
        let members = self.members.read();
        let current_owner = self.owner.lock().clone();
        
        // Find the earliest joined member who is not the current owner
        let new_owner = members.iter()
            .filter(|m| Some(&m.id) != current_owner.as_ref())
            .min_by_key(|m| m.joined_at)
            .map(|m| m.id.clone());
            
        if let Some(ref new_owner_id) = new_owner {
            let mut owner = self.owner.lock();
            *owner = Some(new_owner_id.clone());
        }
        
        new_owner
    }
    
    /// Archive the lobby (prevent new joins)
    pub fn archive(&self) {
        let mut archived = self.archived.lock();
        *archived = true;
    }
    
    /// Check if the lobby is archived
    pub fn is_archived(&self) -> bool {
        *self.archived.lock()
    }
    
    /// Mark a member as connected/disconnected
    pub fn set_member_connected(&self, username: &str, connected: bool) {
        let mut members = self.members.write();
        if let Some(member) = members.iter_mut().find(|m| m.id == username) {
            member.connected = connected;
        }
    }
    
    /// Get list of member IDs
    pub fn get_member_ids(&self) -> Vec<String> {
        self.members.read()
            .iter()
            .map(|m| m.id.clone())
            .collect()
    }
    
    /// Get list of connected members
    pub fn get_connected_members(&self) -> Vec<String> {
        self.members.read()
            .iter()
            .filter(|m| m.connected)
            .map(|m| m.id.clone())
            .collect()
    }
    
    /// Create a new table in this lobby
    pub fn create_table(&self, bundle_id: String, bundle: Arc<crate::bundle::Bundle>, owner: String) -> Result<String, String> {
        let mut tables = self.tables.lock();
        
        // Check max concurrent tables
        let active_tables = tables.values()
            .filter(|t| {
                let status = t.status.read();
                !matches!(*status, super::table_instance::TableStatus::Finished | super::table_instance::TableStatus::Abandoned)
            })
            .count();
            
        if active_tables >= self.settings.max_concurrent_tables {
            return Err("Too many active tables in lobby".to_string());
        }
        
        let table_instance = Arc::new(super::table_instance::TableInstance::new_simple(bundle_id, bundle, owner));
        let table_id = table_instance.id.clone();
        
        tables.insert(table_id.clone(), table_instance);
        
        Ok(table_id)
    }
    
    /// Get a table by ID
    pub fn get_table(&self, table_id: &str) -> Option<Arc<super::table_instance::TableInstance>> {
        self.tables.lock().get(table_id).cloned()
    }
    
    /// Note: Seat claiming is now handled by SeatManager in table_instance.rs
    /// This method is deprecated and will be removed
    
    /// Update member's active tables when they leave
    pub fn leave_table(&self, table_id: &str, member_id: &str) {
        // Update member's active tables
        let mut members = self.members.write();
        if let Some(member) = members.iter_mut().find(|m| m.id == member_id) {
            member.active_tables.retain(|t| t != table_id);
        }
    }
    
    /// Archive completed tables (now handled automatically by tables)
    pub fn cleanup_abandoned_tables(&self) {
        let mut tables = self.tables.lock();
        
        // Remove abandoned tables after a timeout
        let abandoned_ids: Vec<String> = tables.iter()
            .filter(|(_, table)| {
                let status = table.status.read();
                matches!(*status, super::table_instance::TableStatus::Abandoned)
            })
            .map(|(id, _)| id.clone())
            .collect();
            
        for table_id in abandoned_ids {
            if let Some(table) = tables.remove(&table_id) {
                // Clean up table chat
                self.chat.clear_table_chat(&table_id);
                
                // Update member's active tables
                let seated_players = table.get_seated_players();
                let mut members = self.members.write();
                for player_id in seated_players {
                    if let Some(member) = members.iter_mut().find(|m| m.id == player_id) {
                        member.active_tables.retain(|t| t != &table_id);
                    }
                }
            }
        }
    }
    
    /// Check if lobby should auto-close
    pub fn should_auto_close(&self) -> bool {
        if !self.settings.auto_close {
            return false;
        }
        
        let members = self.members.read();
        // Only auto-close if truly empty (not just all disconnected)
        members.is_empty()
    }
    
    /// Get spectators for a table
    pub fn get_table_spectators(&self, table_id: &str) -> Vec<String> {
        // Get table first, then release lock before acquiring members lock
        let table = self.tables.lock().get(table_id).cloned();
        
        if let Some(table) = table {
            // Tables now have dedicated spectator lists
            table.spectators.read().clone()
        } else {
            Vec::new()
        }
    }
    
    /// Check if a member is already seated at any table
    pub fn is_member_seated_anywhere(&self, member_id: &str) -> bool {
        let members = self.members.read();
        if let Some(member) = members.iter().find(|m| m.id == member_id) {
            !member.active_tables.is_empty()
        } else {
            false
        }
    }
    
    /// Update member's active tables when they take a seat
    pub fn add_member_to_table(&self, table_id: &str, member_id: &str) -> Result<(), String> {
        // Check if member exists
        let mut members = self.members.write();
        if let Some(member) = members.iter_mut().find(|m| m.id == member_id) {
            // For now, allow multiple table seating
            // TODO: Add policy setting to control this behavior
            if !member.active_tables.contains(&table_id.to_string()) {
                member.active_tables.push(table_id.to_string());
            }
            Ok(())
        } else {
            Err("Member not in lobby".to_string())
        }
    }
    
    /// Remove member from a table's active list
    pub fn remove_member_from_table(&self, table_id: &str, member_id: &str) {
        let mut members = self.members.write();
        if let Some(member) = members.iter_mut().find(|m| m.id == member_id) {
            member.active_tables.retain(|t| t != table_id);
        }
    }
}
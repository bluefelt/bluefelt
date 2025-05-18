use serde::{Serialize, Deserialize};
use uuid::Uuid;
use dashmap::{DashMap, DashSet};

#[derive(Serialize, Deserialize, Clone)]
pub struct Lobby {
    pub id: Uuid,
    pub game_id: String,
}

pub type LobbyMap = DashMap<Uuid, Lobby>;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

/// Map of user name to user info
pub type UserMap = DashMap<String, User>;

/// Tracks which users are in which lobby
pub type LobbyMembers = DashMap<Uuid, DashSet<String>>;

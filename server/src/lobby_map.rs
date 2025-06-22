//! Lobby map for storing lobby instances

use dashmap::DashMap;
use std::sync::Arc;
use crate::lobby::lobby_impl::Lobby;

/// Type alias for the lobby map
pub type LobbyMap = DashMap<String, Arc<Lobby>>;

/// Create a new lobby map
pub fn create_lobby_map() -> Arc<LobbyMap> {
    Arc::new(DashMap::new())
}
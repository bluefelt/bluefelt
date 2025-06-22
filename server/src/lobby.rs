//! Lobby module - manages game lobbies, players, and WebSocket connections

pub mod action_map;
pub mod game_instance;
pub mod table_instance;
pub mod chat;
pub mod countdown_manager;
pub mod lobby_state;
pub mod lobby_impl;
pub mod seat_manager;
pub mod websocket;
pub mod rng;
pub mod lock_helpers;
// pub mod performance; // TODO: Update for new architecture if needed
pub mod connection_manager;
pub mod memory_manager;
pub mod lock_optimization;

pub use self::action_map::compute_action_map;

// LobbyMap type for storing lobby instances
use dashmap::DashMap;
use std::sync::Arc;
pub type LobbyMap = DashMap<String, Arc<lobby_impl::Lobby>>;
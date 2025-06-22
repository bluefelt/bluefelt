//! Application state for handlers

use std::sync::Arc;
use crate::bundle::BundleMap;
use crate::lobby::LobbyMap;
use crate::lobby::connection_manager::ConnectionManager;
use crate::lobby::memory_manager::MemoryManager;
#[cfg(debug_assertions)]
use crate::reload_notifier::ReloadNotifier;

#[derive(Clone)]
pub struct AppState {
    pub bundles: Arc<BundleMap>,
    pub lobbies: Arc<LobbyMap>,
    pub connection_manager: Arc<ConnectionManager>,
    pub memory_manager: Arc<MemoryManager>,
    #[cfg(debug_assertions)]
    pub reload_notifier: ReloadNotifier,
}

impl AppState {
    pub fn new(
        bundles: Arc<BundleMap>,
        lobbies: Arc<LobbyMap>,
        connection_manager: Arc<ConnectionManager>,
        memory_manager: Arc<MemoryManager>,
    ) -> Self {
        Self {
            bundles,
            lobbies,
            connection_manager,
            memory_manager,
            #[cfg(debug_assertions)]
            reload_notifier: ReloadNotifier::new(),
        }
    }
}
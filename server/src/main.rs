use axum::{
    routing::{get, post, delete},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

mod bundle;
mod conditions;
mod engine;
mod http_handlers;
mod lobby;
mod lobby_map;
mod message_format;
mod shorthand;
mod utils;
mod app_state;
mod validation;
mod ws_handlers;
#[cfg(debug_assertions)]
mod debug_handlers;
#[cfg(debug_assertions)]
mod reload_notifier;
#[cfg(debug_assertions)]
mod reload_handlers;

use crate::bundle::BundleMap;
use crate::lobby_map::create_lobby_map;
use crate::app_state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundles = Arc::new(bundle::load_bundles_from_dir("bundles").map_err(|e| e.to_string())?);
    let lobbies = create_lobby_map();
    
    // Create resource managers
    let (connection_manager, cleanup_rx) = lobby::connection_manager::ConnectionManager::new();
    let connection_manager = Arc::new(connection_manager);
    let memory_manager = Arc::new(lobby::memory_manager::MemoryManager::new(Default::default()));
    
    // Start cleanup tasks
    let connections = Arc::clone(&connection_manager.connections);
    let lobbies_for_cleanup = Arc::clone(&connection_manager.lobbies);
    let reconnection_tokens = Arc::clone(&connection_manager.reconnection_tokens);
    tokio::spawn(lobby::connection_manager::ConnectionManager::start_cleanup_task(
        connections,
        lobbies_for_cleanup,
        cleanup_rx,
        reconnection_tokens,
    ));
    
    let memory_manager_clone = Arc::clone(&memory_manager);
    let lobbies_for_memory = Arc::clone(&lobbies);
    tokio::spawn(lobby::memory_manager::MemoryManager::run_cleanup_task(
        memory_manager_clone,
        lobbies_for_memory,
    ));
    
    let state = AppState::new(
        bundles.clone(),
        lobbies.clone(),
        connection_manager.clone(),
        memory_manager.clone(),
    );
    
    let app = Router::new()
        // Health check
        .route("/health", get(|| async { "OK" }))
        
        // API routes
        .route("/api/lobbies", 
            post(http_handlers::create_lobby)
            .get(http_handlers::list_lobbies)
        )
        .route("/api/lobbies/:lobby_id", 
            get(http_handlers::get_lobby)
            .delete(http_handlers::delete_lobby)
        )
        .route("/api/lobbies/:lobby_id/ws", get(ws_handlers::websocket_handler))
        .route("/api/games", get(http_handlers::list_games))
        
        // Table management routes
        .route("/api/lobbies/:lobby_id/tables", 
            post(http_handlers::create_table)
            .get(http_handlers::list_tables)
        )
        .route("/api/lobbies/:lobby_id/tables/:table_id", 
            delete(http_handlers::delete_table)
        )
        .route("/api/lobbies/:lobby_id/tables/:table_id/seats/:seat_index/claim", 
            post(http_handlers::claim_seat)
        )
        .route("/api/lobbies/:lobby_id/tables/:table_id/seats/:seat_index/release", 
            post(http_handlers::release_seat)
        )
        .route("/api/lobbies/:lobby_id/tables/:table_id/ready", 
            post(http_handlers::set_ready_state)
        );
        
    // Add debug routes in debug builds
    #[cfg(debug_assertions)]
    let app = app
        .route("/api/debug", get(debug_handlers::debug_info))
        .route("/api/debug/lobby/:lobby_id", get(debug_handlers::debug_lobby))
        .route("/api/debug/lobby/:lobby_id/table/:table_id/state", 
            get(debug_handlers::debug_game_state))
        .route("/api/debug/test-state", post(debug_handlers::create_test_state))
        .route("/api/reload/ws", get(reload_handlers::reload_websocket))
        .route("/api/reload/bundles", post(reload_handlers::reload_bundles))
        .route("/api/reload/status", get(reload_handlers::reload_status));
        
    let app = app
        // CORS for development
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);
    
    let addr: SocketAddr = "0.0.0.0:8000".parse()?;
    println!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
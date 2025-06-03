use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use axum::extract::ws::Message;
use std::sync::Arc;
use tokio::sync::broadcast;

mod bundle;
mod engine;
mod lobby;
mod utils;
mod shorthand;
mod validation;
mod message_format;
mod http_handlers;
mod ws_handlers;
mod conditions;

use bundle::BundleMap;
use lobby::LobbyMap;
use http_handlers::{create_lobby, list_lobbies, list_games, get_game, get_lobby};
use ws_handlers::{lobbies_ws_handler, ws_handler};

const DEFAULT_PORT: u16 = 8000;
const LOBBY_UPDATE_CHANNEL_SIZE: usize = 16;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bundles = load_bundles()?;
    let (lobbies, lobby_updates_tx) = setup_lobby_system();
    let app = build_app(bundles, lobbies, lobby_updates_tx);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT));
    println!("Server started on http://{}", addr);
    
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

fn load_bundles() -> anyhow::Result<BundleMap> {
    let bundles_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bundles");
    BundleMap::load_dir(bundles_dir.to_str().unwrap())
}

fn setup_lobby_system() -> (Arc<LobbyMap>, broadcast::Sender<Message>) {
    let lobbies = Arc::new(LobbyMap::default());
    let (lobby_updates_tx, _) = broadcast::channel::<Message>(LOBBY_UPDATE_CHANNEL_SIZE);
    (lobbies, lobby_updates_tx)
}

fn build_app(
    bundles: BundleMap,
    lobbies: Arc<LobbyMap>,
    lobby_updates_tx: broadcast::Sender<Message>,
) -> Router {
    // Clone for each route handler
    let bundles_for_games = bundles.clone();
    let bundles_for_lobbies = bundles.clone();
    let bundles_for_manifest = bundles.clone();
    let lobbies_for_lobbies_route = lobbies.clone();
    let lobbies_for_ws = lobbies.clone();
    let lobbies_for_create = lobbies.clone();
    let lobbies_for_ws_list = lobbies.clone();
    let lobby_updates_for_create = lobby_updates_tx.clone();
    let lobby_updates_for_ws_list = lobby_updates_tx.clone();

    let cors = build_cors_layer();

    Router::new()
        .route("/api/games", get(move || list_games(bundles_for_games.clone())))
        .route("/api/games/:id", get(move |path| get_game(path, bundles_for_manifest.clone())))
        .route("/api/lobbies", post(
            move |req| create_lobby(
                req,
                bundles_for_lobbies.clone(),
                lobbies_for_create.clone(),
                lobby_updates_for_create.clone(),
            )
        ).get(
            move || list_lobbies(lobbies_for_lobbies_route.clone())
        ))
        .route("/api/lobbies/:id", get(move |path| get_lobby(path, lobbies.clone())))
        .route("/api/lobbies/ws", get(
            move |ws| lobbies_ws_handler(ws, lobby_updates_for_ws_list.clone(), lobbies_for_ws_list.clone())
        ))
        .route("/api/lobbies/:id/ws", get(
            move |path, ws, query| ws_handler(path, ws, query, lobbies_for_ws.clone())
        ))
        .layer(cors)
}

fn build_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)       // Allow any origin for development
        .allow_methods(Any)      // Allow all methods 
        .allow_headers(Any)      // Allow all headers
        .allow_credentials(false) // Must be false when using wildcard headers
        .expose_headers(Any)     // Use Any to expose all headers
}


//! Watch command - file watching and hot reload

use anyhow::{Context, Result};
use colored::Colorize;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::build;

/// Run the watch command
pub async fn run(
    game_path: PathBuf,
    serve: bool,
    open: bool,
    port: u16,
) -> Result<()> {
    info!("Watching {} for changes...", game_path.display());

    // Initial build
    let output_dir = PathBuf::from("dist");
    match build::run(game_path.clone(), output_dir.clone(), false, false).await {
        Ok(_) => info!("{}", "Initial build successful".green()),
        Err(e) => {
            error!("Initial build failed: {}", e);
            warn!("Waiting for valid files...");
        }
    }

    // Start development server if requested
    if serve {
        start_dev_server(port, open).await?;
    }

    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(1)),
    )?;

    // Watch the game directory
    watcher.watch(&game_path, RecursiveMode::Recursive)?;

    info!("Watching for file changes... Press Ctrl+C to stop");

    // Process file change events
    loop {
        match rx.recv() {
            Ok(event) => {
                if should_rebuild(&event) {
                    info!("{}", "File change detected, rebuilding...".yellow());
                    
                    // Small delay to let file operations complete
                    sleep(Duration::from_millis(100)).await;
                    
                    match build::run(game_path.clone(), output_dir.clone(), false, false).await {
                        Ok(_) => {
                            info!("{}", "✓ Rebuild successful".green());
                            
                            // TODO: Send hot reload signal to connected clients
                            if serve {
                                debug!("TODO: Send hot reload signal to clients");
                            }
                        }
                        Err(e) => {
                            error!("Build failed: {}", e);
                            error!("Fix the errors and save to retry");
                        }
                    }
                }
            }
            Err(e) => {
                error!("Watch error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Check if an event should trigger a rebuild
fn should_rebuild(event: &Event) -> bool {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
            // Check if any path is a relevant file
            event.paths.iter().any(|path| {
                if let Some(ext) = path.extension() {
                    matches!(
                        ext.to_str(),
                        Some("yaml") | Some("yml") | Some("rs") | Some("toml")
                    )
                } else {
                    false
                }
            })
        }
        _ => false,
    }
}

/// Start the development server
async fn start_dev_server(port: u16, open: bool) -> Result<()> {
    info!("Starting development server on port {}", port);
    
    // TODO: Actually start the Bluefelt server
    // For now, just print a message
    warn!("Development server integration not yet implemented");
    warn!("Please run the server manually: cd server && cargo run");
    
    if open {
        // TODO: Open browser
        warn!("Browser auto-open not yet implemented");
    }
    
    Ok(())
}
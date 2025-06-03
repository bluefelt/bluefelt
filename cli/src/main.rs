use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod build;
mod validate;
mod watch;
mod scaffold;
mod bundle;

#[derive(Parser)]
#[command(name = "bluefelt-cli")]
#[command(about = "Bluefelt game authoring and build tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Build game bundle from YAML sources
    Build {
        /// Path to game directory (containing manifest.yaml)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "dist")]
        output: PathBuf,

        /// Create a .bf bundle archive
        #[arg(long)]
        zip: bool,

        /// Build in release mode (optimize WASM)
        #[arg(long)]
        release: bool,
    },

    /// Build all games from games directory
    BuildAll {
        /// Path to games directory (containing subdirectories with game versions)
        #[arg(default_value = "games")]
        games_dir: PathBuf,

        /// Output directory for built bundles
        #[arg(short, long, default_value = "bundles")]
        output: PathBuf,

        /// Build in release mode (optimize WASM)
        #[arg(long)]
        release: bool,
    },

    /// Validate game files without building
    Validate {
        /// Path to game directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Expected spec version
        #[arg(long, default_value = "1")]
        spec_version: String,
    },

    /// Watch for changes and rebuild automatically
    Watch {
        /// Path to game directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Start local development server
        #[arg(long)]
        serve: bool,

        /// Open browser automatically
        #[arg(long)]
        open: bool,

        /// Server port
        #[arg(long, default_value = "8080")]
        port: u16,
    },

    /// Create a new game from template
    Scaffold {
        /// Name of the game
        name: String,

        /// Type of scaffold
        #[arg(short, long, default_value = "game")]
        template: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Clean up unused bundles (not implemented)
    Prune {
        /// Dry run - show what would be deleted
        #[arg(long)]
        dry_run: bool,
    },

    /// Migrate game files between spec versions (not implemented)
    Migrate {
        /// Source spec version
        #[arg(long)]
        from: String,

        /// Target spec version
        #[arg(long)]
        to: String,

        /// Path to game directory
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = match cli.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    match cli.command {
        Commands::Build { path, output, zip, release } => {
            build::run(path, output, zip, release).await?;
        }
        Commands::BuildAll { games_dir, output, release } => {
            build::run_all(games_dir, output, release).await?;
        }
        Commands::Validate { path, spec_version } => {
            validate::run(path, spec_version).await?;
        }
        Commands::Watch { path, serve, open, port } => {
            watch::run(path, serve, open, port).await?;
        }
        Commands::Scaffold { name, template, output } => {
            scaffold::run(template, name, output)?;
        }
        Commands::Prune { dry_run } => {
            eprintln!("Prune command not yet implemented");
            std::process::exit(1);
        }
        Commands::Migrate { from, to, path } => {
            eprintln!("Migrate command not yet implemented");
            std::process::exit(1);
        }
    }

    Ok(())
}
//! Scaffold command - generate game templates

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// Run the scaffold command
pub fn run(
    template: String,
    name: String,
    output: Option<PathBuf>,
) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from(&name));
    
    match template.as_str() {
        "game" => scaffold_game(&name, &output_dir),
        _ => anyhow::bail!("Unknown template type: {}", template),
    }
}

/// Scaffold a new game
fn scaffold_game(name: &str, output_dir: &Path) -> Result<()> {
    info!("Creating new game '{}' at {}", name, output_dir.display());

    // Create directory
    fs::create_dir_all(output_dir)
        .context("Failed to create output directory")?;

    // Generate game ID from name
    let game_id = name
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_ascii_lowercase() || *c == '-')
        .collect::<String>();

    // Create manifest.yaml
    let manifest = format!(
        r#"gameId: {}
version: 1.0
specVersion: 1
metadata:
  name: "{}"
  description: "A new Bluefelt game"
  author: "Your Name"
  players: 2
"#,
        game_id, name
    );
    fs::write(output_dir.join("manifest.yaml"), manifest)?;

    // Create entities.yaml
    let entities = r#"# Game entities (pieces, cards, tokens, etc.)
- id: token_{player}
  name: "{player}'s Token"
  type: token
  props:
    owner: "{player}"
"#;
    fs::write(output_dir.join("entities.yaml"), entities)?;

    // Create zones.yaml
    let zones = r#"# Game zones (board, hands, decks, etc.)
- id: board
  name: "Game Board"
  type: grid
  shape: [3, 3]
  capacity: 1
"#;
    fs::write(output_dir.join("zones.yaml"), zones)?;

    // Create actions.yaml
    let actions = r#"# Player actions
- id: placeToken
  uses: presets.grid.place
  with:
    entity: token_{actor}
    target:
      zone: board
  ui:
    direction: "Place your token"

- id: advanceTurn
  uses: presets.turn.advance
  auto: true
"#;
    fs::write(output_dir.join("actions.yaml"), actions)?;

    // Create phases.yaml
    let phases = format!(
        r#"# Game phases
game:
  - id: setup
    type: automatic
    enterActions:
      - transitionToPhase: game.play
  
  - id: play
    type: playerAction
    prompt: "Place your token on the board"
    possibleActions:
      - placeToken
"#
    );
    fs::write(output_dir.join("phases.yaml"), phases)?;

    // Create .gitignore
    let gitignore = r#"dist/
*.bf
.DS_Store
"#;
    fs::write(output_dir.join(".gitignore"), gitignore)?;

    // Create README.md
    let readme = format!(
        r#"# {}

A Bluefelt game.

## Development

Build the game:
```bash
bluefelt-cli build
```

Watch for changes:
```bash
bluefelt-cli watch --serve
```

## Game Rules

TODO: Add your game rules here.
"#,
        name
    );
    fs::write(output_dir.join("README.md"), readme)?;

    println!("\n{}", "✓ Game scaffolded successfully!".green().bold());
    println!("\nNext steps:");
    println!("  cd {}", output_dir.display());
    println!("  bluefelt-cli build");
    println!("  bluefelt-cli watch --serve");

    Ok(())
}
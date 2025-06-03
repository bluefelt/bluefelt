//! Build command - transforms YAML to JSON and creates game bundles

use anyhow::{Context, Result};
use colored::Colorize;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::bundle::{Bundle, BundleManifest};
use crate::validate;

/// Main entry point for the build command
pub async fn run(
    game_path: PathBuf,
    output_dir: PathBuf,
    create_zip: bool,
    release_mode: bool,
) -> Result<()> {
    info!("Building game from {}", game_path.display());

    // Step 1: Validate the game files
    let validation_result = validate::validate_game(&game_path).await?;
    if !validation_result.is_valid() {
        anyhow::bail!("Validation failed. Run 'bluefelt-cli validate' for details.");
    }

    // Step 2: Load all YAML files
    let manifest = load_yaml::<BundleManifest>(&game_path.join("manifest.yaml"))
        .context("Failed to load manifest.yaml")?;
    
    let entities = if game_path.join("entities.yaml").exists() {
        Some(load_yaml_raw(&game_path.join("entities.yaml"))?)
    } else {
        None
    };

    let zones = if game_path.join("zones.yaml").exists() {
        Some(load_yaml_raw(&game_path.join("zones.yaml"))?)
    } else {
        None
    };

    let actions = if game_path.join("actions.yaml").exists() {
        Some(load_yaml_raw(&game_path.join("actions.yaml"))?)
    } else {
        None
    };

    let phases = if game_path.join("phases.yaml").exists() {
        Some(load_yaml_raw(&game_path.join("phases.yaml"))?)
    } else {
        None
    };

    // Step 3: Process templates if they exist
    // TODO: Implement template expansion for entities

    // Step 4: Convert YAML to JSON
    let mut bundle = Bundle {
        manifest: manifest.clone(),
        entities: entities.map(yaml_to_json),
        zones: zones.map(yaml_to_json),
        actions: actions.map(yaml_to_json),
        phases: phases.map(yaml_to_json),
        hooks: None,
    };

    // Step 5: Compile hooks if they exist
    if let Some(hooks_wasm) = compile_hooks(&game_path, release_mode).await? {
        bundle.hooks = Some(hooks_wasm);
    }

    // Step 6: Calculate bundle hash
    let bundle_hash = calculate_bundle_hash(&bundle)?;
    info!("Bundle hash: {}", bundle_hash.green());

    // Step 7: Create output directory
    let bundle_output_dir = output_dir.join(&bundle_hash);
    fs::create_dir_all(&bundle_output_dir)
        .context("Failed to create output directory")?;

    // Step 8: Write JSON files
    write_json_file(&bundle_output_dir.join("manifest.json"), &bundle.manifest)?;
    
    if let Some(entities) = &bundle.entities {
        write_json_file(&bundle_output_dir.join("entities.json"), entities)?;
    }
    
    if let Some(zones) = &bundle.zones {
        write_json_file(&bundle_output_dir.join("zones.json"), zones)?;
    }
    
    if let Some(actions) = &bundle.actions {
        write_json_file(&bundle_output_dir.join("actions.json"), actions)?;
    }
    
    if let Some(phases) = &bundle.phases {
        write_json_file(&bundle_output_dir.join("phases.json"), phases)?;
    }

    // Step 9: Write hooks.wasm if it exists
    if let Some(hooks) = &bundle.hooks {
        fs::write(bundle_output_dir.join("hooks.wasm"), hooks)
            .context("Failed to write hooks.wasm")?;
    }

    // Step 10: Copy assets if they exist
    let assets_dir = game_path.join("assets");
    if assets_dir.exists() {
        copy_dir_recursive(&assets_dir, &bundle_output_dir.join("assets"))?;
    }

    // Step 11: Create .bf archive if requested
    if create_zip {
        create_bundle_archive(&bundle_output_dir, &output_dir, &bundle_hash)?;
    }

    info!(
        "{}",
        format!("✓ Build complete: {}", bundle_output_dir.display()).green()
    );

    Ok(())
}

/// Load a YAML file and deserialize it
fn load_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    
    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML from {}", path.display()))
}

/// Load raw YAML value
fn load_yaml_raw(path: &Path) -> Result<YamlValue> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    
    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML from {}", path.display()))
}

/// Convert YAML value to JSON value
fn yaml_to_json(yaml: YamlValue) -> JsonValue {
    match yaml {
        YamlValue::Null => JsonValue::Null,
        YamlValue::Bool(b) => JsonValue::Bool(b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                JsonValue::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
            } else {
                JsonValue::Null
            }
        }
        YamlValue::String(s) => JsonValue::String(s),
        YamlValue::Sequence(seq) => {
            JsonValue::Array(seq.into_iter().map(yaml_to_json).collect())
        }
        YamlValue::Mapping(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                if let YamlValue::String(key) = k {
                    json_map.insert(key, yaml_to_json(v));
                }
            }
            JsonValue::Object(json_map)
        }
        _ => JsonValue::Null,
    }
}

/// Write JSON to file with pretty formatting
fn write_json_file<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Compile hooks if they exist
async fn compile_hooks(game_path: &Path, release_mode: bool) -> Result<Option<Vec<u8>>> {
    let hooks_dir = game_path.join("hooks");
    if !hooks_dir.exists() {
        return Ok(None);
    }

    // Check for Rust hooks
    if hooks_dir.join("Cargo.toml").exists() {
        info!("Compiling Rust hooks...");
        compile_rust_hooks(&hooks_dir, release_mode).await
    } else {
        warn!("Hooks directory exists but no supported language found");
        Ok(None)
    }
}

/// Compile Rust hooks to WASM
async fn compile_rust_hooks(hooks_dir: &Path, release_mode: bool) -> Result<Option<Vec<u8>>> {
    use tokio::process::Command;

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .current_dir(hooks_dir);

    if release_mode {
        cmd.arg("--release");
    }

    let output = cmd.output().await
        .context("Failed to run cargo build for hooks")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to compile hooks:\n{}", stderr);
    }

    // Find the compiled WASM file
    let target_dir = hooks_dir.join("target/wasm32-unknown-unknown");
    let wasm_path = if release_mode {
        target_dir.join("release/hooks.wasm")
    } else {
        target_dir.join("debug/hooks.wasm")
    };

    if !wasm_path.exists() {
        anyhow::bail!("Expected WASM output not found at {}", wasm_path.display());
    }

    // Optimize with wasm-strip if available
    let wasm_bytes = if which::which("wasm-strip").is_ok() {
        info!("Optimizing WASM with wasm-strip...");
        let output = Command::new("wasm-strip")
            .arg(&wasm_path)
            .arg("-o")
            .arg("-")
            .output()
            .await?;
        
        if output.status.success() {
            output.stdout
        } else {
            fs::read(&wasm_path)?
        }
    } else {
        fs::read(&wasm_path)?
    };

    Ok(Some(wasm_bytes))
}

/// Calculate SHA-256 hash of the bundle
fn calculate_bundle_hash(bundle: &Bundle) -> Result<String> {
    let mut hasher = Sha256::new();
    
    // Hash manifest
    let manifest_json = serde_json::to_string(&bundle.manifest)?;
    hasher.update(manifest_json.as_bytes());
    
    // Hash other components if they exist
    if let Some(entities) = &bundle.entities {
        hasher.update(serde_json::to_string(entities)?.as_bytes());
    }
    
    if let Some(zones) = &bundle.zones {
        hasher.update(serde_json::to_string(zones)?.as_bytes());
    }
    
    if let Some(actions) = &bundle.actions {
        hasher.update(serde_json::to_string(actions)?.as_bytes());
    }
    
    if let Some(phases) = &bundle.phases {
        hasher.update(serde_json::to_string(phases)?.as_bytes());
    }
    
    if let Some(hooks) = &bundle.hooks {
        hasher.update(hooks);
    }
    
    let result = hasher.finalize();
    Ok(hex::encode(&result[..8])) // Use first 8 bytes for shorter hash
}

/// Copy directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

/// Create .bf bundle archive
fn create_bundle_archive(bundle_dir: &Path, output_dir: &Path, hash: &str) -> Result<()> {
    // TODO: Implement tar.zstd compression
    warn!("Bundle archive creation not yet implemented");
    Ok(())
}

/// Build all games from a games directory
pub async fn run_all(
    games_dir: PathBuf,
    output_dir: PathBuf,
    release_mode: bool,
) -> Result<()> {
    info!("Building all games from {}", games_dir.display());

    if !games_dir.exists() {
        anyhow::bail!("Games directory {} does not exist", games_dir.display());
    }

    let mut built_count = 0;
    let mut failed_count = 0;

    // Scan for game directories
    for entry in fs::read_dir(&games_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let game_name = entry.file_name().to_string_lossy().to_string();
        info!("Processing game: {}", game_name.green());

        // Find latest version directory
        let mut versions: Vec<String> = fs::read_dir(entry.path())?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().ok().map(|f| f.is_dir()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        if versions.is_empty() {
            warn!("No version directories found for game: {}", game_name);
            continue;
        }

        versions.sort();
        let latest_version = versions.last().unwrap();
        let game_path = entry.path().join(latest_version);

        // Check if manifest exists
        if !game_path.join("manifest.yaml").exists() {
            warn!("No manifest.yaml found for game {} v{}, skipping", game_name, latest_version);
            continue;
        }

        // Build this game
        let game_output_dir = output_dir.join(&game_name).join(latest_version);
        match build_single_game(&game_path, &game_output_dir, release_mode).await {
            Ok(()) => {
                info!("✓ Built {} v{}", game_name.green(), latest_version);
                built_count += 1;
            }
            Err(e) => {
                warn!("✗ Failed to build {} v{}: {}", game_name.red(), latest_version, e);
                failed_count += 1;
            }
        }
    }

    info!(
        "{}",
        format!(
            "Build complete: {} succeeded, {} failed",
            built_count, failed_count
        ).green()
    );

    if failed_count > 0 {
        anyhow::bail!("{} games failed to build", failed_count);
    }

    Ok(())
}

/// Build a single game to the specified output directory
async fn build_single_game(
    game_path: &Path,
    output_dir: &Path,
    release_mode: bool,
) -> Result<()> {
    // Step 1: Validate the game files
    let validation_result = validate::validate_game(game_path).await?;
    if !validation_result.is_valid() {
        anyhow::bail!("Validation failed for game at {}", game_path.display());
    }

    // Step 2: Load all YAML files
    let manifest = load_yaml::<BundleManifest>(&game_path.join("manifest.yaml"))
        .context("Failed to load manifest.yaml")?;
    
    let entities = if game_path.join("entities.yaml").exists() {
        Some(load_yaml_raw(&game_path.join("entities.yaml"))?)
    } else {
        None
    };

    let zones = if game_path.join("zones.yaml").exists() {
        Some(load_yaml_raw(&game_path.join("zones.yaml"))?)
    } else {
        None
    };

    let actions = if game_path.join("actions.yaml").exists() {
        Some(load_yaml_raw(&game_path.join("actions.yaml"))?)
    } else {
        None
    };

    let phases = if game_path.join("phases.yaml").exists() {
        Some(load_yaml_raw(&game_path.join("phases.yaml"))?)
    } else {
        None
    };

    // Step 3: Convert YAML to JSON
    let mut bundle = Bundle {
        manifest: manifest.clone(),
        entities: entities.map(yaml_to_json),
        zones: zones.map(yaml_to_json),
        actions: actions.map(yaml_to_json),
        phases: phases.map(yaml_to_json),
        hooks: None,
    };

    // Step 4: Compile hooks if they exist
    if let Some(hooks_wasm) = compile_hooks(game_path, release_mode).await? {
        bundle.hooks = Some(hooks_wasm);
    }

    // Step 5: Create output directory
    fs::create_dir_all(output_dir)
        .context("Failed to create output directory")?;

    // Step 6: Write JSON files
    write_json_file(&output_dir.join("manifest.json"), &bundle.manifest)?;
    
    if let Some(entities) = &bundle.entities {
        write_json_file(&output_dir.join("entities.json"), entities)?;
    }
    
    if let Some(zones) = &bundle.zones {
        write_json_file(&output_dir.join("zones.json"), zones)?;
    }
    
    if let Some(actions) = &bundle.actions {
        write_json_file(&output_dir.join("actions.json"), actions)?;
    }
    
    if let Some(phases) = &bundle.phases {
        write_json_file(&output_dir.join("phases.json"), phases)?;
    }

    // Step 7: Write hooks.wasm if it exists
    if let Some(hooks) = &bundle.hooks {
        fs::write(output_dir.join("hooks.wasm"), hooks)
            .context("Failed to write hooks.wasm")?;
    }

    // Step 8: Copy assets if they exist
    let assets_dir = game_path.join("assets");
    if assets_dir.exists() {
        copy_dir_recursive(&assets_dir, &output_dir.join("assets"))?;
    }

    Ok(())
}
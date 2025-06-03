# Bluefelt SDK Reference

The Bluefelt SDK enables developers to write custom game logic using WebAssembly (WASM) modules. This allows for complex rules, AI opponents, and advanced game mechanics that go beyond the built-in verb system.

## Overview

The SDK provides:
- **Rust bindings** for writing WASM modules
- **Host function interface** for interacting with game state
- **Memory management** utilities for safe WASM operation
- **Development tools** for building and testing hooks

## Architecture

### WebAssembly Integration

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Game Logic    │    │   WASM Runtime  │    │   Host          │
│   (Rust/WASM)   │───▶│   (Wasmtime)    │───▶│   Functions     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Custom Hooks  │    │   Memory        │    │   Game State    │
│   check_win()   │    │   Management    │    │   Access        │
│   ai_move()     │    │   (Linear)      │    │   (JSON)        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Hook Types

- **Validation Hooks** - Check if actions are valid
- **State Hooks** - Modify game state after actions
- **AI Hooks** - Generate computer player moves
- **Win Condition Hooks** - Determine game end states
- **Scoring Hooks** - Calculate scores and rankings

## Getting Started

### Prerequisites

```bash
# Install Rust with WASM target
rustup target add wasm32-wasi

# Install wasm-pack for building
cargo install wasm-pack
```

### Project Setup

```bash
# Create new WASM project
cargo new --lib my-game-hooks
cd my-game-hooks

# Configure Cargo.toml
```

**Cargo.toml:**
```toml
[package]
name = "my-game-hooks"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
bluefelt-sdk = { path = "../sdk/rust" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
]
```

### Basic Hook

**src/lib.rs:**
```rust
use bluefelt_sdk::*;
use serde_json::Value;

#[no_mangle]
pub extern "C" fn check_win_condition(state_ptr: *const u8, state_len: usize) -> *const u8 {
    // Parse game state from memory
    let state = unsafe { 
        std::slice::from_raw_parts(state_ptr, state_len) 
    };
    let game_state: Value = serde_json::from_slice(state).unwrap();
    
    // Check win condition logic
    let winner = check_tic_tac_toe_win(&game_state);
    
    // Return result as JSON
    let result = serde_json::json!({
        "gameEnded": winner.is_some(),
        "winner": winner
    });
    
    // Convert to memory pointer for return
    to_memory_ptr(&result.to_string())
}

fn check_tic_tac_toe_win(state: &Value) -> Option<String> {
    let board = &state["zones"]["board"]["cells"];
    
    // Check rows, columns, diagonals
    for i in 0..3 {
        // Check row
        if let (Some(a), Some(b), Some(c)) = (
            get_cell_entity(board, i, 0),
            get_cell_entity(board, i, 1), 
            get_cell_entity(board, i, 2)
        ) {
            if a == b && b == c {
                return Some(get_player_from_entity(&a));
            }
        }
        
        // Check column  
        if let (Some(a), Some(b), Some(c)) = (
            get_cell_entity(board, 0, i),
            get_cell_entity(board, 1, i),
            get_cell_entity(board, 2, i)
        ) {
            if a == b && b == c {
                return Some(get_player_from_entity(&a));
            }
        }
    }
    
    // Check diagonals
    if let (Some(a), Some(b), Some(c)) = (
        get_cell_entity(board, 0, 0),
        get_cell_entity(board, 1, 1),
        get_cell_entity(board, 2, 2)
    ) {
        if a == b && b == c {
            return Some(get_player_from_entity(&a));
        }
    }
    
    if let (Some(a), Some(b), Some(c)) = (
        get_cell_entity(board, 0, 2),
        get_cell_entity(board, 1, 1), 
        get_cell_entity(board, 2, 0)
    ) {
        if a == b && b == c {
            return Some(get_player_from_entity(&a));
        }
    }
    
    None
}
```

### Building

```bash
# Build WASM module
wasm-pack build --target web --out-dir pkg

# Copy to game directory
cp pkg/my_game_hooks_bg.wasm ../games/my-game/1.0/hooks.wasm
```

## SDK API Reference

### Core Types

```rust
// Game state representation
pub type GameState = serde_json::Value;

// Entity reference
#[derive(Debug, Clone)]
pub struct EntityRef {
    pub entity_id: String,
    pub zone_id: String,
    pub location: Option<Location>,
}

// Zone location
#[derive(Debug, Clone)]
pub enum Location {
    GridCell { row: usize, col: usize },
    ListIndex { index: usize },
    Whole,
}

// Action result
#[derive(Debug)]
pub struct ActionResult {
    pub patches: Vec<JsonPatch>,
    pub valid: bool,
    pub message: Option<String>,
}

// Game status
#[derive(Debug, Clone)]
pub struct GameStatus {
    pub state: GameState,
    pub winner: Option<String>, 
    pub tie: bool,
    pub ended: bool,
}
```

### Host Functions

The SDK provides access to host functions for interacting with the game engine:

#### State Access

```rust
// Get full game state
pub fn get_game_state() -> GameState;

// Get specific zone contents
pub fn get_zone_contents(zone_id: &str) -> Vec<EntityRef>;

// Get entity at specific location
pub fn get_entity_at(zone_id: &str, location: Location) -> Option<EntityRef>;

// Check if zone is empty
pub fn is_zone_empty(zone_id: &str) -> bool;

// Get zone capacity/size
pub fn get_zone_size(zone_id: &str) -> usize;
```

#### State Modification

```rust
// Apply JSON patch to state
pub fn apply_patch(patch: &JsonPatch) -> Result<(), String>;

// Move entity between locations
pub fn move_entity(from: &EntityRef, to_zone: &str, to_location: Option<Location>) -> Result<(), String>;

// Create new entity instance
pub fn create_entity(entity_id: &str, zone_id: &str, location: Option<Location>) -> Result<EntityRef, String>;

// Remove entity from game
pub fn remove_entity(entity_ref: &EntityRef) -> Result<(), String>;

// Set entity properties
pub fn set_entity_props(entity_ref: &EntityRef, props: &serde_json::Value) -> Result<(), String>;
```

#### Game Flow

```rust
// Get current player
pub fn get_current_player() -> String;

// Get turn number
pub fn get_turn_number() -> u64;

// Get tick number (state version)
pub fn get_tick_number() -> u64;

// Set game status
pub fn set_game_status(status: &GameStatus) -> Result<(), String>;

// Advance turn to next player
pub fn advance_turn() -> Result<(), String>;

// Set current game phase
pub fn set_phase(phase_set: &str, phase: &str) -> Result<(), String>;
```

#### Utility Functions

```rust
// Generate random number
pub fn random_range(min: i32, max: i32) -> i32;

// Get entity definition by ID
pub fn get_entity_definition(entity_id: &str) -> Option<serde_json::Value>;

// Get zone definition by ID  
pub fn get_zone_definition(zone_id: &str) -> Option<serde_json::Value>;

// Log message (for debugging)
pub fn log_message(level: LogLevel, message: &str);

// Get player count
pub fn get_player_count() -> usize;

// Check if entity belongs to player
pub fn entity_belongs_to_player(entity_ref: &EntityRef, player_id: &str) -> bool;
```

## Hook Examples

### Win Condition Hook

```rust
#[no_mangle]
pub extern "C" fn check_win_condition(state_ptr: *const u8, state_len: usize) -> *const u8 {
    let state = parse_state(state_ptr, state_len);
    
    // Check for win conditions specific to your game
    let winner = check_game_win(&state);
    let tie = check_game_tie(&state);
    
    let result = serde_json::json!({
        "gameEnded": winner.is_some() || tie,
        "winner": winner,
        "tie": tie
    });
    
    to_memory_ptr(&result.to_string())
}

fn check_game_win(state: &GameState) -> Option<String> {
    // Implement your win condition logic
    // Return player ID if someone won, None otherwise
    None
}

fn check_game_tie(state: &GameState) -> bool {
    // Implement tie condition logic
    false
}
```

### AI Move Generation

```rust
#[no_mangle]
pub extern "C" fn generate_ai_move(state_ptr: *const u8, state_len: usize, player_ptr: *const u8, player_len: usize) -> *const u8 {
    let state = parse_state(state_ptr, state_len);
    let player_id = parse_string(player_ptr, player_len);
    
    // Generate AI move using minimax, neural network, etc.
    let ai_move = calculate_best_move(&state, &player_id);
    
    let result = serde_json::json!({
        "action": ai_move.action,
        "args": ai_move.args
    });
    
    to_memory_ptr(&result.to_string())
}

struct AIMove {
    action: String,
    args: serde_json::Value,
}

fn calculate_best_move(state: &GameState, player_id: &str) -> AIMove {
    // Implement AI logic (minimax, Monte Carlo, etc.)
    AIMove {
        action: "place".to_string(),
        args: serde_json::json!({
            "location": "/zones/board/cells/1/1",
            "entity": format!("piece_{}", player_id)
        })
    }
}
```

### Action Validation Hook

```rust
#[no_mangle]
pub extern "C" fn validate_action(
    state_ptr: *const u8, state_len: usize,
    action_ptr: *const u8, action_len: usize,
    player_ptr: *const u8, player_len: usize
) -> *const u8 {
    let state = parse_state(state_ptr, state_len);
    let action = parse_json(action_ptr, action_len);
    let player_id = parse_string(player_ptr, player_len);
    
    // Custom validation logic
    let valid = validate_custom_rules(&state, &action, &player_id);
    let message = if valid { 
        None 
    } else { 
        Some("Action violates custom rule".to_string()) 
    };
    
    let result = serde_json::json!({
        "valid": valid,
        "message": message
    });
    
    to_memory_ptr(&result.to_string())
}

fn validate_custom_rules(state: &GameState, action: &serde_json::Value, player_id: &str) -> bool {
    // Implement custom validation logic
    // Return true if action is valid, false otherwise
    true
}
```

### Scoring Hook

```rust
#[no_mangle]
pub extern "C" fn calculate_scores(state_ptr: *const u8, state_len: usize) -> *const u8 {
    let state = parse_state(state_ptr, state_len);
    
    // Calculate scores for all players
    let mut scores = std::collections::HashMap::new();
    let player_count = get_player_count();
    
    for i in 1..=player_count {
        let player_id = format!("p{}", i);
        let score = calculate_player_score(&state, &player_id);
        scores.insert(player_id, score);
    }
    
    let result = serde_json::json!({
        "scores": scores,
        "rankings": get_rankings(&scores)
    });
    
    to_memory_ptr(&result.to_string())
}

fn calculate_player_score(state: &GameState, player_id: &str) -> i32 {
    // Implement scoring logic specific to your game
    0
}

fn get_rankings(scores: &std::collections::HashMap<String, i32>) -> Vec<String> {
    let mut players: Vec<_> = scores.iter().collect();
    players.sort_by(|a, b| b.1.cmp(a.1)); // Sort by score descending
    players.into_iter().map(|(player, _)| player.clone()).collect()
}
```

## Memory Management

### Safe Memory Handling

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Convert Rust string to WASM memory pointer
pub fn to_memory_ptr(s: &str) -> *const u8 {
    let c_string = CString::new(s).unwrap();
    let ptr = c_string.as_ptr() as *const u8;
    std::mem::forget(c_string); // Prevent deallocation
    ptr
}

// Parse string from WASM memory
pub fn parse_string(ptr: *const u8, len: usize) -> String {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(slice).to_string()
}

// Parse JSON from WASM memory
pub fn parse_json(ptr: *const u8, len: usize) -> serde_json::Value {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    serde_json::from_slice(slice).unwrap()
}

// Parse game state from memory
pub fn parse_state(ptr: *const u8, len: usize) -> GameState {
    parse_json(ptr, len)
}
```

### Memory Allocation

```rust
// Allocate memory in WASM for host to write to
#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    let mut vec = Vec::with_capacity(size);
    let ptr = vec.as_mut_ptr();
    std::mem::forget(vec);
    ptr
}

// Deallocate memory
#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, capacity: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, capacity);
    }
}
```

## Testing Hooks

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_win_condition() {
        let state = json!({
            "zones": {
                "board": {
                    "type": "grid",
                    "cells": [
                        [{"entity": "x_token"}, {"entity": "x_token"}, {"entity": "x_token"}],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            }
        });
        
        let winner = check_tic_tac_toe_win(&state);
        assert_eq!(winner, Some("p1".to_string()));
    }
    
    #[test]
    fn test_no_winner() {
        let state = json!({
            "zones": {
                "board": {
                    "type": "grid", 
                    "cells": [
                        [{"entity": "x_token"}, {"entity": "o_token"}, null],
                        [null, null, null],
                        [null, null, null]
                    ]
                }
            }
        });
        
        let winner = check_tic_tac_toe_win(&state);
        assert_eq!(winner, None);
    }
}
```

### Integration Testing

```rust
// Test with actual game state
#[test]
fn test_full_game_flow() {
    let initial_state = load_test_state("test_data/initial_state.json");
    
    // Simulate game moves
    let moves = vec![
        ("place", json!({"location": "/zones/board/cells/0/0", "entity": "x_token"})),
        ("place", json!({"location": "/zones/board/cells/0/1", "entity": "o_token"})),
        // ... more moves
    ];
    
    let mut state = initial_state;
    for (action, args) in moves {
        // Apply move to state
        state = apply_test_move(&state, action, &args);
        
        // Check win condition after each move
        let winner = check_tic_tac_toe_win(&state);
        if winner.is_some() {
            break;
        }
    }
    
    // Verify final state
    assert!(check_tic_tac_toe_win(&state).is_some());
}
```

## Performance Optimization

### Efficient State Access

```rust
// Cache frequently accessed data
struct GameCache {
    board_state: Vec<Vec<Option<String>>>,
    player_scores: std::collections::HashMap<String, i32>,
    last_tick: u64,
}

static mut CACHE: Option<GameCache> = None;

fn get_cached_board(state: &GameState) -> &Vec<Vec<Option<String>>> {
    unsafe {
        let current_tick = get_tick_number();
        
        if CACHE.is_none() || CACHE.as_ref().unwrap().last_tick != current_tick {
            CACHE = Some(GameCache {
                board_state: extract_board_state(state),
                player_scores: calculate_all_scores(state),
                last_tick: current_tick,
            });
        }
        
        &CACHE.as_ref().unwrap().board_state
    }
}
```

### Minimize Allocations

```rust
// Use iterators instead of collecting vectors
fn count_player_pieces(state: &GameState, player_id: &str) -> usize {
    state["zones"]["board"]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| row.as_array().unwrap())
        .filter_map(|cell| cell.get("entity"))
        .filter(|entity| entity.as_str().map_or(false, |e| e.contains(player_id)))
        .count()
}

// Reuse buffers when possible
thread_local! {
    static TEMP_BUFFER: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
}

fn get_available_moves(state: &GameState) -> Vec<String> {
    TEMP_BUFFER.with(|buffer| {
        let mut buf = buffer.borrow_mut();
        buf.clear();
        
        // Fill buffer with moves
        for row in 0..3 {
            for col in 0..3 {
                if is_cell_empty(state, row, col) {
                    buf.push(format!("/zones/board/cells/{}/{}", row, col));
                }
            }
        }
        
        buf.clone()
    })
}
```

## Debugging

### Logging

```rust
pub enum LogLevel {
    Debug,
    Info, 
    Warn,
    Error,
}

// Log from WASM to host console
pub fn log_message(level: LogLevel, message: &str) {
    // Implementation provided by host
    unsafe {
        log_to_host(level as i32, message.as_ptr(), message.len());
    }
}

// Debug macro for development
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        log_message(LogLevel::Debug, &format!($($arg)*));
    };
}

// Usage
debug_log!("Checking win condition for player {}", player_id);
debug_log!("Board state: {:?}", board_state);
```

### Error Handling

```rust
// Structured error types
#[derive(Debug)]
pub enum HookError {
    InvalidState(String),
    InvalidAction(String), 
    InvalidPlayer(String),
    InternalError(String),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HookError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            HookError::InvalidAction(msg) => write!(f, "Invalid action: {}", msg),
            HookError::InvalidPlayer(msg) => write!(f, "Invalid player: {}", msg),
            HookError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

// Safe hook wrapper
fn safe_hook_wrapper<F, R>(f: F) -> *const u8 
where
    F: FnOnce() -> Result<R, HookError>,
    R: serde::Serialize,
{
    match f() {
        Ok(result) => {
            let json = serde_json::json!({
                "success": true,
                "data": result
            });
            to_memory_ptr(&json.to_string())
        }
        Err(error) => {
            let json = serde_json::json!({
                "success": false,
                "error": error.to_string()
            });
            to_memory_ptr(&json.to_string())
        }
    }
}
```

## Deployment

### Building for Production

```bash
# Optimize for size and speed
wasm-pack build --target web --release --opt-level=3 --out-dir pkg

# Strip debug information
wasm-strip pkg/my_game_hooks_bg.wasm

# Copy to game bundle
cp pkg/my_game_hooks_bg.wasm ../bundles/my-game/1.0/hooks.wasm
```

### Version Management

```toml
# Cargo.toml - Semantic versioning
[package]
version = "1.2.3"

# Game manifest compatibility
[package.metadata.bluefelt]
min_spec_version = "1.0"
max_spec_version = "2.0"
```

The Bluefelt SDK provides a powerful foundation for extending games with custom logic while maintaining safety and performance through WebAssembly sandboxing.
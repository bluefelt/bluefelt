# Developing Games: Custom Hooks

Custom hooks allow you to implement complex game logic using WebAssembly (WASM), extending beyond Bluefelt's built-in verbs. This comprehensive reference covers everything from simple validation hooks to complex AI systems.

## Overview

Custom hooks in Bluefelt enable:
- **Complex Rule Validation** - Multi-step rule checking beyond basic conditions
- **Advanced Win Conditions** - Custom victory/defeat logic
- **AI Opponents** - Computer player behavior and decision making
- **Dynamic Content** - Procedural generation and adaptive content
- **Custom Scoring** - Complex point calculation systems
- **Special Effects** - Custom game mechanics and behaviors

Hooks are written in Rust, compiled to WebAssembly, and executed in a secure sandbox with controlled access to game state.

## Hook Architecture

### Hook Types

Bluefelt supports several types of custom hooks:

```rust
// Validation hooks - Check if actions are legal
#[no_mangle]
pub extern "C" fn validate_action(
    state_ptr: *const u8, state_len: usize,
    action_ptr: *const u8, action_len: usize,
    player_ptr: *const u8, player_len: usize
) -> *const u8;

// State modification hooks - Change game state
#[no_mangle]
pub extern "C" fn process_action(
    state_ptr: *const u8, state_len: usize,
    action_ptr: *const u8, action_len: usize
) -> *const u8;

// Win condition hooks - Determine game end
#[no_mangle]
pub extern "C" fn check_win_condition(
    state_ptr: *const u8, state_len: usize
) -> *const u8;

// AI hooks - Generate computer moves
#[no_mangle]
pub extern "C" fn ai_move(
    state_ptr: *const u8, state_len: usize,
    player_ptr: *const u8, player_len: usize,
    difficulty_ptr: *const u8, difficulty_len: usize
) -> *const u8;

// Scoring hooks - Calculate points
#[no_mangle]
pub extern "C" fn calculate_score(
    state_ptr: *const u8, state_len: usize,
    player_ptr: *const u8, player_len: usize
) -> *const u8;

// Setup hooks - Initialize game state
#[no_mangle]
pub extern "C" fn custom_setup(
    config_ptr: *const u8, config_len: usize
) -> *const u8;
```

### Hook Integration

Hooks are referenced in your game's action definitions:

```yaml
# In actions.yaml
- id: "complex_move"
  uses: "custom"
  hook: "validate_complex_move"    # References WASM function
  ui:
    name: "Special Move"
    description: "Perform a complex movement with custom rules"
  conditions:
    - type: "current_player_turn"
  effects:
    - hook: "process_complex_move"
      args: 
        moveType: "{move_type}"
        complexity: "high"

# Automatic hooks for game events
- id: "turn_start_hook"
  uses: "custom"
  hook: "process_turn_start"
  auto: true
  trigger: "turn_start"

# Win condition checking
- id: "victory_check"
  uses: "custom"
  hook: "check_win_condition"
  auto: true
  trigger: "after_action"
```

## Development Environment Setup

### Project Structure

Create a Rust WebAssembly project:

```bash
# Create new Rust library project
cargo new --lib my-game-hooks
cd my-game-hooks

# Project structure
my-game-hooks/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Main hook implementations
│   ├── game_state.rs    # State parsing utilities
│   ├── ai.rs           # AI logic
│   ├── validation.rs   # Rule validation
│   └── scoring.rs      # Scoring systems
├── tests/
│   ├── integration_tests.rs
│   └── test_data/
│       └── sample_states.json
└── examples/
    └── basic_usage.rs
```

### Cargo.toml Configuration

```toml
[package]
name = "my-game-hooks"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# JSON parsing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# WebAssembly utilities
wasm-bindgen = "0.2"

# Optional: Advanced features
rand = { version = "0.8", features = ["small_rng"] }
pathfinding = "4.0"
regex = "1.0"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
]

# Development dependencies
[dev-dependencies]
wasm-bindgen-test = "0.3"

# Build configuration
[profile.release]
opt-level = "s"          # Optimize for size
lto = true              # Link-time optimization
panic = "abort"         # Reduce binary size
```

### Basic Hook Template

```rust
// src/lib.rs
use serde_json::{Value, json};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Memory management utilities
mod memory;
use memory::{to_memory_ptr, parse_json_from_memory, parse_string_from_memory};

// Game-specific modules
mod game_state;
mod validation;
mod ai;
mod scoring;

// Export hook functions
pub use validation::validate_action;
pub use ai::ai_move;
pub use scoring::calculate_score;

// Example: Basic win condition check
#[no_mangle]
pub extern "C" fn check_win_condition(
    state_ptr: *const u8, 
    state_len: usize
) -> *const u8 {
    // Parse game state
    let state = match parse_json_from_memory(state_ptr, state_len) {
        Ok(s) => s,
        Err(e) => return to_memory_ptr(&json!({
            "error": format!("Failed to parse state: {}", e)
        }).to_string()),
    };
    
    // Check for win conditions
    let result = check_victory_conditions(&state);
    
    // Return result
    to_memory_ptr(&result.to_string())
}

fn check_victory_conditions(state: &Value) -> Value {
    // Implementation will vary by game
    json!({
        "gameEnded": false,
        "winner": null,
        "tie": false,
        "reason": null
    })
}
```

## Memory Management

### Safe Memory Utilities

```rust
// src/memory.rs
use serde_json::Value;
use std::ffi::{CStr, CString};

// Convert Rust string to WebAssembly memory pointer
pub fn to_memory_ptr(s: &str) -> *const u8 {
    let c_string = CString::new(s).expect("Failed to create CString");
    let ptr = c_string.as_ptr() as *const u8;
    std::mem::forget(c_string);  // Prevent automatic deallocation
    ptr
}

// Parse JSON from WebAssembly memory
pub fn parse_json_from_memory(
    ptr: *const u8, 
    len: usize
) -> Result<Value, Box<dyn std::error::Error>> {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let json_str = std::str::from_utf8(slice)?;
    let json_value = serde_json::from_str(json_str)?;
    Ok(json_value)
}

// Parse string from WebAssembly memory
pub fn parse_string_from_memory(
    ptr: *const u8, 
    len: usize
) -> Result<String, Box<dyn std::error::Error>> {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let string = std::str::from_utf8(slice)?.to_string();
    Ok(string)
}

// Allocate memory for host to write to
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

### Error Handling

```rust
// src/error.rs
use serde_json::{Value, json};

#[derive(Debug)]
pub enum HookError {
    ParseError(String),
    ValidationError(String),
    StateError(String),
    LogicError(String),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HookError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            HookError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            HookError::StateError(msg) => write!(f, "State error: {}", msg),
            HookError::LogicError(msg) => write!(f, "Logic error: {}", msg),
        }
    }
}

impl std::error::Error for HookError {}

// Convert errors to JSON responses
pub fn error_response(error: &HookError) -> Value {
    json!({
        "success": false,
        "error": error.to_string(),
        "errorType": match error {
            HookError::ParseError(_) => "parse",
            HookError::ValidationError(_) => "validation",
            HookError::StateError(_) => "state",
            HookError::LogicError(_) => "logic",
        }
    })
}

// Success response wrapper
pub fn success_response(data: Value) -> Value {
    json!({
        "success": true,
        "data": data
    })
}
```

## Game State Access

### State Parsing Utilities

```rust
// src/game_state.rs
use serde_json::Value;
use crate::error::{HookError, Result};

pub struct GameState<'a> {
    pub raw: &'a Value,
}

impl<'a> GameState<'a> {
    pub fn new(state: &'a Value) -> Self {
        Self { raw: state }
    }
    
    // Zone access
    pub fn get_zone(&self, zone_id: &str) -> Result<&Value> {
        self.raw
            .get("zones")
            .and_then(|zones| zones.get(zone_id))
            .ok_or_else(|| HookError::StateError(format!("Zone '{}' not found", zone_id)))
    }
    
    pub fn get_zone_contents(&self, zone_id: &str) -> Result<Vec<&Value>> {
        let zone = self.get_zone(zone_id)?;
        match zone.get("type").and_then(|t| t.as_str()) {
            Some("grid") => {
                let cells = zone.get("cells")
                    .ok_or_else(|| HookError::StateError("Grid zone missing cells".to_string()))?
                    .as_array()
                    .ok_or_else(|| HookError::StateError("Grid cells not an array".to_string()))?;
                
                let mut entities = Vec::new();
                for row in cells {
                    if let Some(row_array) = row.as_array() {
                        for cell in row_array {
                            if !cell.is_null() {
                                entities.push(cell);
                            }
                        }
                    }
                }
                Ok(entities)
            },
            Some("list") | Some("deck") => {
                let items = zone.get("items")
                    .ok_or_else(|| HookError::StateError("List zone missing items".to_string()))?
                    .as_array()
                    .ok_or_else(|| HookError::StateError("Zone items not an array".to_string()))?;
                Ok(items.iter().collect())
            },
            _ => Err(HookError::StateError(format!("Unknown zone type for '{}'", zone_id)))
        }
    }
    
    // Grid-specific access
    pub fn get_grid_cell(&self, zone_id: &str, row: usize, col: usize) -> Result<&Value> {
        let zone = self.get_zone(zone_id)?;
        let cells = zone.get("cells")
            .and_then(|c| c.as_array())
            .ok_or_else(|| HookError::StateError("Zone is not a grid".to_string()))?;
        
        cells.get(row)
            .and_then(|r| r.as_array())
            .and_then(|row_array| row_array.get(col))
            .ok_or_else(|| HookError::StateError(format!("Cell [{}, {}] not found", row, col)))
    }
    
    pub fn is_cell_empty(&self, zone_id: &str, row: usize, col: usize) -> bool {
        self.get_grid_cell(zone_id, row, col)
            .map(|cell| cell.is_null())
            .unwrap_or(false)
    }
    
    // Entity access
    pub fn get_entity_at_path(&self, path: &str) -> Result<&Value> {
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        let mut current = self.raw;
        
        for part in parts {
            current = if let Ok(index) = part.parse::<usize>() {
                current.as_array()
                    .and_then(|arr| arr.get(index))
                    .ok_or_else(|| HookError::StateError(format!("Array index {} not found", index)))?
            } else {
                current.get(part)
                    .ok_or_else(|| HookError::StateError(format!("Key '{}' not found", part)))?
            };
        }
        
        Ok(current)
    }
    
    // Player access
    pub fn get_current_player(&self) -> Result<&str> {
        self.raw
            .get("meta")
            .and_then(|meta| meta.get("currentPlayer"))
            .and_then(|player| player.as_str())
            .ok_or_else(|| HookError::StateError("Current player not found".to_string()))
    }
    
    pub fn get_player_count(&self) -> usize {
        self.raw
            .get("meta")
            .and_then(|meta| meta.get("players"))
            .and_then(|players| players.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0)
    }
    
    // Turn and tick info
    pub fn get_turn_number(&self) -> u64 {
        self.raw
            .get("meta")
            .and_then(|meta| meta.get("turn"))
            .and_then(|turn| turn.as_u64())
            .unwrap_or(0)
    }
    
    pub fn get_tick(&self) -> u64 {
        self.raw
            .get("meta")
            .and_then(|meta| meta.get("tick"))
            .and_then(|tick| tick.as_u64())
            .unwrap_or(0)
    }
    
    // Game status
    pub fn is_game_ended(&self) -> bool {
        self.raw
            .get("meta")
            .and_then(|meta| meta.get("gameStatus"))
            .and_then(|status| status.get("state"))
            .and_then(|state| state.as_str())
            .map(|s| s == "ended")
            .unwrap_or(false)
    }
}
```

### Entity Utilities

```rust
// src/entities.rs
use serde_json::Value;
use crate::error::{HookError, Result};

pub struct Entity<'a> {
    pub data: &'a Value,
}

impl<'a> Entity<'a> {
    pub fn new(data: &'a Value) -> Self {
        Self { data }
    }
    
    pub fn get_id(&self) -> Result<&str> {
        self.data
            .get("entity")
            .and_then(|id| id.as_str())
            .ok_or_else(|| HookError::StateError("Entity missing ID".to_string()))
    }
    
    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.data
            .get("props")
            .and_then(|props| props.get(key))
    }
    
    pub fn get_property_str(&self, key: &str) -> Option<&str> {
        self.get_property(key)
            .and_then(|prop| prop.as_str())
    }
    
    pub fn get_property_i64(&self, key: &str) -> Option<i64> {
        self.get_property(key)
            .and_then(|prop| prop.as_i64())
    }
    
    pub fn belongs_to_player(&self, player_id: &str) -> bool {
        self.get_property_str("player")
            .map(|p| p == player_id)
            .or_else(|| {
                // Check if entity ID contains player ID
                self.get_id().ok()
                    .map(|id| id.contains(player_id))
            })
            .unwrap_or(false)
    }
    
    pub fn is_type(&self, entity_type: &str) -> bool {
        self.get_id()
            .map(|id| id.starts_with(entity_type))
            .unwrap_or(false)
    }
}
```

## Validation Hooks

### Action Validation

```rust
// src/validation.rs
use serde_json::{Value, json};
use crate::game_state::GameState;
use crate::entities::Entity;
use crate::error::{HookError, success_response, error_response};
use crate::memory::{parse_json_from_memory, parse_string_from_memory, to_memory_ptr};

#[no_mangle]
pub extern "C" fn validate_action(
    state_ptr: *const u8, state_len: usize,
    action_ptr: *const u8, action_len: usize,
    player_ptr: *const u8, player_len: usize
) -> *const u8 {
    let result = validate_action_impl(state_ptr, state_len, action_ptr, action_len, player_ptr, player_len);
    to_memory_ptr(&result.to_string())
}

fn validate_action_impl(
    state_ptr: *const u8, state_len: usize,
    action_ptr: *const u8, action_len: usize,
    player_ptr: *const u8, player_len: usize
) -> Value {
    // Parse inputs
    let state = match parse_json_from_memory(state_ptr, state_len) {
        Ok(s) => s,
        Err(e) => return error_response(&HookError::ParseError(e.to_string())),
    };
    
    let action = match parse_json_from_memory(action_ptr, action_len) {
        Ok(a) => a,
        Err(e) => return error_response(&HookError::ParseError(e.to_string())),
    };
    
    let player_id = match parse_string_from_memory(player_ptr, player_len) {
        Ok(p) => p,
        Err(e) => return error_response(&HookError::ParseError(e.to_string())),
    };
    
    // Perform validation
    match validate_complex_move(&state, &action, &player_id) {
        Ok(result) => success_response(result),
        Err(e) => error_response(&e),
    }
}

// Example: Chess-like movement validation
fn validate_complex_move(
    state: &Value, 
    action: &Value, 
    player_id: &str
) -> Result<Value, HookError> {
    let game_state = GameState::new(state);
    
    // Extract action parameters
    let from_path = action.get("args")
        .and_then(|args| args.get("from"))
        .and_then(|from| from.as_str())
        .ok_or_else(|| HookError::ValidationError("Missing 'from' parameter".to_string()))?;
    
    let to_path = action.get("args")
        .and_then(|args| args.get("to"))
        .and_then(|to| to.as_str())
        .ok_or_else(|| HookError::ValidationError("Missing 'to' parameter".to_string()))?;
    
    // Parse coordinates from paths
    let (from_row, from_col) = parse_grid_coordinates(from_path)?;
    let (to_row, to_col) = parse_grid_coordinates(to_path)?;
    
    // Check if source cell contains player's piece
    let source_entity = game_state.get_grid_cell("board", from_row, from_col)?;
    if source_entity.is_null() {
        return Err(HookError::ValidationError("No piece at source location".to_string()));
    }
    
    let entity = Entity::new(source_entity);
    if !entity.belongs_to_player(player_id) {
        return Err(HookError::ValidationError("Piece does not belong to current player".to_string()));
    }
    
    // Check if destination is valid
    let dest_entity = game_state.get_grid_cell("board", to_row, to_col)?;
    if !dest_entity.is_null() {
        let dest_entity_obj = Entity::new(dest_entity);
        if dest_entity_obj.belongs_to_player(player_id) {
            return Err(HookError::ValidationError("Cannot move to cell occupied by own piece".to_string()));
        }
    }
    
    // Check movement pattern based on piece type
    let piece_type = entity.get_property_str("piece_type")
        .unwrap_or("unknown");
    
    if !is_valid_movement(piece_type, from_row, from_col, to_row, to_col, &game_state)? {
        return Err(HookError::ValidationError(format!("{} cannot move from [{},{}] to [{},{}]", 
            piece_type, from_row, from_col, to_row, to_col)));
    }
    
    Ok(json!({
        "valid": true,
        "message": "Move is valid"
    }))
}

fn parse_grid_coordinates(path: &str) -> Result<(usize, usize), HookError> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 5 {
        return Err(HookError::ValidationError("Invalid grid path".to_string()));
    }
    
    let row = parts[3].parse::<usize>()
        .map_err(|_| HookError::ValidationError("Invalid row coordinate".to_string()))?;
    let col = parts[4].parse::<usize>()
        .map_err(|_| HookError::ValidationError("Invalid column coordinate".to_string()))?;
    
    Ok((row, col))
}

fn is_valid_movement(
    piece_type: &str,
    from_row: usize, from_col: usize,
    to_row: usize, to_col: usize,
    game_state: &GameState
) -> Result<bool, HookError> {
    let row_diff = (to_row as i32 - from_row as i32).abs();
    let col_diff = (to_col as i32 - from_col as i32).abs();
    
    match piece_type {
        "pawn" => {
            // Pawn movement logic (simplified)
            if col_diff == 0 && row_diff == 1 {
                // Forward movement
                Ok(game_state.is_cell_empty("board", to_row, to_col))
            } else if col_diff == 1 && row_diff == 1 {
                // Diagonal capture
                Ok(!game_state.is_cell_empty("board", to_row, to_col))
            } else {
                Ok(false)
            }
        },
        "rook" => {
            // Rook moves in straight lines
            if row_diff == 0 || col_diff == 0 {
                Ok(is_path_clear(from_row, from_col, to_row, to_col, game_state)?)
            } else {
                Ok(false)
            }
        },
        "bishop" => {
            // Bishop moves diagonally
            if row_diff == col_diff {
                Ok(is_path_clear(from_row, from_col, to_row, to_col, game_state)?)
            } else {
                Ok(false)
            }
        },
        "queen" => {
            // Queen combines rook and bishop
            if row_diff == 0 || col_diff == 0 || row_diff == col_diff {
                Ok(is_path_clear(from_row, from_col, to_row, to_col, game_state)?)
            } else {
                Ok(false)
            }
        },
        "king" => {
            // King moves one square in any direction
            Ok(row_diff <= 1 && col_diff <= 1)
        },
        "knight" => {
            // Knight moves in L-shape
            Ok((row_diff == 2 && col_diff == 1) || (row_diff == 1 && col_diff == 2))
        },
        _ => Ok(false)
    }
}

fn is_path_clear(
    from_row: usize, from_col: usize,
    to_row: usize, to_col: usize,
    game_state: &GameState
) -> Result<bool, HookError> {
    let row_step = if to_row > from_row { 1 } else if to_row < from_row { -1 } else { 0 };
    let col_step = if to_col > from_col { 1 } else if to_col < from_col { -1 } else { 0 };
    
    let mut current_row = from_row as i32 + row_step;
    let mut current_col = from_col as i32 + col_step;
    
    while current_row != to_row as i32 || current_col != to_col as i32 {
        if !game_state.is_cell_empty("board", current_row as usize, current_col as usize) {
            return Ok(false);
        }
        current_row += row_step;
        current_col += col_step;
    }
    
    Ok(true)
}
```

## AI Implementation

### Basic AI Framework

```rust
// src/ai.rs
use serde_json::{Value, json};
use crate::game_state::GameState;
use crate::entities::Entity;
use crate::validation::is_valid_movement;
use crate::error::{HookError, success_response, error_response};
use crate::memory::{parse_json_from_memory, parse_string_from_memory, to_memory_ptr};

#[no_mangle]
pub extern "C" fn ai_move(
    state_ptr: *const u8, state_len: usize,
    player_ptr: *const u8, player_len: usize,
    difficulty_ptr: *const u8, difficulty_len: usize
) -> *const u8 {
    let result = ai_move_impl(state_ptr, state_len, player_ptr, player_len, difficulty_ptr, difficulty_len);
    to_memory_ptr(&result.to_string())
}

fn ai_move_impl(
    state_ptr: *const u8, state_len: usize,
    player_ptr: *const u8, player_len: usize,
    difficulty_ptr: *const u8, difficulty_len: usize
) -> Value {
    // Parse inputs
    let state = match parse_json_from_memory(state_ptr, state_len) {
        Ok(s) => s,
        Err(e) => return error_response(&HookError::ParseError(e.to_string())),
    };
    
    let player_id = match parse_string_from_memory(player_ptr, player_len) {
        Ok(p) => p,
        Err(e) => return error_response(&HookError::ParseError(e.to_string())),
    };
    
    let difficulty = match parse_string_from_memory(difficulty_ptr, difficulty_len) {
        Ok(d) => d,
        Err(e) => return error_response(&HookError::ParseError(e.to_string())),
    };
    
    // Generate AI move
    match generate_ai_move(&state, &player_id, &difficulty) {
        Ok(action) => success_response(action),
        Err(e) => error_response(&e),
    }
}

#[derive(Debug, Clone)]
pub struct Move {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub score: i32,
    pub piece_type: String,
}

fn generate_ai_move(state: &Value, player_id: &str, difficulty: &str) -> Result<Value, HookError> {
    let game_state = GameState::new(state);
    
    // Get all possible moves for AI player
    let possible_moves = get_all_possible_moves(&game_state, player_id)?;
    
    if possible_moves.is_empty() {
        return Err(HookError::LogicError("No possible moves available".to_string()));
    }
    
    // Select move based on difficulty
    let selected_move = match difficulty {
        "easy" => select_random_move(&possible_moves),
        "medium" => select_good_move(&possible_moves, &game_state),
        "hard" => select_best_move(&possible_moves, &game_state, player_id, 3)?,
        _ => select_random_move(&possible_moves),
    };
    
    // Convert move to action format
    Ok(json!({
        "verb": "moveEntity",
        "args": {
            "from": format!("/zones/board/cells/{}/{}", selected_move.from.0, selected_move.from.1),
            "to": format!("/zones/board/cells/{}/{}", selected_move.to.0, selected_move.to.1)
        }
    }))
}

fn get_all_possible_moves(game_state: &GameState, player_id: &str) -> Result<Vec<Move>, HookError> {
    let mut moves = Vec::new();
    
    // Scan all board positions for player's pieces
    for row in 0..8 {
        for col in 0..8 {
            let cell = game_state.get_grid_cell("board", row, col)?;
            if cell.is_null() {
                continue;
            }
            
            let entity = Entity::new(cell);
            if !entity.belongs_to_player(player_id) {
                continue;
            }
            
            let piece_type = entity.get_property_str("piece_type").unwrap_or("unknown");
            
            // Check all possible destinations for this piece
            for dest_row in 0..8 {
                for dest_col in 0..8 {
                    if row == dest_row && col == dest_col {
                        continue; // Can't move to same position
                    }
                    
                    // Check if move is valid for this piece type
                    if is_valid_movement(piece_type, row, col, dest_row, dest_col, game_state)? {
                        let score = evaluate_move(game_state, row, col, dest_row, dest_col, player_id)?;
                        moves.push(Move {
                            from: (row, col),
                            to: (dest_row, dest_col),
                            score,
                            piece_type: piece_type.to_string(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(moves)
}

fn select_random_move(moves: &[Move]) -> &Move {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..moves.len());
    &moves[index]
}

fn select_good_move(moves: &[Move], game_state: &GameState) -> &Move {
    // Select move with highest immediate score
    moves.iter()
        .max_by_key(|m| m.score)
        .unwrap_or(&moves[0])
}

fn select_best_move(
    moves: &[Move], 
    game_state: &GameState, 
    player_id: &str, 
    depth: i32
) -> Result<&Move, HookError> {
    // Minimax algorithm with alpha-beta pruning
    let mut best_move = &moves[0];
    let mut best_score = i32::MIN;
    
    for mov in moves {
        let score = minimax(game_state, mov, depth, i32::MIN, i32::MAX, true, player_id)?;
        if score > best_score {
            best_score = score;
            best_move = mov;
        }
    }
    
    Ok(best_move)
}

fn minimax(
    game_state: &GameState,
    mov: &Move,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    ai_player_id: &str
) -> Result<i32, HookError> {
    if depth == 0 {
        return Ok(mov.score);
    }
    
    // This is a simplified minimax - in a real implementation,
    // you would apply the move, generate opponent responses, etc.
    Ok(mov.score)
}

fn evaluate_move(
    game_state: &GameState,
    from_row: usize, from_col: usize,
    to_row: usize, to_col: usize,
    player_id: &str
) -> Result<i32, HookError> {
    let mut score = 0;
    
    // Check if move captures an opponent piece
    let dest_cell = game_state.get_grid_cell("board", to_row, to_col)?;
    if !dest_cell.is_null() {
        let dest_entity = Entity::new(dest_cell);
        if !dest_entity.belongs_to_player(player_id) {
            // Capturing opponent piece is good
            let piece_value = get_piece_value(dest_entity.get_property_str("piece_type").unwrap_or("pawn"));
            score += piece_value;
        }
    }
    
    // Evaluate position control
    score += evaluate_position_control(to_row, to_col);
    
    // Check for threats to own pieces
    score -= evaluate_threats_created(game_state, from_row, from_col, to_row, to_col, player_id)?;
    
    Ok(score)
}

fn get_piece_value(piece_type: &str) -> i32 {
    match piece_type {
        "pawn" => 10,
        "knight" => 30,
        "bishop" => 30,
        "rook" => 50,
        "queen" => 90,
        "king" => 900,
        _ => 0,
    }
}

fn evaluate_position_control(row: usize, col: usize) -> i32 {
    // Center squares are more valuable
    let center_distance = ((row as i32 - 3).abs() + (col as i32 - 3).abs()) as i32;
    10 - center_distance
}

fn evaluate_threats_created(
    game_state: &GameState,
    from_row: usize, from_col: usize,
    to_row: usize, to_col: usize,
    player_id: &str
) -> Result<i32, HookError> {
    // Simplified threat evaluation
    // In a real implementation, you would check if moving this piece
    // exposes other pieces to capture
    Ok(0)
}
```

## Win Condition Hooks

### Complex Victory Detection

```rust
// src/win_conditions.rs
use serde_json::{Value, json};
use crate::game_state::GameState;
use crate::entities::Entity;
use crate::error::{HookError, success_response, error_response};
use crate::memory::{parse_json_from_memory, to_memory_ptr};

#[no_mangle]
pub extern "C" fn check_win_condition(
    state_ptr: *const u8,
    state_len: usize
) -> *const u8 {
    let result = check_win_condition_impl(state_ptr, state_len);
    to_memory_ptr(&result.to_string())
}

fn check_win_condition_impl(state_ptr: *const u8, state_len: usize) -> Value {
    let state = match parse_json_from_memory(state_ptr, state_len) {
        Ok(s) => s,
        Err(e) => return error_response(&HookError::ParseError(e.to_string())),
    };
    
    match check_multiple_win_conditions(&state) {
        Ok(result) => success_response(result),
        Err(e) => error_response(&e),
    }
}

fn check_multiple_win_conditions(state: &Value) -> Result<Value, HookError> {
    let game_state = GameState::new(state);
    
    // Check various win conditions
    
    // 1. Checkmate (chess-style)
    if let Ok(checkmate_result) = check_checkmate(&game_state) {
        if checkmate_result.get("gameEnded").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Ok(checkmate_result);
        }
    }
    
    // 2. Capture all pieces
    if let Ok(capture_result) = check_piece_elimination(&game_state) {
        if capture_result.get("gameEnded").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Ok(capture_result);
        }
    }
    
    // 3. Control territory
    if let Ok(territory_result) = check_territory_control(&game_state) {
        if territory_result.get("gameEnded").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Ok(territory_result);
        }
    }
    
    // 4. Time/turn limit
    if let Ok(limit_result) = check_game_limits(&game_state) {
        if limit_result.get("gameEnded").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Ok(limit_result);
        }
    }
    
    // No win condition met
    Ok(json!({
        "gameEnded": false,
        "winner": null,
        "tie": false,
        "reason": null
    }))
}

fn check_checkmate(game_state: &GameState) -> Result<Value, HookError> {
    // Find kings for each player
    let mut kings = std::collections::HashMap::new();
    
    for row in 0..8 {
        for col in 0..8 {
            let cell = game_state.get_grid_cell("board", row, col)?;
            if cell.is_null() {
                continue;
            }
            
            let entity = Entity::new(cell);
            if entity.get_property_str("piece_type") == Some("king") {
                if let Some(player) = entity.get_property_str("player") {
                    kings.insert(player.to_string(), (row, col));
                }
            }
        }
    }
    
    // Check if any king is in checkmate
    for (player_id, (king_row, king_col)) in &kings {
        if is_in_checkmate(game_state, player_id, *king_row, *king_col)? {
            let winner = get_opponent_player(game_state, player_id)?;
            return Ok(json!({
                "gameEnded": true,
                "winner": winner,
                "tie": false,
                "reason": "checkmate"
            }));
        }
    }
    
    // Check for stalemate
    let current_player = game_state.get_current_player()?;
    if is_in_stalemate(game_state, current_player)? {
        return Ok(json!({
            "gameEnded": true,
            "winner": null,
            "tie": true,
            "reason": "stalemate"
        }));
    }
    
    Ok(json!({
        "gameEnded": false,
        "winner": null,
        "tie": false,
        "reason": null
    }))
}

fn is_in_checkmate(
    game_state: &GameState, 
    player_id: &str, 
    king_row: usize, 
    king_col: usize
) -> Result<bool, HookError> {
    // Check if king is in check
    if !is_king_in_check(game_state, player_id, king_row, king_col)? {
        return Ok(false);
    }
    
    // Check if player has any legal moves to escape check
    has_legal_moves_to_escape_check(game_state, player_id)
}

fn is_king_in_check(
    game_state: &GameState,
    player_id: &str,
    king_row: usize,
    king_col: usize
) -> Result<bool, HookError> {
    // Check if any opponent piece can attack the king
    for row in 0..8 {
        for col in 0..8 {
            let cell = game_state.get_grid_cell("board", row, col)?;
            if cell.is_null() {
                continue;
            }
            
            let entity = Entity::new(cell);
            if entity.belongs_to_player(player_id) {
                continue; // Skip own pieces
            }
            
            let piece_type = entity.get_property_str("piece_type").unwrap_or("unknown");
            if can_piece_attack(piece_type, row, col, king_row, king_col, game_state)? {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

fn can_piece_attack(
    piece_type: &str,
    from_row: usize, from_col: usize,
    target_row: usize, target_col: usize,
    game_state: &GameState
) -> Result<bool, HookError> {
    use crate::validation::is_valid_movement;
    is_valid_movement(piece_type, from_row, from_col, target_row, target_col, game_state)
}

fn has_legal_moves_to_escape_check(
    game_state: &GameState,
    player_id: &str
) -> Result<bool, HookError> {
    // This would involve trying all possible moves for the player
    // and checking if any of them result in the king not being in check
    // Simplified implementation returns false (checkmate)
    Ok(false)
}

fn is_in_stalemate(game_state: &GameState, player_id: &str) -> Result<bool, HookError> {
    // Player has no legal moves but is not in check
    // Simplified implementation
    Ok(false)
}

fn get_opponent_player(game_state: &GameState, player_id: &str) -> Result<String, HookError> {
    let player_count = game_state.get_player_count();
    if player_count == 2 {
        if player_id == "p1" {
            Ok("p2".to_string())
        } else {
            Ok("p1".to_string())
        }
    } else {
        Err(HookError::LogicError("Multiple player games not implemented".to_string()))
    }
}

fn check_piece_elimination(game_state: &GameState) -> Result<Value, HookError> {
    let mut player_pieces = std::collections::HashMap::new();
    
    // Count pieces for each player
    for row in 0..8 {
        for col in 0..8 {
            let cell = game_state.get_grid_cell("board", row, col)?;
            if cell.is_null() {
                continue;
            }
            
            let entity = Entity::new(cell);
            if let Some(player) = entity.get_property_str("player") {
                *player_pieces.entry(player.to_string()).or_insert(0) += 1;
            }
        }
    }
    
    // Check if any player has no pieces left
    for player in ["p1", "p2"] {
        if player_pieces.get(player).unwrap_or(&0) == &0 {
            let winner = get_opponent_player(game_state, player)?;
            return Ok(json!({
                "gameEnded": true,
                "winner": winner,
                "tie": false,
                "reason": "elimination"
            }));
        }
    }
    
    Ok(json!({
        "gameEnded": false,
        "winner": null,
        "tie": false,
        "reason": null
    }))
}

fn check_territory_control(game_state: &GameState) -> Result<Value, HookError> {
    // Count controlled squares for each player
    let mut control_count = std::collections::HashMap::new();
    
    for row in 0..8 {
        for col in 0..8 {
            let controlling_player = get_square_controller(game_state, row, col)?;
            if let Some(player) = controlling_player {
                *control_count.entry(player).or_insert(0) += 1;
            }
        }
    }
    
    // Check if any player controls more than 75% of the board
    let total_squares = 64;
    let threshold = (total_squares * 3) / 4; // 75%
    
    for (player, count) in &control_count {
        if *count >= threshold {
            return Ok(json!({
                "gameEnded": true,
                "winner": player,
                "tie": false,
                "reason": "territory_control"
            }));
        }
    }
    
    Ok(json!({
        "gameEnded": false,
        "winner": null,
        "tie": false,
        "reason": null
    }))
}

fn get_square_controller(
    game_state: &GameState,
    row: usize,
    col: usize
) -> Result<Option<String>, HookError> {
    // Simplified: square is controlled by the player who has a piece on it
    let cell = game_state.get_grid_cell("board", row, col)?;
    if cell.is_null() {
        return Ok(None);
    }
    
    let entity = Entity::new(cell);
    Ok(entity.get_property_str("player").map(|s| s.to_string()))
}

fn check_game_limits(game_state: &GameState) -> Result<Value, HookError> {
    let turn_number = game_state.get_turn_number();
    const MAX_TURNS: u64 = 100; // Example limit
    
    if turn_number >= MAX_TURNS {
        // Determine winner by piece count or other criteria
        let winner = determine_winner_by_points(game_state)?;
        
        if let Some(winner_id) = winner {
            Ok(json!({
                "gameEnded": true,
                "winner": winner_id,
                "tie": false,
                "reason": "turn_limit"
            }))
        } else {
            Ok(json!({
                "gameEnded": true,
                "winner": null,
                "tie": true,
                "reason": "turn_limit"
            }))
        }
    } else {
        Ok(json!({
            "gameEnded": false,
            "winner": null,
            "tie": false,
            "reason": null
        }))
    }
}

fn determine_winner_by_points(game_state: &GameState) -> Result<Option<String>, HookError> {
    let mut player_scores = std::collections::HashMap::new();
    
    // Calculate scores based on remaining pieces
    for row in 0..8 {
        for col in 0..8 {
            let cell = game_state.get_grid_cell("board", row, col)?;
            if cell.is_null() {
                continue;
            }
            
            let entity = Entity::new(cell);
            if let Some(player) = entity.get_property_str("player") {
                let piece_type = entity.get_property_str("piece_type").unwrap_or("pawn");
                let piece_value = get_piece_value(piece_type);
                *player_scores.entry(player.to_string()).or_insert(0) += piece_value;
            }
        }
    }
    
    // Find player with highest score
    let max_score = player_scores.values().max().copied().unwrap_or(0);
    let winners: Vec<_> = player_scores.iter()
        .filter(|(_, &score)| score == max_score)
        .map(|(player, _)| player.clone())
        .collect();
    
    if winners.len() == 1 {
        Ok(Some(winners[0].clone()))
    } else {
        Ok(None) // Tie
    }
}

use crate::ai::get_piece_value;
```

## Scoring Systems

### Complex Score Calculation

```rust
// src/scoring.rs
use serde_json::{Value, json};
use crate::game_state::GameState;
use crate::entities::Entity;
use crate::error::{HookError, success_response, error_response};
use crate::memory::{parse_json_from_memory, parse_string_from_memory, to_memory_ptr};

#[no_mangle]
pub extern "C" fn calculate_score(
    state_ptr: *const u8, state_len: usize,
    player_ptr: *const u8, player_len: usize
) -> *const u8 {
    let result = calculate_score_impl(state_ptr, state_len, player_ptr, player_len);
    to_memory_ptr(&result.to_string())
}

fn calculate_score_impl(
    state_ptr: *const u8, state_len: usize,
    player_ptr: *const u8, player_len: usize
) -> Value {
    let state = match parse_json_from_memory(state_ptr, state_len) {
        Ok(s) => s,
        Err(e) => return error_response(&HookError::ParseError(e.to_string())),
    };
    
    let player_id = match parse_string_from_memory(player_ptr, player_len) {
        Ok(p) => p,
        Err(e) => return error_response(&HookError::ParseError(e.to_string())),
    };
    
    match calculate_complex_score(&state, &player_id) {
        Ok(score_data) => success_response(score_data),
        Err(e) => error_response(&e),
    }
}

fn calculate_complex_score(state: &Value, player_id: &str) -> Result<Value, HookError> {
    let game_state = GameState::new(state);
    let mut total_score = 0;
    let mut score_breakdown = json!({});
    
    // 1. Piece value score
    let piece_score = calculate_piece_value_score(&game_state, player_id)?;
    total_score += piece_score;
    score_breakdown["piece_value"] = json!(piece_score);
    
    // 2. Position control score
    let position_score = calculate_position_score(&game_state, player_id)?;
    total_score += position_score;
    score_breakdown["position_control"] = json!(position_score);
    
    // 3. Development score
    let development_score = calculate_development_score(&game_state, player_id)?;
    total_score += development_score;
    score_breakdown["development"] = json!(development_score);
    
    // 4. King safety score
    let safety_score = calculate_king_safety_score(&game_state, player_id)?;
    total_score += safety_score;
    score_breakdown["king_safety"] = json!(safety_score);
    
    // 5. Material advantage
    let material_advantage = calculate_material_advantage(&game_state, player_id)?;
    total_score += material_advantage;
    score_breakdown["material_advantage"] = json!(material_advantage);
    
    Ok(json!({
        "total_score": total_score,
        "breakdown": score_breakdown,
        "rank": calculate_player_rank(&game_state, player_id, total_score)?,
        "percentile": calculate_percentile(&game_state, player_id, total_score)?
    }))
}

fn calculate_piece_value_score(game_state: &GameState, player_id: &str) -> Result<i32, HookError> {
    let mut score = 0;
    
    for row in 0..8 {
        for col in 0..8 {
            let cell = game_state.get_grid_cell("board", row, col)?;
            if cell.is_null() {
                continue;
            }
            
            let entity = Entity::new(cell);
            if entity.belongs_to_player(player_id) {
                let piece_type = entity.get_property_str("piece_type").unwrap_or("pawn");
                score += get_piece_base_value(piece_type);
            }
        }
    }
    
    Ok(score)
}

fn calculate_position_score(game_state: &GameState, player_id: &str) -> Result<i32, HookError> {
    let mut score = 0;
    
    for row in 0..8 {
        for col in 0..8 {
            let cell = game_state.get_grid_cell("board", row, col)?;
            if cell.is_null() {
                continue;
            }
            
            let entity = Entity::new(cell);
            if entity.belongs_to_player(player_id) {
                let piece_type = entity.get_property_str("piece_type").unwrap_or("pawn");
                score += get_position_value(piece_type, row, col);
            }
        }
    }
    
    Ok(score)
}

fn calculate_development_score(game_state: &GameState, player_id: &str) -> Result<i32, HookError> {
    let mut developed_pieces = 0;
    let starting_row = if player_id == "p1" { 0 } else { 7 };
    
    // Count pieces that have moved from starting positions
    for col in 0..8 {
        let cell = game_state.get_grid_cell("board", starting_row, col)?;
        if cell.is_null() {
            // Piece has moved from starting position
            developed_pieces += 1;
        } else {
            let entity = Entity::new(cell);
            if !entity.belongs_to_player(player_id) {
                // Enemy piece on our starting row - we lost material
                developed_pieces -= 1;
            }
        }
    }
    
    Ok(developed_pieces * 10)
}

fn calculate_king_safety_score(game_state: &GameState, player_id: &str) -> Result<i32, HookError> {
    // Find player's king
    let king_position = find_king_position(game_state, player_id)?;
    
    if let Some((king_row, king_col)) = king_position {
        let mut safety_score = 0;
        
        // Check for pawn shield
        safety_score += calculate_pawn_shield_score(game_state, player_id, king_row, king_col)?;
        
        // Penalty for exposed king
        if is_king_exposed(game_state, player_id, king_row, king_col)? {
            safety_score -= 50;
        }
        
        // Bonus for castled king (simplified check)
        if is_king_castled(king_row, king_col, player_id) {
            safety_score += 30;
        }
        
        Ok(safety_score)
    } else {
        // King missing - major penalty
        Ok(-1000)
    }
}

fn calculate_material_advantage(game_state: &GameState, player_id: &str) -> Result<i32, HookError> {
    let player_material = calculate_piece_value_score(game_state, player_id)?;
    let opponent_id = if player_id == "p1" { "p2" } else { "p1" };
    let opponent_material = calculate_piece_value_score(game_state, opponent_id)?;
    
    Ok(player_material - opponent_material)
}

fn get_piece_base_value(piece_type: &str) -> i32 {
    match piece_type {
        "pawn" => 100,
        "knight" => 320,
        "bishop" => 330,
        "rook" => 500,
        "queen" => 900,
        "king" => 20000, // King is invaluable
        _ => 0,
    }
}

fn get_position_value(piece_type: &str, row: usize, col: usize) -> i32 {
    // Position-specific bonuses
    match piece_type {
        "pawn" => get_pawn_position_value(row, col),
        "knight" => get_knight_position_value(row, col),
        "bishop" => get_bishop_position_value(row, col),
        "rook" => get_rook_position_value(row, col),
        "queen" => get_queen_position_value(row, col),
        "king" => get_king_position_value(row, col),
        _ => 0,
    }
}

fn get_pawn_position_value(row: usize, col: usize) -> i32 {
    // Pawn position values (for white, adjust for black)
    let pawn_table = [
        [ 0,  0,  0,  0,  0,  0,  0,  0],
        [50, 50, 50, 50, 50, 50, 50, 50],
        [10, 10, 20, 30, 30, 20, 10, 10],
        [ 5,  5, 10, 25, 25, 10,  5,  5],
        [ 0,  0,  0, 20, 20,  0,  0,  0],
        [ 5, -5,-10,  0,  0,-10, -5,  5],
        [ 5, 10, 10,-20,-20, 10, 10,  5],
        [ 0,  0,  0,  0,  0,  0,  0,  0],
    ];
    pawn_table[row][col]
}

fn get_knight_position_value(row: usize, col: usize) -> i32 {
    // Knights are better in the center
    let knight_table = [
        [-50,-40,-30,-30,-30,-30,-40,-50],
        [-40,-20,  0,  0,  0,  0,-20,-40],
        [-30,  0, 10, 15, 15, 10,  0,-30],
        [-30,  5, 15, 20, 20, 15,  5,-30],
        [-30,  0, 15, 20, 20, 15,  0,-30],
        [-30,  5, 10, 15, 15, 10,  5,-30],
        [-40,-20,  0,  5,  5,  0,-20,-40],
        [-50,-40,-30,-30,-30,-30,-40,-50],
    ];
    knight_table[row][col]
}

fn get_bishop_position_value(row: usize, col: usize) -> i32 {
    // Bishops prefer long diagonals
    let bishop_table = [
        [-20,-10,-10,-10,-10,-10,-10,-20],
        [-10,  0,  0,  0,  0,  0,  0,-10],
        [-10,  0,  5, 10, 10,  5,  0,-10],
        [-10,  5,  5, 10, 10,  5,  5,-10],
        [-10,  0, 10, 10, 10, 10,  0,-10],
        [-10, 10, 10, 10, 10, 10, 10,-10],
        [-10,  5,  0,  0,  0,  0,  5,-10],
        [-20,-10,-10,-10,-10,-10,-10,-20],
    ];
    bishop_table[row][col]
}

fn get_rook_position_value(row: usize, col: usize) -> i32 {
    // Rooks prefer open files and 7th rank
    let rook_table = [
        [ 0,  0,  0,  0,  0,  0,  0,  0],
        [ 5, 10, 10, 10, 10, 10, 10,  5],
        [-5,  0,  0,  0,  0,  0,  0, -5],
        [-5,  0,  0,  0,  0,  0,  0, -5],
        [-5,  0,  0,  0,  0,  0,  0, -5],
        [-5,  0,  0,  0,  0,  0,  0, -5],
        [-5,  0,  0,  0,  0,  0,  0, -5],
        [ 0,  0,  0,  5,  5,  0,  0,  0],
    ];
    rook_table[row][col]
}

fn get_queen_position_value(row: usize, col: usize) -> i32 {
    // Queen prefers center, but not too early
    let queen_table = [
        [-20,-10,-10, -5, -5,-10,-10,-20],
        [-10,  0,  0,  0,  0,  0,  0,-10],
        [-10,  0,  5,  5,  5,  5,  0,-10],
        [ -5,  0,  5,  5,  5,  5,  0, -5],
        [  0,  0,  5,  5,  5,  5,  0, -5],
        [-10,  5,  5,  5,  5,  5,  0,-10],
        [-10,  0,  5,  0,  0,  0,  0,-10],
        [-20,-10,-10, -5, -5,-10,-10,-20],
    ];
    queen_table[row][col]
}

fn get_king_position_value(row: usize, col: usize) -> i32 {
    // King safety is paramount
    let king_table = [
        [-30,-40,-40,-50,-50,-40,-40,-30],
        [-30,-40,-40,-50,-50,-40,-40,-30],
        [-30,-40,-40,-50,-50,-40,-40,-30],
        [-30,-40,-40,-50,-50,-40,-40,-30],
        [-20,-30,-30,-40,-40,-30,-30,-20],
        [-10,-20,-20,-20,-20,-20,-20,-10],
        [ 20, 20,  0,  0,  0,  0, 20, 20],
        [ 20, 30, 10,  0,  0, 10, 30, 20],
    ];
    king_table[row][col]
}

fn find_king_position(game_state: &GameState, player_id: &str) -> Result<Option<(usize, usize)>, HookError> {
    for row in 0..8 {
        for col in 0..8 {
            let cell = game_state.get_grid_cell("board", row, col)?;
            if cell.is_null() {
                continue;
            }
            
            let entity = Entity::new(cell);
            if entity.belongs_to_player(player_id) && 
               entity.get_property_str("piece_type") == Some("king") {
                return Ok(Some((row, col)));
            }
        }
    }
    Ok(None)
}

fn calculate_pawn_shield_score(
    game_state: &GameState,
    player_id: &str,
    king_row: usize,
    king_col: usize
) -> Result<i32, HookError> {
    let mut shield_score = 0;
    let pawn_row = if player_id == "p1" { king_row + 1 } else { king_row - 1 };
    
    // Check for pawns in front of king
    for col_offset in -1..=1i32 {
        let check_col = (king_col as i32 + col_offset) as usize;
        if check_col < 8 {
            let cell = game_state.get_grid_cell("board", pawn_row, check_col)?;
            if !cell.is_null() {
                let entity = Entity::new(cell);
                if entity.belongs_to_player(player_id) && 
                   entity.get_property_str("piece_type") == Some("pawn") {
                    shield_score += 10;
                }
            }
        }
    }
    
    Ok(shield_score)
}

fn is_king_exposed(
    game_state: &GameState,
    player_id: &str,
    king_row: usize,
    king_col: usize
) -> Result<bool, HookError> {
    // Check if king is on open files or ranks
    // Simplified implementation
    Ok(king_col >= 3 && king_col <= 5) // Center files are more exposed
}

fn is_king_castled(king_row: usize, king_col: usize, player_id: &str) -> bool {
    // Simplified castling detection
    let expected_row = if player_id == "p1" { 0 } else { 7 };
    king_row == expected_row && (king_col == 2 || king_col == 6)
}

fn calculate_player_rank(
    game_state: &GameState,
    player_id: &str,
    score: i32
) -> Result<u32, HookError> {
    // Calculate rank among all players
    let mut all_scores = Vec::new();
    
    for player in ["p1", "p2"] {
        if player == player_id {
            all_scores.push(score);
        } else {
            let other_score = calculate_piece_value_score(game_state, player)?;
            all_scores.push(other_score);
        }
    }
    
    all_scores.sort_by(|a, b| b.cmp(a)); // Sort descending
    
    let rank = all_scores.iter().position(|&s| s == score).unwrap_or(0) + 1;
    Ok(rank as u32)
}

fn calculate_percentile(
    game_state: &GameState,
    _player_id: &str,
    score: i32
) -> Result<f32, HookError> {
    // Simplified percentile calculation
    // In a real implementation, this might compare against historical data
    let normalized_score = (score as f32 + 1000.0) / 2000.0; // Normalize to 0-1
    Ok((normalized_score * 100.0).min(100.0).max(0.0))
}
```

## Building and Deployment

### Build Configuration

```bash
# Build for production
cargo build --release --target wasm32-unknown-unknown

# Optimize binary size
wasm-opt -Os target/wasm32-unknown-unknown/release/my_game_hooks.wasm -o hooks.wasm

# Copy to game directory
cp hooks.wasm ../games/my-game/1.0/hooks.wasm
```

### Testing Hooks

```rust
// tests/integration_tests.rs
use serde_json::json;
use my_game_hooks::*;

#[test]
fn test_win_condition_detection() {
    let test_state = json!({
        "zones": {
            "board": {
                "type": "grid",
                "cells": [
                    [{"entity": "x_token"}, {"entity": "x_token"}, {"entity": "x_token"}],
                    [null, null, null],
                    [null, null, null]
                ]
            }
        },
        "meta": {
            "currentPlayer": "p1",
            "tick": 5
        }
    });
    
    let state_str = test_state.to_string();
    let result_ptr = check_win_condition(
        state_str.as_ptr(),
        state_str.len()
    );
    
    // Parse result and verify
    // Implementation depends on your memory management
}
```

## Best Practices

### Performance Optimization

1. **Minimize Memory Allocations**
   ```rust
   // ✓ Reuse buffers
   thread_local! {
       static MOVE_BUFFER: RefCell<Vec<Move>> = RefCell::new(Vec::new());
   }
   
   // ✗ Allocate new vectors repeatedly
   fn get_moves() -> Vec<Move> {
       Vec::new() // New allocation each time
   }
   ```

2. **Efficient State Access**
   ```rust
   // ✓ Cache frequently accessed data
   struct GameAnalysis {
       piece_positions: HashMap<String, Vec<(usize, usize)>>,
       control_map: [[Option<String>; 8]; 8],
   }
   
   // ✗ Scan entire board repeatedly
   fn count_pieces_slow(state: &GameState) -> usize {
       // Scans board multiple times
   }
   ```

3. **Early Returns**
   ```rust
   // ✓ Fast exit conditions
   fn check_win_condition(state: &GameState) -> Result<Value, HookError> {
       if state.is_game_ended() {
           return Ok(json!({"gameEnded": true}));
       }
       // Continue with expensive checks
   }
   ```

### Error Handling

1. **Graceful Degradation**
   ```rust
   // ✓ Handle errors gracefully
   fn ai_move_with_fallback(state: &Value) -> Value {
       match generate_complex_move(state) {
           Ok(move_action) => move_action,
           Err(_) => generate_simple_move(state).unwrap_or_else(|_| {
               json!({"verb": "pass"}) // Ultimate fallback
           })
       }
   }
   ```

2. **Informative Error Messages**
   ```rust
   // ✓ Helpful errors
   return Err(HookError::ValidationError(
       format!("Invalid move: {} cannot move from [{},{}] to [{},{}] because path is blocked", 
               piece_type, from_row, from_col, to_row, to_col)
   ));
   
   // ✗ Vague errors
   return Err(HookError::ValidationError("Invalid move".to_string()));
   ```

### Security Considerations

1. **Input Validation**
   ```rust
   // ✓ Validate all inputs
   fn validate_coordinates(row: usize, col: usize) -> Result<(), HookError> {
       if row >= 8 || col >= 8 {
           return Err(HookError::ValidationError("Coordinates out of bounds".to_string()));
       }
       Ok(())
   }
   ```

2. **Resource Limits**
   ```rust
   // ✓ Prevent infinite loops
   fn minimax_with_limit(depth: i32, max_depth: i32) -> i32 {
       if depth >= max_depth {
           return evaluate_position();
       }
       // Continue minimax
   }
   ```

This comprehensive guide covers all aspects of custom hook development in Bluefelt. Use it to implement sophisticated game logic while maintaining performance, security, and reliability.
# Bluefelt Server

The Bluefelt server is the core engine that powers multiplayer turn-based games. Built in Rust using modern async technologies, it provides real-time game state management, WebSocket communication, and WebAssembly-based rule enforcement.

## Overview

The server handles:
- **Game State Management** - Authoritative game state with JSON Patch synchronization
- **Real-time Communication** - WebSocket connections for instant updates
- **Rule Enforcement** - Built-in verbs and WebAssembly hooks for game logic
- **Lobby Management** - Multi-game lobby system with player management
- **Game Loading** - Dynamic loading of game bundles at runtime

## Architecture

### Core Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   WebSocket     │    │   Game Engine   │    │   Bundle        │
│   Handler       │───▶│   (State mgmt)  │───▶│   Loader        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Lobby         │    │   Action        │    │   WASM          │
│   Manager       │    │   Processor     │    │   Runtime       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Technology Stack

- **Rust** - Systems programming language for performance and safety
- **Axum** - Modern async web framework
- **Tokio** - Async runtime for handling concurrent connections
- **Wasmtime** - WebAssembly runtime for custom game logic
- **DashMap** - Concurrent hash map for lobby management
- **serde_json** - JSON serialization/deserialization

## Game Engine

### State Management

The engine maintains authoritative game state as JSON and synchronizes changes using JSON Patch. For a comprehensive guide on state structure and client/server synchronization, see [State Structure and Synchronization](./state-structure.md).

```json
{
  "zones": {
    "board": {
      "type": "grid",
      "cells": [[null, {"entity": "x_token"}, null], ...]
    },
    "hand_p1": {
      "type": "list", 
      "items": [{"entity": "card_hearts_a"}, ...]
    }
  },
  "meta": {
    "players": [{"id": "p1"}, {"id": "p2"}],
    "tick": 42,
    "turn": 1,
    "currentPlayer": "p2",
    "gameStatus": {
      "state": "playing",
      "winner": null,
      "tie": false
    }
  }
}
```

### Built-in Verbs

The engine provides built-in verbs for common game operations:

#### Core Movement Verbs

```rust
// Place entity at grid location
apply_verb(state, "place", {
    "location": "/zones/board/cells/0/0",
    "entity": "x_token"
})

// Move entity between zones
apply_verb(state, "moveEntity", {
    "from": "/zones/hand_p1/items/0", 
    "to": "/zones/discard/items"
})

// Draw from deck to hand
apply_verb(state, "draw", {
    "from": "/zones/deck",
    "to": "/zones/hand_p1", 
    "count": 3
})
```

#### Game Flow Verbs

```rust
// Advance turn
apply_verb(state, "nextTurn", {})

// Change phase
apply_verb(state, "setPhase", {
    "phaseSet": "main",
    "phase": "scoring"
})
```

### Path System

The engine uses JSON Pointer-style paths to reference game state:

- **Grid cells**: `/zones/board/cells/row/col`
- **List items**: `/zones/hand_p1/items/index`
- **Zone contents**: `/zones/deck/items`
- **Metadata**: `/meta/currentPlayer`

### Action Processing

Actions flow through the system as follows:

1. **WebSocket Message** - Player sends action JSON
2. **Validation** - Check action availability and conditions  
3. **Verb Application** - Execute built-in or custom logic
4. **Patch Generation** - Create JSON Patch for state changes
5. **Broadcast** - Send patches to all connected clients

## Lobby System

### Lobby Management

Each game session is managed by a `Lobby` instance:

```rust
pub struct Lobby {
    pub id: String,
    pub bundle: Bundle,           // Game definition
    pub state: Mutex<Value>,      // Current game state
    pub tick: Mutex<u64>,         // Version counter
    pub connections: DashMap<String, WebSocket>, // Player connections
}
```

### Connection Handling

Players connect via WebSocket to `/ws/lobby/{lobby_id}/{player_id}`:

```
ws://localhost:8000/ws/lobby/my-game-123/player1
```

### Message Protocol

#### Client to Server

```json
{
  "type": "action",
  "data": {
    "verb": "place",
    "args": {
      "location": "/zones/board/cells/0/0",
      "entity": "x_token"
    }
  }
}
```

#### Server to Client

```json
{
  "type": "welcome",
  "data": {
    "playerId": "p1",
    "state": { /* full game state */ },
    "tick": 0
  }
}
```

```json
{
  "type": "diff", 
  "data": {
    "tick": 1,
    "patches": [
      {
        "op": "replace",
        "path": "/zones/board/cells/0/0", 
        "value": {"entity": "x_token"}
      }
    ]
  }
}
```

## Bundle Loading

### Bundle Structure

Games are loaded as JSON bundles from the `bundles/` directory:

```
bundles/
└── tic-tac-toe/
    └── 1.0/
        ├── manifest.json
        ├── entities.json
        ├── zones.json
        ├── actions.json
        ├── phases.json
        └── hooks.wasm (optional)
```

### Loading Process

1. **Scan Directory** - Find all game versions
2. **Parse Manifests** - Load game metadata
3. **Expand Shorthands** - Process player templates and built-ins
4. **Validate** - Check for required fields and references
5. **Store** - Cache in memory for fast access

### Shorthand Expansion

The server automatically expands shorthand syntax:

```yaml
# Input: Player template
- id: "token_{player}"
  type: "token"

# Output: Individual entities  
- id: "token_p1"
  type: "token"
- id: "token_p2" 
  type: "token"
```

```yaml
# Input: Standard deck
- type: "standardDeck"

# Output: 52 individual cards
- id: "card_hearts_a"
  props: { suit: "hearts", rank: "A", value: 1 }
# ... 51 more cards
```

## WebAssembly Integration

### Custom Hooks

For complex game logic, the server can execute WebAssembly modules:

```rust
// Host functions available to WASM
pub fn get_zone_contents(zone_path: &str) -> Vec<Entity>;
pub fn set_game_status(status: GameStatus);
pub fn get_current_player() -> PlayerId;
```

### Hook Execution

```rust
// Execute WASM hook
let result = wasmtime_instance.call(
    "check_win_condition", 
    &[state_ptr]
)?;
```

### Safety and Sandboxing

- **Memory Isolation** - WASM runs in isolated memory space
- **Limited API** - Only specific host functions exposed
- **Timeout Protection** - Hooks have execution time limits
- **Resource Limits** - Memory and CPU usage controlled

## Performance Features

### Concurrent Processing

- **Async WebSockets** - Handle thousands of concurrent connections
- **Lock-free Reads** - DashMap allows concurrent state reads
- **Background Tasks** - Phase processing runs asynchronously

### Memory Management

- **Shared State** - Game state shared between connections
- **Patch Compression** - Only send state changes, not full state
- **Bundle Caching** - Games loaded once and cached in memory

### Optimization Strategies

```rust
// Efficient path navigation
fn get_cell_value(state: &Value, path: &str) -> Result<Value, String> {
    let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in &path_parts {
        if let Ok(index) = part.parse::<usize>() {
            current = &current.as_array()?[index];
        } else {
            current = current.get(part)?;
        }
    }
    
    Ok(current.clone())
}
```

## Configuration

### Server Settings

```bash
# Environment variables
BLUEFELT_PORT=8000           # Server port
BLUEFELT_HOST=0.0.0.0        # Bind address
BLUEFELT_BUNDLES_DIR=bundles # Game bundles directory
BLUEFELT_LOG_LEVEL=info      # Logging level
```

### Runtime Configuration

The server automatically configures based on available games:

```rust
// Load games from bundles directory
let bundles = BundleMap::load_dir("bundles")?;
println!("Loaded {} games", bundles.len());

// Start server with CORS for development
let app = Router::new()
    .route("/ws/lobby/:lobby_id/:player_id", get(websocket_handler))
    .route("/api/lobbies", get(list_lobbies))
    .layer(CorsLayer::permissive());
```

## Monitoring and Debugging

### Logging

The server provides comprehensive logging:

```
[INFO] Loading game: tic-tac-toe v1.0
[DEBUG] Expanding shorthand: {player} -> p1, p2
[INFO] Server started on http://0.0.0.0:8000
[DEBUG] WebSocket connection: lobby=game123, player=p1
[DEBUG] Action received: place at /zones/board/cells/0/0
[DEBUG] State updated: tick 42 -> 43
```

### Health Endpoints

```bash
# Check server status
curl http://localhost:8000/api/health

# List active lobbies  
curl http://localhost:8000/api/lobbies

# Get lobby details
curl http://localhost:8000/api/lobbies/my-game-123
```

### Performance Metrics

Monitor key metrics:
- **Active Connections** - Number of WebSocket connections
- **Lobby Count** - Number of active game sessions
- **Action Latency** - Time to process actions
- **Memory Usage** - Game state and bundle memory consumption

## Development Workflow

### Building

```bash
cd server
cargo build          # Debug build
cargo build --release # Production build
```

### Testing

```bash
# Run all tests (29 total)
cargo test

# Run specific test categories
cargo test --lib engine::tests        # Unit tests (13)
cargo test --test engine_integration  # Integration tests (8) 
cargo test --test websocket_tests     # WebSocket tests (8)
```

### Running

```bash
# Development mode
cargo run

# Production mode  
./target/release/bluefelt-core

# With custom bundle directory
BLUEFELT_BUNDLES_DIR=/path/to/bundles cargo run
```

## Error Handling

### Game Errors

```rust
// Action validation errors
pub enum ActionError {
    InvalidPath(String),
    ConditionNotMet(String), 
    UnknownVerb(String),
    StateCorruption(String),
}

// Graceful error responses
if let Err(e) = apply_action(&bundle, &mut state, &action) {
    send_error_message(&mut socket, &format!("Action failed: {}", e));
}
```

### Connection Errors

- **WebSocket Drops** - Automatic cleanup of disconnected players
- **Message Parsing** - Invalid JSON handled gracefully
- **Rate Limiting** - Protection against message flooding

### Recovery Strategies

- **State Validation** - Periodic state consistency checks
- **Lobby Cleanup** - Remove empty lobbies automatically  
- **Bundle Reloading** - Hot reload games without restart
- **Graceful Shutdown** - Save active game states on exit

## Deployment

### Production Setup

```dockerfile
# Dockerfile example
FROM rust:1.70 AS builder
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/bluefelt-core /usr/local/bin/
COPY bundles/ /app/bundles/
EXPOSE 8000
CMD ["bluefelt-core"]
```

### Scaling Considerations

- **Horizontal Scaling** - Multiple server instances behind load balancer
- **State Persistence** - Redis or database for cross-instance state
- **Bundle Distribution** - Shared storage for game bundles
- **WebSocket Affinity** - Sticky sessions for WebSocket connections

### Security

- **Input Validation** - All player actions validated
- **Rate Limiting** - Prevent spam and DoS attacks
- **CORS Configuration** - Restrict cross-origin requests in production
- **WASM Sandboxing** - Custom hooks run in isolated environment

The Bluefelt server provides a robust, scalable foundation for multiplayer turn-based games with real-time synchronization and extensible game logic through WebAssembly integration.
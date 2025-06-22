# Bluefelt Server

The Bluefelt server is the core engine that powers multiplayer turn-based games. Built in Rust using modern async technologies, it provides real-time game state management, WebSocket communication, and comprehensive declarative rule enforcement.

## Related Server Documentation

- **[State Structure](./state-structure.md)** - **CRITICAL**: How game state is synchronized between client and server
- **[Game Log Parameters](./game-log-parameters.md)** - Template system for game action logging

These documents provide essential details for understanding server internals and client-server integration.

## Overview

The server handles:
- **Game State Management** - Authoritative game state with JSON Patch synchronization
- **Real-time Communication** - WebSocket connections for instant updates
- **Rule Enforcement** - Powerful built-in verbs and declarative conditions for game logic
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
│   Lobby         │    │   Action        │    │   Condition     │
│   Manager       │    │   Processor     │    │   Engine        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Connection     │    │    Memory       │    │   Lock          │
│   Manager       │    │    Manager      │    │  Optimizer      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Technology Stack

- **Rust** - Systems programming language for performance and safety
- **Axum** - Modern async web framework
- **Tokio** - Async runtime for handling concurrent connections
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

The lobby system has been refactored for better separation of concerns:

```rust
pub struct Lobby {
    pub state: Arc<LobbyState>,     // Persistent lobby state
    pub bundles: Arc<BundleMap>,    // Available game bundles
    pub clients: RwLock<HashMap<String, ClientInfo>>, // Connection preferences
    pub rng: Arc<GameRng>,          // Seeded RNG for determinism
    pub lobby_map: Arc<LobbyMap>,   // Global lobby registry
}

pub struct LobbyState {
    pub id: String,
    pub name: String,
    pub owner: Arc<Mutex<Option<String>>>,  // Lobby owner (first player to join)
    pub members: Arc<RwLock<Vec<LobbyMember>>>,  // Lobby members
    pub tables: Arc<Mutex<HashMap<String, Arc<TableInstance>>>>, // Game tables
    pub chat: Arc<ChatSystem>,      // Integrated chat
    pub invite_code: String,        // For joining
    pub settings: LobbySettings,    // Configurable options
}
```

### Connection Handling

Players connect via WebSocket with username:

```
ws://localhost:8000/api/lobbies/{lobby_id}/ws?player={username}&join=true
```

### Lobby Ownership

The first player to join a lobby becomes the owner/admin. The owner has special permissions:

- **Rename Lobby** - Change the lobby name using `renameLobby` action
- **Future Permissions** - Additional admin controls can be added (kick players, change settings, etc.)

#### Ownership Transfer

When the owner leaves the lobby, ownership automatically transfers to the earliest joined remaining member. This ensures there's always an owner as long as the lobby has members.

#### Lobby Archival

When the last member leaves a lobby, it becomes archived:
- **Archived Status** - The lobby's `archived` flag is set to `true`
- **No New Joins** - Archived lobbies reject all join attempts
- **Hidden from Lists** - Archived lobbies are filtered out of the public lobby list
- **Permanent State** - Archived lobbies cannot be unarchived

This prevents ghost lobbies from accumulating and ensures clean resource management.

### Message Protocol

#### Client Actions

```json
// Table management
{"action": "createTable", "bundleId": "tic-tac-toe"}
{"action": "joinTable", "tableId": "table_123"}  // Auto-seat assignment
{"action": "leaveTable", "tableId": "table_123"}
{"action": "start_game", "tableId": "table_123"}

// Chat
{"action": "sendChatMessage", "message": "Hello!", "scope": "lobby"}
{"action": "sendChatMessage", "message": "GL!", "scope": "table", "tableId": "table_123"}

// Lobby management (owner only)
{"action": "renameLobby", "name": "New Lobby Name"}

// Legacy game actions (deprecated - use table system instead)
{"action": "createGame", "gameType": "tic-tac-toe"}  // Use createTable
{"action": "gameAction", "gameId": "game_123", "verb": "place", ...}  // Use action
```

#### Server Messages

```json
// Initial connection
{
  "type": "lobbyJoined",
  "lobby": {
    "id": "lobby_xyz",
    "name": "My Lobby",
    "owner": "alice",  // Username of lobby owner (first to join)
    "archived": false,  // Whether lobby is archived (no new joins)
    "myId": "alice",
    "tables": [...],
    "recentChat": [...]
  }
}

// Table updates
{
  "type": "tableCreated",
  "table": {
    "id": "table_123",
    "bundleId": "tic-tac-toe",
    "seats": [null, null],
    "status": "Open"
  }
}

{
  "type": "tableUpdated",
  "tableId": "table_123",
  "seats": [{"playerId": "alice", "username": "alice"}, null],
  "readyStates": [true, false],
  "status": "Open"
}

// Game started from table
{
  "type": "gameStarted",
  "tableId": "table_123",
  "gameInstanceId": "game_abc",
  "you": "p1",
  "state": {...},
  "ui": {...}
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

## Resource Management (v0.0.2+)

The server includes comprehensive resource management systems to ensure reliability under high load:

### Connection Manager

Manages WebSocket connections with automatic cleanup and resource limits:

- **Connection Pooling** - Limits connections per user (3) and globally (100)
- **Automatic Cleanup** - Removes stale connections after 5 minutes of inactivity
- **Health Monitoring** - Tracks failed sends and disconnection rates
- **RAII Guards** - Automatic resource cleanup on connection drop

```rust
// Connection limits enforced automatically
let pool = ConnectionPool::new(
    max_per_user: 3,     // Per-user connection limit
    max_total: 100       // Global connection limit
);

// Health monitoring
let monitor = HealthMonitor::new();
monitor.record_failed_send();  // Track failures
match monitor.check_health() {
    HealthStatus::Healthy => {},
    HealthStatus::Warning(msg) => log::warn!("{}", msg),
    HealthStatus::Critical(msg) => log::error!("{}", msg),
}
```

### Memory Manager

Prevents unbounded memory growth with intelligent limits:

- **Bounded Collections** - Auto-evicting collections for chat and logs
- **Table Limits** - Maximum tables per lobby (100)
- **Chat History** - Limited to 500 messages per table
- **Automatic Cleanup** - Removes idle tables after 1 hour

```rust
// Bounded collections automatically evict oldest items
let chat_messages = BoundedVec::new(500);
let action_log = BoundedHashMap::new(1000);

// Memory configuration
let config = MemoryConfig {
    max_tables_per_lobby: 100,
    max_chat_messages: 500,
    max_game_log_entries: 1000,
    table_idle_timeout: Duration::from_secs(3600),
};
```

### Lock Optimizer

Reduces lock contention for better concurrency:

- **Timeout-based Acquisition** - Prevents indefinite blocking
- **Batched Updates** - Groups multiple updates to reduce lock frequency
- **Read Caching** - Reduces read lock contention with TTL cache
- **Metrics Tracking** - Monitors lock acquisition patterns

```rust
// Optimized lock acquisition with timeout
let optimizer = LockOptimizer::new();
if let Some(guard) = optimizer.try_read_with_timeout(&data, Duration::from_millis(100)) {
    // Use data
}

// Batched updates for efficiency
let updater = BatchedUpdater::new(max_batch_size: 100);
updater.add_update(|state| {
    // Update operation
});
```

### High-Load Testing

The server includes comprehensive load testing tools:

```bash
# Basic load test (50 users, 30 seconds)
cargo test high_load_basic -- --ignored --nocapture

# Stress test (200 users, 60 seconds)
cargo test high_load_stress -- --ignored --nocapture

# Memory leak test (5 minutes)
cargo test memory_leak_test -- --ignored --nocapture
```

Load test metrics include:
- Connection success/failure rates
- Message throughput (messages/sec)
- Average latency
- Games completed
- Error rates

### Patch Optimizer

Reduces patch sizes for better network efficiency:

- **Duplicate Removal** - Eliminates redundant patches
- **Path Consolidation** - Merges related patches
- **Deep Merging** - Combines object updates intelligently
- **Metrics Tracking** - Reports optimization effectiveness

```rust
let optimizer = PatchOptimizer::new();
let optimized = optimizer.optimize_patches(patches);
// Typically reduces patch count by 30-70%
```

### Entity Pool

Manages game entities efficiently:

- **Object Pooling** - Reuses entity objects to reduce allocations
- **Pre-allocation** - Creates entities in advance for common types
- **Template Caching** - Caches entity templates for fast instantiation
- **Automatic Growth** - Expands pool as needed

```rust
let pool = EntityPool::new();
pool.pre_allocate("card", 52);  // Pre-create a deck
let entity = pool.acquire("token", &template);
// Entity automatically returned to pool when dropped
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

## HTTP API

All HTTP endpoints are prefixed with `/api`:

### Lobby Management

```bash
# Create a new lobby
POST /api/lobbies
Content-Type: application/json
{
  "name": "Friday Night Games",
  "max_members": 20,
  "private_room": false
}

# List all lobbies
GET /api/lobbies

# Get specific lobby
GET /api/lobbies/{lobby_id}

# Delete lobby
DELETE /api/lobbies/{lobby_id}
```

### Table Management

```bash
# List tables in a lobby
GET /api/lobbies/{lobby_id}/tables

# Create a new table
POST /api/lobbies/{lobby_id}/tables
Content-Type: application/json
{
  "bundleId": "tic-tac-toe",
  "minPlayers": 2,
  "maxPlayers": 2
}

# Delete a table
DELETE /api/lobbies/{lobby_id}/tables/{table_id}
```

### Seat Management

```bash
# Claim a seat
POST /api/lobbies/{lobby_id}/tables/{table_id}/seats/{seat_index}/claim
Content-Type: application/json
{
  "playerId": "alice",
  "username": "alice"
}

# Release a seat
POST /api/lobbies/{lobby_id}/tables/{table_id}/seats/{seat_index}/release

# Set ready state
POST /api/lobbies/{lobby_id}/tables/{table_id}/ready
Content-Type: application/json
{
  "playerId": "alice",
  "ready": true
}
```

### Game Information

```bash
# List available games
GET /api/games
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

### Debug Endpoints (Development Only)

The server provides comprehensive debug endpoints in development builds:

```bash
# Get debug overview
GET /api/debug
# Returns list of all lobbies, loaded games, and server version

# Get detailed lobby debug info
GET /api/debug/lobby/{lobby_id}
# Returns member list, table details, and game states

# Get raw game state
GET /api/debug/lobby/{lobby_id}/table/{table_id}/state
# Returns complete game state including:
# - zones, entities, UI data, meta
# - phase stack and bundle info

# Create test state (placeholder)
POST /api/debug/test-state
Content-Type: application/json
{
  "game_type": "tic-tac-toe",
  "scenario": "mid-game"
}
```

Example debug response:
```json
{
  "lobbies": [{
    "id": "abc123",
    "name": "Test Lobby",
    "member_count": 2,
    "table_count": 1,
    "active_games": 1
  }],
  "loaded_games": ["tic-tac-toe", "connect-four", "hex-tic-tac-toe"],
  "server_version": "0.0.1"
}
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

## Internal APIs for Testing

### Creating Lobbies Programmatically

When writing tests, you may need to create lobbies directly. The `new_lobby` function has the following signature:

```rust
pub fn new_lobby(
    id: String,
    bundle: Bundle,
    lobbies: Arc<LobbyMap>,
    lobby_updates: broadcast::Sender<Message>,
    broadcast_sender: Option<tokio::sync::mpsc::Sender<BroadcastRequest>>,
) -> Arc<Lobby>
```

**Parameters:**
- `id`: Unique identifier for the lobby
- `bundle`: The loaded game bundle
- `lobbies`: Shared map of all active lobbies
- `lobby_updates`: Broadcast channel for lobby state updates
- `broadcast_sender`: Optional channel for requesting lobby list broadcasts

Example usage in tests:
```rust
use server::{new_lobby, LobbyMap};
use tokio::sync::broadcast;

// Setup
let lobbies = Arc::new(LobbyMap::new());
let (lobby_updates, _) = broadcast::channel(100);
let bundle = bundles.get_latest("tic-tac-toe").unwrap();

// Create lobby
let lobby = new_lobby(
    "test-lobby".to_string(),
    bundle,
    lobbies.clone(),
    lobby_updates,
    None  // No broadcast sender needed for tests
);
```

### Creating Lobbies with Specific Seeds

For deterministic testing, use `new_lobby_with_seed`:

```rust
pub fn new_lobby_with_seed(
    id: String,
    bundle: Bundle,
    lobbies: Arc<LobbyMap>,
    lobby_updates: broadcast::Sender<Message>,
    broadcast_sender: Option<tokio::sync::mpsc::Sender<BroadcastRequest>>,
    seed: [u8; 32],
) -> Arc<Lobby>
```

The `seed` parameter ensures consistent random number generation for testing.

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
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bluefelt is a platform for creating and playing turn-based multiplayer games. It consists of:
- **Rust server** (`server/`) - WebSocket-based game server with WebAssembly rule enforcement
- **React web client** (`clients/react/`) - TypeScript/React frontend with real-time updates
- **Game definitions** (`games/`) - YAML-based declarative game rules
- **Game CLI** (`cli/`) - Rust CLI tool for building and validating games
- **Rust SDK** (`sdk/rust/`) - For writing WebAssembly game hooks
- **Documentation** (`docs/`) - Docusaurus documentation site

## Claude's Responsibilities

**IMPORTANT**: Claude is responsible for running both the server and client during development sessions:

### Server Management
```bash
cd server
cargo run            # Start server (port 8000)
```

### Client Management  
```bash
cd clients/react
pnpm dev             # Start dev server (port 5173)
```

### Key Responsibilities:
- **Always restart the server** after changes to game YAML files or server code
- **Build games with CLI** when YAML files change: `cd cli && cargo run -- build-all`
- **Read debug logs** directly instead of asking user to copy/paste them
- **Start both services** before asking user to test in browser
- **Reset services** when changes require it to be reflected in the browser

The reason Claude manages the servers is to read debug logs directly and ensure the latest changes are reflected when asking users to test functionality.

## Common Commands

### Server Development
```bash
cd server
cargo build          # Build server
cargo test           # Run comprehensive test suite
cargo run            # Start server (port 8000)
```

The server includes comprehensive test coverage. See `docs/docs/testing.md` for complete testing documentation.

### Client Development
```bash
cd clients/react
pnpm install         # Install dependencies
pnpm dev             # Start dev server (port 5173)
pnpm build           # Production build
pnpm lint            # Run linter
pnpm preview         # Preview production build
pnpm test            # Run tests
pnpm test:ui         # Run tests with UI
pnpm test:coverage   # Run tests with coverage
```

### Game CLI Development
```bash
cd cli
cargo build          # Build CLI tool
cargo run -- build   # Build single game from current directory
cargo run -- build-all # Build all games from games/ directory
cargo run -- validate # Validate game files
```

### Game Deployment Process
```bash
# 1. Develop games as YAML files in games/<game-name>/<version>/
# 2. Build optimized JSON bundles for server deployment
./cli/target/debug/bluefelt-cli build-all
# 3. Server automatically loads from bundles/ directory (not games/)
cd server && cargo run
```

### Documentation
```bash
cd docs
yarn start           # Start docs dev server
yarn build           # Build documentation
yarn typecheck       # TypeScript check
```

**IMPORTANT**: All documentation should now be stored in `docs/docs/` directory. The old scattered .md files have been consolidated into this central location.

If any changes are made to the code which will affect documentation, please make changes to the documentation when the changes are finalized. If the user ever says that something should be a standard, then make sure that is documented.

**State Structure (CRITICAL)**: For the canonical documentation on how game state is structured and synchronized between client and server, see `docs/docs/state-structure.md`. This is **absolutely crucial** for implementing games correctly:
- The server maintains a specific state structure that must be honored
- All new state properties MUST be initialized in `/server/src/engine/state.rs`
- Client and server communicate via JSON patches with specific path conventions
- Understanding this structure prevents patch failures and state synchronization issues
- Example: `currentPlayer` is at `lobbyState.game.currentPlayer`, action maps are at `lobbyState.ui.actionMap`

### Monorepo Commands (from root)
```bash
yarn nx affected -t lint test build    # Run affected targets
yarn nx run-many -t lint test build    # Run all targets
```

## Architecture Overview

Bluefelt is a platform for turn-based multiplayer games using declarative YAML-based game definitions.

**Key Concepts**:
- **Server**: Authoritative game engine (Rust) that loads JSON bundles and enforces rules
- **Client**: React frontend that renders zones dynamically based on server data
- **Games**: YAML definitions in `games/` compiled to JSON bundles in `bundles/`
- **Real-time sync**: WebSocket with JSON Patch for state synchronization

**Game Development Process**:
1. Write YAML files in `games/<game-name>/<version>/`
2. Build bundles: `cd cli && cargo run -- build-all`
3. Server automatically loads from `bundles/` directory

📖 **For complete documentation see**:
- [Game Implementation Guide](docs/docs/game-implementation-guide.md) - How to create games
- [State Structure](docs/docs/state-structure.md) - **CRITICAL** for understanding client-server sync
- [Server Documentation](docs/docs/bluefelt-server.md) - Server API and architecture
- [Client Development](docs/docs/building-clients.md) - React client development

### CRITICAL ARCHITECTURAL PRINCIPLES

**NEVER ADD GAME-SPECIFIC UI CODE TO THE CLIENT**

The Bluefelt engine is designed to be a generalizable platform for ANY turn-based game. This means:

1. **NO hardcoded game-specific UI** - The client should NEVER have code like "if gameId === 'go-fish'" or hardcoded rank selection UI
2. **Data-driven rendering** - ALL game UI must be rendered based on data from the server (zones, action maps, entity definitions)
3. **Generic components only** - Client components should be generic (Board, CardZone, ChoiceZone) that work for ANY game
4. **Server defines everything** - The server sends all necessary data for the client to render the game properly

If you find yourself wanting to add game-specific UI code, STOP and ask:
- What data is missing from the server?
- How can we make the server send the right data?
- How can we enhance our generic components to handle this case?

Adding hardcoded UI is a shortcut that undermines the entire architecture. It's better to enhance the engine's capabilities than to add game-specific hacks.

### Server API Reference
**IMPORTANT**: All HTTP API routes are prefixed with `/api`

#### Create Lobby
```bash
POST /api/lobbies
Content-Type: application/json

{
  "game_id": "go-fish"  # Note: Use game_id (with underscore), not gameId
}

# Response: {"game_id": "go-fish", "id": "0c07fef62b"}
```

#### List Lobbies
```bash
GET /api/lobbies
```

#### Get Lobby Info
```bash
GET /api/lobbies/{lobby_id}
```

#### WebSocket Connection
```
ws://localhost:8000/api/lobbies/{lobby_id}/ws?player={player_name}&join=true
```

#### Game Flow
1. Create lobby via POST `/api/lobbies`
2. Players connect via WebSocket
3. **IMPORTANT**: Games do NOT auto-start when minimum players join
4. To start a game, send WebSocket message: `{"action": "start_game"}`
5. Server processes phases and sends patches to clients

#### Common Mistakes to Avoid
- ❌ Using `/lobbies` instead of `/api/lobbies`
- ❌ Using `gameId` instead of `game_id` in JSON
- ❌ Expecting games to auto-start (they require explicit start)
- ❌ Forgetting that patches are for client sync only (state is already mutated server-side)

📖 **For zone implementation details see**: [Developing Games - Zones](docs/docs/developing-games-zones.md)

📖 **For game development details see**:
- [Actions](docs/docs/developing-games-actions.md) - Action definitions, verbs, and triggers
- [Entities](docs/docs/developing-games-entities.md) - Entity configuration and shorthand
- [Phases](docs/docs/developing-games-phases.md) - Game flow and phase transitions
- [Conditions](docs/docs/engine-enhancements-conditions.md) - Advanced condition system
- [Custom Hooks](docs/docs/developing-games-custom-hooks.md) - WebAssembly extensions

### Implementing New Game Features

📖 **CRITICAL**: Read [State Structure](docs/docs/state-structure.md) first to understand the state contract.

**Key Rules**:
1. Initialize new state properties in `/server/src/engine/state.rs`
2. Use correct patch paths: `/game/...` for game state, `/ui/...` for UI data
3. Test patches thoroughly with both empty and populated states

📖 **For future enhancements see**: [Future Development Roadmap](docs/docs/future-development-roadmap.md)

## Testing

📖 **See**: [Testing Guide](docs/docs/testing.md) for complete documentation including three-layer architecture, workflow, and debugging.

**Quick Commands**:
```bash
# Server tests
cd server && cargo test
cd server && node tests/regression/run-all-tests.js  # (server must be running)

# Client tests  
cd clients/react && pnpm test
```


# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
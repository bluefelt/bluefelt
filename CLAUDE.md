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

## Common Commands

### Server Development
```bash
cd server
cargo build          # Build server
cargo test           # Run tests (comprehensive test suite with 41 tests)
cargo run            # Start server (port 8000)
```

The server includes comprehensive test coverage:
- **Unit tests** (18) - engine verbs, shorthand expansion, path navigation, grid.lineOfMarks functionality
- **Integration tests** (8) - full game simulation, bundle loading
- **WebSocket tests** (8) - connection handling, protocol testing

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

**State Structure**: For the canonical documentation on how game state is structured and synchronized between client and server, see `docs/docs/state-structure.md`. This is crucial for avoiding confusion about where to find game data (e.g., `currentPlayer` is at `lobbyState.game.currentPlayer`, action maps are at `lobbyState.ui.actionMap`).

### Monorepo Commands (from root)
```bash
yarn nx affected -t lint test build    # Run affected targets
yarn nx run-many -t lint test build    # Run all targets
```

## Architecture Overview

### Game Engine
Games are developed using YAML files in `games/<game-name>/<version>/`:
- `manifest.yaml` - Game metadata - **IMPORTANT: Required fields are gameId, version, specVersion, and metadata (with name, description, author, players)**
- `entities.yaml` - Game rules including:
  - Entities (pieces, cards, tokens)
  - Zones (areas where entities exist)
  - Actions (player actions)
  - Phases (game flow)
  - Setup (initial state)
  - Hooks (WebAssembly functions)

**Deployment**: Games are built from YAML to optimized JSON bundles using `bluefelt-cli build-all`. The server loads from `bundles/` directory, not directly from `games/`.

### Available Games
The platform currently includes these games:
- **Tic-tac-toe** - Classic 3x3 grid game with win/tie detection
- **Checkers** - Traditional board game with piece movement
- **Reversi** - Disc-flipping strategy game
- **Gomoku** - Five-in-a-row stone placement game
- **Stone Age** - Worker placement Euro game
- **Gin Rummy** - Card melding and knocking game
- **Crazy Eights** - Card shedding game
- **Connect-4** - Vertical four-in-a-row game
- **Go Fish** - Card collecting game

### State Synchronization
- Server maintains authoritative game state
- Client-server communication via WebSocket (port 8000)
- State updates sent as JSON Patches
- Client applies patches to maintain synchronized state
- Real-time multiplayer with deterministic replay capability

### Key Technologies
- **Server**: Rust, Axum, Tokio, Wasmtime (binary: `bluefelt-core`)
- **Client**: React, TypeScript, Vite, TanStack Query, Zustand, Tailwind CSS
- **Real-time**: WebSocket with JSON Patch
- **Game Logic**: WebAssembly modules for complex rules
- **Testing**: Comprehensive test suite with unit, integration, and WebSocket tests

### Interactive Zones
- Server provides an action map that directly maps locations to available actions
- Action map structure: `{"/zones/board/0/0": {"action": "place", "direction": "Choose a cell"}, ...}`
- Location paths follow a consistent format:
  - Grid zones: `/zones/{zoneId}/{row}/{col}`
  - List/deck zones: `/zones/{zoneId}/{index}`
  - Whole zones: `/zones/{zoneId}`
- Client renders zones as interactive components where players can click to execute actions
- UI directions are defined in the game's `actions.yaml` under `ui.direction`
- Supports multiple zones and actions per game without hardcoding assumptions

### Custom Hooks and Game End Detection
- Actions can trigger other actions using the `triggers` field in `actions.yaml`
- Custom hooks can be defined with `hook: hookName` and `auto: true` for automatic actions
- Game end detection for tic-tac-toe checks for wins (3 in a row) and ties (board full)
- Game status stored in `/meta/gameStatus` with fields: `state`, `winner`, `tie`
- When game ends, turn advancement is prevented and UI shows game result
- WebAssembly hooks can be compiled and placed as `hooks.wasm` in game directory

### Built-in Verbs and Shorthand
The engine supports built-in verbs for common game operations:
- `draw` - Move entities from deck to hand/zone
- `place` - Place entities on grid locations
- `moveEntity` - Move entities between zones or locations
- `nextTurn` - Advance turn and update game state
- `setPhase` - Change game phase
- `grid.lineOfMarks` - Check for winning lines on grid zones (configurable)

The `grid.lineOfMarks` verb provides flexible line detection for multiple game types:
- **Parameters**: `zone` (grid path), `entity` (pattern like "mark_{player}"), `lineLength` (default 3), `directions` (array: ["horizontal", "vertical", "diagonal"])
- **Usage**: Suitable for tic-tac-toe (3-in-a-row), Connect-4 (4-in-a-row), Gomoku (5-in-a-row), etc.
- **Actions**: Sets `gameStatus` to `{state: "ended", winner: "p1", tie: false}` on win, or `{state: "ended", winner: null, tie: true}` on tie
- **Entity Pattern**: Use `"mark_{player}"` to match any player's entities (e.g., "mark_p1", "mark_p2")

Shorthand syntax includes:
- `{player}` replacement in entity IDs and names
- `standardDeck` for 52-card deck generation
- `cards.deal` for distributing cards to players
- `cards.reveal` for revealing cards

## Testing Infrastructure

The project has comprehensive test coverage following these patterns:
- **Pure unit tests** - Individual verb functions and parsing helpers
- **Engine integration tests** - Complete game simulations with tic-tac-toe
- **WebSocket harness** - Real connection testing with handshake and protocol validation
- **Deterministic replay** - State mutations via structured verb application
- **Error scenario coverage** - Invalid moves, malformed data, edge cases

Run tests with: `cargo test` (all 41 tests should pass)

## Client Testing Infrastructure

The React client includes a focused testing suite to validate critical functionality:

### Test Categories
- **Core functionality tests** - Key game logic validation
- **WebSocket tests** - Message handling scenarios
- **Component unit tests** - Individual component behavior

### Key Test Files
- `src/__tests__/TicTacToeGameFlow.simplified.test.tsx` - Core game logic validation

### Critical Behaviors Validated
- ✓ Current player detection from `lobbyState.game.currentPlayer` with proper UI/game data separation
- ✓ JSON Patch application with partial failure handling
- ✓ Action map path format parsing (`/zones/board/cells/{row}/{col}`)
- ✓ Turn determination logic for different player scenarios
- ✓ Action message construction for server communication
- ✓ Game state updates and transitions
- ✓ Game end detection (win/tie scenarios)

### Test Setup
Tests use Vitest with jsdom environment. The simplified test suite focuses on validating the core logic that was debugged and fixed without complex UI mocking.

The test suite prioritizes testing business logic over UI interactions to maintain stability and focus on preventing regressions in the core game functionality.

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
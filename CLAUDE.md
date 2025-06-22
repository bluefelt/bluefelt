# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bluefelt is a platform for creating and playing turn-based multiplayer games. It consists of:
- **Rust server** (`server/`) - WebSocket-based game server with comprehensive rule enforcement
- **React web client** (`clients/react/`) - TypeScript/React frontend with real-time updates
- **Game definitions** (`games/`) - YAML-based declarative game rules
- **Game CLI** (`cli/`) - Rust CLI tool for building and validating games
- **Documentation** (`docs/`) - Docusaurus documentation site

## Log File Management

**IMPORTANT**: All log files should be placed in the `/logs` directory with appropriate subdirectories:

- `/logs/server/` - Server runtime logs
- `/logs/client/` - Client development logs  
- `/logs/tests/` - Test execution logs
- `/logs/build/` - Build process logs

When running commands that generate output, always redirect to the appropriate log directory:
```bash
# Server logs
cd server && cargo run > ../logs/server/server_$(date +%Y%m%d_%H%M%S).log 2>&1

# Client logs
cd clients/react && pnpm dev > ../../logs/client/client_dev_$(date +%Y%m%d_%H%M%S).log 2>&1

# Test logs
cd server && cargo test > ../logs/tests/server_test_$(date +%Y%m%d_%H%M%S).log 2>&1

# Build logs
cd cli && cargo run -- build-all > ../logs/build/bundle_build_$(date +%Y%m%d_%H%M%S).log 2>&1
```

All log files are ignored by git. See `/logs/README.md` for naming conventions.

## Test File Organization

**IMPORTANT**: Test files should be organized in their appropriate directories:

### Automated Tests
- **Rust tests**: `server/tests/` - Use `cargo test` to run
- **React tests**: `clients/react/src/__tests__/` - Use `pnpm test` to run
- **E2E tests**: `server/tests/regression/` - JavaScript tests that test full game flows

### Manual Test Scripts
- **Location**: `server/tests/manual/`
- **JS scripts**: `server/tests/manual/js-scripts/` - Node.js scripts for debugging
- **Shell scripts**: `server/tests/manual/` - Bash scripts for testing scenarios

### CLI Test Files
- **YAML tests**: `cli/test-yaml-includes/` - Test YAML files for CLI features
- **Test output**: `cli/test-output/` - Generated test bundles (git ignored)

**Never place test files in the root directory.** All test files should be in their designated locations.

## Claude's Responsibilities

**IMPORTANT**: Claude is responsible for running both the server and client during development sessions:

### Server Management
```bash
cd server
cargo run            # Start server (port 8000)
# For debugging with logs:
cargo run > ../logs/server/server_$(date +%Y%m%d_%H%M%S).log 2>&1
```

### Client Management  
```bash
cd clients/react
pnpm dev             # Start dev server (port 5173)
# For debugging with logs:
pnpm dev > ../../logs/client/client_dev_$(date +%Y%m%d_%H%M%S).log 2>&1
```

### Key Responsibilities:
- **Always restart the server** after changes to game YAML files or server code
- **Build games with CLI** when YAML files change: `cd cli && cargo run -- build-all`
- **Read debug logs** directly instead of asking user to copy/paste them
- **Start both services** before asking user to test in browser
- **Reset services** when changes require it to be reflected in the browser

The reason Claude manages the servers is to read debug logs directly and ensure the latest changes are reflected when asking users to test functionality.

## Server and Client Process Management

### When to Restart the Server

**ALWAYS restart the server after ANY of these changes:**
- ✅ Changes to Rust code in `server/src/`
- ✅ Changes to game YAML files in `games/`
- ✅ After building games with the CLI (`cargo run -- build-all`)
- ✅ Changes to `Cargo.toml` dependencies
- ✅ When switching between different games for testing

**The server does NOT hot-reload. Every change requires a restart.**

### How to Properly Restart the Server

1. **Stop the current server process:**
   ```bash
   # First, try Ctrl+C in the terminal running cargo run
   # If that doesn't work, find and kill the process:
   lsof -ti:8000 | xargs -r kill -9
   ```

2. **Verify the port is free:**
   ```bash
   lsof -i:8000  # Should return nothing
   ```

3. **Start the server again:**
   ```bash
   cd server
   cargo run
   # Or with logging:
   cargo run > ../logs/server/server_$(date +%Y%m%d_%H%M%S).log 2>&1
   ```

4. **Wait for the startup message:**
   ```
   Starting server on 0.0.0.0:8000
   ```

### When to Restart the Client

**The React client has hot-reload, but restart it after:**
- ✅ Changes to `package.json` dependencies
- ✅ Changes to environment variables
- ✅ Changes to Vite configuration
- ✅ If hot-reload seems broken or inconsistent

**The client usually does NOT need restart for:**
- ❌ Changes to React components
- ❌ Changes to CSS
- ❌ Changes to TypeScript files

### How to Properly Restart the Client

1. **Stop the current dev server:**
   ```bash
   # Ctrl+C in the terminal running pnpm dev
   # If that doesn't work:
   lsof -ti:5173 | xargs -r kill -9
   ```

2. **Start the client again:**
   ```bash
   cd clients/react
   pnpm dev
   ```

3. **For WSL2 users, use host flag:**
   ```bash
   pnpm dev --host
   ```

### Common Process Management Issues

**"Address already in use" error:**
```bash
# Find and kill the process using the port
lsof -ti:8000 | xargs -r kill -9  # For server
lsof -ti:5173 | xargs -r kill -9  # For client
```

**Server changes not reflected:**
- You forgot to restart the server
- You forgot to build games after YAML changes
- The old process is still running on the port

**Can't connect to localhost in WSL2:**
```bash
# Start client with host flag
cd clients/react
pnpm dev --host

# Use the WSL2 IP address shown in the output
```

### Best Practices for Claude

1. **Always check if services are running before testing:**
   ```bash
   lsof -i:8000  # Should show cargo/bluefelt-core
   lsof -i:5173  # Should show node/vite
   ```

2. **After making changes, follow this sequence:**
   - Build games if YAML changed: `cd cli && cargo run -- build-all`
   - Restart server if needed: Kill process → Start server
   - **Run tests for affected functionality** (see Testing Requirements below)
   - Refresh browser or restart client if needed
   - Clear browser cache if seeing stale data

3. **Monitor both consoles:**
   - Server console shows WebSocket connections, game actions, errors
   - Client console shows React errors, WebSocket messages, patch failures

4. **When in doubt, restart both:**
   - It's better to restart unnecessarily than to debug phantom issues
   - A full restart takes less than 30 seconds

### Testing Requirements (CRITICAL)

**NEVER ask the user to test functionality if there are failing tests for that area.**

After making changes, you MUST run relevant tests:

1. **Server Changes**:
   ```bash
   cd server
   cargo test                                    # Run all server tests
   cargo test tic_tac_toe                       # Run specific game tests
   cargo test websocket                         # Run WebSocket tests
   cargo test action_map                        # Run action map tests
   ```

2. **Client Changes**:
   ```bash
   cd clients/react
   pnpm test                                    # Run all client tests
   pnpm test TicTacToe                         # Run Tic-Tac-Toe specific tests
   pnpm test Animation                         # Run animation tests
   pnpm test GameView                          # Run game view tests
   pnpm test useLobbyWebSocket                 # Run WebSocket hook tests
   ```

3. **Game Definition Changes**:
   ```bash
   # After changing YAML files
   cd cli && cargo run -- build-all            # Build games
   cd ../server && cargo test                  # Test server with new definitions
   cd ../server && node tests/regression/games/test-tic-tac-toe.js  # E2E tests
   ```

4. **Common Test Commands**:
   ```bash
   # Run all E2E regression tests (server must be running)
   cd server && node tests/regression/run-all-tests.js
   
   # Run specific game E2E test
   cd server && node tests/regression/games/test-tic-tac-toe.js
   
   # Run client tests with coverage
   cd clients/react && pnpm test:coverage
   ```

**Test Categories by Feature**:
- **Animation Changes**: Run AnimationEngine, PatchAnalyzer, AnimationIntegration tests
- **Game State Changes**: Run StateSynchronization, ClientRequestGeneration tests  
- **WebSocket Changes**: Run useLobbyWebSocket, WebSocketMessageHandling tests
- **UI Changes**: Run GameView, GameZones, Board, UIAffordances tests
- **Specific Game Changes**: Run that game's complete test suite

### Troubleshooting Process Issues

**"I made changes but they're not showing up"**
1. Did you restart the server after Rust code changes?
2. Did you build games after YAML changes?
3. Is the old server process still running?
4. Check: `lsof -i:8000` - is it the new or old process?

**"The user says the old lobby is still visible"**
- This means the server wasn't properly restarted
- The WebSocket connections are still alive to the old process
- Solution: Force kill and restart: `lsof -ti:8000 | xargs -r kill -9`

**"Tests pass but the game doesn't work in browser"**
1. Server might not be running
2. Client might not be running
3. Browser might have cached old JavaScript
4. Server might be running old code (needs restart)

**"WebSocket connection fails"**
1. Check server is running: `lsof -i:8000`
2. Check client is running: `lsof -i:5173`
3. For WSL2: Use `--host` flag and WSL2 IP address
4. Check browser console for CORS errors

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

**CRITICAL**: The [Development Roadmap](docs/docs/development-roadmap.md) is the ONLY authoritative source for development tasks and future plans. DO NOT create new .md files for refactoring plans, roadmap items, or todo lists. All such items should be added to the development roadmap instead.

## Documentation Philosophy (January 2025)

The Bluefelt documentation follows these principles to maintain clarity and usability:

### 1. Hub Document Pattern
Major topics have an overview "hub" document that links to detailed sub-documents:
- **Server**: `bluefelt-server.md` → links to `state-structure.md`, `game-log-parameters.md`
- **Game Development**: `developing-games.md` → links to actions, entities, zones, phases, UI sub-docs
- **Testing**: Single comprehensive `testing.md` (consolidated from multiple docs)

This allows AI agents and developers to quickly find the right level of detail without loading unnecessary content.

### 2. Consolidation Over Proliferation
- Multiple documents on the same topic should be merged (e.g., all testing docs → one testing guide)
- System-specific documents should be integrated into broader categories (e.g., hex grid → zones doc)
- Research and analysis documents should be removed once their insights are incorporated

### 3. Clear Document Categories
Documents are organized into logical groups:
- **Core Platform**: Server, client, architecture documents
- **Game Development**: Overview + detailed sub-documents for each aspect
- **Development Process**: Testing, roadmap, business strategy

### 4. No Redundancy
- Each concept should be documented in exactly one place
- Cross-references should link to the authoritative source
- Outdated or superseded content should be removed entirely

### 5. Scratchpad for Loose Notes
- Use `scratchpad.md` for speculative ideas, research notes, or information that doesn't fit elsewhere
- Clearly mark it as non-authoritative
- Periodically review and either integrate valuable content or remove outdated notes

### 6. Sidebar Organization
Use Docusaurus categories to group related documents:
```typescript
{
  type: 'category',
  label: 'Topic Name',
  link: {
    type: 'doc',
    id: 'hub-document-id',
  },
  items: ['sub-doc-1', 'sub-doc-2'],
}
```

This creates a collapsible section with the hub document as the category page.

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

**Code Organization** (Updated Jan 2025):
- Server code is modularized: `lobby/` and `engine/verbs/` split into focused modules
- All log files and ad-hoc test scripts have been cleaned up
- Tests are organized in `server/tests/regression/` with proper framework
- See [Code Organization](docs/docs/code-organization.md) for detailed structure

📖 **For complete documentation see**:
- [Game Implementation Guide](docs/docs/game-implementation-guide.md) - How to create games
- [State Structure](docs/docs/state-structure.md) - **CRITICAL** for understanding client-server sync
- [Server Documentation](docs/docs/bluefelt-server.md) - Server API and architecture
- [Client Development](docs/docs/building-clients.md) - React client development
- [Testing Strategy](docs/docs/testing-strategy.md) - Comprehensive testing guidelines
- [Code Organization](docs/docs/code-organization.md) - Module structure and best practices

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

Adding hardcoded UI is a shortcut that undermines the entire architecture. 

Improving the engine when we encounter limitations is not just good practice, it's the whole point. We implement new games because it introduces new situations the engine must handle, or provides an opportunity to abstract functionality to be handled in a more general way. 

This is the primary development loop:
1. Attempt to implement a new game authentically based on its mechanics/rules
2. Realize limitations in the engine or client
3. Improve the engine or client to handle those limitations in a general way
4. Repeat

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

#### Game Flow (Table System)
1. Create lobby via POST `/api/lobbies` 
2. Players connect via WebSocket
3. Create table: `{"action": "createTable", "bundleId": "tic-tac-toe"}`
4. Players join table: `{"action": "joinTable", "tableId": "table_id"}` (auto-seat assignment)
5. Start game: `{"action": "start_game", "tableId": "table_id"}`
6. Server processes phases and sends patches to clients

**Note**: The table system supports multiple concurrent games per lobby with automatic seat assignment.

#### Common Mistakes to Avoid
- ❌ Using `/lobbies` instead of `/api/lobbies`
- ❌ Using `gameId` instead of `bundleId` in table creation
- ❌ Trying to join specific seats (use `joinTable` for auto-assignment)
- ❌ Expecting games to auto-start (they require explicit `start_game` action)
- ❌ Forgetting that patches are for client sync only (state is already mutated server-side)

#### Debug Endpoints (Development Only)
When debugging game state or lobby issues, use these endpoints:
- `GET /api/debug` - Overview of all lobbies and loaded games
- `GET /api/debug/lobby/{lobby_id}` - Detailed lobby information
- `GET /api/debug/lobby/{lobby_id}/table/{table_id}/state` - Raw game state

These are only available in debug builds and provide comprehensive state inspection.

### Critical Development Notes (Phase A & B1 Lessons)

**🔴 CRITICAL BUG PATTERNS TO AVOID:**

1. **Zone Path Errors**: Always use `/zones/board` (with leading slash), not `zones/board`
   - Incorrect: `"zone": "zones/board"`
   - Correct: `"zone": "/zones/board"`
   - **Impact**: Win detection fails silently

2. **Wrong Win Condition**: Use `grid.lineOfMarks`, not `zone.hasLine`
   - Incorrect: `"condition": "zone.hasLine"`
   - Correct: `"condition": "grid.lineOfMarks"`
   - **Impact**: Games end immediately or never detect wins

3. **Server Restart Required**: Server does NOT hot-reload game bundles
   - After `cli build-all`, always restart server
   - Bundle changes only take effect after server restart
   - **Impact**: Testing with stale game logic

4. **Test Framework Compatibility**: E2E tests require table system API
   - Use `createTable`/`joinTable`/`start_game` in tests
   - Auto-seat assignment means no manual seat selection
   - **Impact**: All E2E tests fail with old API

**🔧 TABLE SYSTEM IMPLEMENTATION DETAILS (Phase B1):**

5. **Table Owner Auto-Seating**: `TableInstance::new_simple()` automatically seats table creator
   - Table creator is automatically placed in `seats[0]` when creating table
   - **Do NOT** try to have table creator join again with `joinTable`
   - Only other players should use `joinTable` for auto-seat assignment
   - **Impact**: "Player already has a seat at this table" error if creator tries to join

6. **Game Action Message Flow**: Actions use `gameAction` with table ID
   - Send: `{"action": "gameAction", "gameId": table.id, "data": {"action": "placeMark", ...}}`
   - Response: `{"type": "gameUpdate", "gameInstanceId": "table_id", "ui": {"actionMap": ...}}`
   - Note: `gameId` in request corresponds to `gameInstanceId` in response
   - **Impact**: Action execution fails if wrong table ID used

7. **WebSocket Response Handling**: Test framework needs proper Promise chain management
   - **Wrong**: `return new Promise((resolve) => { return anotherPromise(); })`
   - **Correct**: `return await anotherPromise();`
   - Event listeners must be added/removed properly to avoid hanging
   - **Impact**: Tests hang indefinitely waiting for responses

8. **Table Status Updates**: Lobby state must be manually updated for test framework
   - `gameStarted` message contains `gameInstanceId` (not `tableId`)
   - Test framework must update `lobbyState.tables[].status = 'Playing'` manually
   - Server doesn't send automatic table status updates for games
   - **Impact**: Action execution fails to find "Playing" tables

## PHASE B1 HOLISTIC TEST ANALYSIS (January 2025)

After completing the test framework updates and running comprehensive E2E tests on all games, several critical patterns of failure have emerged:

### 🔴 CRITICAL: Game Logic Implementation Issues

**Universal Problem**: Game end detection and action map cleanup are fundamentally broken across ALL games:
- **Tic-tac-toe**: Win detection not working, game continues after victory, action maps not cleared
- **Connect Four**: Player slot mappings failing to establish during game start
- **Three Men's Morris**: Movement phase selection logic missing pieces (0/3 expected selection actions)
- **Go Fish**: Multi-step action system completely non-functional (askForCards action missing)

### 🟡 HIGH: Server-Side Game Engine Problems

**Action Map Generation Timing**:
- Action maps are not being cleared when games end (tic-tac-toe continues after win)
- Phase transitions not triggering proper action map updates
- Win condition detection not stopping action generation
- Game status (`gameStatus.state`, `gameStatus.winner`) not being set properly

**Player State Management**:
- Player slot mapping inconsistencies between client and server (Connect Four)
- CurrentPlayer assignments failing or not persisting properly
- Player identity resolution breaking during game transitions

**Game State Synchronization**:
- State patches not reflecting actual game completion
- Client-server state divergence during critical game moments
- Patches show moves being made but game state doesn't update accordingly

### 🟡 HIGH: Game-Specific Implementation Gaps

**Multi-Step Actions (Go Fish)**:
- The `askForCards` action system is completely missing or broken
- No rank selection UI being generated in `selectingRank` phase
- Multi-step state management not working at all

**Movement Phases (Three Men's Morris)**:
- Piece selection in movement phase not generating correct actions
- Expected 3 selection actions for player pieces, getting 0
- Phase transitions from placement to movement not working correctly

**Gravity Systems (Connect Four)**:
- Player slot mapping failures preventing game start
- "Failed to establish player slot mappings" error consistently

### 🟢 MEDIUM: Test Framework Reliability

The test framework itself is now working correctly after Phase B1 fixes, but it reveals that the underlying game implementations have significant gaps that prevent proper gameplay.

### Strategic Assessment

**Root Cause**: The issue is NOT with the table system or WebSocket communication (those are working correctly). The core game engine logic for handling game completion, action map generation, and complex game mechanics is incomplete or broken.

**Impact**: This means the platform, while technically functional for basic interactions, is not ready for production use with actual gameplay. Players can connect, create tables, start games, but the games themselves don't work properly.

**Next Steps**: Phase B1 identified that the real work needed is in Phase B3 (Individual Game Quality) - specifically implementing proper:
1. Win detection and game end logic across all games
2. Action map cleanup when games end  
3. Multi-step action systems (Go Fish)
4. Phase transition management (Three Men's Morris)
5. Player state consistency and mapping (Connect Four)

The test framework is now a reliable diagnostic tool that clearly shows these game engine issues need to be addressed before the platform can be considered stable.

## CRITICAL PHASE B3 DISCOVERY: Test Framework Isolation Issue (January 2025)

After extensive debugging of "failing" win detection, a crucial discovery was made:

### The False Positive: Win Detection Works Perfectly

**Initial Symptoms**:
- Tests reported: "Game should have ended, but action maps still exist"
- Tests showed: `gameStatus: null, gameEnded: false` after winning moves
- Multiple games appeared to have broken win detection

**Root Cause Analysis**:
Through careful debugging, discovered that **win detection is working perfectly**. The issue was entirely in the test framework design:

1. **Test Scenario Isolation Failure**:
   - Test methods like `testActionMaps()` run multiple scenarios sequentially
   - Each scenario uses `testScenario()` which resets test framework state
   - BUT scenarios continue using the **same server game instance**
   - Result: Later scenarios inherit state from earlier ones

2. **Evidence of the Issue**:
   ```
   Scenario 1: "Initial Action Map" - starts at tick 0 ✅
   Scenario 2: "Action Map After Move" - continues from tick 1 ✅
   Scenario 3: "Action Map When Game Ends" - tries to play winning sequence from tick 2 ❌
   ```

3. **Why Standalone Tests Work**:
   - Individual debug tests create fresh game instances
   - Each test is properly isolated
   - Win detection triggers correctly: `gameStatus: {"state":"ended","winner":"p1"}`

### Key Insights

1. **Platform Works Correctly**: 
   - Game end detection ✅
   - Action map cleanup ✅
   - Win condition evaluation ✅
   - State synchronization ✅

2. **Test Framework Design Flaw**:
   - Scenarios within a test method share game state
   - State accumulates across scenarios
   - Later scenarios fail due to polluted state

3. **Actual Issues to Address**:
   - **Connect Four**: Player slot mapping (unrelated to win detection)
   - **Three Men's Morris**: Movement phase action generation
   - **Go Fish**: Multi-step action system missing

### Test Framework Fix Required

Each test scenario must:
1. Create a completely new lobby
2. Connect fresh player instances
3. Start a brand new game
4. Run its test in isolation
5. Clean up completely before next scenario

This discovery shows that hours of debugging "broken" win detection were actually revealing a test framework design issue, not platform bugs. The platform's core game mechanics are solid.

## Phase B3 Game-Specific Issues (January 2025)

After fixing the test isolation issue, the real game-specific problems were identified:

### ✅ Connect Four: Fixed
1. **Player mapping issue**: Fixed by adding `onGameStarted` handler to initialize board from game state
2. **Win detection bug**: CLI build incorrectly converted conditionalAction. Manually fixed bundle JSON

### ✅ Three Men's Morris: Partially Fixed  
1. **Phase detection**: Fixed test to check both `/game/phases/game` and `/phases/game` paths
2. **Multi-step actions**: Still broken - action map generation needs fix (see below)

### 🔴 Go Fish: Multi-Step Actions Broken
The askForCards multi-step action is not working due to action map generation issue.

### Root Cause: Multi-Step Action Map Generation

The action map computation in `/server/src/lobby/action_map.rs` was generating regular cell-based entries for multi-step actions instead of the special format required:

**Expected format** (per documentation):
```json
{
  "p1": {
    "_multiStep_movePiece": {
      "action": "movePiece",
      "type": "multiStep",
      "direction": "Move one of your pieces",
      "args": {}
    }
  }
}
```

**What was generated** (incorrect):
```json
{
  "p1": {
    "/zones/board/cells/0/0": { "action": "movePiece", "args": {} },
    "/zones/board/cells/0/1": { "action": "movePiece", "args": {} },
    // ... one entry per piece location
  }
}
```

**Fix Applied**:
1. Updated action_map.rs to detect `isMultiStep: true` actions
2. Generate single entry with key `_multiStep_{actionId}`
3. Include `type: "multiStep"` in the action map entry
4. Added condition checking to respect phase requirements

## Phase B2: Connection Management Optimization (January 2025)

Implemented comprehensive connection management improvements to address production stability issues:

### CLOSE_WAIT Connection Fix
- Properly close WebSocket connections by removing the reunite attempt
- Let WebSocket halves drop naturally from their respective tasks
- This prevents CLOSE_WAIT state accumulation on the server

### Heartbeat/Ping Mechanism
- Send ping every 30 seconds to detect dead connections
- Close connection if no pong received within 60 seconds
- Update activity timestamps on successful pong responses
- Prevents zombie connections from consuming resources

### Connection Pooling
- Implemented ConnectionPool with rate limiting:
  - Max 3 connections per user
  - Max 100 total connections
  - RAII guards ensure proper cleanup
- Prevents resource exhaustion from connection spam

### Reconnection Support
- Generate UUID-based reconnection tokens on disconnect
- Tokens valid for 5 minutes (configurable)
- Tokens sent to client before connection closes
- Reconnection bypasses connection limits
- Automatic cleanup of expired tokens

### Implementation Details
- ConnectionManager tracks all active connections
- Background cleanup task runs every 60 seconds
- Stale connections purged after 5 minutes of inactivity
- Proper cleanup on disconnect preserves member state
- Health monitoring with metrics for failed sends and disconnections

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

📖 **For future enhancements see**: [Development Roadmap](docs/docs/development-roadmap.md) - The authoritative source for all development tasks and plans

## Testing

📖 **See**: [Testing Guide](docs/docs/testing.md) and [Testing Strategy](docs/docs/testing-strategy.md) for complete documentation.

**Quick Commands**:
```bash
# Server tests
cd server && cargo test

# E2E regression tests (server must be running)
cd server && node tests/regression/run-all-tests.js

# Client tests  
cd clients/react && pnpm test

# UI component stress testing (development mode)
# Visit http://localhost:5173/ui-test
```

**Test Organization**:
- Unit tests: Colocated with code
- Integration tests: `server/tests/` and `clients/react/src/__tests__/`
- E2E regression tests: `server/tests/regression/games/`
- UI stress tests: Available at `/ui-test` in development


# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
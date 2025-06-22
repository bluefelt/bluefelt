# V2 Architecture Migration Status

## Overview

The V2 architecture separates lobby state from game state, allowing players to remain connected to a persistent lobby across multiple games. This document tracks the migration progress from V1 to V2.

## Current Status (2025-06-20)

### ✅ Completed

1. **Core V2 Infrastructure**
   - Created parallel V2 system alongside V1 (no breaking changes)
   - Implemented `LobbyState` (persistent) and `GameInstance` (temporary) separation
   - Created V2 HTTP endpoints under `/api/v2/`
   - Implemented V2 WebSocket handler with lobby persistence

2. **State Structure Simplification**
   - Simplified `gameStatus` from nested object to single string
   - Moved `you` property inside game object
   - Removed redundant `turn` property
   - Made observers a derived state (not stored)

3. **V2 Endpoints**
   - `POST /api/v2/lobbies` - Create persistent lobbies
   - `GET /api/v2/lobbies` - List all V2 lobbies  
   - `GET /api/v2/lobbies/{id}` - Get specific lobby info
   - `POST /api/v2/lobbies/{id}/games` - Create games within lobbies
   - `DELETE /api/v2/lobbies/{id}` - Delete lobbies
   - `ws://localhost:8000/api/v2/lobbies/{id}/ws?player={name}` - WebSocket connection

4. **Basic Game Flow**
   - Players can create/join lobbies
   - Games can be created within lobbies
   - Games can be started with initial state
   - Players remain connected after games end

### ✅ Completed

1. **Message Filtering** ✅ IMPROVED
   - `should_send_to_user` function now checks game participation
   - Players only receive messages for games they're in
   - Observers properly filtered from game messages

2. **Entity-Centric UI** ✅ PARTIALLY IMPLEMENTED
   - Core structure implemented in `engine/entity_ui.rs`
   - Helper functions implemented (player checking, phase checking, conditions, filters)
   - Being called in V2 WebSocket handler

3. **V2 Game Creation and Joining** ✅ IMPLEMENTED
   - Games can be created within lobbies
   - Players can join games with proper slot assignment
   - Game state initialization works with V2 structure
   - Tictactoe-vtwo bundle created and loaded

### ✅ Recently Fixed

1. **Server Performance Issue** ✅ FIXED
   - Server was becoming unresponsive when processing game start
   - Issue was in action map computation expecting V1 state structure
   - Created V2-specific action map computation (action_map_v2.rs)
   - V2 games now process action maps without infinite loops
   - Successfully handles simplified gameStatus string format

2. **V2-Specific Action Map** ✅ IMPLEMENTED
   - Created dedicated `compute_action_map_v2` function
   - Handles simplified V2 state structure (string gameStatus)
   - Generates basic place actions for tic-tac-toe
   - No longer causes server hangs

### 🚧 In Progress

1. **Game Action Processing**
   - Need to test actual gameplay with V2 system
   - Verify action execution and state updates
   - Ensure patches are generated correctly

### ❌ Not Started

1. **Game Migrations**
   - Migrate `tic-tac-toe` to V2 format
   - Migrate `three-mens-morris` to V2 format
   - Migrate `connect-four` to V2 format
   - Migrate `go-fish` to V2 format

2. **Client Updates**
   - Update React client to use V2 endpoints
   - Implement lobby UI separate from game UI
   - Handle persistent connections across games

3. **V1 Removal & Cleanup**
   - Remove all V1 code after migration complete
   - Rename V2 methods/types to remove "V2" suffix
   - Update all endpoints to remove version numbers
   - Update documentation

## Migration Plan

### Phase 1: Complete V2 Implementation ✅ DONE
- [x] Create parallel V2 infrastructure
- [x] Implement basic lobby/game separation
- [x] Test basic game flow

### Phase 2: Fix Remaining Issues 🚧 CURRENT
- [x] Fix message filtering/targeting (basic implementation done)
- [ ] Debug `gameStarted` message delivery
- [x] Implement entity-centric UI system (core implementation done)
- [ ] Test entity UI with actual game data
- [ ] Complete action map computation for V2
- [ ] Add proper game action processing

### Phase 3: Migrate Games
- [x] Create V2 version of `tic-tac-toe` ✅ DONE
  - [x] Created tictactoe-vtwo game with simplified gameStatus
  - [x] Built and deployed bundle to server
  - [ ] Test WebSocket connectivity
  - [ ] Verify game mechanics
- [ ] Create V2 version of `three-mens-morris`
  - [ ] Handle multi-step actions
  - [ ] Test piece movement
  - [ ] Verify mill detection
- [ ] Create V2 version of `connect-four`
  - [ ] Handle gravity mechanics
  - [ ] Test win detection
  - [ ] Verify draw conditions
- [ ] Create V2 version of `go-fish`
  - [ ] Handle card visibility
  - [ ] Test turn mechanics
  - [ ] Verify book formation

### Phase 4: Client Migration
- [ ] Update WebSocket connection logic
- [ ] Implement lobby UI components
- [ ] Update game state handling
- [ ] Test reconnection logic

### Phase 5: Cleanup
- [ ] Remove all V1 code
- [ ] Rename V2 types/methods (remove "V2" suffix)
- [ ] Update endpoints (remove "/v2")
- [ ] Update all documentation
- [ ] Update tests

## Technical Details

### Key Architecture Changes

1. **Lobby/Game Separation**
   ```
   V1: Lobby = Game (1:1 relationship)
   V2: Lobby contains multiple GameInstances (1:N relationship)
   ```

2. **State Structure**
   ```
   V1: Complex nested gameStatus object
   V2: Simple string gameStatus ("preparing", "playing", "ended")
   ```

3. **Player Identity**
   ```
   V1: Player joins as slot (p1, p2, etc.)
   V2: Player has lobby identity, mapped to game slots
   ```

### Files to Update/Remove

**New V2 Files:**
- `server/src/lobby/lobby_v2.rs`
- `server/src/lobby/lobby_state.rs`
- `server/src/lobby/game_instance.rs`
- `server/src/lobby/websocket_v2.rs`
- `server/src/http_handlers_v2.rs`
- `server/src/engine/state_v2.rs`
- `server/src/engine/entity_ui.rs`
- `server/src/v2_state.rs`
- `server/src/lobby_v2_map.rs`

**Files to Remove (Phase 5):**
- All original lobby files after migration
- V1 HTTP handlers
- V1 WebSocket handlers
- V1 state initialization

**Files to Update:**
- `server/src/main.rs` - Remove V2 routes, make them primary
- `clients/react/src/ws/useLobbyWebSocket.ts` - Use new endpoints
- All game YAML files - Update for V2 state structure

## Success Criteria

1. All four games (tic-tac-toe, three-mens-morris, connect-four, go-fish) work in V2
2. Players can play multiple games in sequence without reconnecting
3. No references to "V1" or "V2" remain in code
4. All tests pass with new architecture
5. Documentation reflects new architecture

## Notes

- The V2 system runs alongside V1 during migration (no breaking changes)
- Message targeting needs careful attention to avoid information leaks
- Entity-centric UI is a major improvement over centralized action maps
- Multi-step actions become regular state transitions in V2
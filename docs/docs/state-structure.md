# State Structure and Synchronization

This document defines the canonical structure for game state in Bluefelt and how it's synchronized between the server and clients.

## Overview

Bluefelt uses a clear separation between:
- **Game State**: The authoritative game data (zones, pieces, turns, etc.)
- **UI Data**: Additional information for client interface (action maps, player names, etc.)

## Canonical Game State Structure

The game state stored on the server has this flat, clear structure:

```json
{
  "zones": {
    "board": {
      "type": "grid",
      "cells": [[null, null, null], ...]
    },
    "hand_p1": {
      "type": "list",
      "items": [...]
    }
  },
  "tick": 0,
  "turn": 0,
  "currentPlayer": "p1",
  "players": [
    {"id": "p1"},
    {"id": "p2"}
  ],
  "gameStatus": {
    "state": "playing",  // or "ended"
    "winner": null,      // or "p1", "p2"
    "tie": false
  },
  "phases": {
    "game": "play",
    "round": "draw"
  },
  "selection": {}        // Always present - stores temporary selections/state
}
```

### Important: The Selection Object

The `selection` object is **always initialized** in the game state to store temporary game data like:
- Selected pieces or cards
- Multi-step action state
- Player choices during actions

This object is guaranteed to exist, so verbs can safely add properties to it without worrying about patch failures.

## Client Message Structure

When the server sends messages to clients, it wraps the game state with additional metadata:

### Welcome Message
```json
{
  "type": "welcome",
  "you": "p1",              // or "p2", "spectator"
  "started": true,
  "game": {                 // The canonical game state
    "zones": { ... },
    "currentPlayer": "p1",
    "turn": 0,
    "gameStatus": { ... }
  },
  "ui": {                   // Client interface data
    "actionMap": {          // What actions each player can take
      "p1": {
        "/zones/board/cells/0/0": {
          "action": "placeMarker",
          "direction": "Click to place"
        }
      },
      "p2": {}
    },
    "players": ["alice", "bob"],     // Player usernames
    "entities": [...],               // Entity definitions
    "zones": [...],                  // Zone metadata
    "zoneGroups": [...],            // UI grouping hints
    "gameLog": [...],               // Game event log
    "currentPhasePrompt": "..."     // Optional UI prompt
  },
  "tick": 0
}
```

### Diff Message (State Updates)
```json
{
  "type": "diff",
  "tick": 1,
  "patch": [
    {
      "op": "replace",
      "path": "/game/zones/board/cells/0/0",
      "value": {"entity": "mark_p1"}
    },
    {
      "op": "replace", 
      "path": "/game/currentPlayer",
      "value": "p2"
    },
    {
      "op": "replace",
      "path": "/game/turn",
      "value": 1
    },
    {
      "op": "replace",
      "path": "/ui/actionMap",
      "value": {
        "p1": {},
        "p2": {
          "/zones/board/cells/0/1": {
            "action": "placeMarker",
            "direction": "Click to place"
          }
        }
      }
    }
  ]
}
```

## Patch Path Convention

All patches use JSON Patch format with these clear, semantic path conventions:

### Game State Patches
Patches that modify the authoritative game state use `/game`:
- `/game/zones/[zoneId]/...` - Zone contents
- `/game/turn` - Turn counter
- `/game/currentPlayer` - Current player ID
- `/game/gameStatus` - Game end state
- `/game/phases` - Phase states
- `/game/selection` - Selected piece info

### Client Interface Patches
Patches that modify client interface data use `/ui`:
- `/ui/actionMap` - Available actions
- `/ui/gameLog` - Game event log
- `/ui/currentPhasePrompt` - UI prompts

## Accessing State in Clients

### React Client Convention

In the React client, state should be accessed with clear, semantic patterns:

```typescript
// Game state fields - authoritative game data
const currentPlayer = lobbyState.game?.currentPlayer;
const turn = lobbyState.game?.turn;
const gameStatus = lobbyState.game?.gameStatus;
const zones = lobbyState.game?.zones;
const phases = lobbyState.game?.phases;

// Client interface fields - UI-specific data
const actionMap = lobbyState.ui?.actionMap;
const playerNames = lobbyState.ui?.players;
const entityDefs = lobbyState.ui?.entities;
const gameLog = lobbyState.ui?.gameLog;

// Connection info
const you = lobbyState.you;  // "p1", "p2", or "spectator"
const started = lobbyState.started;
```

### Important Notes

1. **Clear separation**: Game data lives in `game`, interface data lives in `ui`. Never mix them!

2. **Action maps are UI data**: The `actionMap` is computed by the server and sent as client interface data, not part of the game state.

3. **Player IDs vs usernames**: 
   - Game state uses player IDs: "p1", "p2"
   - UI data includes usernames: ["alice", "bob"]

4. **Zone data structure**: Zones can have different structures:
   - Grid zones: `{ type: "grid", cells: [[...]] }`
   - List zones: `{ type: "list", items: [...] }`
   - Legacy format (arrays): `[[...]]` for grids, `[...]` for lists

## State Synchronization Flow

1. **Initial State**: Server sends welcome message with full state
2. **Action Processing**: Client sends action, server validates and applies
3. **State Update**: Server computes diff and broadcasts to all clients
4. **Patch Application**: Clients apply patches to maintain synchronized state

## Patch Application Strategy

The client handles patch application robustly:

### Path Preprocessing
Before applying patches, the client ensures parent paths exist:
- `/meta/actionMap/p2` requires `actionMap.p2` to exist
- `/meta/phaseStates/game/current` requires `phaseStates.game` to exist

### Individual Patch Application
Patches are applied one by one to handle partial failures:
```typescript
for (const patch of patches) {
  try {
    workingState = applyPatch(workingState, [patch], true, false);
    successfulPatches++;
  } catch (error) {
    console.error('Failed to apply patch:', patch, error);
    // Continue with remaining patches
  }
}
```

This ensures that successful patches still update the state even if some patches fail.

## Player Count Handling

The game state is initialized with the maximum number of players defined in the game manifest, but when a game starts, it adjusts to the actual number of connected players:

1. **Initial Creation**: `load_initial_state` creates state for max players
2. **Game Start**: `start_game` adjusts the state to match actual player count:
   - Updates the `players` array to only include connected players
   - Removes player-specific zones for non-existent players (e.g., `hand_p3`, `hand_p4`)
3. **Dynamic Adjustment**: This ensures games work correctly whether started with minimum or maximum players

### Example
A game supporting 2-4 players:
- Initial state creates: `p1`, `p2`, `p3`, `p4` and their zones
- If only 2 players join and start: State is adjusted to only have `p1`, `p2`
- Player-specific zones like `hand_p3` and `hand_p4` are removed

## Adding New State Properties

When implementing new game features that require state storage:

1. **Always initialize in `load_initial_state`** (in `/server/src/engine/state.rs`):
   ```rust
   let initial_state = json!({
       // ... existing properties ...
       "myNewProperty": {}  // or appropriate initial value
   });
   ```

2. **Use consistent patch paths**:
   - Game state: `/game/myNewProperty/...`
   - UI state: `/ui/myNewProperty/...`

3. **Document the new property** in this file

4. **Test patch operations** work with both empty and populated states

### Example: Adding Selection Support

When Go Fish needed to track selected ranks, we:
1. Added `"selection": {}` to initial state
2. Used the setState verb to add properties: `/selection/selectedRank`
3. Generated patches with appropriate operations ("add" for new keys)

## Common Pitfalls

1. **Mixing data sources**: Don't look for game state in `lobbyState.ui` or UI data in `lobbyState.game`
2. **Wrong patch prefixes**: Game state patches use `/game`, UI patches use `/ui`
3. **Assuming state structure**: Always check if objects exist before accessing
4. **Direct state mutation**: Never modify state directly, always use patches
5. **All-or-nothing patch failures**: Apply patches individually to handle partial failures
6. **Missing parent paths**: Ensure nested objects exist before applying patches to them
7. **Not initializing state properties**: Always add new properties to `load_initial_state` to avoid patch failures

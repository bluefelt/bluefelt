# Implementing a Client

This guide explains how to create a new Bluefelt client that can connect to the server and allow users to play games. Whether you're building a web app, mobile app, desktop application, or even a VR experience, the core concepts remain the same.

## Core Responsibilities

A Bluefelt client has four primary responsibilities:

1. **WebSocket Communication** - Maintain a connection to the server
2. **State Management** - Apply JSON patches to maintain game state
3. **User Interface** - Render zones and entities based on server data
4. **Action Handling** - Capture user input and send actions to server

## Communication Protocol

### WebSocket Connection

Connect to the server using the WebSocket endpoint:

```
ws://[server]/api/lobbies/[lobby_id]/ws?player=[player_name]&join=true
```

Parameters:
- `lobby_id` - The game lobby to join
- `player_name` - The player's display name
- `join=true` - Indicates the player wants to join (not just spectate)

### Message Format

#### Client to Server

Clients send action messages in this format:

```json
{
  "action": "place",
  "args": {
    "location": "/zones/board/cells/0/0",
    "entity": "x_token"
  }
}
```

Table management actions:
- `"action": "createTable"` - Create a new table in the lobby
- `"action": "claimSeat"` - Claim a seat at a table  
- `"action": "setReady"` - Mark yourself as ready to start
- `"action": "sendChatMessage"` - Send a chat message

Legacy actions (deprecated):
- `"action": "start_game"` - Use ready states instead

#### Server to Client

The server sends several message types:

**Lobby Joined** (sent immediately after connection):
```json
{
  "type": "lobbyJoined",
  "lobby": {
    "id": "abc123",
    "name": "Friday Night Games",
    "members": [{"id": "alice", "name": "alice", "connected": true}],
    "tables": [
      {
        "id": "table1",
        "bundleId": "tic-tac-toe", 
        "seats": ["alice", null],
        "status": "open",
        "readyStates": [false, false]
      }
    ]
  },
  "you": "alice"
}
```

**Game Started** (sent when countdown completes and game begins):
```json
{
  "type": "gameStarted",
  "tableId": "table1",
  "gameInstanceId": "game_xyz",
  "tick": 0,
  "you": "p1",
  "state": { /* initial game state */ },
  "ui": { /* UI hints and metadata */ }
}
```

**State Updates** (sent after each action):
```json
{
  "patches": [
    {
      "op": "replace",
      "path": "/zones/board/cells/0/0",
      "value": {"entity": "x_token"}
    }
  ]
}
```

**Error Messages**:
```json
{
  "error": "Invalid move: cell already occupied"
}
```

## State Structure

Bluefelt uses a lobby-first architecture where players connect to lobbies, create/join tables, and then play games:

```typescript
interface LobbyState {
  lobby: {
    id: string;
    name: string;
    members: LobbyMember[];
    tables: Table[];
  };
  you: string;  // Your username/player ID
}

interface Table {
  id: string;
  bundleId: string;               // Game type
  owner: string;
  status: 'open' | 'countdown' | 'playing' | 'finished';
  seats: (string | null)[];       // Player IDs in each seat
  readyStates: boolean[];          // Ready state for each seat
  minPlayers: number;
  maxPlayers: number;
  countdownEndsAt?: number;        // Timestamp when countdown ends
}

// Once a game starts, you receive the game state:
interface GameState {
  zones: Record<string, Zone>;     // All game zones
  currentPlayer: string;           // Who can act now
  turn: number;                    // Current turn number
  gameStatus: {                    // Game end state
    state: 'playing' | 'ended';
    winner?: string;
    tie?: boolean;
  };
  phases?: Record<string, string>; // Current phase states
}
```

### Understanding Action Maps

The action map tells clients which interactions are currently available:

```json
{
  "p1": {
    "/zones/board/cells/0/0": {
      "action": "place",
      "args": {
        "location": "/zones/board/cells/0/0"
      }
    },
    "/zones/board/cells/0/1": {
      "action": "place",
      "args": {
        "location": "/zones/board/cells/0/1"
      }
    }
  }
}
```

Keys are paths to interactive elements, values describe the action to send.

## Typical Workflow

Here's the typical flow for joining and playing a game:

### 1. Connect to Lobby
```javascript
// Connect to a lobby
const ws = new WebSocket('ws://localhost:8000/api/lobbies/abc123/ws?player=alice&join=true');

// Wait for lobby state
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.type === 'lobbyJoined') {
    // Display lobby: members, available tables, chat
    showLobby(message.lobby);
  }
};
```

### 2. Create or Join a Table
```javascript
// Create a new table
ws.send(JSON.stringify({
  action: 'createTable',
  bundleId: 'tic-tac-toe',
  name: 'My Table'
}));

// Or claim a seat at existing table
ws.send(JSON.stringify({
  action: 'claimSeat',
  tableId: 'table_123',
  seatIndex: 0
}));
```

### 3. Ready Up and Wait for Game
```javascript
// Mark yourself as ready
ws.send(JSON.stringify({
  action: 'setReady',
  tableId: 'table_123',
  ready: true
}));

// When all players are ready, 10-second countdown starts
// Game begins automatically when countdown ends
```

### 4. Play the Game
```javascript
// Listen for game start
if (message.type === 'gameStarted') {
  // Switch to game view
  showGame(message.state, message.ui);
}

// Handle game updates
if (message.patches) {
  // Apply JSON patches to update game state
  applyPatches(gameState, message.patches);
  renderGame(gameState);
}

// Send game actions
ws.send(JSON.stringify({
  action: 'place',
  args: {
    location: '/zones/board/cells/0/0',
    entity: 'x_token'
  }
}));
```

## Zone Types and Rendering

Zones are the containers for game entities. Each zone has a type that suggests how to render it:

### Grid Zones
Used for game boards (chess, tic-tac-toe, etc.):

```json
{
  "type": "grid",
  "cells": [
    [null, {"entity": "x_token"}, null],
    [{"entity": "o_token"}, null, null],
    [null, null, null]
  ]
}
```

Rendering suggestions:
- Display as a 2D grid
- Make empty cells clickable if they appear in the action map
- Show entity graphics/text in occupied cells

### List Zones
Used for hands, card rows, etc.:

```json
{
  "type": "list",
  "items": [
    {"entity": "card_hearts_a"},
    {"entity": "card_spades_k"}
  ]
}
```

Rendering suggestions:
- Display as a row or column of items
- Allow selection if items appear in action map
- Consider overlapping for card hands

### Stack Zones
Used for decks, discard piles:

```json
{
  "type": "stack",
  "items": [
    {"entity": "card_back"},
    {"entity": "card_back"},
    {"entity": "card_back"}
  ]
}
```

Rendering suggestions:
- Show as a pile with slight offset
- Display count if many items
- Show top card if visible

### Choice Zones
Used for presenting options to players:

```json
{
  "type": "choice",
  "choices": [
    {"id": "rock", "display": "Rock ✊"},
    {"id": "paper", "display": "Paper ✋"},
    {"id": "scissors", "display": "Scissors ✌️"}
  ]
}
```

## Implementation Steps

### 1. WebSocket Management

Create a robust WebSocket connection with reconnection logic:

```javascript
class GameConnection {
  constructor(url) {
    this.url = url;
    this.reconnectAttempts = 0;
    this.connect();
  }

  connect() {
    this.ws = new WebSocket(this.url);
    
    this.ws.onopen = () => {
      this.reconnectAttempts = 0;
      this.onConnected();
    };
    
    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      this.handleMessage(data);
    };
    
    this.ws.onclose = () => {
      this.scheduleReconnect();
    };
  }

  scheduleReconnect() {
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
    setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, delay);
  }

  sendAction(action, args) {
    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ action, args }));
    }
  }
}
```

### 2. State Management

Implement JSON Patch application:

```javascript
function applyPatches(state, patches) {
  let newState = JSON.parse(JSON.stringify(state)); // Deep clone
  
  for (const patch of patches) {
    const path = patch.path.split('/').filter(p => p);
    let target = newState;
    
    // Navigate to parent
    for (let i = 0; i < path.length - 1; i++) {
      target = target[path[i]];
    }
    
    const key = path[path.length - 1];
    
    switch (patch.op) {
      case 'add':
      case 'replace':
        target[key] = patch.value;
        break;
      case 'remove':
        if (Array.isArray(target)) {
          target.splice(parseInt(key), 1);
        } else {
          delete target[key];
        }
        break;
    }
  }
  
  return newState;
}
```

### 3. Zone Rendering

Create generic zone components:

```javascript
function renderZone(zoneId, zoneData, actionMap) {
  switch (zoneData.type) {
    case 'grid':
      return renderGridZone(zoneId, zoneData, actionMap);
    case 'list':
      return renderListZone(zoneId, zoneData, actionMap);
    case 'stack':
      return renderStackZone(zoneId, zoneData, actionMap);
    case 'choice':
      return renderChoiceZone(zoneId, zoneData, actionMap);
    default:
      return renderUnknownZone(zoneId, zoneData);
  }
}

function renderGridZone(zoneId, zone, actionMap) {
  return zone.cells.map((row, rowIdx) => 
    row.map((cell, colIdx) => {
      const path = `/zones/${zoneId}/cells/${rowIdx}/${colIdx}`;
      const action = actionMap?.[path];
      
      return {
        content: cell,
        clickable: !!action,
        onClick: action ? () => sendAction(action) : null
      };
    })
  );
}
```

### 4. Action Handling

Process user interactions based on the action map:

```javascript
function handleCellClick(zoneId, row, col) {
  const path = `/zones/${zoneId}/cells/${row}/${col}`;
  const actionInfo = gameState.ui.actionMap?.[gameState.you]?.[path];
  
  if (actionInfo) {
    connection.sendAction(actionInfo.action, actionInfo.args);
  }
}
```

## Best Practices

### 1. Never Assume Game Rules

❌ **Wrong**:
```javascript
// Don't check win conditions client-side
if (checkThreeInARow(board)) {
  showWinMessage();
}
```

✅ **Correct**:
```javascript
// Let the server tell you the game state
if (gameState.game.gameStatus.state === 'ended') {
  showWinMessage(gameState.game.gameStatus.winner);
}
```

### 2. Handle All Zone Types

Even if your first game only uses grids, implement rendering for all zone types. New games might use different zones.

### 3. Respect the Action Map

Only allow interactions that appear in the action map. Don't assume certain moves are valid.

### 4. Graceful Degradation

Handle missing or unknown data gracefully:

```javascript
function getEntityDisplay(entity) {
  const definition = gameState.ui.entities?.find(e => e.id === entity.entity);
  return definition?.display || entity.entity; // Fallback to ID
}
```

### 5. Efficient Updates

Use efficient rendering techniques to handle frequent state updates:
- Virtual DOM (React)
- Change detection (Angular)
- Reactive bindings (Vue)
- Direct DOM manipulation with diffing

## Testing Your Client

### 1. Multi-Game Support

Test your client with different game types:
- Grid games (Tic-Tac-Toe)
- Card games (Go Fish)
- Complex games (Chess)

### 2. Edge Cases

- Empty zones
- Large boards (10x10 or bigger)
- Many players (4+)
- Rapid state updates
- Connection drops and reconnects

### 3. Accessibility

- Keyboard navigation
- Screen reader support
- Color blind friendly
- Touch targets for mobile

## Common Pitfalls

1. **Hardcoding Game Logic** - Never assume rules; always use server data
2. **Ignoring Action Maps** - These tell you what's clickable
3. **State Mutation** - Always create new state objects
4. **Missing Reconnection** - Networks are unreliable
5. **Assuming Entity Types** - Games can define any entities

## Next Steps

Once you have a basic client working:

1. Add visual polish (animations, transitions)
2. Implement sound effects
3. Add player preferences (themes, settings)
4. Create responsive layouts
5. Add social features (chat, emotes)

Remember: The server handles all game logic. Your client's job is to create an excellent user experience for playing those games.
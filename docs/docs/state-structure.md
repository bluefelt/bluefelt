# State Structure and Synchronization

This document defines the canonical structure for game state in Bluefelt and how it's synchronized between the server and clients.

## Overview

Bluefelt uses a Lobby → Table → Seat architecture where:
- **Lobbies** are persistent social spaces where players gather
- **Tables** are created within lobbies where players can claim seats
- **Games** start when all seated players are ready
- **Chat** operates at both lobby and table levels
- State synchronization uses JSON Patch for efficient updates

## Lobby → Table → Seat Architecture

### Lobby Structure
- Lobbies persist indefinitely and have invite codes
- First player to join becomes the lobby owner
- Owners can rename the lobby
- Ownership transfers to earliest joined member when owner leaves
- Lobbies become archived when all members leave (cannot be rejoined)
- Members can create multiple tables within a lobby
- Multiple tables can exist concurrently in one lobby
- Members can browse tables and join available seats
- Dual-scope chat system (lobby-wide and table-specific)

### Table Structure
- Tables are created by players with a specific game type
- Each table has a fixed number of seats based on the game
- Players claim seats atomically (no race conditions)
- All seated players must mark "ready" before game starts
- 10-second countdown begins when all players are ready
- Tables can be in states: Open, Countdown, Playing, Finished

### Identity Management
- **In Lobby**: Users are identified by their username
- **At Table**: Players are identified by their seat and username
- **In Game**: Players are identified by their slot (p1, p2, etc.)
- The `you` field indicates your player slot once game starts

## State Structure

### Lobby State
```json
{
  "id": "lobby_xyz",
  "name": "Friday Night Games",
  "owner": "alice",
  "archived": false,
  "inviteCode": "ABCD1234",
  "members": [
    {
      "id": "alice",
      "name": "alice",
      "connected": true,
      "activeTables": ["table_123"]
    }
  ],
  "tables": [
    {
      "id": "table_123",
      "bundleId": "tic-tac-toe",
      "owner": "alice",
      "status": "Open",
      "seats": [
        { "playerId": "alice", "username": "alice" },
        null
      ],
      "readyStates": [true, false],
      "minPlayers": 2,
      "maxPlayers": 2,
      "countdownEndsAt": null
    }
  ],
  "recentChat": [
    {
      "id": "msg_123",
      "scope": "lobby",
      "sender": "alice",
      "message": "Anyone up for a game?",
      "timestamp": 1703001234
    }
  ],
  "settings": {
    "maxMembers": 50,
    "maxConcurrentTables": 10,
    "allowObservers": true
  }
}
```

### Game State (unchanged)
Once a game starts from a table, the state structure remains the same:

```json
{
  "zones": {
    "board": {
      "cells": [[null, null, null], ...]
    }
  },
  "tick": 0,
  "currentPlayer": "p1",
  "players": [
    {"id": "p1"},
    {"id": "p2"}
  ],
  "gameStatus": {
    "state": "playing",
    "winner": null,
    "tie": false
  },
  "phases": {
    "game": "play"
  },
  "selection": {}
}
```

## WebSocket Messages

### Connection
```
ws://localhost:8000/api/lobbies/{lobbyId}/ws?player={username}&join=true
```

### LobbyJoined Message
Sent when a member joins a lobby:

```json
{
  "type": "lobbyJoined",
  "lobby": {
    "id": "lobby_xyz",
    "myId": "alice",
    "name": "Friday Night Games",
    "inviteCode": "ABCD1234",
    "members": [...],
    "tables": [...],
    "recentChat": [...],
    "games": [...],  // Legacy, for backward compatibility
    "settings": {...}
  }
}
```

### Table Messages

#### TableCreated
```json
{
  "type": "tableCreated",
  "table": {
    "id": "table_123",
    "bundleId": "tic-tac-toe",
    "owner": "alice",
    "status": "Open",
    "seats": [null, null],
    "readyStates": [false, false],
    "minPlayers": 2,
    "maxPlayers": 2
  }
}
```

#### TableUpdated
```json
{
  "type": "tableUpdated",
  "tableId": "table_123",
  "seats": [
    { "playerId": "alice", "username": "alice" },
    { "playerId": "bob", "username": "bob" }
  ],
  "readyStates": [true, true],
  "status": "Countdown",
  "countdownEndsAt": 1703001244
}
```

#### CountdownStarted
```json
{
  "type": "countdownStarted",
  "tableId": "table_123",
  "endsAt": 1703001244
}
```

#### GameStarted (from table)
```json
{
  "type": "gameStarted",
  "tableId": "table_123",
  "gameInstanceId": "game_abc",
  "you": "p1",
  "state": {...},
  "ui": {...}
}
```

### Chat Messages

```json
{
  "type": "chatMessage",
  "scope": "lobby",  // or "table"
  "tableId": null,   // or "table_123" for table chat
  "sender": "alice",
  "message": "Good game!",
  "timestamp": 1703001234
}
```

### Client Actions

#### Table Management
```json
// Create table
{
  "action": "createTable",
  "bundleId": "tic-tac-toe",
  "minPlayers": 2,         // optional
  "maxPlayers": 2          // optional
}

// Claim seat
{
  "action": "claimSeat",
  "tableId": "table_123",
  "seatIndex": 0
}

// Release seat
{
  "action": "releaseSeat",
  "tableId": "table_123",
  "seatIndex": 0
}

// Set ready state
{
  "action": "setReady",
  "tableId": "table_123",
  "ready": true
}
```

#### Chat
```json
{
  "action": "sendChatMessage",
  "message": "Hello everyone!",
  "scope": "lobby",        // or "table"
  "tableId": null          // or "table_123" for table chat
}
```

## State Synchronization Flow

1. **Join Lobby**: Connect and receive lobby state with tables
2. **Browse Tables**: See all tables and their seat availability  
3. **Claim Seat**: Atomically claim an available seat
4. **Ready Up**: Mark yourself ready when prepared to play
5. **Countdown**: 10-second countdown when all players ready
6. **Game Start**: Transition from table to active game
7. **Play Game**: Standard game flow with patches
8. **Game End**: Table transitions to Finished state
9. **Return to Lobby**: Can create or join new tables

## Benefits of Table Architecture

1. **Social Persistence**: Players stay in lobby between games
2. **Self-Organization**: Players create and name their own tables
3. **No Race Conditions**: Atomic seat claiming prevents conflicts
4. **Clear Intent**: Ready system ensures all players want to start
5. **Spectator Support**: Non-seated players can watch games
6. **Flexible Matching**: Support for 2-8 player games at tables
7. **Dual Chat**: Separate lobby and table conversations

## Client Implementation Notes

### React Client Patterns

```typescript
// Check if seated at any table
const myTable = lobbyState.tables.find(table =>
  table.seats.some(seat => seat?.playerId === username)
);

// Get my seat index
const mySeatIndex = myTable?.seats.findIndex(
  seat => seat?.playerId === username
);

// Check if ready to start
const allReady = myTable?.seats.every((seat, idx) => 
  seat === null || myTable.readyStates[idx]
);

// Countdown timer
const timeLeft = myTable?.countdownEndsAt 
  ? Math.max(0, myTable.countdownEndsAt - Date.now() / 1000)
  : null;
```

### Managing Table State

```typescript
// Handle seat claiming with optimistic updates
const claimSeat = async (tableId: string, seatIndex: number) => {
  // Optimistic update
  updateTableSeats(tableId, seatIndex, { playerId: myId, username: myName });
  
  // Send to server
  socket.send({ action: 'claimSeat', tableId, seatIndex });
  
  // Server will send authoritative update
};
```

## Table System Implementation Status

The table system is the primary architecture with auto-seat assignment:

1. **Current Standard**: Table system with `createTable`/`joinTable` is the primary API
2. **Auto-Seat Assignment**: Players use `joinTable` for automatic seat claiming
3. **Multiple Tables**: Lobbies support multiple concurrent tables per lobby
4. **Production Ready**: Table system validated for production use (Phase A completed)

## Common Patterns

### Table Status Checks
```typescript
const canClaimSeat = (table, seatIndex) => {
  return table.status === 'Open' && 
         table.seats[seatIndex] === null &&
         !isSeatedAtAnyTable();
};

const canStartCountdown = (table) => {
  const seatedCount = table.seats.filter(s => s !== null).length;
  const allReady = table.seats.every((seat, idx) => 
    seat === null || table.readyStates[idx]
  );
  return seatedCount >= table.minPlayers && allReady;
};
```

### Chat Scoping
```typescript
// Send to appropriate chat
const sendChat = (message: string) => {
  if (currentTable) {
    socket.send({ 
      action: 'sendChatMessage', 
      message, 
      scope: 'table',
      tableId: currentTable.id 
    });
  } else {
    socket.send({ 
      action: 'sendChatMessage', 
      message, 
      scope: 'lobby' 
    });
  }
};
```

## Error Handling

Common error responses from the server:

- "Seat already taken" - Another player claimed the seat first
- "Already seated at a table" - Player can only sit at one table
- "Cannot claim seat during countdown" - Table is about to start
- "Table not found" - Table was deleted or doesn't exist
- "Not seated at this table" - Trying to ready up without a seat
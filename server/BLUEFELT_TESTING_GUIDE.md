# Bluefelt Game Testing Guide

## Overview

This guide documents the testing approach for Bluefelt games, covering both server-side and client-side testing.

## Server-Side Testing

### Key Principles

1. **Complete Game Flow**: Tests should play through an entire game from lobby creation to game end
2. **State Verification**: Verify game state at each critical transition
3. **Message Deduplication**: Handle duplicate messages (both players receive all updates)
4. **Error Handling**: Test invalid moves and edge cases

### Test Structure

```javascript
// 1. Create lobby via HTTP API
const lobby = await createLobby(gameId);

// 2. Connect players via WebSocket
const players = await connectPlayers(lobby.id, ['Alice', 'Bob']);

// 3. Start game
await startGame(players[0]);

// 4. Play through game with assertions
while (!gameEnded) {
  const currentPlayer = getCurrentPlayer();
  const action = selectAction(gameState);
  await executeAction(currentPlayer, action);
  verifyState(gameState);
}

// 5. Verify final state
assert(gameState.ended);
assert(gameState.winner || gameState.tie);
```

### Common Issues Found

1. **Go Fish Turn Bug**: Turn advances to non-existent players (p3, p4) when only 2 players
2. **State Initialization**: Client state must match server state structure (e.g., selection object)
3. **Action Map Tracking**: Action maps are sent differently in gameStarted vs diff messages

## Client-Side Testing

### Key Areas to Test

1. **State Synchronization**
   - Client state mirrors server state after each patch
   - Patches apply successfully without errors
   - State structure matches expected format

2. **UI Affordances**
   - Correct actions available for current player
   - UI elements appear/disappear based on game phase
   - Turn indicators accurate

3. **Request Generation**
   - Actions formatted correctly
   - Arguments match server expectations
   - Player ID included where needed

### Test Patterns

```javascript
// 1. State sync test
it('should sync state with server', async () => {
  const serverState = await getServerState();
  const clientState = component.state;
  
  expect(clientState.game).toEqual(serverState.game);
  expect(clientState.ui.actionMap).toBeDefined();
});

// 2. UI affordance test
it('should show actions only on player turn', () => {
  const { actionMap, isYourTurn } = component.props;
  
  if (isYourTurn) {
    expect(Object.keys(actionMap).length).toBeGreaterThan(0);
  } else {
    expect(component.find('.action-button').length).toBe(0);
  }
});

// 3. Request generation test
it('should generate correct action request', () => {
  const action = component.generateAction('placeMarker', { row: 0, col: 0 });
  
  expect(action).toEqual({
    action: 'placeMarker',
    args: {
      location: '/zones/board/cells/0/0',
      entity: 'mark_p1'
    }
  });
});
```

## Game-Specific Notes

### Tic-Tac-Toe
- Simple turn-based flow
- Fixed 2 players (p1, p2)
- Single phase: play
- Actions: placeMarker

### Go Fish
- Complex phase transitions: dealing → selectingRank → selectingPlayer → responding → fishing
- Variable players (2-4)
- **Bug**: nextTurn advances beyond actual player count
- Actions: selectRank, selectPlayer, automatic card transfers

### Three Men's Morris
- Two main phases: placement → movement
- Selection-based actions
- Actions: placeToken, selectPiece, moveSelectedPiece, clearSelection

## Regression Test Checklist

- [ ] Lobby creation successful
- [ ] All players connect properly
- [ ] Game starts with correct initial state
- [ ] Each turn processes correctly
- [ ] Invalid actions rejected
- [ ] Game ends properly (win/tie)
- [ ] Final scores/state correct
- [ ] No infinite loops or hangs
- [ ] Client receives all updates
- [ ] UI reflects current state

## Known Issues to Test For

1. **Patch Application Failures**
   - Missing parent objects (e.g., selection not initialized)
   - Incorrect path structure
   - Operation on non-existent paths

2. **Turn Management**
   - Turn advancing to non-existent players
   - Turn not advancing when it should
   - Current player not updating

3. **Phase Transitions**
   - Phases getting stuck
   - Multiple transitions in same tick
   - UI not updating with phase

4. **Action Availability**
   - Actions not appearing when they should
   - Wrong player getting actions
   - Actions persisting after use
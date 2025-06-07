# Bluefelt Client Test Suite

This comprehensive test suite ensures the client-side handling of all Bluefelt games is robust and reliable. The tests cover critical scenarios for state synchronization, WebSocket communication, and game-specific behaviors.

## Test Structure Overview

### Core Test Files

#### 1. `WebSocketMessageHandling.test.tsx`
**Purpose**: WebSocket communication reliability
- **Message Type Processing**: Welcome, diff, playerUpdate message handling
- **Patch Application Edge Cases**: Missing parent paths, out-of-order patches, invalid operations
- **Message Format Validation**: Malformed JSON, missing fields, unknown types
- **Game State Consistency**: Multiple patch sequences maintaining state integrity

**Key Scenarios Tested**:
- Turn advancement via diff messages
- Action map updates for different players
- Patch application failures that don't crash the client
- Out-of-order message delivery

#### 2. `ServerClientIntegration.test.tsx`
**Purpose**: Server-client protocol validation
- Message construction and formatting
- Action request/response cycles
- State update propagation
- Error handling and recovery

#### 3. `StateSynchronization.test.tsx`
**Purpose**: Client state management
- JSON Patch application
- State consistency across updates
- Handling partial state updates
- Recovery from corrupted state

#### 4. `UIAffordances.test.tsx`
**Purpose**: User interface interaction testing
- Visual feedback for valid actions
- Disabled state handling
- Touch and click event processing
- Responsive design behaviors

#### 5. `ClientRequestGeneration.test.tsx`
**Purpose**: Action message generation
- Correct message formatting for different action types
- Parameter validation and serialization
- Edge case handling in message construction

### Game-Specific Regression Tests

Located in `regression/` subdirectory, organized by game:

#### Tic-Tac-Toe (`regression/tic-tac-toe/`)
- `TicTacToeComplete.test.tsx` - Full game flow testing including:
  - Win detection (horizontal, vertical, diagonal)
  - Tie game scenarios
  - Turn switching
  - Invalid move prevention

#### Connect Four (`regression/connect-four/`)
- `ConnectFourActions.test.tsx` - Column-based action handling
- `ConnectFourUISync.test.tsx` - UI state synchronization for gravity mechanics

#### Go Fish (`regression/go-fish/`)
- `GoFishCardHandling.test.tsx` - Card selection and transfer mechanics
- `GoFishUISync.test.tsx` - Hand display and update synchronization

#### Three Men's Morris (`regression/three-mens-morris/`)
- `ThreeMensMorrisPhases.test.tsx` - Phase transition handling
- `ThreeMensMorrisUISync.test.tsx` - Selection state and movement display

## What These Tests Catch

### 1. **Turn Switching Issues** (The Original Bug)
- ✅ Server properly validates `player.isActor` condition
- ✅ Client correctly processes turn advancement patches
- ✅ UI reflects whose turn it is accurately
- ✅ Action maps update to show correct clickable cells

### 2. **State Synchronization Problems**
- ✅ `game.currentPlayer` vs `ui.actionMap` data separation
- ✅ JSON patch application order and partial failures
- ✅ Inconsistent state between server and client

### 3. **Action Validation Issues**
- ✅ Clicking when not your turn
- ✅ Clicking occupied cells
- ✅ Invalid message construction
- ✅ Game actions after game has ended

### 4. **Communication Failures**
- ✅ WebSocket message parsing errors
- ✅ Missing or malformed server responses
- ✅ Out-of-order message delivery
- ✅ Connection drops and reconnections

### 5. **Performance Issues**
- ✅ Memory leaks from repeated state updates
- ✅ Slow performance with large game states
- ✅ Inefficient action map lookups

### 6. **Edge Cases**
- ✅ Spectator mode functionality
- ✅ Player disconnections mid-game
- ✅ Corrupted or missing game state
- ✅ Multiple rapid clicks/actions

## Running the Tests

```bash
# Run all tests
pnpm test

# Run core test suites
pnpm test WebSocketMessageHandling
pnpm test ServerClientIntegration
pnpm test StateSynchronization
pnpm test UIAffordances
pnpm test ClientRequestGeneration

# Run game-specific regression tests
pnpm test TicTacToeComplete
pnpm test ConnectFourActions
pnpm test GoFishCardHandling
pnpm test ThreeMensMorrisPhases

# Run tests with coverage
pnpm test:coverage

# Run tests in watch mode
pnpm test --watch
```

## Benefits for Other Games

This test structure provides a template for testing other games:

1. **Message Handling**: Tests can be adapted for different message types
2. **State Management**: Core logic for state synchronization applies universally
3. **Action Validation**: Pattern can be extended for different action types
4. **Game Flow**: Integration tests can model any game's progression
5. **Error Scenarios**: Edge cases and error recovery apply to all games

## Continuous Integration

These tests should be run:
- ✅ Before every commit
- ✅ In CI/CD pipeline
- ✅ Before deploying new game types
- ✅ When modifying WebSocket or state management code

## Test Coverage

The test suite provides comprehensive coverage across:
- **Core functionality**: WebSocket handling, state synchronization, UI interactions
- **Game-specific behaviors**: Each game has dedicated regression tests
- **Edge cases**: Error handling, network issues, invalid states
- **Performance**: Rapid updates, large state changes
- **Integration**: Complete game flows from start to finish

This test suite ensures that client-side handling of all Bluefelt games is reliable, performant, and resilient against various failure scenarios.
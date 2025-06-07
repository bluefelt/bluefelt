# Bluefelt Game Regression Tests

This directory contains comprehensive server-side regression tests for all Bluefelt games. These tests are completely independent of any client code and test game mechanics directly through WebSocket connections.

## Test Coverage

### 🎮 Games Tested

1. **Tic-Tac-Toe** (`test-tic-tac-toe.js`)
   - All win conditions (horizontal, vertical, diagonal)
   - Tie games
   - Invalid moves (occupied cells, out of turn)
   - Edge cases (minimum moves to win, moves after game end)

2. **Connect Four** (`test-connect-four.js`)
   - Gravity mechanics
   - All win conditions (horizontal, vertical, both diagonals)
   - Tie games (full board)
   - Invalid moves (full columns, out of turn, invalid columns)
   - Edge cases (wins at edges, top row wins)

3. **Three Men's Morris** (`test-three-mens-morris.js`)
   - Placement phase (3 pieces per player)
   - Movement phase (select and move mechanics)
   - Phase transitions
   - Win conditions during both phases
   - Selection and cancellation
   - Invalid moves

4. **Go Fish** (`test-go-fish.js`)
   - Card dealing (7 cards per player)
   - Asking for cards mechanics
   - Go Fish (drawing from pool)
   - Book/pair formation
   - Win conditions
   - **Known Bug**: Turn advancement to non-existent players (p3/p4) in 2-player games

## Running Tests

### Prerequisites

1. Start the Bluefelt server:
   ```bash
   cd server
   cargo run
   ```

2. Ensure the server is running on port 8000

### Running Individual Tests

Run a specific game's tests:
```bash
node tests/regression/games/test-tic-tac-toe.js
node tests/regression/games/test-connect-four.js
node tests/regression/games/test-three-mens-morris.js
node tests/regression/games/test-go-fish.js
```

### Running All Tests

Run the complete test suite:
```bash
node tests/regression/run-all-tests.js
```

## Test Framework

All tests extend the `GameTestFramework` class which provides:

- WebSocket connection management
- Player setup and lobby creation
- Message handling and state tracking
- Action execution helpers
- Assertion utilities

### Test Structure

Each test file follows this pattern:

1. **Setup**: Create lobby, connect players, start game
2. **Test Scenarios**: Run specific test cases
3. **Assertions**: Verify expected behavior
4. **Cleanup**: Close connections

### Example Test Scenario

```javascript
await this.testScenario('Horizontal Win', async () => {
  const moves = [
    { player: 'p1', row: 0, col: 0 },
    { player: 'p2', row: 1, col: 0 },
    { player: 'p1', row: 0, col: 1 },
    { player: 'p2', row: 1, col: 1 },
    { player: 'p1', row: 0, col: 2 }, // P1 wins
  ];
  
  await this.playMoves(moves);
  assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win');
});
```

## Known Issues

### Go Fish
- **Turn Advancement Bug**: The game attempts to advance turns to players p3 and p4 even when only 2 players are in the game. This causes the game to become unplayable after a few turns.

## Adding New Tests

To add tests for a new game:

1. Create a new test file in `games/` directory
2. Extend `GameTestFramework`
3. Implement game-specific state tracking in `processPatch`
4. Add test scenarios covering:
   - Normal gameplay
   - Win conditions
   - Invalid moves
   - Edge cases
5. Add the test to `gameTests` array in `run-all-tests.js`

## Test Output

Tests provide detailed output including:
- Setup confirmation (lobby creation, player connections)
- Each test scenario with pass/fail status
- Game state debugging information
- Overall test summary

Failed tests will show:
- Error messages
- Current game state
- Board/hand visualization (where applicable)
# Bluefelt Testing Guide

## Overview

Bluefelt uses a comprehensive multi-layer testing architecture to ensure quality at every level of the platform. This document is the primary entry point for understanding our testing approach.

## Related Documentation

- **[Comprehensive Testing Strategy](./comprehensive-testing-strategy.md)** - Detailed testing pyramid with clear criteria for each test level
- **[UI Zone Test Harness](./ui-zone-test-harness.md)** - Specialized tool for stress testing UI components in isolation

## Testing Philosophy

**Core Principle**: "We shouldn't even touch the client until the game can be played and we have a test suite which can handle all possible outcomes."

**CRITICAL**: End-to-end tests must validate ACTUAL UI functionality, not just data flow. Tests that pass while the UI remains unusable are worse than no tests at all.

This means:
1. Server functionality is tested first, independently
2. Client behavior is tested second, with mocked server
3. Integration is tested last, with real client-server communication
4. **E2E tests must simulate real user interactions** - clicking UI elements and verifying the game reaches expected states

### E2E Testing Standards (MANDATORY)

**The Ultimate Source of Truth**: E2E tests are the final arbiter of whether a user can actually complete a game using the client app.

**What E2E Tests MUST Do**:
- Simulate actual clicking on UI elements (buttons, cells, cards, etc.)
- Verify that clicks result in the expected game state changes
- Test the complete user journey from game start to completion
- Validate that the UI correctly reflects the current game state
- Ensure users can actually interact with and complete games

**What E2E Tests MUST NOT Do**:
- Bypass the UI layer by directly calling functions or APIs
- Test only data structures without verifying UI rendering
- Pass when the UI is broken or non-functional
- Be "simplified" in ways that skip actual user interactions

**Red Flag**: If you can simplify an E2E test by removing UI interactions, you're probably breaking its purpose. The complexity of clicking through the UI is the point - that's what users actually do.

## Quick Start: When to Write Which Test

| Scenario | Test Type | Location | Example |
|----------|-----------|----------|---------|
| Pure function logic | Unit Test | Colocated with code | `verbs.test.rs`, `utils.test.ts` |
| Component interactions | Integration Test | `tests/` directory | `action_flow_test.rs` |
| UI edge cases | Visual Test | `__tests__/visual/` | `BoardZoneStress.test.tsx` |
| **Complete game flow** | **E2E Test** | `__tests__/regression/` | `TicTacToeComplete.test.tsx` |
| Zone rendering issues | UI Harness Test | Test harness page | Interactive debugging |

**E2E Test Requirements**:
- Must use `fireEvent.click()` or similar to simulate real user clicks
- Must verify game state changes after each UI interaction
- Must test complete game flows from start to win/loss/tie
- Must validate that users can actually complete games through the UI
- Any "simplification" that bypasses UI interactions defeats the purpose

## Testing Architecture

### Layer 1: Server Tests (Independent)

**Location**: `/server/tests/`

#### Rust Unit & Integration Tests
```bash
cd server
cargo test           # Run all Rust tests
cargo test tic_tac_toe   # Run specific game tests
cargo test --test connect_four_test  # Run specific test file
```

**Test Files**:
- `tests/*.rs` - Integration tests for each game
- `src/**/*.rs` - Unit tests (in `#[cfg(test)]` modules)

#### WebSocket Regression Tests
**Location**: `/server/tests/regression/`

```
tests/regression/
├── framework/
│   └── GameTestFramework.js     # Base WebSocket testing framework
├── games/
│   ├── test-tic-tac-toe.js      # ✅ Complete
│   ├── test-connect-four.js     # ❌ Needs debugging
│   ├── test-three-mens-morris.js # ❌ Needs debugging
│   └── test-go-fish.js          # ❌ Blocked by turn bug
├── run-all-tests.js             # Master test runner
└── README.md
```

**Running WebSocket Tests**:
```bash
# Start server first
cd server && cargo run

# In another terminal
cd server
node tests/regression/run-all-tests.js           # Run all games
node tests/regression/games/test-tic-tac-toe.js  # Run specific game
```

### Layer 2: Client Tests (Mocked Server)

**Location**: `/clients/react/src/__tests__/`

```
__tests__/
├── regression/                  # Game-specific tests
│   ├── tic-tac-toe/
│   │   └── TicTacToeComplete.test.tsx
│   ├── connect-four/
│   │   ├── ConnectFourUISync.test.tsx
│   │   └── ConnectFourActions.test.tsx
│   ├── three-mens-morris/
│   │   ├── ThreeMensMorrisUISync.test.tsx
│   │   └── ThreeMensMorrisPhases.test.tsx
│   └── go-fish/
│       ├── GoFishUISync.test.tsx
│       └── GoFishCardHandling.test.tsx
├── WebSocketMessageHandling.test.tsx    # General WebSocket tests
├── ServerClientIntegration.test.tsx     # General integration tests
├── StateSynchronization.test.tsx        # State sync tests
├── UIAffordances.test.tsx              # UI behavior tests
└── ClientRequestGeneration.test.tsx    # Request generation tests
```

**Component & Hook Tests**:
- `/clients/react/src/components/__tests__/` - UI component unit tests
- `/clients/react/src/hooks/__tests__/` - React hook tests

**Running Client Tests**:
```bash
cd clients/react
pnpm test                    # Run all tests
pnpm test TicTacToe         # Run tests matching pattern
pnpm test:ui                # Run with UI
pnpm test:coverage          # Generate coverage report
```

### Layer 3: Integration Tests (Future)

**Planned Location**: `/e2e/`

Will include:
- Cypress or Playwright tests
- Real browser automation
- Multi-player scenarios
- Cross-browser testing

## Test Coverage Requirements

### Game Log Requirements

All game implementations MUST ensure:

1. **Player Name Replacement**: Game logs should NEVER show "p1" or "p2" - these must be replaced with actual player names
2. **Automatic Action Logs**: Actions that happen automatically (like "Go Fish!" responses) must generate log entries
3. **Turn Switching**: When turns switch, there should be a clear log entry (e.g., "Turn passes to Bob")
4. **Complete Game Flow**: Players must be able to complete an entire game using only UI interactions

### What 100% Coverage Means

For Bluefelt, 100% coverage means testing **all possible behaviors**:

```
Coverage = (Tested Scenarios / Total Scenarios) × 100%

Where Total Scenarios = 
  Outcomes + Actions + States + Errors + Edge Cases
```

### Coverage by Category

1. **Outcomes**: All ways a game can end
   - Win conditions (all variations)
   - Tie/draw conditions
   - Abandonment (if applicable)

2. **Actions**: Every move a player can make
   - Valid actions in correct context
   - Invalid attempts (should fail)
   - Actions with wrong parameters

3. **States**: All reachable game states
   - Initial state
   - Phase transitions
   - Turn changes
   - End conditions

4. **Errors**: All failure modes
   - Wrong turn
   - Invalid location
   - After game end
   - Malformed data

5. **Edge Cases**: Boundary conditions
   - First/last moves
   - Min/max players
   - Full board/deck
   - Resource exhaustion

## Current Test Status

| Game | Server Tests | Client Tests | Known Issues |
|------|--------------|--------------|--------------|
| **Tic-Tac-Toe** | ✅ 100% Pass | ✅ Ready | Server allows moves after game end |
| **Connect Four** | ❌ Gravity failing | ✅ Ready | Implementation needs debugging |
| **Three Men's Morris** | ❌ Placement failing | ✅ Ready | Implementation needs debugging |
| **Go Fish** | ❌ Deal failing | ✅ Ready | Turn advances to non-existent players |

## ⚠️ CRITICAL WARNING: Test Simplification Anti-Pattern

**DANGER**: The most insidious testing mistake is "simplifying" E2E tests in ways that make them pass while the UI remains broken.

### What NOT to Do (Common Anti-Patterns)

❌ **Bypassing UI interactions**:
```javascript
// WRONG - This bypasses the actual UI clicking
const result = gameLogic.makeMove(player, position);
expect(result.success).toBe(true);
```

❌ **Testing only data without UI validation**:
```javascript
// WRONG - Tests the data but ignores whether UI works
mockWebSocket.trigger('gameUpdate', newState);
expect(component.state.currentPlayer).toBe('player2');
// Missing: Does the UI actually show "Player 2's turn"?
```

❌ **Removing "complex" UI interactions**:
```javascript
// WRONG - This skips the actual user experience
// gameActions.playFullGame(['A1', 'B1', 'A2', 'B2', 'A3']); // Shortcut
// expect(screen.getByText(/Player 1 wins/)).toBeInTheDocument();
```

### The Right Approach ✅

```javascript
// CORRECT - Forces actual UI clicking and validation
test('complete tic-tac-toe game with player 1 winning', async () => {
  // Each click represents what a real user would do
  fireEvent.click(screen.getByTestId('cell-0-0')); // Player 1: A1
  await waitFor(() => expect(screen.getByText("Player 2's turn")).toBeInTheDocument());
  
  fireEvent.click(screen.getByTestId('cell-1-0')); // Player 2: B1
  await waitFor(() => expect(screen.getByText("Player 1's turn")).toBeInTheDocument());
  
  fireEvent.click(screen.getByTestId('cell-0-1')); // Player 1: A2
  await waitFor(() => expect(screen.getByText("Player 2's turn")).toBeInTheDocument());
  
  fireEvent.click(screen.getByTestId('cell-1-1')); // Player 2: B2
  await waitFor(() => expect(screen.getByText("Player 1's turn")).toBeInTheDocument());
  
  fireEvent.click(screen.getByTestId('cell-0-2')); // Player 1: A3 (winning move)
  await waitFor(() => {
    expect(screen.getByText(/Player 1 wins!/i)).toBeInTheDocument();
  });
});
```

**Remember**: If it feels "tedious" to click through every move, that's the point. That's what users have to do. If the test is complex, it's because the user experience is complex.

## Test Development Workflow

### When Adding a New Game

1. **Design Phase**
   - Create `games/{game-name}/RULES.md` with complete rules
   - List all possible scenarios
   - Plan test coverage

2. **Server Implementation**
   - Write server regression tests FIRST
   - Create test in `server/tests/regression/games/test-{game-name}.js`
   - Implement game logic in YAML
   - Ensure ALL tests pass
   - **DO NOT start client work yet**

3. **Client Implementation**
   - Write client tests in `clients/react/src/__tests__/regression/{game-name}/`
   - **CRITICAL**: Write E2E tests that click through complete games
   - Implement UI components
   - Ensure tests pass **AND** verify that users can actually complete games

4. **Integration (Future)**
   - Write e2e tests
   - Test full stack
   - Performance validation

### Test File Template (Server)

```javascript
const { GameTestFramework } = require('../framework/GameTestFramework.js');

class MyGameRegressionTest extends GameTestFramework {
  constructor() {
    super('my-game');
  }

  async testScenario(name, testFunc) {
    console.log(`Testing: ${name}`);
    try {
      await testFunc();
      console.log('✅ PASSED');
    } catch (error) {
      console.log(`❌ FAILED: ${error.message}`);
      throw error;
    }
  }

  async testWinConditions() {
    // Test all ways to win
  }

  async testInvalidMoves() {
    // Test move rejection
  }
}
```

## Writing Effective Tests

### Server Test Best Practices

1. **Test behavior, not implementation**
2. **Each test should be independent**
3. **Use fresh lobbies for test isolation**
4. **Clear naming**: `testScenario('Player wins with three in a row', ...)`
5. **Test both success and failure cases**

### Client Test Best Practices

1. **Mock WebSocket completely**
2. **Test state synchronization**
3. **Verify UI updates from patches**
4. **Test user interactions**
5. **Check error handling**

### E2E Test Best Practices (CRITICAL)

**NEVER compromise on these standards - they are what make E2E tests valuable:**

1. **Always simulate real user interactions**
   ```javascript
   // ✅ CORRECT - Tests actual UI clicking
   fireEvent.click(screen.getByTestId('cell-0-0'));
   
   // ❌ WRONG - Bypasses UI layer
   gameActions.makeMove('A1');
   ```

2. **Verify state changes after each UI action**
   ```javascript
   // ✅ CORRECT - Confirms UI click had expected effect
   fireEvent.click(cell);
   await waitFor(() => {
     expect(screen.getByText('Player 2\'s turn')).toBeInTheDocument();
   });
   ```

3. **Test complete game scenarios, not just happy paths**
   - Full games from start to win/loss/tie
   - Invalid move attempts (clicking occupied cells)
   - Game end conditions (no more moves possible)
   - Turn switching verification

4. **Resist the urge to "simplify" E2E tests**
   - If removing UI interactions makes the test simpler, you're breaking its purpose
   - The complexity of clicking through the UI is intentional - that's what users do
   - False simplicity leads to tests that pass while the UI is broken

5. **Each E2E test should answer: "Can a user actually complete this game?"**
   - Not just "Does the data flow work?"
   - But "Can someone click buttons and win/lose/tie?"

## Common Testing Patterns

### Testing Turn-Based Logic
```javascript
// Server test
await this.executeAction('p1', 'makeMove', { location: 'A1' });
assert.strictEqual(this.currentPlayer, 'p2', 'Turn should switch');

// E2E Client test (PREFERRED for complete validation)
fireEvent.click(cell);
await waitFor(() => {
  expect(screen.getByText("Player 2's turn")).toBeInTheDocument();
});
expect(mockWebSocket.send).toHaveBeenCalledWith(
  JSON.stringify({ action: 'makeMove', args: { location: 'A1' } })
);
```

### Testing Game End Conditions
```javascript
// Server test
await this.playWinningMoves();
assert.strictEqual(this.gameStatus.state, 'ended');
assert.strictEqual(this.gameStatus.winner, 'p1');

// E2E Client test (CRITICAL - must validate UI shows win state)
// Click through the actual winning sequence
fireEvent.click(screen.getByTestId('cell-0-0')); // Player 1
fireEvent.click(screen.getByTestId('cell-1-0')); // Player 2
fireEvent.click(screen.getByTestId('cell-0-1')); // Player 1
fireEvent.click(screen.getByTestId('cell-1-1')); // Player 2
fireEvent.click(screen.getByTestId('cell-0-2')); // Player 1 wins

await waitFor(() => {
  expect(screen.getByText(/Player 1 wins!/i)).toBeInTheDocument();
});
```

### Testing Invalid Actions
```javascript
// Server test
await this.executeAction('p1', 'makeMove', { location: 'occupied' });
assert.strictEqual(this.lastError, 'Cell already occupied');

// E2E Client test (must verify UI prevents invalid clicks)
const occupiedCell = screen.getByTestId('cell-0-0');
// First click should work
fireEvent.click(occupiedCell);
await waitFor(() => {
  expect(occupiedCell).toHaveTextContent('X');
});

// Second click on same cell should be ignored
fireEvent.click(occupiedCell);
expect(mockWebSocket.send).toHaveBeenCalledTimes(1); // Only the first click
```

## Debugging Failed Tests

### Server Test Failures

1. **Check server is running**: `curl http://localhost:8000/api/lobbies`
2. **Look for infinite loops**: Repeated log messages
3. **Verify action definitions**: Check YAML files
4. **Check phase transitions**: Missing `enterActions`
5. **Validate state paths**: `/game/...` vs `/ui/...`

### Client Test Failures

1. **Check mock setup**: WebSocket mock properly configured
2. **Verify state structure**: Must match server exactly
3. **Check patch paths**: Ensure correct JSON Patch format
4. **Test timing**: Add `waitFor` for async updates

## Known Issues & Workarounds

### Server Issues

1. **Moves After Game End** (All games)
   - Server doesn't check `gameStatus.state === 'ended'`
   - Tests currently check game remains ended instead

2. **Go Fish Turn Bug**
   - Advances to p3/p4 in 2-player games
   - Blocks many test scenarios
   - Fix: Update `nextTurn` to use actual player count

### Test Infrastructure Issues

1. **WebSocket Timing**
   - Some tests need `setTimeout` waits
   - Framework handles most deduplication

2. **Test Isolation**
   - Each test group needs fresh lobby
   - Cannot reuse lobby between test scenarios

## Continuous Integration

### Current Setup
Tests should be run:
- On every commit
- Before merging PRs
- During deployment

### Commands for CI
```bash
# Server tests
cd server && cargo test
cd server && node tests/regression/run-all-tests.js

# Client tests
cd clients/react && pnpm test

# Future: Integration tests
cd e2e && npm run test
```

## Maintaining Tests

### When Changing Game Logic
1. Update server tests first
2. Run tests to ensure they fail (TDD)
3. Fix implementation
4. Update client tests if needed
5. Document any new patterns

### When Adding Features
1. Add test cases for new feature
2. Implement feature
3. Update this documentation if needed

### Test Review Checklist
- [ ] All scenarios covered?
- [ ] Tests are independent?
- [ ] Clear test names?
- [ ] Both success and failure cases?
- [ ] Documentation updated?

## Getting Help

### Common Issues

**"Cannot find module"**
- Ensure you're in the correct directory
- Run `npm install` or `pnpm install`

**"Connection refused"**
- Start the server: `cd server && cargo run`
- Check port 8000 is free

**"Test timeout"**
- Increase timeout in test
- Check for infinite loops
- Verify async operations complete

### Resources
- Example: `test-tic-tac-toe.js` is the reference implementation
- Framework: `GameTestFramework.js` has helpful methods
- Ask questions in GitHub issues

## Summary

The three-layer testing architecture ensures:
1. **Server correctness** before client work begins
2. **Client behavior** matches server expectations  
3. **Integration** validates the full stack

**MOST IMPORTANTLY**: E2E tests are the ultimate validation that users can actually play and complete games through the UI. Never compromise on this standard.

### Key Takeaways for E2E Testing:

- ✅ **Tests must click through actual UI elements** - `fireEvent.click()` is mandatory
- ✅ **Complete game flows must be tested** - from start to win/loss/tie
- ✅ **UI state must be validated after each action** - don't just test data
- ❌ **Never bypass the UI layer** - no direct function calls in E2E tests
- ❌ **Never "simplify" by removing interactions** - the complexity is the point
- ❌ **Tests that pass while UI is broken are worse than no tests**

By following this guide, you can confidently add new games and features while ensuring users can actually use them through the client interface.
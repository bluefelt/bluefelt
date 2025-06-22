# Testing Guide

## Overview

Bluefelt uses a comprehensive multi-layer testing architecture to ensure quality at every level of the platform. This document is the primary entry point for understanding our testing approach.

**Related Resources:**
- **UI Test Harness** - Available at `/ui-test` in development mode for stress testing zones
- **GameTestFramework** - Located at `server/tests/regression/framework/GameTestFramework.js`

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

## Critical Test Structure Requirements

**DISCOVERED ISSUE**: Many tests fail due to incorrect data structures. Follow these patterns exactly:

### Zone Metadata Must Be Arrays
```typescript
// ❌ WRONG - causes "zoneMetadata is not iterable" 
ui: { zones: {} }

// ✅ CORRECT
ui: { 
  zones: [
    {
      id: 'choice_p1',
      name: 'Select Rank',
      resolved_name: 'Select Rank',
      visibility: 'owner', 
      owner: 'p1',
      layout_order: 0,
      renderType: 'choice',  // Required for GameZones to render
      items: [...],
      prompt: 'Choose a rank'
    }
  ]
}
```

### Action Map Paths Must Match Client
```typescript
// ❌ WRONG - extra path segments
actionMap: {
  p1: {
    '/zones/choice_p1/ranks/a': { ... }  // Extra /ranks/
  }
}

// ✅ CORRECT - matches getChoiceActionLocation()
actionMap: {
  p1: {
    '/zones/choice_p1/a': {
      action: 'selectRank',
      direction: 'Choose a rank'
    }
  }
}
```

### Text Content Must Match Actual UI
```typescript
// ❌ WRONG - tests expect game description instead of name
expect(screen.getByText('Three Mens Morris game')).toBeInTheDocument();

// ✅ CORRECT - use actual displayed text or specific selectors
expect(screen.getByRole('heading', { name: 'Three Mens Morris' })).toBeInTheDocument();

// Or for duplicate text, use more specific selectors
expect(screen.getByText('Three Mens Morris')).toBeInTheDocument(); // ❌ Fails if text appears multiple times
expect(screen.getAllByText('Three Mens Morris')).toHaveLength(2); // ✅ Better approach
```

### Required Test IDs
Many components need data-testid for tests to work:
- CardZone: `data-testid="zone-${zoneId}"`
- ChoiceZone: `data-testid="choice-zone"`

### Animation Context Interface Mismatch
**MAJOR ISSUE**: Animation tests expect methods that don't exist:
- Tests expect: `processPatches()`, `getAnimationStatus()`, `engine` property, `state.queue` 
- Context provides: `updateConfig()`, `isAnimating`, `state`, `addAnimation`, `removeAnimation`, `clearQueue`

This affects 23+ failing tests requiring interface redesign.

### Audio Testing Environment Requirements

**CRITICAL**: AudioManager attempts to create real audio buffers in test environments, causing errors. Tests must detect the environment and skip audio initialization.

#### Required Test Environment Detection

**AudioManager.constructor()** must skip AudioContext initialization in tests:
```typescript
// AudioManager.tsx - Test environment detection
constructor() {
  // Skip AudioContext in test environment
  if (typeof window !== 'undefined' && 
      (window.location?.href?.includes('vitest') || 
       process.env.NODE_ENV === 'test' ||
       typeof global !== 'undefined' && global.expect)) {
    this.audioContext = null;
    this.soundBuffers = new Map();
    return;
  }
  
  // Normal initialization for browser
  this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
  this.soundBuffers = new Map();
}
```

**AnimationEngine.constructor()** must skip placeholder sound creation:
```typescript
// AnimationEngine.ts - Test environment detection  
constructor(options: AnimationEngineOptions) {
  this.audioManager = options.audioManager;
  
  // Skip placeholder sounds in test environment
  if (typeof process !== 'undefined' && process.env.NODE_ENV === 'test') {
    return;
  }
  
  // Create placeholder sounds for browser
  this.createPlaceholderSounds();
}
```

**AudioManager.preloadSounds()** must use mock buffers instead of fetch:
```typescript
// AudioManager.tsx - Test-safe preloading
async preloadSounds(sounds: string[]): Promise<void> {
  if (!this.audioContext) {
    // Test environment - create mock buffers
    sounds.forEach(sound => {
      this.soundBuffers.set(sound, null); // Mock buffer
    });
    return;
  }
  
  // Normal fetch and decode for browser
  const promises = sounds.map(async sound => {
    const response = await fetch(`/sounds/${sound}.mp3`);
    const arrayBuffer = await response.arrayBuffer();
    const audioBuffer = await this.audioContext!.decodeAudioData(arrayBuffer);
    this.soundBuffers.set(sound, audioBuffer);
  });
  
  await Promise.all(promises);
}
```

#### Test Setup Requirements

**Test files must mock audio before importing components:**
```typescript
// Before any imports that use AudioManager
beforeEach(() => {
  // Mock AudioContext
  global.AudioContext = vi.fn(() => ({
    decodeAudioData: vi.fn().mockResolvedValue({}),
    createBufferSource: vi.fn(() => ({
      connect: vi.fn(),
      start: vi.fn(),
      buffer: null
    })),
    destination: {}
  }));
  
  // Mock fetch for audio files
  global.fetch = vi.fn().mockResolvedValue({
    arrayBuffer: () => Promise.resolve(new ArrayBuffer(0))
  });
});
```

**Component tests must provide audio-safe context:**
```typescript
// Wrap components with test providers
const TestWrapper = ({ children }) => (
  <AnimationProvider audioManager={mockAudioManager}>
    <PlayerPreferencesProvider>
      {children}
    </PlayerPreferencesProvider>
  </AnimationProvider>
);

const mockAudioManager = {
  playSound: vi.fn(),
  preloadSounds: vi.fn().mockResolvedValue(undefined),
  setVolume: vi.fn()
};
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

## Recent Test Additions (June 2025)

### New Test Files Created

1. **Debug Endpoints Testing** (`server/tests/debug_endpoints_test.rs`)
   - Tests for `/api/debug` endpoints
   - Validates debug information structure
   - Tests error cases for non-existent lobbies
   - Ensures development-only exposure

2. **Hex Tic-Tac-Toe Integration** (`server/tests/hex_tic_tac_toe_test.rs`)
   - Tests hexagonal grid coordinate system
   - Validates hex board shape and configuration
   - Tests placement on axial coordinates
   - Verifies tie detection on full board

3. **WebSocket Cleanup Testing** (`server/tests/websocket_cleanup_test.rs`)
   - Tests normal and abrupt disconnections
   - Validates rapid reconnection handling
   - Tests error recovery scenarios
   - Verifies multiple connections per user

4. **ActionExecutor Integration** (`server/tests/action_executor_integration_test.rs`)
   - Comprehensive game flow testing
   - Tests conditional action execution
   - Validates template variable replacement
   - Tests depth limit protection

### Test Infrastructure Improvements

- **Debug Mode Support**: Debug endpoints provide deep state inspection during test failures
- **WebSocket Reliability**: Improved connection cleanup prevents test flakiness
- **ActionExecutor Coverage**: Unit tests now cover all major verb execution paths

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

## Test Structure Requirements

### Required Structures in Tests

When writing tests that create or mock Bluefelt structures, ensure they include all required fields:

#### MultiStepState Structure
```rust
MultiStepState {
    action_id: String,
    player_id: String, 
    step_index: usize,
    total_steps: usize,
    state: serde_json::Value,
    created_at: SystemTime,     // Required field
    last_activity: SystemTime,  // Required field
}
```

**Common Error**: Forgetting `created_at` and `last_activity` fields will cause compilation errors.

#### Bundle Structure
When accessing bundles in tests:
```rust
// ✅ Correct - Use get_latest
let bundle = bundles.get_latest("tic-tac-toe").unwrap();

// ❌ Wrong - No get() method exists
let bundle = bundles.get("tic-tac-toe");
```

#### Action Map Structure
Action maps are keyed by player ID, not username:
```javascript
// ✅ Correct - Using player ID
const playerActions = actionMap['p1'];

// ❌ Wrong - Using username (only works before game starts)
const playerActions = actionMap['alice'];
```

### Phase System Paths
Tests must handle both phase system formats:

```javascript
// Legacy phase system
const currentPhase = state.phases.game; // Simple string

// Enhanced phase system
const currentPhase = state.game.phases.game.current; // Nested object

// Safe access that handles both
const phase = state.phases?.current?.game || state.phases?.game;
```

## Writing Effective Tests

### Server Test Best Practices

1. **Test behavior, not implementation**
2. **Each test should be independent**
3. **Use fresh lobbies for test isolation**
4. **Clear naming**: `testScenario('Player wins with three in a row', ...)`
5. **Test both success and failure cases**
6. **Initialize all required fields in test structures**

### Client Test Best Practices

1. **Mock WebSocket completely**
2. **Test state synchronization**
3. **Verify UI updates from patches**
4. **Test user interactions**
5. **Check error handling**
6. **Use correct animation expectations**

#### Animation Test Expectations

When testing animations, use the current sound and duration values:

```javascript
// Sound expectations (as of Jan 2025)
expect(audioManager.playSound).toHaveBeenCalledWith('place_yours_soft');     // Player's piece
expect(audioManager.playSound).toHaveBeenCalledWith('place_opponent_soft');  // Opponent's piece

// Duration expectations
const ENTITY_SPAWN_DURATION = 300;  // Not 400ms
expect(animateCall[1].duration).toBe(300);

// Mock element structure
const mockElement = {
  classList: {
    add: vi.fn(),
    remove: vi.fn()
  },
  appendChild: vi.fn(),  // Required for some animations
  animate: vi.fn(() => ({
    finished: Promise.resolve(),
    addEventListener: vi.fn()
  }))
};
```

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
5. **Validate state paths**: Direct paths without prefixes (e.g., `/zones/...`, `/actionMap/...`)

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

## Test-First Development Strategy

Following the principle from comprehensive testing strategy:

1. **Test-First Development**: Write tests before implementing features
2. **Comprehensive Coverage**: Every verb, condition, and pattern must have tests
3. **Continuous Validation**: Automated tests run at every stage
4. **Real-World Scenarios**: Tests must reflect actual gameplay
5. **Client-Server Integration**: Tests must verify end-to-end functionality

### Test Templates by Game Type

#### Grid-Based Games (Tic-Tac-Toe, Connect-4, Go)
```javascript
// Key test areas:
// - Line detection (horizontal, vertical, diagonal)
// - Board full detection
// - Gravity effects (Connect-4)
// - Territory calculation (Go)

testPatterns = {
  winConditions: ['rows', 'columns', 'diagonals'],
  invalidMoves: ['occupied', 'outOfBounds'],
  edgeCases: ['cornerMoves', 'centerMove', 'fullBoard']
};
```

#### Card Games (Go Fish, Gin Rummy, Hearts)
```javascript
// Key test areas:
// - Deck shuffling determinism
// - Hand management
// - Valid play rules
// - Scoring calculations

testPatterns = {
  dealing: ['initialDeal', 'drawFromDeck', 'deckEmpty'],
  playing: ['validCard', 'followSuit', 'trump'],
  scoring: ['sets', 'runs', 'points']
};
```

### Game-Specific Test Templates

Create templates for each game following this structure:

```javascript
// server/tests/regression/templates/GameTestTemplate.js
class GameTestTemplate extends GameTestFramework {
  async runAllTests() {
    await this.testGameSetup();
    await this.testAllPhases();
    await this.testAllActions();
    await this.testEndConditions();
    await this.testEdgeCases();
  }
  
  async testGameSetup() {
    console.log('🎮 Testing Game Setup');
    
    // Verify initial phase completes
    await this.waitForPhase('play', 5000);
    
    // Verify all players have initial resources
    for (const player of this.getPlayers()) {
      const hasActions = this.playerHasActions(player);
      if (!hasActions) {
        throw new Error(`${player} has no available actions after setup`);
      }
    }
  }
}
```

### Comprehensive Verb Testing

Every verb must have comprehensive unit tests:

```rust
// server/tests/unit/verbs/draw_test.rs
use bluefelt_core::engine::verbs::draw::apply_draw;

#[test]
fn test_draw_single_card() {
    let mut state = create_test_state();
    add_cards_to_zone(&mut state, "/zones/deck", vec!["card_hearts_5"]);
    
    let args = json!({
        "from": "/zones/deck",
        "to": "/zones/hand_p1",
        "count": 1
    });
    
    let patches = apply_draw(&mut state, &args).unwrap();
    
    assert_eq!(patches.len(), 2); // Remove + Add patches
    assert_zone_count(&state, "/zones/deck", 0);
    assert_zone_count(&state, "/zones/hand_p1", 1);
}
```

### Automated Test Generation

Generate tests from YAML:

```javascript
// cli/src/generate-tests.js
function generateGameTest(gameDir) {
  const manifest = yaml.load(fs.readFileSync(`${gameDir}/manifest.yaml`));
  const phases = yaml.load(fs.readFileSync(`${gameDir}/phases.yaml`));
  const actions = yaml.load(fs.readFileSync(`${gameDir}/actions.yaml`));
  
  const testCode = `
const { GameTestTemplate } = require('../templates/GameTestTemplate');

class ${manifest.gameId}Test extends GameTestTemplate {
  constructor() {
    super('${manifest.gameId}');
  }
  
  // Generated phase tests
  ${generatePhaseTests(phases)}
  
  // Generated action tests
  ${generateActionTests(actions)}
}

module.exports = { ${manifest.gameId}Test };
`;
  
  fs.writeFileSync(`server/tests/regression/games/test-${manifest.gameId}.js`, testCode);
}
```

### Coverage Requirements

- **Unit Tests**: 100% of verbs and conditions
- **Integration Tests**: 100% of common patterns
- **E2E Tests**: 100% of games
- **Action Coverage**: Every action in every game tested

### Quality Metrics
- **Bug Discovery Rate**: Find bugs before users do
- **Regression Prevention**: No feature breaks existing games
- **Performance**: Tests complete in < 5 minutes
- **Reliability**: Tests pass consistently

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
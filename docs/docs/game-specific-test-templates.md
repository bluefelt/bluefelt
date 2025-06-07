# Game-Specific Test Templates

## Overview

This document provides templates and structure for implementing comprehensive tests for new games in the Bluefelt platform. Each game should follow this standardized approach to ensure consistent quality and complete coverage.

## Directory Structure

```
games/{game-name}/
├── 1.0/
│   ├── manifest.yaml
│   ├── entities.yaml
│   ├── actions.yaml
│   ├── phases.yaml
│   ├── zones.yaml
│   └── hooks.wasm (optional)
├── RULES.md
└── tests/
    ├── unit/
    │   ├── conditions.test.js
    │   ├── verbs.test.js
    │   └── hooks.test.js
    ├── integration/
    │   ├── complete-game.test.js
    │   ├── phase-transitions.test.js
    │   └── action-sequences.test.js
    └── visual/
        └── zone-configurations.json
```

## Test Templates by Level

### 1. Unit Tests

#### Condition Tests Template
```javascript
// games/{game-name}/tests/unit/conditions.test.js
const { TestHelper } = require('../../../../server/tests/helpers');

describe('{GameName} Conditions', () => {
  describe('Win Conditions', () => {
    test('detects horizontal win', () => {
      const state = TestHelper.createState({
        board: [
          ['X', 'X', 'X'],
          ['-', 'O', '-'],
          ['-', 'O', '-']
        ]
      });
      
      const result = checkWinCondition(state, 'X');
      expect(result).toBe(true);
    });
    
    test('detects no win with incomplete line', () => {
      const state = TestHelper.createState({
        board: [
          ['X', 'X', '-'],
          ['-', 'O', '-'],
          ['-', 'O', '-']
        ]
      });
      
      const result = checkWinCondition(state, 'X');
      expect(result).toBe(false);
    });
  });
  
  describe('Valid Move Conditions', () => {
    test('allows move to empty space', () => {
      const state = TestHelper.createState({
        board: createEmptyBoard(3, 3)
      });
      
      const canMove = isValidMove(state, { row: 0, col: 0 });
      expect(canMove).toBe(true);
    });
    
    test('prevents move to occupied space', () => {
      const state = TestHelper.createState({
        board: [['X', '-', '-']]
      });
      
      const canMove = isValidMove(state, { row: 0, col: 0 });
      expect(canMove).toBe(false);
    });
  });
});
```

#### Verb Tests Template
```javascript
// games/{game-name}/tests/unit/verbs.test.js
describe('{GameName} Verbs', () => {
  describe('Custom Placement Verb', () => {
    test('places piece correctly', () => {
      const state = { zones: { board: createEmptyBoard(3, 3) } };
      const patches = executePlacement(state, {
        location: '/zones/board/0/0',
        entity: 'piece_p1'
      });
      
      expect(patches).toContainEqual({
        op: 'add',
        path: '/zones/board/0/0',
        value: { entity: 'piece_p1' }
      });
    });
    
    test('validates placement rules', () => {
      const state = { 
        zones: { board: [['piece_p1']] },
        rules: { allowStacking: false }
      };
      
      expect(() => {
        executePlacement(state, {
          location: '/zones/board/0/0',
          entity: 'piece_p2'
        });
      }).toThrow('Cannot place on occupied space');
    });
  });
});
```

### 2. Integration Tests

#### Complete Game Test Template
```javascript
// games/{game-name}/tests/integration/complete-game.test.js
const { GameTestFramework } = require('../../../../server/tests/regression/framework/GameTestFramework');

class {GameName}IntegrationTest extends GameTestFramework {
  constructor() {
    super('{game-id}');
  }
  
  async runAllTests() {
    await this.testCompleteGame();
    await this.testAllWinConditions();
    await this.testDrawCondition();
    await this.testInvalidActions();
    await this.testConcurrency();
  }
  
  async testCompleteGame() {
    await this.testScenario('Complete game with winner', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Play winning sequence
      await this.executeAction('Alice', 'place', { location: '/zones/board/0/0' });
      await this.executeAction('Bob', 'place', { location: '/zones/board/1/0' });
      // ... continue moves
      
      // Verify win
      assert.strictEqual(this.gameStatus.state, 'ended');
      assert.strictEqual(this.gameStatus.winner, 'Alice');
    });
  }
  
  async testAllWinConditions() {
    const winPatterns = [
      { name: 'horizontal', moves: [...] },
      { name: 'vertical', moves: [...] },
      { name: 'diagonal', moves: [...] }
    ];
    
    for (const pattern of winPatterns) {
      await this.testScenario(`Win by ${pattern.name}`, async () => {
        // Test implementation
      });
    }
  }
  
  async testInvalidActions() {
    await this.testScenario('Reject move on occupied space', async () => {
      // Setup game
      await this.executeAction('Alice', 'place', { location: '/zones/board/0/0' });
      
      // Try invalid move
      const error = await this.expectError(() => 
        this.executeAction('Bob', 'place', { location: '/zones/board/0/0' })
      );
      
      assert.strictEqual(error.message, 'Space already occupied');
    });
  }
}

// Run tests
if (require.main === module) {
  const test = new {GameName}IntegrationTest();
  test.runAllTests().catch(console.error);
}
```

#### Phase Transition Test Template
```javascript
// games/{game-name}/tests/integration/phase-transitions.test.js
describe('{GameName} Phase Transitions', () => {
  let game;
  
  beforeEach(() => {
    game = new GameEngine('{game-id}');
  });
  
  test('transitions from setup to play phase', async () => {
    // Start in setup phase
    expect(game.currentPhase).toBe('setup');
    
    // Complete setup actions
    await game.executeAction('dealer', 'deal', { 
      cardsPerPlayer: 7 
    });
    
    // Verify transition
    expect(game.currentPhase).toBe('play');
    expect(game.currentPlayer).toBe('p1');
  });
  
  test('transitions to end phase on win condition', async () => {
    // Setup near-win state
    game.setState(nearWinState);
    
    // Make winning move
    await game.executeAction('p1', 'place', {
      location: '/zones/board/2/2'
    });
    
    // Verify transition
    expect(game.currentPhase).toBe('end');
    expect(game.gameStatus.state).toBe('ended');
  });
  
  test('handles phase-specific action restrictions', async () => {
    game.setPhase('setup');
    
    // Try play-phase action during setup
    await expect(
      game.executeAction('p1', 'place', { location: '/zones/board/0/0' })
    ).rejects.toThrow('Action not allowed in setup phase');
  });
});
```

### 3. Client Tests

#### UI Synchronization Test Template
```typescript
// clients/react/src/__tests__/regression/{game-name}/{GameName}UISync.test.tsx
describe('{GameName} UI Synchronization', () => {
  let mockWebSocket: MockWebSocket;
  
  beforeEach(() => {
    mockWebSocket = createMockWebSocket();
  });
  
  it('updates board when receiving place action', async () => {
    const { getByTestId } = render(<GameView gameId="{game-id}" />);
    
    // Simulate incoming patch
    mockWebSocket.receiveMessage({
      patches: [{
        op: 'add',
        path: '/zones/board/0/0',
        value: { entity: 'piece_p1', owner: 'p1' }
      }]
    });
    
    await waitFor(() => {
      const cell = getByTestId('cell-0-0');
      expect(cell).toHaveTextContent('X');
      expect(cell).toHaveClass('bg-blue-500');
    });
  });
  
  it('disables invalid moves based on action map', async () => {
    const { getByTestId } = render(<GameView gameId="{game-id}" />);
    
    // Set action map with only specific valid moves
    mockWebSocket.receiveMessage({
      patches: [{
        op: 'replace',
        path: '/ui/actionMap',
        value: {
          '/zones/board/0/0': { action: 'place' },
          // Other cells not included = not clickable
        }
      }]
    });
    
    // Valid cell should be interactive
    const validCell = getByTestId('cell-0-0');
    expect(validCell).toHaveClass('cursor-pointer');
    
    // Invalid cell should not be interactive
    const invalidCell = getByTestId('cell-1-1');
    expect(invalidCell).not.toHaveClass('cursor-pointer');
  });
});
```

#### Action Handling Test Template
```typescript
// clients/react/src/__tests__/regression/{game-name}/{GameName}Actions.test.tsx
describe('{GameName} Action Handling', () => {
  it('sends correct action when clicking valid cell', async () => {
    const mockSend = jest.fn();
    mockWebSocket.send = mockSend;
    
    const { getByTestId } = render(<GameView gameId="{game-id}" />);
    
    // Setup action map
    setupActionMap(mockWebSocket, {
      '/zones/board/0/0': { action: 'place', direction: 'Click to place' }
    });
    
    // Click cell
    const cell = getByTestId('cell-0-0');
    fireEvent.click(cell);
    
    // Verify sent message
    expect(mockSend).toHaveBeenCalledWith(JSON.stringify({
      action: 'place',
      args: { location: '/zones/board/0/0' }
    }));
  });
  
  it('handles special actions correctly', async () => {
    // Test game-specific actions like:
    // - Card selection
    // - Piece movement
    // - Resource spending
    // - Multi-step actions
  });
});
```

### 4. Visual/Stress Tests

#### Zone Configuration File
```json
// games/{game-name}/tests/visual/zone-configurations.json
{
  "testCases": [
    {
      "name": "empty-board",
      "description": "Empty game board",
      "state": {
        "zones": {
          "board": {
            "type": "grid",
            "dimensions": [3, 3],
            "entities": []
          }
        }
      }
    },
    {
      "name": "full-board",
      "description": "Completely filled board",
      "state": {
        "zones": {
          "board": {
            "type": "grid",
            "dimensions": [3, 3],
            "entities": [
              { "location": [0, 0], "entity": "piece_p1" },
              { "location": [0, 1], "entity": "piece_p2" },
              // ... all positions
            ]
          }
        }
      }
    },
    {
      "name": "large-board",
      "description": "Stress test with 20x20 grid",
      "state": {
        "zones": {
          "board": {
            "type": "grid",
            "dimensions": [20, 20],
            "entities": []
          }
        }
      }
    },
    {
      "name": "many-cards",
      "description": "Hand with 50 cards",
      "state": {
        "zones": {
          "hand_p1": {
            "type": "list",
            "layout": "fan",
            "entities": [/* 50 card entities */]
          }
        }
      }
    }
  ]
}
```

## Test Coverage Checklist

For each new game, ensure coverage of:

### Actions
- [ ] Each action succeeds with valid parameters
- [ ] Each action fails with invalid parameters
- [ ] Actions respect turn order
- [ ] Actions respect phase restrictions
- [ ] Actions update state correctly
- [ ] Actions trigger appropriate follow-up actions

### Conditions
- [ ] All win conditions detected correctly
- [ ] Draw/tie conditions detected
- [ ] Invalid move conditions enforced
- [ ] Resource availability checked
- [ ] Player eligibility validated

### Phases
- [ ] Initial phase set correctly
- [ ] All phase transitions triggered properly
- [ ] Phase-specific actions restricted
- [ ] End phase reached on game completion

### UI Synchronization
- [ ] Zones render initial state
- [ ] Patches update UI correctly
- [ ] Action map enables/disables interactions
- [ ] Game end state displayed
- [ ] Turn indicator updates
- [ ] All visual states tested (empty, full, partial)

### Edge Cases
- [ ] Minimum players (usually 2)
- [ ] Maximum players
- [ ] Empty zones
- [ ] Full zones
- [ ] Rapid actions
- [ ] Concurrent actions
- [ ] Network disconnection/reconnection

## Common Patterns by Game Type

### Grid-Based Games (Tic-Tac-Toe, Connect-4, Go)
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

### Card Games (Go Fish, Gin Rummy, Hearts)
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

### Resource Games (Stone Age, Settlers)
```javascript
// Key test areas:
// - Resource generation
// - Resource spending
// - Action availability
// - Victory points

testPatterns = {
  resources: ['gain', 'spend', 'trade', 'limits'],
  actions: ['available', 'blocked', 'cost'],
  scoring: ['immediate', 'endGame', 'bonus']
};
```

## Test Maintenance

### When to Update Tests
1. **Rule changes**: Update condition and action tests
2. **UI changes**: Update visual tests and selectors
3. **New features**: Add comprehensive test coverage
4. **Bug fixes**: Add regression test first

### Test Review Process
1. Run full test suite before committing
2. Review coverage reports
3. Add tests for any uncovered scenarios
4. Update this template with new patterns

## Performance Benchmarks

Each game should establish performance baselines:

```javascript
// games/{game-name}/tests/benchmarks.js
const benchmarks = {
  'action-execution': { target: '< 50ms', critical: '< 100ms' },
  'state-update': { target: '< 20ms', critical: '< 50ms' },
  'ui-render': { target: '< 16ms', critical: '< 33ms' },
  'full-game': { target: '< 500ms', critical: '< 1000ms' }
};
```
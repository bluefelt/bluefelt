# Client-Side Regression Tests

This directory contains comprehensive regression tests for the React client, organized by game. These tests focus on validating UI behavior, state synchronization, and user interactions without depending on the server.

## Test Structure

Each game has its own directory with tests covering:

### 1. UI Synchronization Tests (`*UISync.test.tsx`)
- Tests how the UI renders based on different game states
- Validates proper display of game elements (boards, cards, pieces)
- Checks visual feedback (highlighting, animations, etc.)
- Ensures correct rendering during different game phases
- Tests game end scenarios and winner display

### 2. Action Generation Tests (`*Actions.test.tsx`)
- Tests the `useGameActions` hook behavior
- Validates action map interpretation
- Tests turn-based restrictions
- Verifies correct message construction for server
- Tests edge cases and error scenarios

### 3. Phase-Specific Tests (`*Phases.test.tsx`)
- Tests phase transition handling
- Validates UI changes between phases
- Tests phase-specific action availability
- Ensures proper state management during transitions

### 4. Game-Specific Tests
Additional tests for unique game mechanics:
- **Connect Four**: Gravity animation, column selection
- **Three Men's Morris**: Selection states, mill detection
- **Go Fish**: Card handling, player targeting, book formation

## Test Patterns

### WebSocket Mocking
All tests mock WebSocket connections to avoid server dependencies:

```typescript
class MockWebSocket {
  send(data: string) {
    const message = JSON.parse(data);
    // Capture or validate sent messages
  }
}
```

### State Management
Tests use realistic game states that match the server's state structure:

```typescript
const mockLobbyState: LobbyState = {
  game: { currentPlayer, meta: { gameStatus } },
  ui: { actionMap, selection },
  zones: { /* game-specific zones */ },
  phases: { game: { current, stack } }
};
```

### Component Testing
Tests render components with proper context providers:

```typescript
const TestWrapper = ({ children }) => (
  <QueryClientProvider client={queryClient}>
    <PlayerProvider initialName="Alice">
      <WebSocketProvider>
        {children}
      </WebSocketProvider>
    </PlayerProvider>
  </QueryClientProvider>
);
```

## Running Tests

```bash
# Run all regression tests
pnpm test src/__tests__/regression/

# Run tests for a specific game
pnpm test src/__tests__/regression/connect-four/
pnpm test src/__tests__/regression/three-mens-morris/
pnpm test src/__tests__/regression/go-fish/

# Run with coverage
pnpm test:coverage src/__tests__/regression/

# Run in watch mode for development
pnpm test --watch src/__tests__/regression/
```

## Adding New Tests

When adding tests for a new game:

1. Create a directory: `src/__tests__/regression/[game-name]/`
2. Add UI synchronization tests: `[GameName]UISync.test.tsx`
3. Add action handling tests: `[GameName]Actions.test.tsx`
4. Add phase-specific tests if needed: `[GameName]Phases.test.tsx`
5. Include game-specific mechanics tests as appropriate

## Test Coverage Goals

Each game should have tests covering:

- ✅ Initial game rendering
- ✅ Action generation based on game state
- ✅ Turn-based restrictions
- ✅ Phase transitions
- ✅ Selection/deselection mechanics
- ✅ Game end scenarios (win/tie)
- ✅ Error handling
- ✅ Visual feedback (highlighting, animations)
- ✅ Multi-player scenarios
- ✅ Edge cases specific to the game

## Debugging Tips

1. **Use `screen.debug()`** to see the current DOM state
2. **Add `console.log` in mock WebSocket** to see sent messages
3. **Use `waitFor` for async updates** after user interactions
4. **Check data-testid attributes** for reliable element selection
5. **Validate state structure** matches server expectations
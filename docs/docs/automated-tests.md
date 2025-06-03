# Automated Tests

The Bluefelt project includes comprehensive automated testing to ensure reliability, performance, and correctness across all components. This guide covers the testing architecture, running tests, and contributing new test cases.

## Overview

The testing strategy includes:
- **Unit Tests** - Test individual functions and modules
- **Integration Tests** - Test component interactions
- **WebSocket Tests** - Test real-time communication
- **End-to-End Tests** - Test complete user workflows
- **Performance Tests** - Validate system performance

## Testing Architecture

### Server Tests (Rust)

The server includes 29 comprehensive tests covering all major functionality:

```
server/src/
├── lib.rs              # Test module exports
├── engine.rs           # 13 unit tests
├── shorthand.rs        # Unit tests for expansion
├── lobby.rs            # Unit tests for lobby management
└── tests/
    ├── engine_integration_tests.rs  # 8 integration tests
    └── websocket_tests.rs           # 8 WebSocket tests
```

### Client Tests (TypeScript)

```
clients/react/src/
├── components/         # Component unit tests
├── hooks/             # Custom hook tests
├── utils/             # Utility function tests
└── __tests__/         # Integration tests
```

### CLI Tests (Rust)

```
cli/src/
├── build.rs           # Build process tests
├── validate.rs        # Validation tests
└── tests/             # Integration tests
```

## Server Testing

### Running Server Tests

```bash
cd server

# Run all tests (29 total)
cargo test

# Run with detailed output
cargo test -- --nocapture

# Run specific test categories
cargo test --lib                     # Unit tests
cargo test --test engine_integration # Integration tests
cargo test --test websocket_tests    # WebSocket tests

# Run tests in parallel (default)
cargo test -- --test-threads=4

# Run tests serially (for debugging)
cargo test -- --test-threads=1
```

### Unit Tests

#### Engine Tests

The engine unit tests cover all built-in verbs:

```rust
// Example: Testing the place verb
#[test]
fn test_apply_place_verb() {
    let mut state = json!({
        "zones": {
            "board": {
                "type": "grid",
                "cells": [[null, null], [null, null]]
            }
        }
    });
    
    let args = json!({
        "location": "/zones/board/cells/0/0",
        "entity": "x_token"
    });
    
    let result = apply_verb(&mut state, "place", &args);
    assert!(result.is_ok());
    assert_eq!(state["zones"]["board"]["cells"][0][0]["entity"], "x_token");
}
```

**Covered Verbs:**
- `place` - Entity placement on grids
- `moveEntity` - Moving entities between zones
- `draw` - Drawing from decks to other zones
- `nextTurn` - Turn advancement
- `setPhase` - Phase management

#### Shorthand Tests

Tests for template expansion and built-in components:

```rust
#[test]
fn test_player_template_expansion() {
    let input = json!([{
        "id": "piece_{player}",
        "type": "piece"
    }]);
    
    let expanded = expand_entities(&input, 2);
    assert_eq!(expanded.len(), 2);
    assert_eq!(expanded[0]["id"], "piece_p1");
    assert_eq!(expanded[1]["id"], "piece_p2");
}
```

### Integration Tests

#### Game Simulation Tests

Full game simulations that test complete workflows:

```rust
#[test]
fn test_complete_tic_tac_toe_game() {
    let bundle = load_test_bundle("tic-tac-toe");
    let mut state = load_initial_state(&bundle).unwrap();
    
    // Simulate complete game
    let moves = vec![
        ("place", json!({"location": "/zones/board/cells/0/0", "entity": "x_token"})),
        ("place", json!({"location": "/zones/board/cells/0/1", "entity": "o_token"})),
        ("place", json!({"location": "/zones/board/cells/1/0", "entity": "x_token"})),
        ("place", json!({"location": "/zones/board/cells/1/1", "entity": "o_token"})),
        ("place", json!({"location": "/zones/board/cells/2/0", "entity": "x_token"})),
    ];
    
    for (verb, args) in moves {
        apply_verb(&mut state, verb, &args).unwrap();
    }
    
    // Verify game end state
    assert_eq!(state["meta"]["gameStatus"]["state"], "ended");
    assert_eq!(state["meta"]["gameStatus"]["winner"], "p1");
}
```

#### Multi-Game Tests

Tests that verify behavior across different game types:

```rust
#[test]
fn test_all_bundled_games_load() {
    let bundles = BundleMap::load_dir("../bundles").unwrap();
    
    for (game_id, bundle) in bundles.iter() {
        // Verify each game loads correctly
        let state = load_initial_state(bundle);
        assert!(state.is_ok(), "Failed to load {}: {:?}", game_id, state.err());
        
        // Verify basic state structure
        let state = state.unwrap();
        assert!(state.get("zones").is_some());
        assert!(state.get("meta").is_some());
    }
}
```

### WebSocket Tests

#### Connection Management

```rust
#[test]
async fn test_websocket_connection_and_welcome() {
    let lobby = create_test_lobby("test-game").await;
    let mut ws = connect_websocket(&lobby.id, "p1").await;
    
    // Should receive welcome message
    let message = ws.next().await.unwrap();
    let welcome: WelcomeMessage = serde_json::from_str(&message).unwrap();
    
    assert_eq!(welcome.message_type, "welcome");
    assert_eq!(welcome.data.player_id, "p1");
    assert!(welcome.data.state.is_object());
}
```

#### Action Processing

```rust
#[test]
async fn test_action_processing_and_diff() {
    let lobby = create_test_lobby("tic-tac-toe").await;
    let mut ws1 = connect_websocket(&lobby.id, "p1").await;
    let mut ws2 = connect_websocket(&lobby.id, "p2").await;
    
    // Skip welcome messages
    ws1.next().await;
    ws2.next().await;
    
    // Send action from player 1
    ws1.send(json!({
        "type": "action",
        "data": {
            "verb": "place",
            "args": {
                "location": "/zones/board/cells/0/0",
                "entity": "x_token"
            }
        }
    })).await;
    
    // Both players should receive diff
    let diff1 = ws1.next().await.unwrap();
    let diff2 = ws2.next().await.unwrap();
    
    assert_eq!(diff1, diff2);
    
    let diff: DiffMessage = serde_json::from_str(&diff1).unwrap();
    assert_eq!(diff.message_type, "diff");
    assert!(!diff.data.patches.is_empty());
}
```

#### Error Handling

```rust
#[test]
async fn test_invalid_action_error() {
    let lobby = create_test_lobby("tic-tac-toe").await;
    let mut ws = connect_websocket(&lobby.id, "p1").await;
    
    ws.next().await; // Skip welcome
    
    // Send invalid action
    ws.send(json!({
        "type": "action",
        "data": {
            "verb": "invalid_verb",
            "args": {}
        }
    })).await;
    
    // Should receive error message
    let error = ws.next().await.unwrap();
    let error_msg: ErrorMessage = serde_json::from_str(&error).unwrap();
    
    assert_eq!(error_msg.message_type, "error");
    assert!(error_msg.data.message.contains("Unknown verb"));
}
```

## Client Testing

### Setup

```bash
cd clients/react
pnpm install
pnpm test
```

### Component Tests

```typescript
// Example: Testing GameView component
import { render, screen } from '@testing-library/react';
import { GameView } from '../components/GameView';

test('renders game board', () => {
  const mockState = {
    zones: {
      board: {
        type: 'grid',
        cells: [[null, null, null], [null, null, null], [null, null, null]]
      }
    },
    meta: {
      currentPlayer: 'p1',
      gameStatus: { state: 'playing' }
    }
  };
  
  render(<GameView gameState={mockState} />);
  expect(screen.getByText('Current Player: p1')).toBeInTheDocument();
});
```

### WebSocket Hook Tests

```typescript
import { renderHook, act } from '@testing-library/react';
import { useWebSocket } from '../hooks/useWebSocket';

test('handles welcome message', async () => {
  const { result } = renderHook(() => 
    useWebSocket('ws://localhost:8000/ws/lobby/test/p1')
  );
  
  act(() => {
    // Simulate welcome message
    result.current.mockReceive({
      type: 'welcome',
      data: {
        playerId: 'p1',
        state: mockGameState,
        tick: 0
      }
    });
  });
  
  expect(result.current.playerId).toBe('p1');
  expect(result.current.gameState).toEqual(mockGameState);
});
```

### State Management Tests

```typescript
import { useGameStore } from '../store/gameStore';
import { renderHook, act } from '@testing-library/react';

test('applies JSON patches correctly', () => {
  const { result } = renderHook(() => useGameStore());
  
  act(() => {
    result.current.setGameState(initialState);
  });
  
  act(() => {
    result.current.applyPatches([
      {
        op: 'replace',
        path: '/zones/board/cells/0/0',
        value: { entity: 'x_token' }
      }
    ]);
  });
  
  const updatedState = result.current.gameState;
  expect(updatedState.zones.board.cells[0][0].entity).toBe('x_token');
});
```

## CLI Testing

### Build Process Tests

```rust
#[test]
fn test_build_game_from_yaml() {
    let temp_dir = create_temp_game_dir();
    write_test_manifests(&temp_dir);
    
    let result = build_game(&temp_dir);
    assert!(result.is_ok());
    
    // Verify output files exist
    assert!(temp_dir.join("bundles/test-game/1.0/manifest.json").exists());
    assert!(temp_dir.join("bundles/test-game/1.0/entities.json").exists());
}
```

### Validation Tests

```rust
#[test]
fn test_validation_catches_missing_fields() {
    let invalid_manifest = json!({
        "gameId": "test",
        // Missing required fields
    });
    
    let errors = validate_manifest(&invalid_manifest);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.message.contains("version")));
}
```

## Performance Tests

### Load Testing

```rust
#[tokio::test]
async fn test_concurrent_connections() {
    let lobby = create_test_lobby("stress-test").await;
    let connection_count = 100;
    
    let mut handles = Vec::new();
    
    for i in 0..connection_count {
        let lobby_id = lobby.id.clone();
        let handle = tokio::spawn(async move {
            let player_id = format!("player_{}", i);
            let mut ws = connect_websocket(&lobby_id, &player_id).await;
            
            // Send rapid actions
            for j in 0..10 {
                ws.send(create_test_action(j)).await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all connections to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify lobby is still functional
    assert_eq!(lobby.connections.len(), connection_count);
}
```

### Memory Usage Tests

```rust
#[test]
fn test_memory_usage_within_limits() {
    let initial_memory = get_memory_usage();
    
    // Load large number of games
    let bundles = BundleMap::load_dir("../test_bundles").unwrap();
    
    // Create many lobbies
    let mut lobbies = Vec::new();
    for i in 0..1000 {
        lobbies.push(create_lobby(format!("lobby_{}", i), &bundles["tic-tac-toe"]));
    }
    
    let final_memory = get_memory_usage();
    let memory_increase = final_memory - initial_memory;
    
    // Should not exceed reasonable memory limits
    assert!(memory_increase < 100_000_000); // 100MB limit
}
```

## Test Data Management

### Fixtures

```rust
// Create test game bundles
pub fn create_test_bundle(game_type: &str) -> Bundle {
    match game_type {
        "tic-tac-toe" => Bundle {
            manifest: load_test_manifest("tic-tac-toe.json"),
            entities: load_test_entities("tic-tac-toe-entities.json"),
            zones: load_test_zones("tic-tac-toe-zones.json"),
            actions: load_test_actions("tic-tac-toe-actions.json"),
            phases: None,
        },
        _ => panic!("Unknown test game type: {}", game_type)
    }
}
```

### Mock Data

```rust
pub fn create_mock_game_state() -> Value {
    json!({
        "zones": {
            "board": {
                "type": "grid",
                "cells": [
                    [null, {"entity": "x_token"}, null],
                    [null, {"entity": "o_token"}, null],
                    [null, null, null]
                ]
            }
        },
        "meta": {
            "players": [{"id": "p1"}, {"id": "p2"}],
            "tick": 5,
            "turn": 3,
            "currentPlayer": "p1",
            "gameStatus": {
                "state": "playing",
                "winner": null,
                "tie": false
            }
        }
    })
}
```

## Continuous Integration

### GitHub Actions

```yaml
name: Test Suite
on: [push, pull_request]

jobs:
  test-server:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run Server Tests
        run: |
          cd server
          cargo test --verbose
      
      - name: Run CLI Tests
        run: |
          cd cli
          cargo test --verbose

  test-client:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Install Dependencies
        run: |
          cd clients/react
          pnpm install
      
      - name: Run Client Tests
        run: |
          cd clients/react
          pnpm test --coverage

  integration-tests:
    runs-on: ubuntu-latest
    needs: [test-server, test-client]
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Games
        run: |
          cd cli
          cargo run -- build-all ../games/
      
      - name: Start Server
        run: |
          cd server
          cargo run &
          sleep 10
      
      - name: Run E2E Tests
        run: |
          cd clients/react
          pnpm e2e
```

### Test Coverage

```bash
# Server coverage
cd server
cargo install cargo-tarpaulin
cargo tarpaulin --out Html

# Client coverage
cd clients/react
pnpm test --coverage
```

## Writing New Tests

### Test Guidelines

1. **Test Naming** - Use descriptive test names
   ```rust
   #[test]
   fn test_place_verb_updates_grid_cell_correctly() { ... }
   ```

2. **Arrange-Act-Assert** - Structure tests clearly
   ```rust
   #[test]
   fn test_example() {
       // Arrange
       let mut state = create_test_state();
       let args = json!({"key": "value"});
       
       // Act
       let result = function_under_test(&mut state, &args);
       
       // Assert
       assert!(result.is_ok());
       assert_eq!(state["expected_path"], expected_value);
   }
   ```

3. **Error Cases** - Test both success and failure paths
   ```rust
   #[test]
   fn test_invalid_action_returns_error() {
       let result = apply_invalid_action();
       assert!(result.is_err());
       assert!(result.unwrap_err().contains("expected error message"));
   }
   ```

### Adding Server Tests

1. **Unit Tests** - Add to existing module files
2. **Integration Tests** - Create new files in `tests/` directory
3. **Mock Dependencies** - Use test fixtures for consistent data

### Adding Client Tests

1. **Component Tests** - Test rendering and user interactions
2. **Hook Tests** - Test custom React hooks
3. **Utils Tests** - Test utility functions

### Performance Test Guidelines

1. **Baseline Measurements** - Establish performance baselines
2. **Realistic Load** - Use representative test scenarios
3. **Resource Monitoring** - Track memory and CPU usage
4. **Regression Detection** - Fail tests when performance degrades

## Debugging Tests

### Server Test Debugging

```bash
# Run with debug output
RUST_LOG=debug cargo test test_name -- --nocapture

# Run single test
cargo test test_specific_function -- --exact

# Debug with gdb
cargo test --no-run
gdb target/debug/deps/test_binary
```

### Client Test Debugging

```bash
# Run tests in watch mode
pnpm test --watch

# Debug specific test
pnpm test --testNamePattern="specific test"

# Debug in browser
pnpm test --debug
```

### Common Test Issues

1. **Timing Issues** - Use proper async/await patterns
2. **State Isolation** - Ensure tests don't affect each other
3. **Resource Cleanup** - Clean up connections and files
4. **Flaky Tests** - Add retries for non-deterministic behavior

## Test Metrics

Current test coverage:

- **Server**: 29 tests covering all major functionality
  - Unit tests: 13 (engine verbs, shorthand expansion)
  - Integration tests: 8 (game simulations, multi-game)
  - WebSocket tests: 8 (connections, actions, errors)

- **CLI**: Comprehensive validation and build testing
- **Client**: Component, hook, and integration testing

The automated test suite ensures Bluefelt maintains high quality and reliability as new features are added and existing functionality is improved.
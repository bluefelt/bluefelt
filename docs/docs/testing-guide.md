# Testing Guide

Comprehensive testing is critical for maintaining platform quality and preventing regressions. This guide covers testing strategies, post-implementation validation, and deployment readiness for Bluefelt game development.

## Testing Philosophy

### Core Principles

1. **Regression Prevention**: Every new feature must not break existing functionality
2. **Comprehensive Coverage**: Test every action, edge case, and user flow
3. **Multi-Layer Testing**: Unit, integration, and end-to-end testing
4. **Automated First**: Prefer automated tests over manual testing
5. **Platform Stability**: All existing games must continue working

### Testing Pyramid

```
    🔺 End-to-End Tests
   🔺🔺 Integration Tests  
  🔺🔺🔺 Unit Tests
```

- **Unit Tests**: Individual functions and components
- **Integration Tests**: Game flow and component interaction
- **End-to-End Tests**: Complete user scenarios

## Server-Side Testing

The server includes comprehensive tests covering all major functionality. Currently there are 41 tests total:
- **Unit tests** (18) - Engine verbs, shorthand expansion, path navigation, grid.lineOfMarks functionality
- **Integration tests** (8) - Full game simulation, bundle loading
- **WebSocket tests** (8) - Connection handling, protocol testing
- **Game-specific tests** (7) - Connect Four and Tic-tac-toe game log tests

### Running Server Tests

```bash
cd server

# Run all tests (currently 41 tests)
cargo test

# Run with detailed output
cargo test -- --nocapture

# Run specific test categories
cargo test --lib                     # Unit tests (18)
cargo test --test engine_integration # Integration tests (8)
cargo test --test websocket_tests    # WebSocket tests (8)
cargo test connect_four_test         # Connect Four specific tests
cargo test tic_tac_toe_test         # Tic-tac-toe specific tests

# Run tests in parallel (default)
cargo test -- --test-threads=4

# Run tests serially (for debugging)
cargo test -- --test-threads=1
```

### Test File Structure

Create comprehensive test files for each game:

```
server/src/
├── lib.rs              # Test module exports
├── engine/
│   ├── mod.rs          # Engine tests
│   ├── grid.rs         # Grid-specific tests including lineOfMarks
│   └── verbs.rs        # Verb implementation tests
├── shorthand.rs        # Unit tests for expansion
├── lobby.rs            # Unit tests for lobby management
└── tests/
    ├── engine_integration_tests.rs  # 8 integration tests
    ├── websocket_tests.rs           # 8 WebSocket tests
    ├── connect_four_test.rs         # Connect Four specific tests
    ├── tic_tac_toe_test.rs          # Tic-tac-toe specific tests
    ├── connect_four_game_log_test.rs # Connect Four game log tests
    ├── tic_tac_toe_game_log_test.rs  # Tic-tac-toe game log tests
    └── game_log_location_parsing_test.rs # Shared utilities
```

### Basic Game Setup Tests

Every game should have setup validation:

```rust
#[test]
fn test_game_setup() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("game-name").expect("Failed to get bundle");
    let state = load_initial_state(&bundle);
    
    // Verify initial state
    assert_eq!(state["turn"], 0);
    assert_eq!(state["currentPlayer"], "p1");
    
    // Verify board structure and dimensions
    let board = &state["zones"]["board"];
    assert_eq!(board["cells"].as_array().unwrap().len(), expected_rows);
    
    // Verify initial emptiness
    for row in board["cells"].as_array().unwrap() {
        for cell in row.as_array().unwrap() {
            assert!(cell.is_null());
        }
    }
    
    // Verify action configuration exists
    let actions = bundle.actions.as_array().expect("actions should be an array");
    assert!(!actions.is_empty());
}
```

### Verb-Specific Testing

Test each custom verb thoroughly:

```rust
#[test]
fn test_custom_verb_mechanics() {
    use bluefelt_core::engine::verbs::apply_verb;
    use serde_json::json;
    
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("game-name").expect("Failed to get bundle");
    let mut state = load_initial_state(&bundle);
    
    // Test normal operation
    let args = json!({
        "param1": "value1",
        "param2": 42
    });
    
    let result = apply_verb(&mut state, "customVerb", &args, &bundle);
    assert!(result.is_ok(), "Verb should succeed: {:?}", result);
    
    // Verify state changes
    assert_eq!(state["zones"]["board"]["cells"][0][0]["entity"], "expected_entity");
    
    // Test edge cases
    let invalid_args = json!({ "invalid": "params" });
    let error_result = apply_verb(&mut state, "customVerb", &invalid_args, &bundle);
    assert!(error_result.is_err(), "Should fail with invalid args");
}
```

### Game Flow Testing

Test complete game scenarios:

```rust
#[test]
fn test_complete_game_flow() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("game-name").expect("Failed to get bundle");
    let mut state = load_initial_state(&bundle);
    
    // Simulate a complete game
    let moves = vec![
        (0, 0), (0, 1), (1, 0), (1, 1), (2, 0) // Winning sequence
    ];
    
    for (i, (row, col)) in moves.iter().enumerate() {
        let current_player = if i % 2 == 0 { "p1" } else { "p2" };
        assert_eq!(state["currentPlayer"], current_player);
        
        // Perform move
        let args = json!({
            "location": format!("/zones/board/cells/{}/{}", row, col),
            "entity": format!("mark_{}", current_player)
        });
        
        let result = apply_verb(&mut state, "place", &args, &bundle);
        assert!(result.is_ok());
        
        // Check for win condition after each move
        if i >= 4 { // Minimum moves for a win
            if state["meta"]["gameStatus"]["state"] == "ended" {
                assert_eq!(state["meta"]["gameStatus"]["winner"], "p1");
                break;
            }
        }
    }
}
```

### Game Log Testing

Verify log message generation:

```rust
#[test]
fn test_game_log_messages() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("game-name").expect("Failed to get bundle");
    
    // Test log template exists and is correct
    let actions = bundle.actions.as_array().expect("actions should be an array");
    let action = actions.iter()
        .find(|a| a["id"].as_str() == Some("mainAction"))
        .expect("Action not found");
    
    let log_template = action["ui"]["logTemplate"].as_str()
        .expect("logTemplate should exist");
    
    assert!(log_template.contains("{player}"));
    assert!(log_template.contains("{row}") || log_template.contains("{column}"));
}

#[test]
fn test_log_parameter_replacement() {
    // Test the logic for replacing parameters in log templates
    let test_cases = vec![
        (("/zones/board/cells/0/0", "p1"), "p1 placed at (1, 1)"),
        (("/zones/board/cells/1/2", "p2"), "p2 placed at (2, 3)"),
        (("/zones/board/cells/2/2", "p1"), "p1 placed at (3, 3)"),
    ];
    
    for ((location, player), expected) in test_cases {
        let result = simulate_log_processing("{player} placed at ({row}, {col})", player, location);
        assert_eq!(result, expected);
    }
}
```

### Win Condition Testing

Test every possible way to win:

```rust
#[test]
fn test_horizontal_win() {
    // Test horizontal wins for each row
}

#[test]
fn test_vertical_win() {
    // Test vertical wins for each column
}

#[test]
fn test_diagonal_win() {
    // Test diagonal wins (both directions)
}

#[test]
fn test_tie_condition() {
    // Test games that end in ties
}

#[test]
fn test_no_premature_wins() {
    // Test that wins don't trigger too early
}
```

## Client-Side Testing

### Action Handling Tests

Test action construction and sending:

```typescript
import { describe, it, expect, vi } from 'vitest';
import { useGameActions } from '../hooks/useGameActions';
import { renderHook } from '@testing-library/react';

describe('Game Action Handling', () => {
  it('should construct correct action messages', () => {
    const mockSendMessage = vi.fn().mockReturnValue(true);
    const lobbyState = {
      you: 'p1',
      ui: {
        actionMap: {
          'p1': {
            '/zones/board/cells/1/2': {
              action: 'placeMarker',
              direction: 'Place your mark'
            }
          }
        }
      }
    };

    const { result } = renderHook(() => 
      useGameActions({
        isYourTurn: true,
        lobbyState,
        sendMessage: mockSendMessage
      })
    );

    result.current.handleCellClick(1, 2);

    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'placeMarker',
        args: {
          location: '/zones/board/cells/1/2',
          entity: 'mark_p1'
        }
      })
    );
  });

  it('should handle column-based actions', () => {
    // Test column actions for gravity-based games
  });

  it('should respect turn restrictions', () => {
    // Test that actions only work on player's turn
  });
});
```

### Component Integration Tests

Test how components handle different game states:

```typescript
describe('BoardZone Component', () => {
  it('should render board correctly', () => {
    // Test basic rendering
  });

  it('should detect column actions', () => {
    // Test column action detection for gravity games
  });

  it('should handle click events', () => {
    // Test click handling and event propagation
  });

  it('should display visual affordances', () => {
    // Test visual feedback for valid moves
  });
});
```

### Game Flow Testing

Test complete client-side game scenarios:

```typescript
describe('Game Flow Integration', () => {
  it('should handle complete tic-tac-toe game', () => {
    // Test full game from start to finish
  });

  it('should handle WebSocket state updates', () => {
    // Test state synchronization
  });

  it('should display game results correctly', () => {
    // Test win/tie display
  });
});
```

## CLI Testing

### Running CLI Tests

```bash
cd cli

# Run all CLI tests
cargo test

# Test build process
cargo test build

# Test validation
cargo test validate
```

### Build Process Tests

Test that games build correctly from YAML to JSON:

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

Ensure validation catches missing or invalid fields:

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

## Integration Testing

### Bundle Integration

Test that YAML games compile to working JSON bundles:

```rust
#[test]
fn test_bundle_compilation() {
    // Verify YAML -> JSON compilation
    // Test bundle loading
    // Verify all required fields exist
}

#[test]
fn test_bundle_validation() {
    // Test CLI validation passes
    // Test no missing references
    // Test action configuration completeness
}

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

### WebSocket Integration

Test real-time communication:

```rust
#[test]
fn test_websocket_game_actions() {
    // Start test server
    // Connect WebSocket client
    // Send game actions
    // Verify state updates
    // Test disconnection handling
}
```

### Cross-Game Integration

Test that new games don't break existing ones:

```rust
#[test]
fn test_multiple_game_loading() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    
    // Verify all games load
    assert!(bundles.get_latest("tic-tac-toe").is_some());
    assert!(bundles.get_latest("connect-four").is_some());
    
    // Test they can run simultaneously
}
```

## Regression Testing Strategy

### Automated Regression Tests

After any change, run the full test suite:

```bash
#!/bin/bash
# Full regression test script

echo "🧪 Running Server Tests..."
cd server
cargo test tic_tac_toe
cargo test connect_four
cargo test engine_integration
cargo test websocket

echo "🧪 Running Client Tests..."
cd ../clients/react
pnpm test TicTacToeGameFlow
pnpm test ConnectFourColumnActions

echo "✅ All tests passed!"
```

### Manual Regression Checklist

For each existing game, verify:

- [ ] Game loads without errors
- [ ] Basic actions work (place, move, etc.)
- [ ] Turn advancement works
- [ ] Win conditions trigger correctly
- [ ] Game log shows proper messages
- [ ] Visual affordances work
- [ ] Mobile experience is smooth
- [ ] No console errors

### Performance Regression Testing

Monitor for performance degradation:

```rust
#[test]
fn test_action_performance() {
    let start = std::time::Instant::now();
    
    // Perform 1000 actions
    for _ in 0..1000 {
        perform_test_action();
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000, "Actions too slow: {:?}", duration);
}
```

## Performance Testing

### Load Testing

Test the system under concurrent load:

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

Ensure the system doesn't leak memory:

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

### Performance Benchmarks

Set baseline performance metrics:

```rust
#[bench]
fn bench_verb_application(b: &mut Bencher) {
    let mut state = create_test_state();
    let args = create_test_args();
    
    b.iter(|| {
        apply_verb(&mut state.clone(), "place", &args);
    });
}
```

## Test Data Management

### Deterministic Test Data

Use consistent test data for reproducible results:

```rust
fn create_test_game_state() -> Value {
    json!({
        "turn": 0,
        "currentPlayer": "p1",
        "zones": {
            "board": {
                "cells": [
                    [null, null, null],
                    [null, null, null], 
                    [null, null, null]
                ]
            }
        }
    })
}
```

### Test Scenarios Library

Maintain a library of test scenarios:

```rust
pub struct TestScenarios;

impl TestScenarios {
    pub fn tic_tac_toe_near_win() -> Value { /* ... */ }
    pub fn connect_four_full_column() -> Value { /* ... */ }
    pub fn complex_board_state() -> Value { /* ... */ }
}
```

## Continuous Integration

### Automated Test Pipeline

```yaml
# .github/workflows/test.yml
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

### Test Coverage Requirements

Maintain minimum coverage thresholds:

- **Server code**: 80% line coverage
- **Client components**: 70% line coverage
- **Critical paths**: 95% coverage (game actions, win detection)

### Measuring Test Coverage

```bash
# Server coverage using tarpaulin
cd server
cargo install cargo-tarpaulin
cargo tarpaulin --out Html

# Client coverage using Vitest
cd clients/react
pnpm test --coverage
```

### Current Test Metrics

The platform maintains comprehensive test coverage:

- **Server**: 41 tests covering all major functionality
  - Unit tests: 18 (engine verbs, shorthand expansion)
  - Integration tests: 8 (game simulations, multi-game)
  - WebSocket tests: 8 (connections, actions, errors)
  - Game-specific tests: 7 (Connect Four, Tic-tac-toe)

- **CLI**: Comprehensive validation and build testing
- **Client**: Component, hook, and integration testing

## Testing Tools and Utilities

### Custom Test Helpers

```rust
// Test utilities
pub fn assert_game_state(state: &Value, expected_player: &str, expected_turn: u64) {
    assert_eq!(state["currentPlayer"], expected_player);
    assert_eq!(state["turn"], expected_turn);
}

pub fn simulate_action(state: &mut Value, action: &str, args: &Value) -> Result<(), String> {
    // Helper for simulating actions in tests
}
```

### Mock Objects

```typescript
// Client test mocks
export const mockLobbyState = {
  you: 'testPlayer',
  ui: { actionMap: {} },
  game: { currentPlayer: 'testPlayer' }
};

export const mockWebSocket = {
  send: vi.fn(),
  close: vi.fn(),
  readyState: WebSocket.OPEN
};
```

### Test Fixtures and Mock Data

Create reusable test data for consistency:

```rust
// Server test fixtures
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

## Writing Effective Tests

### Test Naming Guidelines

Use descriptive test names that explain what is being tested:

```rust
// Good test names
#[test]
fn test_place_verb_updates_grid_cell_correctly() { ... }

#[test]
fn test_invalid_location_returns_error() { ... }

#[test]
fn test_horizontal_win_detection_for_three_in_row() { ... }

// Bad test names
#[test]
fn test_1() { ... }

#[test]
fn test_place() { ... }
```

### Test Organization

Follow the Arrange-Act-Assert pattern:

```rust
#[test]
fn test_example() {
    // Arrange - Set up test data and environment
    let mut state = create_test_state();
    let args = json!({"key": "value"});
    
    // Act - Perform the action being tested
    let result = function_under_test(&mut state, &args);
    
    // Assert - Verify the results
    assert!(result.is_ok());
    assert_eq!(state["expected_path"], expected_value);
}
```

### Test Both Success and Failure

Always test error conditions:

```rust
#[test]
fn test_valid_action_succeeds() {
    let result = apply_valid_action();
    assert!(result.is_ok());
}

#[test]
fn test_invalid_action_returns_error() {
    let result = apply_invalid_action();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expected error message"));
}
```

## Best Practices Summary

### Do's ✅

- ✅ Test every new feature comprehensively
- ✅ Run regression tests after every change
- ✅ Use descriptive test names and error messages
- ✅ Test edge cases and error conditions
- ✅ Mock external dependencies appropriately
- ✅ Maintain test data consistency
- ✅ Test both happy path and error scenarios
- ✅ Use automated testing wherever possible

### Don'ts ❌

- ❌ Skip testing because "it's just a small change"
- ❌ Write tests that depend on external services
- ❌ Ignore test failures or flaky tests
- ❌ Test implementation details instead of behavior
- ❌ Write overly complex tests that are hard to understand
- ❌ Forget to test game log message generation
- ❌ Skip manual testing of visual components
- ❌ Deploy without running the full test suite

## Troubleshooting Test Issues

### Common Test Failures

**Bundle loading failures**
- Check that `../bundles` directory exists
- Rebuild bundles: `./cli/target/debug/bluefelt-cli build-all`
- Verify YAML syntax in game files

**WebSocket test failures**
- Check test server startup
- Verify port availability
- Test connection timing

**Flaky integration tests**
- Add proper wait conditions
- Use deterministic test data
- Avoid timing-dependent assertions

### Debugging Test Failures

#### Server Test Debugging

```bash
# Run with debug output
RUST_LOG=debug cargo test test_name -- --nocapture

# Run single test
cargo test test_specific_function -- --exact

# Debug with gdb
cargo test --no-run
gdb target/debug/deps/test_binary
```

#### Client Test Debugging

```bash
# Run tests in watch mode
pnpm test --watch

# Debug specific test
pnpm test --testNamePattern="specific test"

# Debug in browser
pnpm test --debug
```

#### Common Debugging Steps

1. **Read error messages carefully** - They often contain the exact issue
2. **Use debug output** - `println!` in Rust, `console.log` in TypeScript
3. **Test components in isolation** - Narrow down the problem
4. **Check test data setup** - Ensure initial state is correct
5. **Verify mock configurations** - Mocks may not match real behavior

### Common Test Issues and Solutions

#### Timing Issues
- **Problem**: Tests fail intermittently due to async timing
- **Solution**: Use proper async/await patterns and explicit waits

```rust
// Bad
ws.send(message);
let response = ws.next(); // May not be ready

// Good
ws.send(message).await;
let response = ws.next().await.unwrap();
```

#### State Isolation
- **Problem**: Tests affect each other's state
- **Solution**: Clean up after each test

```rust
#[teardown]
fn cleanup() {
    // Reset global state
    // Clear test databases
    // Close connections
}
```

#### Resource Cleanup
- **Problem**: Tests leave files or connections open
- **Solution**: Use RAII patterns or explicit cleanup

```rust
// Use drop traits for automatic cleanup
struct TestResource {
    file: File,
}

impl Drop for TestResource {
    fn drop(&mut self) {
        // Cleanup happens automatically
    }
}
```

#### Flaky Tests
- **Problem**: Tests pass/fail randomly
- **Solution**: Make tests deterministic

```rust
// Bad: Time-dependent
assert!(timestamp > now());

// Good: Mock time
let mock_time = MockTime::new();
assert!(timestamp > mock_time.now());
```

## Post-Implementation Testing

After implementing any new game or platform feature, comprehensive testing ensures backward compatibility and system stability. This checklist prevents regressions and maintains platform quality.

### Overview

**Critical Principle**: Every change to the platform must be validated against ALL existing functionality. A single new game should never break existing games or core platform features.

#### Testing Scope

- ✅ **New Implementation**: The feature you just built
- ✅ **All Existing Games**: Every game must continue working
- ✅ **Core Platform**: Server, client, WebSocket, bundling
- ✅ **Integration Points**: Where systems connect
- ✅ **User Experience**: Visual, performance, accessibility

### Pre-Testing Setup

#### Environment Preparation

```bash
# Ensure clean build state
cd /path/to/bluefelt

# Rebuild all game bundles
./cli/target/debug/bluefelt-cli build-all

# Ensure server builds
cd server && cargo build

# Ensure client builds  
cd ../clients/react && pnpm build
```

#### Test Data Preparation

- Clear browser cache and local storage
- Reset any test databases
- Ensure test environments are clean
- Verify no lingering WebSocket connections

### Manual Testing Phase

#### Core Platform Testing

**Server Startup**
- [ ] Server starts without errors
- [ ] All games load successfully
- [ ] WebSocket endpoint responds
- [ ] No error logs on startup

**Bundle System**
- [ ] YAML games compile to JSON bundles
- [ ] Bundle validation passes for all games
- [ ] Server loads bundles without errors
- [ ] Bundle hot-reloading works (if applicable)

#### Game-by-Game Testing

For **EACH existing game**, complete this checklist:

**Tic-Tac-Toe**
- [ ] **Game loads**: No console errors, renders properly
- [ ] **Basic actions**: Can place marks in cells
- [ ] **Turn switching**: Alternates between players correctly
- [ ] **Win detection**: Horizontal, vertical, diagonal wins work
- [ ] **Tie detection**: Full board without winner shows tie
- [ ] **Game log**: Shows messages like "alice placed their mark at (2, 3)"
- [ ] **Visual feedback**: Hover states, click feedback work
- [ ] **Mobile responsive**: Works on mobile devices
- [ ] **Accessibility**: Keyboard navigation works

**Connect 4**
- [ ] **Game loads**: No console errors, renders properly
- [ ] **Column actions**: Can click column headers to drop discs
- [ ] **Gravity mechanics**: Discs drop to lowest available position
- [ ] **Full columns**: Cannot drop in full columns
- [ ] **Turn switching**: Alternates between players correctly  
- [ ] **Win detection**: Horizontal, vertical, diagonal (4-in-a-row) works
- [ ] **Tie detection**: Full board without winner shows tie
- [ ] **Game log**: Shows messages like "bob dropped a disc in column 4"
- [ ] **Visual affordances**: Column drop zones appear and work
- [ ] **Mobile responsive**: Column headers work on touch

**[Additional Games]**
Repeat the above pattern for each game in the platform.

#### Cross-Game Testing

**Multiple Game Sessions**
- [ ] Can have multiple games running simultaneously
- [ ] Switching between games works correctly
- [ ] Game state is isolated between games
- [ ] No interference between different game types

**WebSocket Stability**
- [ ] WebSocket connections remain stable across games
- [ ] Real-time updates work for all games
- [ ] Reconnection handling works properly
- [ ] No message cross-contamination between games

### User Experience Testing

#### Game Log Functionality
For each game, verify:
- [ ] Actions generate appropriate log messages
- [ ] Coordinates are properly formatted (1-indexed)
- [ ] Player names appear correctly
- [ ] Messages are clear and informative
- [ ] No literal placeholders appear (like `{row}`)

#### Visual Affordances
For each game, verify:
- [ ] Interactive elements are clearly indicated
- [ ] Hover states work on desktop
- [ ] Touch feedback works on mobile
- [ ] Disabled states are visually distinct
- [ ] Loading states appear during actions

#### Performance
- [ ] Games load quickly (< 3 seconds)
- [ ] Actions respond immediately (< 100ms)
- [ ] No memory leaks during extended play
- [ ] Smooth animations and transitions
- [ ] No frame drops or stuttering

### Device and Browser Testing

#### Desktop Testing
- [ ] **Chrome**: All functionality works
- [ ] **Firefox**: All functionality works  
- [ ] **Safari**: All functionality works (if on macOS)
- [ ] **Edge**: All functionality works

#### Mobile Testing
- [ ] **Mobile Chrome**: Touch interactions work
- [ ] **Mobile Safari**: Touch interactions work
- [ ] **Responsive layout**: Games adapt to screen size
- [ ] **Touch targets**: Adequate size and spacing
- [ ] **Portrait/landscape**: Works in both orientations

#### Accessibility Testing
- [ ] **Keyboard navigation**: Can play using only keyboard
- [ ] **Screen reader**: Announces game state clearly
- [ ] **Color contrast**: Meets WCAG guidelines
- [ ] **Focus indicators**: Clear focus outlines
- [ ] **Motion preferences**: Respects reduced motion

### End-to-End Integration Testing

For each game, test complete scenarios:

#### Tic-Tac-Toe Full Game
- [ ] Start new game → Player 1 wins → Game log correct
- [ ] Start new game → Player 2 wins → Game log correct  
- [ ] Start new game → Tie game → Game log correct
- [ ] Invalid moves are prevented and show feedback

#### Connect 4 Full Game
- [ ] Start new game → Horizontal win → Game log correct
- [ ] Start new game → Vertical win → Game log correct
- [ ] Start new game → Diagonal win → Game log correct
- [ ] Full columns prevent further drops
- [ ] Column drop zones work correctly

### Multi-Player Testing
- [ ] Two players can join and play
- [ ] Turn order is enforced correctly
- [ ] Both players see consistent game state
- [ ] Player disconnection is handled gracefully

### Performance Under Load
- [ ] Multiple concurrent games don't affect performance
- [ ] Rapid actions don't cause issues
- [ ] Memory usage remains stable
- [ ] No resource leaks

### Error Condition Testing

#### Network Issues
- [ ] WebSocket disconnection is handled gracefully
- [ ] Reconnection restores game state correctly
- [ ] Actions during disconnection are queued or rejected appropriately

#### Invalid Actions
- [ ] Invalid moves show appropriate error messages
- [ ] Server rejects malformed action requests
- [ ] Client prevents obviously invalid actions
- [ ] Error states don't break the game

#### Edge Cases
- [ ] Very large boards (if applicable) perform adequately
- [ ] Games with maximum player counts work correctly
- [ ] Boundary conditions (corners, edges) work properly
- [ ] Special game states (near-win, tied positions) work correctly

### Documentation Verification

#### User-Facing Documentation
- [ ] Game rules are accurate and complete
- [ ] Controls and interactions are documented
- [ ] Any new features are documented
- [ ] Screenshots/examples are up to date

#### Developer Documentation
- [ ] Implementation guide reflects new patterns
- [ ] Testing documentation is updated
- [ ] API changes are documented
- [ ] Best practices include new learnings

## Deployment Readiness Checklist

### Technical Validation
- [ ] ✅ All automated tests pass
- [ ] ✅ All manual tests pass
- [ ] ✅ No console errors in any game
- [ ] ✅ Performance meets standards
- [ ] ✅ Accessibility requirements met
- [ ] ✅ Mobile experience is smooth

### Quality Assurance
- [ ] ✅ Code review completed
- [ ] ✅ Security review completed (if applicable)
- [ ] ✅ No known bugs or issues
- [ ] ✅ Rollback plan prepared
- [ ] ✅ Monitoring alerts configured

### User Experience
- [ ] ✅ Games feel responsive and polished
- [ ] ✅ Visual design is consistent
- [ ] ✅ Game log messages are informative
- [ ] ✅ Error messages are helpful
- [ ] ✅ Loading states are appropriate

## Post-Deployment Monitoring

After deployment, monitor:

### System Health
- [ ] Server error rates remain normal
- [ ] WebSocket connection success rates
- [ ] Database performance (if applicable)
- [ ] Memory and CPU usage patterns

### User Behavior
- [ ] Game completion rates
- [ ] Error frequencies
- [ ] Support ticket patterns
- [ ] User feedback

### Performance Metrics
- [ ] Page load times
- [ ] Action response times
- [ ] WebSocket latency
- [ ] Client-side error rates

## Rollback Procedures

If issues are discovered:

### Immediate Actions
1. **Assess severity**: Critical bugs require immediate rollback
2. **Document issues**: Clear reproduction steps
3. **Notify stakeholders**: Inform relevant team members
4. **Execute rollback**: Revert to previous working version

### Investigation Process
1. **Reproduce issues**: In development environment
2. **Identify root cause**: Code, configuration, or environment
3. **Develop fix**: Address underlying problem
4. **Test fix**: Complete testing cycle again
5. **Re-deploy**: With proper validation

## Tools and Scripts

### Comprehensive Test Runner

```bash
#!/bin/bash
# comprehensive-test.sh

echo "🧪 Starting Comprehensive Test Suite..."

# Server tests
echo "Testing server..."
cd server
if ! cargo test; then
    echo "❌ Server tests failed"
    exit 1
fi

# Client tests  
echo "Testing client..."
cd ../clients/react
if ! pnpm test; then
    echo "❌ Client tests failed"
    exit 1
fi

echo "✅ All tests passed!"
```

### Manual Test Checklist Generator

```bash
#!/bin/bash
# generate-test-checklist.sh

echo "Generating test checklist for current games..."

for game in ../games/*/; do
    game_name=$(basename "$game")
    echo "- [ ] Test $game_name complete functionality"
    echo "- [ ] Verify $game_name game log messages"
    echo "- [ ] Check $game_name visual affordances"
done
```

## Best Practices Summary

### Before Testing
- ✅ Clean build environment
- ✅ Update all dependencies
- ✅ Clear caches and temporary data
- ✅ Review changes for potential impacts

### During Testing
- ✅ Test systematically, don't skip steps
- ✅ Document any issues found
- ✅ Test both happy path and error conditions
- ✅ Verify on multiple devices/browsers

### After Testing
- ✅ Update documentation with learnings
- ✅ Add new test cases for future coverage
- ✅ Share results with team
- ✅ Monitor post-deployment metrics

## Related Documentation

- [Game Implementation Guide](./game-implementation-guide.md) - Overall development process
- [Game Log Parameters](./game-log-parameters.md) - Log message testing
- [State Structure](./state-structure.md) - Understanding game state
- [Visual Affordances](./visual-affordances.md) - UI interaction testing

Remember: Comprehensive testing is the final safeguard ensuring platform stability. Never skip testing - it's an investment in platform stability and developer confidence. Every test written today prevents bugs tomorrow!
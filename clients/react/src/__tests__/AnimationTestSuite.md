# Animation System Test Suite

This directory contains comprehensive tests for the Bluefelt animation system, ensuring all animations continue working properly as the codebase evolves.

## Test Files

### 1. **AnimationEngineTests.test.tsx**
Tests the core animation engine functionality:
- Entity spawn animations with retry logic
- Game-specific animation logic (tic-tac-toe uses scale, not spin)
- Sound volume differentiation for player vs opponent pieces
- Animation queue management and deduplication
- Gravity drop animations for Connect 4 style games
- Animation speed and configuration handling
- Game end animations with victory celebrations
- Animation cancellation and status reporting
- Complete game coverage (all pieces animated)

### 2. **PatchAnalyzerTests.test.tsx**
Tests the patch analysis system:
- Detection of animatable patches
- Extraction of animation metadata
- Gravity drop detection for Connect 4
- Player ownership identification
- Different patch operation types (add, remove, replace)
- Animation priority and sorting
- Server hint processing
- Animation deduplication logic

### 3. **AnimationIntegrationTests.test.tsx**
Integration tests with real game scenarios:
- Complete tic-tac-toe game animations
- Connect 4 gravity drop sequences
- Animation context integration
- Error handling and edge cases
- Sound integration with volume differentiation
- Performance testing with large batches
- React rendering delay handling

### 4. **AudioManagerTests.test.tsx**
Tests the audio system:
- Sound playback with configuration
- Animation-specific sound effects
- Placeholder sound generation
- Sound preloading and caching
- Pitch adjustment capabilities
- Configuration updates
- Error handling for missing audio context

### 5. **AnimationSystem.test.tsx**
Visual component and DOM integration tests:
- ViewZone fade-in and stagger animations
- ActionIndicator visual feedback
- MultiStepActionDisplay progress animations
- AnimationSettings UI controls
- DOM element finding and retry logic
- Animation engine patch processing
- Complete game animation coverage

## Key Test Scenarios

### Entity Spawn Animations
- Verifies that new game pieces trigger spawn animations
- Tests retry logic for finding DOM elements (important for React rendering delays)
- Ensures game-specific animations (tic-tac-toe uses scale, others use spin)

### Sound Differentiation
- Player pieces play at full volume
- Opponent pieces play at 70% volume
- Turn notifications respect user preferences

### Gravity Drop Detection
- Connect 4 pieces detect empty cells above
- Animation metadata includes from/to positions
- Drop animation uses different keyframes

### Complete Game Coverage
- No animations are missed during a full game
- All pieces are animated in the correct order
- Game end animations trigger appropriately

## Running the Tests

```bash
# Run all animation tests
pnpm test AnimationEngineTests
pnpm test PatchAnalyzerTests
pnpm test AnimationIntegrationTests
pnpm test AudioManagerTests
pnpm test AnimationSystem

# Run with coverage
pnpm test:coverage

# Run in watch mode
pnpm test:watch
```

## Test Coverage Goals

The test suite aims for:
- 90%+ code coverage of animation system files
- All animation types tested
- All edge cases handled
- Integration with real game scenarios
- Performance under load

## Mock Requirements

Tests use the following mocks:
- `Element.prototype.animate` - Web Animations API
- `AudioContext` - Web Audio API
- `document.querySelector` - DOM queries
- `requestAnimationFrame` - Animation timing

## Adding New Tests

When adding new animation features:
1. Add unit tests to the appropriate test file
2. Add integration tests showing real usage
3. Test error cases and edge conditions
4. Verify performance implications
5. Update this README with new test scenarios
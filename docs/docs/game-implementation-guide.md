# Game Implementation Guide

This guide outlines the standard process for implementing new games on the Bluefelt platform. Follow these steps to ensure consistency, quality, and maintainability.

## Overview

Each game implementation follows an 8-step process designed to ensure thorough planning, proper implementation, and comprehensive testing before release.

## Step 1: Document Game Rules

Create a `RULES.md` file in the game's directory (`games/<game-name>/<version>/RULES.md`) that includes:

### Required Sections:
- **Game Overview**: Brief description and objective
- **Players**: Number of players (min/max)
- **Equipment**: What pieces/cards/boards are used
- **Setup**: Initial game state
- **Gameplay**: Turn structure and allowed actions
- **Win Conditions**: How the game ends
- **Special Rules**: Edge cases, variations
- **Quick Start**: Simple explanation for new players
- **Visual Examples**: ASCII diagrams or descriptions of key game states

### Example Structure:
```markdown
# [Game Name] Rules

## Overview
[Brief description and objective]

## Players
- Minimum: X
- Maximum: Y

## Equipment
- [List all game components]

## Setup
[Initial board/card state]

## Gameplay
### Turn Structure
1. [Phase 1]
2. [Phase 2]

### Allowed Actions
- [Action 1]: [Description]
- [Action 2]: [Description]

## Win Conditions
- [Primary win condition]
- [Alternative endings]

## Special Rules
- [Edge case 1]
- [Edge case 2]

## Quick Start
[2-3 sentence summary for new players]
```

## Step 2: Analyze System Requirements

Before implementation, analyze how the game maps to Bluefelt's architecture:

### 2.1 Entities Analysis
Review needed entities and their properties:
- **New entity types needed?**
- **Properties required** (position, owner, state, etc.)
- **Visual representation** (marks, cards, pieces)
- **Behavior patterns** (can move, can capture, can flip)

### 2.2 Zones Analysis
Determine zone requirements:
- **Zone types**: Grid (2D array), List (1D array), Single slot
- **Zone properties**: Size, visibility, ownership
- **Special behaviors**: Gravity, stacking, shuffling
- **Interaction rules**: What can be placed where

### 2.3 Actions/Phases Analysis
Map game rules to actions:
- **Player actions**: What players can do on their turn
- **Automatic actions**: What happens as a result (captures, scoring)
- **Phase structure**: Game phases and transitions
- **Validation rules**: When actions are allowed
- **WebAssembly hooks**: Complex logic that needs custom code

### 2.4 Reusability Check
Look for patterns from existing games:
- Check `games/` directory for similar mechanics
- Identify reusable verbs (place, move, capture, draw)
- Note successful UI patterns from other games

## Step 3: UI/UX Design Considerations

Plan the client-side rendering:

### Desktop Considerations
- **Board layout**: How to display the main game area
- **Player areas**: Where to show hands, scores, captured pieces
- **Action feedback**: How to show valid moves, selections
- **Game state display**: Turn indicator, phase display, scores

### Mobile Considerations
- **Touch targets**: Minimum 44x44px for clickable areas
- **Responsive layout**: How zones stack on small screens
- **Gesture support**: Tap, drag, pinch-to-zoom needs
- **Orientation**: Portrait vs landscape requirements

### Accessibility
- **Keyboard navigation**: Tab order, arrow key movement
- **Screen reader support**: Descriptive labels for game state
- **Color contrast**: Don't rely solely on color
- **Visual indicators**: Multiple ways to show game state

### Performance
- **Large boards**: Pagination or viewport limiting
- **Many entities**: Efficient rendering strategies
- **Animation needs**: State transitions, piece movement
- **Update frequency**: Real-time vs turn-based updates

## Step 4: Implementation Planning

Create an implementation summary:

### 4.1 New Additions Needed
List all new components:
- **Server-side**: New verbs, validation logic, state structures
- **Client-side**: New components, rendering logic, interactions
- **Shared**: New types, constants, utilities

### 4.2 Integration Strategy
Plan how to add features while maintaining:
- **Backward compatibility**: Don't break existing games
- **Declarative design**: Configuration over code
- **Reusability**: Make components generic when possible
- **Performance**: Avoid expensive operations

### 4.3 Implementation Phases
Break down into manageable chunks:
1. **MVP**: Minimum playable version
2. **Core features**: Essential game mechanics
3. **Polish**: Animations, better UX
4. **Advanced**: Optional rules, variations

## Step 5: Implementation

### 5.1 Create Game Directory
```bash
mkdir -p games/<game-name>/1.0
cd games/<game-name>/1.0
```

### 5.2 Create Game Files
Start with YAML files:
- `manifest.yaml`: Game metadata
- `entities.yaml`: Game entities and setup
- `actions.yaml`: Player and automatic actions
- `phases.yaml`: Game flow phases
- `zones.yaml`: Board and player areas

### 5.3 Implementation Order
1. **manifest.yaml**: Define game metadata
2. **zones.yaml**: Set up game areas
3. **entities.yaml**: Define pieces and initial setup
4. **phases.yaml**: Create game flow
5. **actions.yaml**: Implement game mechanics
6. **hooks.wasm**: (Optional) Complex logic

### 5.4 Client-side Updates
If needed, update React components:
- Check if existing components handle the game
- Add new zone renderers if required
- Ensure mobile responsiveness

## Step 6: Testing

### 6.1 Server Tests
Create `server/tests/<game_name>_test.rs`:
- **Setup tests**: Verify initial state
- **Action tests**: Each action works correctly
- **Win condition tests**: All ways to win/draw
- **Edge case tests**: Invalid moves, special rules
- **Integration tests**: Full game simulation

### 6.2 Client Tests
Add tests in `clients/react/src/__tests__/`:
- **Rendering tests**: Components display correctly
- **Interaction tests**: Clicks/taps work
- **State sync tests**: Updates apply properly
- **Mobile tests**: Touch interactions work

### 6.3 Test Scenarios
Cover these scenarios:
- **2+ player games**: All player counts
- **Early/mid/late game**: Different game states
- **Win conditions**: Every way to end
- **Invalid actions**: Proper error handling
- **Concurrent actions**: Race conditions

## Step 7: Validation

### 7.1 Build and Validate
```bash
# Build the game bundle
cd cli
cargo run -- build

# Validate the game files
cargo run -- validate
```

### 7.2 Manual Testing Checklist
- [ ] Game starts correctly
- [ ] All actions work as expected
- [ ] Win conditions trigger properly
- [ ] UI updates reflect game state
- [ ] Mobile experience is smooth
- [ ] No console errors
- [ ] Performance is acceptable

### 7.3 Cross-browser Testing
Test on:
- Chrome/Edge (Chromium)
- Firefox
- Safari (macOS/iOS)
- Mobile browsers

## Step 8: User Testing

### 8.1 Testing Protocol
1. Start fresh (no prior knowledge assumed)
2. Play through multiple games
3. Try edge cases
4. Test on different devices
5. Note any confusion or bugs

### 8.2 Feedback Collection
Document:
- **Bugs found**: Steps to reproduce
- **UX issues**: Confusing interactions
- **Performance problems**: Slow updates, lag
- **Feature requests**: Missing functionality
- **Positive feedback**: What works well

### 8.3 Iteration
Based on feedback:
1. Fix critical bugs
2. Improve UX pain points
3. Consider feature additions
4. Update documentation

## Best Practices

### Do's
- ✅ Use existing patterns from other games
- ✅ Keep games declarative (YAML-driven)
- ✅ Write comprehensive tests
- ✅ Consider mobile from the start
- ✅ Document edge cases
- ✅ Make reusable components

### Don'ts
- ❌ Hardcode game-specific logic in shared components
- ❌ Skip testing edge cases
- ❌ Ignore mobile experience
- ❌ Break existing games
- ❌ Use complex logic when simple will do
- ❌ Forget about accessibility

## Examples

### Simple Game (like Tic-Tac-Toe)
- Few entities (marks)
- Single zone (board)
- Simple actions (place)
- Basic win condition (line of marks)

### Medium Complexity (like Checkers)
- Multiple entity types (pieces, kings)
- Multiple zones (board, captured)
- Complex actions (move, capture, promote)
- Multiple win conditions

### Complex Game (like Card Games)
- Many entities (full deck)
- Many zones (deck, hands, play area)
- Complex phases (deal, play, score)
- WebAssembly hooks for rules

## Troubleshooting

### Common Issues

**Game doesn't load**
- Check manifest.yaml syntax
- Verify all required fields
- Run `cargo run -- validate`

**Actions don't work**
- Check action conditions
- Verify zone paths
- Test with server logs

**UI doesn't update**
- Check WebSocket connection
- Verify patch paths
- Look for console errors

**Performance problems**
- Profile render cycles
- Optimize large boards
- Reduce entity count

## Next Steps

After implementing a game:
1. Update this guide with lessons learned
2. Add reusable patterns to SDK
3. Share components with other games
4. Consider game variations
5. Plan tournament features

Remember: Each game makes the platform better for the next one!
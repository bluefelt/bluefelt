# Game Implementation Guide

This is the comprehensive reference for implementing games on the Bluefelt platform.

## Quick Reference Workflow

Every game implementation follows these 8 steps:

1. **Document Rules** → Create RULES.md
2. **Analyze Requirements** → Map game mechanics to engine capabilities  
3. **Identify Gaps** → Find what the engine can't do yet
4. **Plan Implementation** → Design your approach
5. **Build Game Files** → Create YAML configurations
6. **Add Tests** → Ensure everything works
7. **Validate & Deploy** → Build bundles and run
8. **User Testing** → Get feedback and iterate

## Overview

This guide provides detailed information about each step of the game implementation process.

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
- Identify reusable verbs (place, move, capture, draw, placeWithGravity)
- Note successful UI patterns from other games
- Consider if new verbs are needed for novel mechanics

### 2.5 Game Log Planning
Plan how player actions will appear in the game log:
- **Action descriptions**: What message shows when a player acts
- **Parameter format**: What coordinates/arguments need to appear
- **User-friendly display**: 1-indexed coordinates, column names, etc.
- **Template format**: How to structure log message templates

### 2.6 Visual Affordances Planning
Consider how players will interact with the game:
- **Click targets**: Cells, columns, cards, pieces
- **Visual feedback**: Highlighting valid moves, selections
- **Interaction patterns**: Direct cell clicks vs. column drops
- **Generic components**: How existing UI components can be reused

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

## Step 4: Implementation Planning (Critical Phase)

**This is the most important step!** Before writing any code, analyze what the engine can and cannot do. Each game implementation is an opportunity to improve the Bluefelt engine for all developers.

### 4.1 Analyze Engine Capabilities

#### Built-in Verbs Available:
- **place**: Place entity at specific location
- **placeWithGravity**: Place with automatic falling (Connect 4 style)
- **moveEntity**: Move entity between locations
- **draw**: Move entities from deck to hand/zone
- **nextTurn**: Advance turn and update current player
- **setPhase**: Change game phase
- **grid.lineOfMarks**: Check for winning lines (configurable for 3-in-a-row, 4-in-a-row, etc.)

#### What CAN be done declaratively:
- Turn-based gameplay with automatic turn advancement
- Grid-based games with placement and movement
- Win conditions based on line patterns
- Card/deck mechanics
- Simple phase transitions
- Automatic actions triggered by other actions
- Visibility rules per zone

#### What CANNOT currently be done declaratively:
- Complex phase management (e.g., "place 3 pieces then switch to movement phase")
- Conditional action availability based on game state
- Resource tracking tied to phase transitions
- Custom movement patterns (e.g., chess pieces, checkers jumping)
- Capture mechanics with special rules
- Area control/influence calculations

### 4.2 Identify Missing Capabilities

For each mechanic your game needs that isn't supported:

1. **Document the gap clearly**:
   - What exactly do you need?
   - Why can't current verbs handle it?
   - What games would benefit from this feature?

2. **Propose a reusable solution**:
   - Design a new verb that's generic enough for multiple games
   - Consider parameters that make it flexible
   - Think about edge cases

3. **Examples of good verb design**:
   - `grid.lineOfMarks` - Works for tic-tac-toe (3), Connect 4 (4), Gomoku (5)
   - `placeWithGravity` - Works for Connect 4, Plinko-style games
   - `moveEntity` - Generic enough for cards, tokens, pieces

### 4.3 Implementation Strategy Decision Tree

```
Does your game mechanic exist as a built-in verb?
├─ YES → Use it with appropriate parameters
└─ NO → Can it be generalized for multiple games?
    ├─ YES → Work with maintainers to add new verb
    └─ NO → Is it truly unique to this one game?
        ├─ YES → Consider WebAssembly hook (last resort)
        └─ NO → Rethink the generalization
```

### 4.4 Common Patterns and Solutions

#### Phase Management Pattern
**Problem**: "Game needs different rules in different phases"
**Solution**: Use automatic actions with conditions to manage phase transitions
```yaml
# Action that checks state and transitions phases
- id: checkPhaseTransition
  auto: true  # Runs automatically
  when:
    - condition: "zone.count"
      with:
        zone: "/zones/board"
        entity: "piece_{player}"
        operator: ">="
        value: 6
  then:
    - action: "setPhase"
      with:
        phaseSet: "game"
        phase: "movement"

# Trigger this check after relevant actions
- id: placeToken
  uses: "place"
  then:
    - action: "checkPhaseTransition"
    - action: "advanceTurn"
```

This pattern allows complex phase logic while keeping it declarative.

#### Resource Tracking Pattern
**Problem**: "Need to track how many pieces each player has placed"
**Current Limitation**: No built-in counter that affects game flow
**Proposed Solution**: Add resource conditions and verbs
```yaml
# Potential new verb
- uses: "incrementResource"
  with:
    resource: "piecesPlaced_{player}"
    amount: 1
```

#### Conditional Actions Pattern
**Problem**: "Action should only be available if player has done X"
**Current Limitation**: Limited conditional system
**Proposed Solution**: Extend condition system

### 4.5 Working with Maintainers

When you identify a gap:

1. **Document the use case** in the implementation guide
2. **Propose the solution** with YAML examples
3. **Show how multiple games** would benefit
4. **Test with simplified version** first
5. **Iterate on the design** based on feedback

### 4.6 Real Examples of Gap Identification

#### Example 1: Three Men's Morris
**Game Requirement**: Players place 3 pieces each, then move pieces

**Initial Assessment**:
1. **Phase transitions based on game state** - Can use action conditions!
2. **Different actions in different phases** - Can use phase conditions!
3. **Track pieces placed** - Can count entities in zones!

**Working Solution Using Existing Patterns**:
```yaml
# Place action (available in placement phase)
- id: placeToken
  uses: "place"
  when:
    - condition: "zone.isEmpty"
      with:
        zone: "{target}"
    - condition: "player.isActor"
  then:
    - action: "checkPhaseTransition"
    - action: "advanceTurn"

# Automatic phase check
- id: checkPhaseTransition
  auto: true
  when:
    - condition: "zone.count"
      with:
        zone: "/zones/board"
        entity: "piece_p1"
        operator: ">="
        value: 3
    - condition: "zone.count"
      with:
        zone: "/zones/board"
        entity: "piece_p2"
        operator: ">="
        value: 3
  then:
    - action: "setPhase"
      with:
        phaseSet: "game"
        phase: "movement"

# Move action (only in movement phase)
- id: movePiece
  uses: "moveEntity"
  when:
    - condition: "phase.is"
      with:
        phaseSet: "game"
        phase: "movement"
```

**Enhancement Needed**: More flexible condition syntax for counting

#### Example 2: Chess (Hypothetical)
**Game Requirement**: Pieces move in specific patterns

**Identified Gap**: No piece-specific movement rules

**Proposed Generic Solution**:
```yaml
# Proposed: Movement pattern verb
- uses: "moveWithPattern"
  with:
    piece: "{selected}"
    patterns:
      knight: "L-shape"
      bishop: "diagonal"
      rook: "straight"
```

**Benefits**: Reusable for chess variants, checkers, shogi

#### Example 3: Monopoly-style Game
**Game Requirement**: Move around board, trigger space effects

**Identified Gaps**:
1. No circular track movement
2. No location-triggered effects
3. No property ownership

**Proposed Generic Solutions**:
```yaml
# Proposed: Track movement verb
- uses: "moveOnTrack"
  with:
    track: "board"
    distance: "{diceRoll}"
    direction: "forward"
    
# Proposed: Location triggers
zones:
  - id: "boardwalk"
    onEnter:
      - action: "payRent"
        when:
          - condition: "property.hasOwner"
```

#### Example 4: Deck Building Game
**Game Requirement**: Buy cards from market, build personal deck

**Identified Gaps**:
1. No marketplace mechanics
2. No card purchasing system
3. No deck cycling (discard → draw pile)

**Proposed Generic Solutions**:
```yaml
# Proposed: Purchase verb
- uses: "purchase"
  with:
    from: "market"
    to: "discard_{player}"
    cost:
      gold: 3
      
# Proposed: Deck cycling
- uses: "shuffleDiscardIntoDeck"
  when:
    - condition: "zone.isEmpty"
      with:
        zone: "deck_{player}"
```

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

### 5.4 Server-side Verb Development
If new verbs are needed:

#### 5.4.1 When to Create New Verbs
- **Novel mechanics**: Gravity, stacking, complex movement
- **Performance reasons**: Optimize expensive operations
- **Complex validation**: Multi-step rule checking

#### 5.4.2 Verb Implementation Process
1. **Add to `src/engine/verbs.rs`**:
   ```rust
   fn apply_new_verb(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
       // 1. Extract and validate arguments
       let required_arg = args["required"].as_str().ok_or("Missing required arg")?;
       
       // 2. Perform game logic
       // 3. Update state
       // 4. Return patches for client updates
   }
   ```

2. **Register in verb dispatcher**:
   ```rust
   match verb {
       "newVerb" => apply_new_verb(state, args),
       // ... other verbs
   }
   ```

3. **Add comprehensive tests**:
   - Valid argument handling
   - Edge cases and error conditions
   - State mutations
   - Return patch validation

#### 5.4.3 Verb Design Principles
- **Atomic operations**: Each verb does one clear thing
- **Idempotent**: Same input produces same output
- **Error handling**: Clear error messages for invalid operations
- **Performance**: Minimize expensive operations
- **State consistency**: Always leave game in valid state

### 5.5 Action Map Generation
Plan how actions will be presented to the client:

#### 5.5.1 Action Map Patterns
- **Cell-based**: `/zones/board/cells/{row}/{col}` (tic-tac-toe)
- **Column-based**: `/zones/board/columns/{col}` (Connect 4)
- **Zone-based**: `/zones/hand` (card games)
- **Index-based**: `/zones/deck/{index}` (ordered collections)

#### 5.5.2 Custom Action Map Logic
If needed, update `lobby.rs` action map generation:
- Add new path patterns for novel interaction types
- Ensure consistent naming conventions
- Handle edge cases (full columns, invalid positions)

### 5.6 Game Log Templates
Configure informative game log messages:

#### 5.6.1 Template Syntax
- **Player replacement**: `{player}` → actual player name
- **Coordinate replacement**: `{row}`, `{col}` → 1-indexed coordinates
- **Custom arguments**: `{column}` → custom argument values

#### 5.6.2 Argument Formats
Different action types use different argument structures:
- **Location-based** (tic-tac-toe): `{"location": "/zones/board/cells/1/2"}`
- **Coordinate-based**: `{"row": 1, "col": 2}`
- **Column-based** (Connect 4): `{"column": 3}`

#### 5.6.3 Server Processing
The server automatically handles:
- `{player}` replacement with actual player names
- `{row}`, `{col}` extraction from direct arguments or location paths

### 5.7 Critical Logging Requirements ⚠️

**THIS IS THE #1 CAUSE OF POOR GAME EXPERIENCE!**

Players MUST understand what the game is doing. When automatic decisions happen without explanation, players feel confused and disconnected from the game. This is especially critical for:
- Phase transitions
- Turn advancement  
- Win condition checking
- Card transfers
- Pair/set formation
- Resource changes

#### 5.7.1 Log ALL Automatic Decisions
Every automatic action MUST have a log entry:

```yaml
# BAD: Silent automatic action
- id: checkForPairs
  auto: true
  uses: formPairs
  
# GOOD: Logged automatic action  
- id: checkForPairs
  auto: true
  uses: formPairs
  ui:
    logTemplate: "{player} forms a pair of {rank}s"
```

#### 5.7.2 Player References in Logs ⚠️
**CRITICAL**: Never use hardcoded player names in log templates! Always use player IDs (p1, p2, etc.) which the server automatically replaces with actual player names.

```yaml
# ✗ WRONG - These will show literally in logs
logTemplate: "Player 1 draws a card"
logTemplate: "Alice asks for cards"
logTemplate: "The first player wins"

# ✓ CORRECT - Server replaces these with player names
logTemplate: "p1 draws a card"        # Shows: "Alice draws a card"
logTemplate: "{player} asks for cards" # Shows: "Bob asks for cards"
logTemplate: "p2 wins the game"       # Shows: "Charlie wins the game"
```

See [Game Log Parameters](./game-log-parameters.md#critical-player-references-in-log-templates) for complete details.

**Required Logging Checklist:**
- [ ] `nextTurn` actions - "Turn passes to {nextPlayer}"
- [ ] Phase transitions - "Moving to {phase} phase"
- [ ] Win checking - "Checking for winning conditions..."
- [ ] Game end - "Game Over! {winner} wins!"
- [ ] Automatic card draws - "{player} drew a card"
- [ ] Automatic transfers - "{source} gives cards to {target}"
- [ ] Score changes - "{player} scores {points} points"

#### 5.7.3 Multiple Log Entries for Complex Actions
Break down what happened step by step:

```yaml
# Example: Go Fish asking sequence
- id: askForCards
  ui:
    logTemplate: "{player} asks {target} for {rank}s"
    
- id: giveCards
  ui:
    logTemplate: "{target} has {count} {rank}(s) and gives them to {player}"
    
- id: goFish
  ui:
    logTemplate: "{target} says 'Go Fish!' - {player} draws a card"
    
- id: drewRequestedRank
  ui:
    logTemplate: "{player} drew a {rank} and gets another turn!"
```

#### 5.7.3 Discrete Patches for Animation Support
Each animatable change needs its own patch:

```yaml
# Step 1: Remove from source (patch 1)
- uses: removeEntity
  with:
    from: "/zones/hand_{target}"
    entity: "{cardId}"
    
# Step 2: Add to destination (patch 2)
- uses: addEntity  
  with:
    to: "/zones/hand_{player}"
    entity: "{cardId}"
    
# Step 3: Check consequences (patch 3)
- uses: checkForPairs
  with:
    player: "{player}"
```

This allows clients to:
- Animate card leaving opponent's hand
- Animate card arriving in player's hand  
- Animate pair formation separately

#### 5.7.4 Variable Replacement Testing
Always test that ALL variables are replaced:
- Test with actual game data
- Check edge cases (long names, special characters)
- Verify no `{variable}` appears in production logs
- `{column}` replacement from column arguments
- 1-indexed display conversion (internal 0-indexed → display 1-indexed)

### 5.8 Client-side Updates
If needed, update React components:
- Check if existing components handle the game
- Add new zone renderers if required
- Ensure mobile responsiveness
- Maintain generic component architecture

## Step 6: Testing

### 6.1 Server Tests
Create comprehensive tests in `server/tests/<game_name>_test.rs`:

#### 6.1.1 Basic Setup Tests
```rust
#[test]
fn test_game_setup() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("game-name").expect("Failed to get bundle");
    let state = load_initial_state(&bundle);
    
    // Verify initial state
    assert_eq!(state["turn"], 0);
    assert_eq!(state["currentPlayer"], "p1");
    
    // Verify board dimensions and initial emptiness
    // Verify action configuration
}
```

#### 6.1.2 Verb-Specific Tests
For each new verb (e.g., `placeWithGravity`):
```rust
#[test]
fn test_new_verb_mechanics() {
    // Test core functionality
    // Test edge cases (full zones, invalid positions)
    // Test error conditions
    // Test state consistency
}
```

#### 6.1.3 Game Flow Tests
- **Action execution**: Each action works correctly
- **Turn advancement**: Proper turn switching
- **Win condition detection**: All ways to win/draw
- **Edge case handling**: Invalid moves, special rules

#### 6.1.4 Game Log Tests
Verify log message generation:
```rust
#[test]
fn test_game_log_messages() {
    // Test template processing
    // Test parameter replacement
    // Test coordinate formatting
}
```

### 6.2 Client Tests
Add tests in `clients/react/src/__tests__/`:

#### 6.2.1 Action Handling Tests
```typescript
describe('Game Action Handling', () => {
  it('should handle specific action patterns', () => {
    // Test action message construction
    // Test turn-based restrictions
    // Test error handling
  });
});
```

#### 6.2.2 Component Integration Tests
- **Rendering tests**: Components display correctly
- **Interaction tests**: Clicks/taps work as expected
- **State sync tests**: Updates apply properly
- **Visual affordance tests**: UI feedback works

### 6.3 Integration Testing Strategy
Test complete game scenarios:

#### 6.3.1 Full Game Simulations
- **Complete games**: Play from start to finish
- **Multiple win paths**: Test different victory conditions
- **Edge case games**: Ties, unusual situations

#### 6.3.2 Multi-Player Testing
- **All player counts**: Minimum to maximum players
- **Turn order**: Proper sequence handling
- **Concurrent actions**: Race condition prevention

### 6.4 Bundle Validation
Always verify game bundles:
```bash
# Build updated bundles
./cli/target/debug/bluefelt-cli build-all

# Run comprehensive tests
cargo test <game_name>
```

### 6.5 Post-Implementation Testing (Critical!)
After implementing any new game, test ALL existing games:

#### 6.5.1 Regression Testing Checklist
```bash
# Run all server tests
cargo test tic_tac_toe
cargo test connect_four
cargo test <any_other_games>

# Run all engine tests
cargo test engine_integration
cargo test websocket

# Run all client tests
cd clients/react
pnpm test TicTacToeGameFlow
pnpm test ConnectFourColumnActions
```

#### 6.5.2 Manual Smoke Testing
For each existing game:
- [ ] Game loads correctly
- [ ] Basic actions work
- [ ] Game log shows proper messages
- [ ] Win conditions trigger
- [ ] No console errors

### 6.6 Performance Testing
- **Large boards**: Test with maximum zone sizes
- **Many entities**: Test with full entity counts
- **Rapid actions**: Test quick successive moves
- **Memory usage**: Monitor for leaks during long games

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

### Architecture Principles

#### Generic Client Design ✅
- Keep the React client completely generic
- Avoid game-specific components (like `ConnectFourBoard`)
- Use existing components (`BoardZone`, `CardZone`) for all games
- Add generic features (column actions) rather than game-specific ones

#### Server-Side Logic ✅
- Implement complex mechanics on the server (gravity, validation)
- Keep client simple and reactive
- Use verbs for reusable game operations
- Handle edge cases in server validation

#### Declarative Configuration ✅
- Express game rules in YAML when possible
- Use WebAssembly hooks only for complex logic
- Make games configurable rather than hardcoded
- Follow existing patterns from other games

### Testing Principles

#### Comprehensive Coverage ✅
- Test every action, edge case, and win condition
- Include both unit tests and integration tests
- Test game log message generation
- Verify visual affordances work correctly

#### Regression Prevention ✅
- Always test ALL existing games after changes
- Run complete test suites before deployment
- Manual smoke testing for critical user flows
- Performance testing for resource-intensive games

### Do's ✅
- ✅ Use existing patterns from other games
- ✅ Keep games declarative (YAML-driven)
- ✅ Write comprehensive tests for every new feature
- ✅ Consider mobile from the start
- ✅ Document edge cases and special rules
- ✅ Make reusable server verbs
- ✅ Test all existing games after implementing new ones
- ✅ Plan game log messages for good UX
- ✅ Consider visual affordances early
- ✅ Maintain backward compatibility

### Don'ts ❌
- ❌ Hardcode game-specific logic in shared components
- ❌ Skip testing edge cases and error conditions
- ❌ Ignore mobile experience and touch targets
- ❌ Break existing games with new implementations
- ❌ Use complex logic when simple will do
- ❌ Forget about accessibility and screen readers
- ❌ Create game-specific React components
- ❌ Implement game logic on the client side
- ❌ Skip regression testing after changes
- ❌ Use unclear or uninformative game log messages

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

### Gravity-based Game (like Connect 4)
- Standard entities (discs/pieces)
- Single zone (board) with special mechanics
- Column-based actions (gravity drops)
- Custom server verbs (placeWithGravity)
- Visual affordances (column drop zones)

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

**Game log messages not showing coordinates**
- Check argument format (location vs. row/col)
- Verify template syntax in actions.yaml
- Ensure bundles are rebuilt after changes

**Visual affordances not working**
- Check action map generation patterns
- Verify client component handles new action types
- Test with browser dev tools for click detection

## Lessons Learned from Recent Implementations

### Connect 4 Implementation Insights

#### Server-Side vs Client-Side Logic
**Decision: Implement gravity on server**
- ✅ Ensures consistent game state
- ✅ Prevents cheating/invalid moves
- ✅ Keeps client generic and simple
- ❌ More complex than pure client-side

#### Generic Components vs Game-Specific
**Decision: Enhance BoardZone with column detection**
- ✅ Maintains generic client architecture
- ✅ Reusable for future gravity-based games
- ✅ Automatic visual affordances
- ❌ Required thinking about abstraction

#### Game Log Parameter Handling
**Learning: Different games use different action argument formats**
- Location-based: `{"location": "/zones/board/cells/1/2"}`
- Column-based: `{"column": 3}`
- Coordinate-based: `{"row": 1, "col": 2}`

Server now handles all formats automatically with regex parsing.

#### Testing Strategy Evolution
**Key insight: Regression testing is critical**
- Any new feature can break existing games
- Comprehensive test suites prevent issues
- Manual smoke testing catches UI problems
- Game log testing ensures good UX

### Platform Evolution Patterns

#### When to Add New Verbs
Add server verbs for:
- ✅ Novel mechanics (gravity, complex movement)
- ✅ Performance optimization
- ✅ Complex validation rules
- ❌ Simple operations that existing verbs handle

#### When to Enhance Client Components
Enhance existing components for:
- ✅ New interaction patterns (column clicks)
- ✅ Visual affordances (drop zones)
- ✅ Better accessibility
- ❌ Game-specific features

## Post-Implementation Checklist

After implementing any new game:

### Technical Validation
- [ ] All new tests pass
- [ ] All existing game tests pass
- [ ] Bundle builds successfully
- [ ] Server starts without errors
- [ ] Client builds without errors

### Functional Validation
- [ ] Game plays correctly end-to-end
- [ ] Game log shows informative messages
- [ ] Visual affordances work intuitively
- [ ] Win conditions trigger properly
- [ ] Mobile experience is smooth

### Regression Testing
- [ ] Test all existing games manually
- [ ] Verify no console errors
- [ ] Check game log functionality
- [ ] Validate WebSocket connectivity
- [ ] Performance remains acceptable

### Documentation Updates
- [ ] Update this implementation guide
- [ ] Add new patterns to best practices
- [ ] Document any new verbs or components
- [ ] Update SDK reference if needed

## Documentation Improvements Reflection

Based on implementation experiences, here's what documentation should emphasize:

### Critical Information Often Missing:
1. **Initial State Structure** - How the game state is initialized automatically
2. **Path Formats** - Exact formats for zone paths in actions and patches
3. **Verb Limitations** - What each verb can and cannot do
4. **Conditional Capabilities** - Current limitations of the condition system
5. **Error Messages** - Common errors and their meanings

### Documentation Best Practices:
- **Show complete examples** not just fragments
- **Document edge cases** and limitations explicitly  
- **Include debugging tips** for common issues
- **Provide decision trees** for choosing approaches
- **List all available options** for each configuration

### Recommended Documentation Structure:
1. **Quick Start** - Minimal working example
2. **Concept Overview** - How the system works
3. **Complete Reference** - All options and parameters
4. **Limitations** - What isn't possible yet
5. **Workarounds** - How to handle gaps
6. **Examples** - Full, working implementations

### Key Learning: Action Conditions Enable Complex Logic

The action system with conditional logic is more powerful than initially apparent:

```yaml
# Automatic actions can check any game state
- id: checkCondition
  auto: true
  when:
    - condition: "zone.count"
    - condition: "resource.value"
    - condition: "phase.is"
  then:
    - action: "any_action"
```

This enables:
- **Conditional phase transitions** based on game state
- **Dynamic game flow** without hardcoding
- **Complex win conditions** beyond simple patterns
- **Resource-triggered events** and milestones

The key insight: Think of actions as your game's logic engine, not just player interactions.

## Next Steps

After implementing a game:
1. **Update documentation** with lessons learned
2. **Extract reusable patterns** for future games
3. **Share new components** with the community
4. **Consider game variations** and expansions
5. **Plan performance optimizations** if needed
6. **Test on all platforms** and browsers

### Continuous Improvement
- Review implementation process effectiveness
- Identify pain points in development flow
- Enhance tooling and automation
- Gather feedback from other developers
- Update templates and scaffolding

Remember: Each game implementation should:
- ✅ **Improve the platform** for future games
- ✅ **Maintain quality standards** across all features
- ✅ **Preserve backward compatibility** with existing games
- ✅ **Document learnings** for the next developer

The platform grows stronger with every well-implemented game! 🎮
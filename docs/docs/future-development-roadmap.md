# Future Development Roadmap

This document consolidates all planned enhancements and future development directions for the Bluefelt platform.

## Immediate Priorities

### 1. Game Log and Animation Support
**Status**: PARTIALLY COMPLETE - Critical for player understanding and polished UI

**Current Issues**:
- ✅ Variable replacement broken in game logs - **FIXED**: Added support for `{targetColumn}` and improved replacement logic
- ✅ Automatic game decisions happen too fast without explanation - **FIXED**: Added logging to "then" actions
- ⚠️ Multiple state changes sent as single diff, preventing animation - **NEEDS DESIGN**

**Progress Made**:
- ✅ Fixed variable replacement bug for Connect 4's `{targetColumn}` parameter
- ✅ Updated Go Fish with comprehensive logging for all automatic decisions
- ✅ Added logging to Three Men's Morris phase transitions and turn advancement
- ✅ Added logging to Tic-Tac-Toe automatic actions
- ✅ Updated documentation to emphasize logging requirements
- ✅ Implemented automatic action logging for "then" actions (e.g., nextTurn)
- ⚠️ Phase enter actions still don't generate logs (dealCards, etc.)

**Still Required**:

#### A. Game Log Enhancements
- **Fix variable replacement**: Ensure all template variables are properly replaced
- **Log all rule applications**: Every automatic decision should generate a log entry
- **Detailed action results**: Show what happened, not just that something happened

Example improvements for Go Fish:
```yaml
# Current: Player selects opponent, card appears instantly
# Better:
- "{player} asks {target} for {rank}s"
- "{target} has 1 {rank} and gives it to {player}"
- "{player} forms a pair of {rank}s"

# Or if they don't have it:
- "{player} asks {target} for {rank}s" 
- "{target} says 'Go Fish!'"
- "{player} draws a card"
- "{player} drew a {rank} and gets another turn!" # If they drew what they asked for
```

#### B. Discrete State Updates for Animation
**Requirement**: Server must send separate patches for each logical state change

**Example Sequence**:
1. **Patch 1**: Remove card from opponent's hand
   - Log: "Player 2 gives a 3 to Player 1"
   - Client can animate: Card moving from P2 to P1
   
2. **Patch 2**: Add card to current player's hand
   - Client completes card arrival animation
   
3. **Patch 3**: If pair formed, move both cards to pairs zone
   - Log: "Player 1 forms a pair of 3s"
   - Client can animate: Cards popping out and moving to pairs

**Implementation Requirements**:
- Actions should generate multiple patches when appropriate
- Each patch should represent one animatable change
- Include metadata to identify patch type for animation system

### 2. Selection Pattern Support
**Status**: Needed for Three Men's Morris movement phase

**Required Features**:
- Selection state management
- Two-phase interactions (select → act)
- Cancel mechanism for non-committal actions
- UI support for selection highlighting

**New Verbs Needed**:
```yaml
- selectEntity   # Mark an entity as selected
- moveSelected   # Move the currently selected entity
- clearSelection # Cancel current selection
```

**Benefits**: Enables movement-based board games like Checkers, Chess, etc.

### 2. Enhanced Conditions
**Current gaps identified**:

- **Adjacency checking**: For movement validation
- **Path validation**: For complex movement rules
- **Line-of-sight**: For some board games
- **Mandatory action detection**: For forced captures

**Example Usage**:
```yaml
- condition: location.isAdjacent
  with:
    from: "{selection.location}"
    to: "{target}"
```

### 3. UI Protocol Enhancements
**Current limitations**:
- All interactions must be spatial (clicking on zones)
- No support for abstract UI elements (buttons, menus)
- Limited feedback for selections

**Proposed additions**:
- Cancel action buttons
- Context menus for pieces
- Selection highlighting
- Action confirmation dialogs

## Medium-Term Enhancements

### 1. Movement Validation System
For complex board games that need sophisticated movement rules:

- **Pattern-based movement**: Diagonal, L-shape, straight lines
- **Distance constraints**: Maximum movement distance
- **Obstacle detection**: Blocking pieces
- **Capture mechanics**: Jump captures, multiple captures

### 2. Advanced Card Game Support
Based on patterns from potential card games:

- **Hand management**: Meld detection, set formation
- **Trick-taking mechanics**: Follow suit, trump cards
- **Drafting systems**: Simultaneous selection
- **Deck manipulation**: Searching, inserting cards

### 3. Math and Calculation System
For dice games and scoring:

- **Dice rolling**: Random value generation
- **Pattern detection**: Straights, sets, full houses
- **Score calculation**: Complex formulas
- **Resource management**: Counters, currencies

## Long-Term Vision

### 1. Real-Time Game Support
Currently focused on turn-based games, but could expand to:
- Simultaneous action games
- Timed decisions
- Real-time strategy elements

### 2. Advanced AI Integration
- Built-in AI opponents
- Difficulty levels
- Strategy templates
- Learning systems

### 3. Tournament and Social Features
- Ranking systems
- Tournament brackets
- Spectator mode
- Replay analysis

## Implementation Philosophy

### Design Principles
1. **Declarative over imperative**: Prefer YAML configuration to custom code
2. **Reusable patterns**: Each enhancement should benefit multiple games
3. **Backwards compatibility**: Don't break existing games
4. **Progressive enhancement**: Games can adopt new features incrementally
5. **Player comprehension first**: Every automatic decision must be clearly communicated
6. **Animation-ready**: State changes should enable smooth UI transitions

### Development Approach
1. **Identify common patterns** across multiple games
2. **Design generic solutions** that work for many use cases
3. **Implement incrementally** with working games as proof of concept
4. **Maintain comprehensive tests** for all new features

### Game Implementation Guidelines
When implementing any game:

1. **Log Every Decision**:
   - Every automatic action needs a log entry
   - Use clear templates: `"{player} does {action} to {target}"`
   - Test that all variables are replaced correctly

2. **Discrete State Changes**:
   - Each logical change should be a separate patch
   - Think "what would I animate?" and make that a patch
   - Order matters: removal → transfer → arrival → consequences

3. **Player Communication**:
   - Never assume the player knows what happened
   - Log the cause AND effect of actions
   - Be specific: "drew a 3" not just "drew a card"

## Current Game-Specific Needs

### All Games
- **Critical**: Fix game log variable replacement (`{call}` → actual values)
- **Important**: Add comprehensive logging for all automatic decisions
- **Important**: Ensure state changes are sent as discrete patches

### Three Men's Morris
- **Immediate**: Selection pattern for movement phase
- **Future**: Win detection when no moves available
- **Logging**: Add logs for phase transitions

### Connect Four
- **Current**: Column clicking works
- **Fix**: `{call}` in logs should show actual column number
- **Enhancement**: Separate patches for piece falling animation

### Go Fish
- **Critical**: Add detailed logs for:
  - "Player X asks Player Y for Zs"
  - "Player Y has N cards and gives them"
  - "Player X forms a pair of Zs"
  - "Go Fish! Player X draws a card"
- **Critical**: Send separate patches for:
  - Card transfer from opponent
  - Card arrival in hand
  - Pair formation and movement
- **Enhancement**: Better card organization and sorting

### Tic-Tac-Toe
- **Current**: Works correctly (reference implementation)
- **Enhancement**: Could benefit from clearer win detection logs

## Success Metrics

For each major enhancement:
1. **Enables new game types**: Should unlock a category of games
2. **Improves existing games**: Should enhance current implementations
3. **Maintains simplicity**: Should not complicate basic use cases
4. **Developer friendly**: Should be easy to understand and use

## Review Process

This roadmap should be reviewed and updated:
- **After each major feature implementation**
- **When new game requirements are identified**
- **Based on developer feedback and usage patterns**
- **Quarterly to reassess priorities**

The goal is to maintain a clear vision while remaining flexible enough to adapt to actual usage and needs.
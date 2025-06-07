# Game Log Parameters

Game log messages provide players with clear, informative updates about actions taken during gameplay. This guide explains how to configure log templates and parameter replacement.

## Critical Requirements ⚠️

1. **All automatic decisions MUST generate log entries** - Players need to understand what the game did on their behalf
2. **All template variables MUST be replaced** - Never show `{variable}` in production
3. **Complex actions need multiple log entries** - Break down what happened step by step

## Overview

Game logs are generated when players perform actions AND when the game makes automatic decisions. The server processes log templates defined in `actions.yaml` and replaces placeholders with actual values to create user-friendly messages.

## Log Template Syntax

### Basic Structure

In your `actions.yaml` file, add a `logTemplate` to any action:

```yaml
- id: myAction
  uses: place
  ui:
    direction: "Click to perform action"
    logTemplate: "{player} performed an action"
```

### Placeholder Types

#### Player Placeholders
- **`{player}`** - Replaced with the actual player name (automatically mapped from p1/p2)
- **`{nextPlayer}`** - The name of the next player (used in turn advancement actions)
- **Always available** for any action

```yaml
logTemplate: "{player} made a move"
# Result: "alice made a move"

logTemplate: "Turn passes to {nextPlayer}"
# Result: "Turn passes to bob"
```

#### Coordinate Placeholders
- **`{row}`** - Row coordinate (1-indexed for display)
- **`{col}`** - Column coordinate (1-indexed for display)  
- **`{column}`** - Column number (1-indexed for display)

Coordinates are automatically extracted from multiple formats:
- Direct arguments: `args.row` and `args.col`
- Target format: `args.target` as "0-0" (row-col)
- Location path: `args.location` as "/zones/board/cells/0/0"

```yaml
logTemplate: "{player} placed their mark at ({row}, {col})"
# Result: "bob placed their mark at (2, 3)"
```

#### Action Arguments
Access any argument passed to the action:
- **`{args.fieldName}`** - Any field from action arguments

```yaml
logTemplate: "{player} asks for {args.rank}s"
# Result: "alice asks for Queens"
```

#### Selection State
Access values from the current selection state:
- **`{selection.fieldName}`** - Any field from selection state

```yaml
logTemplate: "{player} selected {selection.selectedRank}"
# Result: "bob selected King"
```

## Action Argument Formats

Different games use different argument structures. The server automatically handles all common formats:

### Location-Based Arguments (Tic-Tac-Toe Style)

**Client sends:**
```json
{
  "action": "placeMarker",
  "args": {
    "location": "/zones/board/cells/1/2",
    "entity": "mark_p1"
  }
}
```

**Server extracts:**
- `row = 1, col = 2` from the location path
- Converts to 1-indexed: `row = 2, col = 3`

**Template:**
```yaml
logTemplate: "{player} placed their mark at ({row}, {col})"
```

**Result:**
```
"alice placed their mark at (2, 3)"
```

### Direct Coordinate Arguments

**Client sends:**
```json
{
  "action": "moveEntity", 
  "args": {
    "row": 0,
    "col": 1,
    "entity": "piece_p1"
  }
}
```

**Server extracts:**
- `row = 0, col = 1` directly from arguments
- Converts to 1-indexed: `row = 1, col = 2`

### Target-Based Arguments (Three Men's Morris Style)

**Client sends:**
```json
{
  "action": "placeToken",
  "args": {
    "target": "0-0",
    "entity": "piece_p1"
  }
}
```

**Server extracts:**
- Parses "0-0" format as `row = 0, col = 0`
- Converts to 1-indexed: `row = 1, col = 1`

**Template:**
```yaml
logTemplate: "{player} placed a piece at ({row}, {col})"
```

**Result:**
```
"alice placed a piece at (1, 1)"
```

### Column-Based Arguments (Connect 4 Style)

**Client sends:**
```json
{
  "action": "dropDisc",
  "args": {
    "zone": "/zones/board",
    "column": 3,
    "entity": "disc_p1"
  }
}
```

**Server extracts:**
- `column = 3` directly from arguments
- Converts to 1-indexed: `column = 4`

**Template:**
```yaml
logTemplate: "{player} dropped a disc in column {column}"
```

**Result:**
```
"bob dropped a disc in column 4"
```

## Server Processing Logic

The server automatically handles placeholder replacement in this order:

1. **Player replacement**: `{player}` → actual player name (mapped from p1/p2 to player list)
2. **Direct coordinates**: `{row}`, `{col}` from action arguments
3. **Column arguments**: `{column}` from action arguments  
4. **Target parsing**: Extract coordinates from "row-col" format (e.g., "0-0")
5. **Location parsing**: Extract coordinates from location paths using regex
6. **Generic arguments**: `{args.fieldName}` replaced with any action argument
7. **Selection state**: `{selection.fieldName}` replaced with selection values
8. **1-indexed conversion**: All coordinates converted for user display

### Coordinate Extraction Methods

#### Target Format Parsing
For actions using `target` argument in "row-col" format:

**Pattern:** `^(\d+)-(\d+)$`

**Examples:**
- `"0-0"` → `row=1, col=1`
- `"2-1"` → `row=3, col=2`
- `"1-2"` → `row=2, col=3`

#### Location Path Parsing
For actions using location paths:

**Pattern:** `/zones/[^/]+/cells/(\d+)/(\d+)`

**Examples:**
- `/zones/board/cells/0/0` → `row=1, col=1`
- `/zones/da-board/cells/2/1` → `row=3, col=2` 
- `/zones/main-board/cells/5/3` → `row=6, col=4`

## Examples by Game Type

### Tic-Tac-Toe
```yaml
logTemplate: "{player} placed their mark at ({row}, {col})"
```
**Messages:**
- "alice placed their mark at (1, 1)"
- "bob placed their mark at (2, 3)"
- "alice placed their mark at (3, 2)"

### Connect 4
```yaml
logTemplate: "{player} dropped a disc in column {column}"
```
**Messages:**
- "alice dropped a disc in column 1"
- "bob dropped a disc in column 4" 
- "alice dropped a disc in column 7"

### Checkers
```yaml
logTemplate: "{player} moved from ({row}, {col}) to ({newRow}, {newCol})"
```
**Messages:**
- "alice moved from (3, 2) to (4, 3)"
- "bob moved from (6, 1) to (5, 2)"

### Card Games
```yaml
logTemplate: "{player} drew {cardCount} cards"
```
**Messages:**
- "alice drew 3 cards"
- "bob drew 1 card"

## Logging Automatic Decisions

### When to Log
Every automatic game decision needs a log entry:

```yaml
# Example: Go Fish - Checking if player has requested cards
- id: checkPlayerCards
  uses: conditionalAction
  auto: true
  with:
    condition:
      - condition: zone.hasMatching
        with:
          zone: "/zones/hand_{selection.targetPlayer}"
          property: rank
          value: "{selection.selectedRank}"
    ifTrue:
      - action: transferCards
      - action: logTransfer  # CRITICAL: Log what happened!
    ifFalse:
      - action: goFish
      - action: logGoFish    # CRITICAL: Log the alternative!
```

### Multi-Step Actions
Break complex actions into multiple log entries:

```yaml
# BAD: One vague log for everything
logTemplate: "{player} completed their turn"

# GOOD: Detailed sequence
- id: askForCards
  logTemplate: "{player} asks {target} for {rank}s"
  
- id: transferCards  
  logTemplate: "{target} has {count} {rank}(s) and gives them to {player}"
  
- id: formPair
  logTemplate: "{player} forms a pair of {rank}s"
```

## Discrete Patches for Animation

**CRITICAL**: Each animatable state change must be a separate patch!

### Wrong Approach ❌
```yaml
# This sends all changes in one patch, preventing animation
- uses: complexAction
  with:
    # Removes card from opponent
    # Adds card to player  
    # Forms pair
    # All in one atomic operation
```

### Correct Approach ✅
```yaml
# Step 1: Remove from opponent (patch 1)
- uses: removeEntity
  with:
    from: "/zones/hand_{target}"
    entity: "{cardId}"
  ui:
    logTemplate: "{target} gives a {rank} to {player}"

# Step 2: Add to player (patch 2)  
- uses: addEntity
  with:
    to: "/zones/hand_{player}"
    entity: "{cardId}"
    
# Step 3: Check and form pairs (patch 3)
- uses: formPairs
  with:
    player: "{player}"
  ui:
    logTemplate: "{player} forms a pair of {rank}s"
```

Each patch enables the client to:
1. Animate card leaving opponent's hand
2. Animate card arriving in player's hand
3. Animate pair formation

## Critical: Player References in Log Templates

⚠️ **NEVER use player names directly in log templates!**

When referencing players in log templates, you must use player IDs (p1, p2, etc.) instead of hardcoded names. The server will automatically replace these with actual player names.

### Wrong ❌
```yaml
logTemplate: "Player 1 draws a card"
logTemplate: "Alice asks for cards"
logTemplate: "The first player wins"
```

### Correct ✅
```yaml
logTemplate: "p1 draws a card"        # Will show: "Alice draws a card"
logTemplate: "{player} asks for cards" # Will show: "Bob asks for cards"
logTemplate: "p2 wins the game"       # Will show: "Charlie wins the game"
```

### Why This Matters
1. **Dynamic player names** - Players choose their own names when joining
2. **Consistency** - All logs show actual player names, not generic labels
3. **Multiplayer support** - Works correctly with any number of players

### Examples for Automatic Actions

```yaml
# Dealing cards to specific players
- id: dealToP1
  auto: true
  ui:
    logTemplate: "Dealing 7 cards to p1"  # Shows: "Dealing 7 cards to Alice"

- id: dealToP2  
  auto: true
  ui:
    logTemplate: "Dealing 7 cards to p2"  # Shows: "Dealing 7 cards to Bob"

# Referencing other players
- id: transferCards
  ui:
    logTemplate: "p2 gives cards to p1"   # Shows: "Bob gives cards to Alice"
```

### Player ID Patterns
The server recognizes and replaces these patterns:
- `p1`, `p2`, `p3`, `p4` - Direct player references
- `{player}` - Current acting player
- `{nextPlayer}` - Next player in turn order
- `{selection.targetPlayer}` - Player selected from game state
- `{args.targetPlayer}` - Player passed as action argument

## Best Practices

### Clear Action Descriptions
✅ **Good:** `"{player} dropped a disc in column {column}"`
❌ **Bad:** `"{player} did something"`

### Informative Coordinates
✅ **Good:** `"{player} moved to ({row}, {col})"`  
❌ **Bad:** `"{player} moved"`

### User-Friendly Language
✅ **Good:** `"{player} captured a piece at ({row}, {col})"`
❌ **Bad:** `"{player} executed capture action"`

### Consistent Terminology
- Use the same terms across all actions in a game
- Match terminology used in game rules
- Be consistent with coordinate format

### Complete Information
✅ **Good:** `"{player} drew a {rank} of {suit} - gets another turn!"`
❌ **Bad:** `"{player} drew a card"`

## Testing Log Templates

### Unit Tests
Create tests to verify log message generation:

```rust
#[test]
fn test_log_template_processing() {
    let log_template = "{player} placed at ({row}, {col})";
    let player_id = "testPlayer";
    let location = "/zones/board/cells/1/2";
    
    // Process template (simulating server logic)
    let result = process_log_template(log_template, player_id, location);
    assert_eq!(result, "testPlayer placed at (2, 3)");
}
```

### Integration Tests
Test with actual game actions:

```rust
#[test] 
fn test_game_log_in_action() {
    // Simulate complete action with log generation
    // Verify log appears in game state
    // Check coordinate formatting
}
```

### Manual Testing
1. Start a game and perform various actions
2. Check the game log panel for messages
3. Verify coordinates are correct and 1-indexed
4. Test edge cases (corners, center, etc.)

## Troubleshooting

### Log Shows Literal Placeholders
**Problem:** `"alice placed their mark at ({row}, {col})"`

**Causes:**
- Action arguments don't match expected format
- Bundle not rebuilt after template changes
- Server doesn't recognize argument structure

**Solutions:**
1. Check action argument format in client
2. Rebuild bundles: `./cli/target/debug/bluefelt-cli build-all`
3. Verify server supports the argument pattern

### Coordinates Are Wrong
**Problem:** Shows (0, 0) instead of (1, 1)

**Causes:**
- 1-indexed conversion not applied
- Wrong coordinate extraction

**Solutions:**
1. Check server processing logic
2. Verify regex pattern matches location format
3. Test coordinate extraction separately

### Missing Log Messages
**Problem:** No log entry appears

**Causes:**
- Missing `logTemplate` in action definition
- Action not executing properly
- WebSocket connection issues

**Solutions:**
1. Add `logTemplate` to action in `actions.yaml`
2. Verify action executes successfully
3. Check browser console for WebSocket errors

## Advanced Usage

### Custom Argument Extraction
For complex games, you may need custom server logic:

```rust
// In lobby.rs, add custom parameter handling
if let Some(special_arg) = args_obj["specialValue"].as_str() {
    log_text = log_text.replace("{special}", special_arg);
}
```

### Dynamic Message Content
Use conditional logic for varied messages:

```yaml
logTemplate: "{player} {actionType} at ({row}, {col})"
```

Where `actionType` is determined by game context.

### Multi-Language Support
Consider how log templates will work with internationalization:

```yaml
logTemplate: 
  en: "{player} placed their mark at ({row}, {col})"
  es: "{player} colocó su marca en ({row}, {col})"
```

## Related Documentation

- [Game Implementation Guide](./game-implementation-guide.md) - Overall game development process
- [State Structure](./state-structure.md) - How game state is organized
- [SDK Reference](./sdk-reference.md) - API documentation

Remember: Good game log messages significantly improve the player experience by providing clear feedback about what happened in the game!
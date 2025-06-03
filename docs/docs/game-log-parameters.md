# Game Log Parameters

Game log messages provide players with clear, informative updates about actions taken during gameplay. This guide explains how to configure log templates and parameter replacement.

## Overview

Game logs are generated when players perform actions. The server processes log templates defined in `actions.yaml` and replaces placeholders with actual values to create user-friendly messages.

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
- **`{player}`** - Replaced with the actual player name
- **Always available** for any action

```yaml
logTemplate: "{player} made a move"
# Result: "alice made a move"
```

#### Coordinate Placeholders
- **`{row}`** - Row coordinate (1-indexed for display)
- **`{col}`** - Column coordinate (1-indexed for display)  
- **`{column}`** - Column number (1-indexed for display)

```yaml
logTemplate: "{player} placed their mark at ({row}, {col})"
# Result: "bob placed their mark at (2, 3)"
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

1. **Player replacement**: `{player}` → actual player name
2. **Direct coordinates**: `{row}`, `{col}` from action arguments
3. **Column arguments**: `{column}` from action arguments  
4. **Location parsing**: Extract coordinates from location paths using regex
5. **1-indexed conversion**: All coordinates converted for user display

### Location Path Parsing

The server uses regex to extract coordinates from location paths:

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
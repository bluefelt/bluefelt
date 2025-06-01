# SDK Conditions

The Bluefelt SDK provides conditions that can be used in game definitions to handle common game patterns without writing custom code.

## Available Conditions

### consecutiveMarksInRow

Checks if a player has a specified number of consecutive marks in a row (horizontally, vertically, or diagonally).

**Parameters:**
- `zone`: The zone name to check (e.g., "board")
- `count`: The number of consecutive marks required (e.g., 3 for tic-tac-toe, 5 for tic-tac-toe-5)
- `entity`: The entity to look for, can use `{actor}` placeholder (e.g., "mark_{actor}")

**Example:**
```yaml
- uses: board.checkConsecutive
  with:
    zone: board
    count: 3
    entity: mark_{actor}
  result:
    - gameWin: actor
```

### allCellsFilled

Checks if all cells in a zone are filled (no null values).

**Parameters:**
- `zone`: The zone name to check (e.g., "board")

**Example:**
```yaml
- uses: board.checkFilled
  with:
    zone: board
  result:
    - gameTie: true
```

## Using Conditions

Conditions are used in action definitions with the `conditions` field. Each condition can have a `result` that specifies what should happen when the condition is met.

### Complete Example (tic-tac-toe)

```yaml
- id: place
  uses: entity.move
  with:
    source: marks_{actor}
    target:
      zone: board
      constraints:
        vacant: true
  ui:
    direction: "Choose a cell to place a mark"
  then:
    - checkGameEnd

- id: checkGameEnd
  auto: true
  conditions:
    - uses: board.checkConsecutive
      with:
        zone: board
        count: 3
        entity: mark_{actor}
      result:
        - gameWin: actor
    - uses: board.checkFilled
      with:
        zone: board
      result:
        - gameTie: true
```

## Result Actions

The following result actions are supported:

### gameWin
Sets the game to ended state with a winner.
- Value: "actor" (uses the player who met the condition) or a specific player ID

### gameTie
Sets the game to ended state with a tie.
- Value: true

## Benefits

Using conditions:
1. **No custom code needed** - Game logic is declarative
2. **Reusable** - Same conditions work for tic-tac-toe, tic-tac-toe-5, Connect Four, etc.
3. **Consistent** - All games using these conditions behave the same way
4. **Optimized** - Conditions are implemented efficiently in the engine

## Future Conditions

Potential additions:
- `capturedPieces`: Check if a player has captured a certain number of pieces
- `reachedPosition`: Check if a piece has reached a specific position
- `scoreThreshold`: Check if a player has reached a score threshold
- `timeExpired`: Check if a timer has expired
- `customExpression`: Evaluate a custom expression on the game state
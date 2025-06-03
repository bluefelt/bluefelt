# Using the grid.lineOfMarks Built-in Verb

The `grid.lineOfMarks` verb provides a flexible, reusable way to detect winning conditions in grid-based games without custom WebAssembly hooks.

## Basic Usage

```yaml
# Check for winning condition after each move
- id: checkWin
  uses: grid.lineOfMarks
  auto: true
  with:
    zone: "zones/board"
    entity: "mark_{player}"
    lineLength: 3
    directions: ["horizontal", "vertical", "diagonal"]
```

## Parameters

- **zone**: Path to the grid zone to check (e.g., "zones/board")
- **entity**: Entity pattern to match. Use `{player}` wildcard (e.g., "token_{player}" matches "token_p1", "token_p2")
- **lineLength**: Number of entities in a row needed to win (default: 3)
- **directions**: Array of directions to check. Options: "horizontal", "vertical", "diagonal"

## Game Examples

### Tic-Tac-Toe (3-in-a-row)
```yaml
- id: checkWin
  uses: grid.lineOfMarks
  auto: true
  with:
    zone: "zones/board"
    entity: "mark_{player}"
    lineLength: 3
    directions: ["horizontal", "vertical", "diagonal"]
```

### Connect-4 (4-in-a-row, no diagonal)
```yaml
- id: checkWin
  uses: grid.lineOfMarks
  auto: true
  with:
    zone: "zones/board"
    entity: "chip_{player}"
    lineLength: 4
    directions: ["horizontal", "vertical"]
```

### Gomoku (5-in-a-row)
```yaml
- id: checkWin
  uses: grid.lineOfMarks
  auto: true
  with:
    zone: "zones/board"
    entity: "stone_{player}"
    lineLength: 5
    directions: ["horizontal", "vertical", "diagonal"]
```

## Integration in Actions

Typically, you'll call `checkWin` after each placement action:

```yaml
# Main placement action
- id: placeToken
  uses: place
  ui:
    direction: "Click to place your token"
  when:
    - condition: zone.isEmpty
      with:
        zone: "{target}"
    - condition: player.isActor
  then:
    - action: checkWin
    - action: advanceTurn

# Win detection action
- id: checkWin
  uses: grid.lineOfMarks
  auto: true
  with:
    zone: "zones/board"
    entity: "token_{player}"
    lineLength: 4
    directions: ["horizontal", "vertical", "diagonal"]
```

## Game Status Output

When a win is detected, `grid.lineOfMarks` sets the game status:

**Win detected:**
```json
{
  "state": "ended",
  "winner": "p1",
  "tie": false
}
```

**Tie detected (board full, no winner):**
```json
{
  "state": "ended", 
  "winner": null,
  "tie": true
}
```

**Game continues (no win, board not full):**
No game status is set, allowing the game to continue.

## Benefits

1. **No Custom Code**: Avoid writing WebAssembly hooks for basic line detection
2. **Reusable**: Same verb works for multiple game types
3. **Configurable**: Adjust line length and directions per game
4. **Pattern Matching**: `{player}` wildcard matches any player's entities
5. **Comprehensive**: Handles wins, ties, and ongoing games
6. **Tested**: Built-in tests ensure reliability across game scenarios
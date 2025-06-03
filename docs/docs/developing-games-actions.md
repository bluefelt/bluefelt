# Developing Games: Actions

Actions define what players can do in your game - from placing pieces to drawing cards. This guide covers how to create actions using Bluefelt's built-in verbs and action system.

## Overview

Actions in Bluefelt handle:
- **Basic Moves** - Place pieces, move entities, draw cards
- **Game Flow** - End turn, change phase
- **Automated Actions** - Triggered by game events
- **Chained Actions** - Actions that trigger other actions

Each action specifies what it does and optionally when it's available.

## Action Structure

### Minimal Action

```yaml
- id: "place_piece"
  uses: "place"
```

### Complete Action Structure

```yaml
- id: "advanced_move"
  uses: "moveEntity"
  ui:
    name: "Tactical Movement"
    description: "Move a unit and potentially attack"
    direction: "Select a unit to move"
    icon: "🏃"
    color: "#0066cc"
    hotkey: "M"
    priority: 1
  conditions:
    - type: "current_player_turn"
    - type: "zone_contains_player_entity"
      zone: "{source}"
    - type: "path_clear"
      from: "{source}"
      to: "{target}"
    - type: "movement_range"
      entity: "{source_entity}"
      range: 3
  effects:
    - verb: "moveEntity"
      args:
        from: "{source}"
        to: "{target}"
    - trigger: "check_combat"
      if: "enemy_adjacent"
  triggers:
    - "update_visibility"
    - "check_victory_conditions"
  auto: false
  availability: "always"
  cooldown: 0
  cost:
    - resource: "action_points"
      amount: 1
  metadata:
    category: "movement"
    complexity: "medium"
    tutorial: "Move units to strategic positions"
```

## Core Properties

### Required Fields

#### `id` (string, required)
Unique identifier for the action.

```yaml
- id: "place_marker"      # ✓ Valid
- id: "draw_cards"        # ✓ Valid
- id: "move_piece"        # ✓ Valid
```

#### `uses` (string, required)
The verb that executes the action. Can be built-in or custom.

```yaml
uses: "place"             # Built-in verb
uses: "moveEntity"        # Built-in verb
uses: "customVerb"        # Custom verb from WASM
```

### UI Properties

Define how actions appear to players:

```yaml
ui:
  # Display properties
  name: "Place Marker"            # Human-readable name
  description: "Place your marker on an empty cell"
  direction: "Choose a cell"      # Instruction shown when available
  
  # Visual properties
  icon: "⭕"                      # Emoji or icon reference
  color: "#0066cc"                # Highlight color
  
  # Interaction
  hotkey: "P"                     # Keyboard shortcut
  priority: 1                     # Display order (lower = higher priority)
  
  # Game log
  logTemplate: "{player} placed their marker at ({row}, {col})"
```

### Conditions (when)

Control when actions are available using the `when` field:

```yaml
when:
  - condition: "zone.isEmpty"
    with:
      zone: "{target}"
  - condition: "player.isActor"
```

#### Current Condition Types

**Basic Conditions:**
- `zone.isEmpty` - Check if a zone/location is empty
- `player.isActor` - Check if player is the current actor

**Proposed Enhanced Conditions (To Be Implemented):**
- `zone.count` - Count entities in a zone
- `resource.value` - Check resource amounts
- `phase.is` - Check current phase
- `zone.compare` - Compare two zones
- `zone.contains` - Check if zone contains specific entity
- `player.hasPlaced` - Track player-specific counters
- `game.turn` - Check turn number conditions

Example of proposed comprehensive syntax:
```yaml
when:
  # Check entity count
  - condition: "zone.count"
    with:
      zone: "/zones/board"
      entity: "piece_{player}"
      operator: ">="  # ==, !=, >, <, >=, <=
      value: 3
      
  # Check resource
  - condition: "resource.value"
    with:
      resource: "gold_{player}"
      operator: ">="
      value: 10
      
  # Check phase
  - condition: "phase.is"
    with:
      phaseSet: "game"
      phase: "combat"
```

### Effects and Triggers

Chain actions together:

#### then vs triggers
- **`then`**: Actions that execute after this action completes
- **`triggers`**: (Legacy) Similar to `then` but less commonly used

```yaml
# Preferred: Using "then" for action chaining
then:
  - action: "checkForWin"
  - action: "checkPhaseTransition"
  - action: "advanceTurn"

# Alternative syntax with parameters
then:
  - action: "updateScore"
    with:
      player: "{player}"
      amount: 10
  - action: "setPhase"
    with:
      phaseSet: "game"
      phase: "combat"

# Legacy: Using "triggers" (still supported)
triggers:
  - "check_win_condition"
  - "refill_market"
  - "advance_phase"
```

## Built-in Verbs Reference

Bluefelt provides powerful built-in verbs for common game operations. These verbs handle the complex logic so you can focus on game design.

### Movement and Placement Verbs

#### `place`
Places an entity at a specific location.

```yaml
- id: "place_marker"
  uses: "place"
  ui:
    direction: "Choose an empty cell"
    logTemplate: "{player} placed at ({row}, {col})"
```

**Arguments:**
- `location` (string): Path to placement location (e.g., "/zones/board/cells/0/0")
- `entity` (string): Entity ID to place

**Example usage in action:**
```yaml
effects:
  - verb: "place"
    args:
      location: "{location}"
      entity: "marker_{player}"
```

#### `placeWithGravity`
Places an entity that falls to the lowest available position in a column.

```yaml
- id: "drop_disc"
  uses: "placeWithGravity"
  ui:
    direction: "Choose a column"
    logTemplate: "{player} dropped a disc in column {column}"
```

**Arguments:**
- `zone` (string): Path to the grid zone
- `column` (number): Column index (0-based)
- `entity` (string): Entity ID to place

**Special behavior:**
- Automatically finds the lowest empty row in the column
- Validates that the column isn't full
- Generates column-based action maps (e.g., `/zones/board/columns/3`)

**Example usage:**
```yaml
effects:
  - verb: "placeWithGravity"
    args:
      zone: "/zones/board"
      column: "{column}"
      entity: "disc_{player}"
```

#### `moveEntity`
Moves entities between locations or zones.

```yaml
- id: "move_piece"
  uses: "moveEntity"
  ui:
    direction: "Select a piece to move"
    logTemplate: "{player} moved from {from} to {to}"
```

**Arguments:**
- `from` (string): Source path
- `to` (string): Destination path
- `entity` (string, optional): Specific entity to move

**Variations:**
```yaml
# Move specific entity
- verb: "moveEntity"
  args:
    from: "/zones/board/cells/0/0"
    to: "/zones/board/cells/1/1"
    entity: "knight_white"

# Move any entity at location
- verb: "moveEntity"
  args:
    from: "/zones/hand_p1/items/0"
    to: "/zones/discard/items"
```

### Card and Deck Verbs

#### `draw`
Draws entities from one zone to another (typically deck to hand).

```yaml
- id: "draw_cards"
  uses: "draw"
  ui:
    direction: "Draw cards"
    logTemplate: "{player} drew {count} cards"
```

**Arguments:**
- `from` (string): Source zone path
- `to` (string): Destination zone path
- `count` (number): Number to draw

**Example:**
```yaml
- verb: "draw"
  args:
    from: "/zones/deck"
    to: "/zones/hand_{player}"
    count: 3
```

#### `shuffle`
Randomizes the order of entities in a zone.

```yaml
- verb: "shuffle"
  args:
    zone: "/zones/deck"
```

#### `cards.deal`
Distributes cards evenly to multiple players.

```yaml
- verb: "cards.deal"
  args:
    from: "/zones/deck"
    count: 7
    players: ["p1", "p2", "p3", "p4"]
```

#### `cards.reveal`
Makes hidden cards visible.

```yaml
- verb: "cards.reveal"
  args:
    zone: "/zones/hand_{player}"
    indices: [0, 1, 2]  # Specific cards
    # or
    all: true           # All cards
```

### Game Flow Verbs

#### `nextTurn`
Advances to the next player's turn.

```yaml
- id: "end_turn"
  uses: "nextTurn"
  ui:
    direction: "End your turn"
    logTemplate: "{player} ended their turn"
```

**Automatic behaviors:**
- Updates `currentPlayer` to next in sequence
- Increments turn counter
- Resets per-turn states
- Triggers turn-start events

#### `setPhase`
Changes the current game phase.

```yaml
- verb: "setPhase"
  args:
    phaseSet: "main"      # Phase set name
    phase: "scoring"      # Phase within set
```

**Example phases:**
```yaml
triggers:
  - verb: "setPhase"
    args:
      phaseSet: "rounds"
      phase: "planning"
```

### Win Condition Verbs

#### `grid.lineOfMarks`
Detects lines of consecutive entities on a grid.

```yaml
- id: "check_win"
  uses: "grid.lineOfMarks"
  auto: true  # Runs automatically after placement
```

**Arguments:**
- `zone` (string): Grid zone path to check
- `entity` (string): Entity pattern to match (use `{player}` for current player)
- `lineLength` (number, default: 3): Consecutive entities needed
- `directions` (array): Directions to check
  - `"horizontal"`: Check rows
  - `"vertical"`: Check columns
  - `"diagonal"`: Check both diagonals

**Examples:**
```yaml
# Tic-tac-toe (3 in a row)
- verb: "grid.lineOfMarks"
  args:
    zone: "/zones/board"
    entity: "mark_{player}"
    lineLength: 3
    directions: ["horizontal", "vertical", "diagonal"]

# Connect 4 (4 in a row)
- verb: "grid.lineOfMarks"
  args:
    zone: "/zones/board"
    entity: "disc_{player}"
    lineLength: 4
    directions: ["horizontal", "vertical", "diagonal"]

# Gomoku (5 in a row, no diagonals)
- verb: "grid.lineOfMarks"
  args:
    zone: "/zones/board"
    entity: "stone_{player}"
    lineLength: 5
    directions: ["horizontal", "vertical"]
```

**Automatic actions:**
- Sets `/meta/gameStatus/state` to `"ended"`
- Sets `/meta/gameStatus/winner` to winning player
- Sets `/meta/gameStatus/tie` to `true` if board full with no winner

#### `checkForWin`
Generic win condition checker (custom implementation).

```yaml
- verb: "checkForWin"
  args:
    condition: "most_territory"
    # or
    condition: "highest_score"
    # or
    condition: "custom_win_check"
```

### Scoring and Resources

#### `updateScore`
Modifies player scores.

```yaml
- verb: "updateScore"
  args:
    player: "{player}"
    amount: 10
    operation: "add"  # add, subtract, set, multiply
```

#### `transferResource`
Moves resources between players or zones.

```yaml
- verb: "transferResource"
  args:
    resource: "gold"
    amount: 5
    from: "bank"
    to: "{player}"
```

### Custom Verbs

For complex game logic, you can create custom verbs:

```yaml
- id: "complex_action"
  uses: "customCombat"
  hook: "resolveCombat"  # WASM function
```

## Action Patterns

### Basic Placement Action

```yaml
- id: "place_token"
  uses: "place"
  ui:
    direction: "Place your token"
    logTemplate: "{player} placed a token at ({row}, {col})"
  conditions:
    - type: "is_current_player_turn"
    - type: "location_is_empty"
```

### Multi-Step Action

```yaml
- id: "attack_with_unit"
  uses: "customAttack"
  ui:
    direction: "Select unit to attack with"
    logTemplate: "{player} attacked {target}"
  conditions:
    - type: "is_current_player_turn"
    - type: "unit_can_attack"
  effects:
    - verb: "damage"
      args:
        target: "{target}"
        amount: "{unit.attack}"
  triggers:
    - "check_unit_destroyed"
    - "award_combat_experience"
```

### Automatic Action

Automatic actions run without player input and are crucial for:
- Phase transitions based on game state
- Win condition checking
- Resource management
- Cleanup operations

```yaml
- id: "checkPhaseTransition"
  auto: true  # Runs automatically when triggered
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

# Another example: refill market
- id: "refillMarket"
  uses: "draw"
  auto: true
  when:
    - condition: "zone.isEmpty"
      with:
        zone: "/zones/market"
  with:
    from: "/zones/deck"
    to: "/zones/market"
    count: 5
```

**Triggering Automatic Actions:**
1. From other actions via `then`
2. From phase enter actions
3. From other automatic actions

### Conditional Triggers

```yaml
- id: "harvest_resources"
  uses: "customHarvest"
  ui:
    direction: "Harvest from this tile"
  triggers:
    - action: "bonus_harvest"
      if:
        condition: "has_upgrade"
        upgrade: "improved_tools"
    - action: "seasonal_bonus"
      if:
        condition: "current_season"
        season: "autumn"
```

## Action Availability

Actions can be made available through different mechanisms:

### Turn-Based Availability

```yaml
conditions:
  - type: "is_current_player_turn"
  - type: "phase_is"
    phase: "main"
```

### Resource-Based Availability

```yaml
conditions:
  - type: "has_resources"
    resources:
      gold: 10
      wood: 5
cost:
  - resource: "gold"
    amount: 10
  - resource: "wood"
    amount: 5
```

### Location-Based Availability

The action map system automatically determines where actions can be performed:

```yaml
# Server generates action map like:
{
  "/zones/board/cells/0/0": {
    "action": "place_marker",
    "direction": "Place your marker here"
  },
  "/zones/board/columns/3": {
    "action": "drop_disc",
    "direction": "Drop disc in this column"
  }
}
```

### Custom Availability

```yaml
conditions:
  - type: "custom"
    hook: "canPerformSpecialMove"
    args:
      checkType: "castling"
```

## Game Log Templates

Good game logs enhance the player experience by clearly communicating what happened.

### Template Syntax

- `{player}` - Replaced with player name
- `{row}`, `{col}` - Grid coordinates (1-indexed for display)
- `{column}` - Column number (1-indexed)
- `{from}`, `{to}` - Movement locations
- `{count}` - Numeric values
- Any action argument can be referenced

### Examples

```yaml
# Simple placement
logTemplate: "{player} placed a marker"

# With coordinates
logTemplate: "{player} placed at ({row}, {col})"

# Column-based
logTemplate: "{player} dropped a disc in column {column}"

# Movement
logTemplate: "{player} moved from ({fromRow}, {fromCol}) to ({toRow}, {toCol})"

# Resource gain
logTemplate: "{player} gained {amount} {resource}"

# Combat
logTemplate: "{player}'s {attacker} dealt {damage} damage to {defender}"
```

### Coordinate Handling

The server automatically extracts and formats coordinates:

- Location paths: `/zones/board/cells/1/2` → row=2, col=3 (1-indexed)
- Direct arguments: `{row: 0, col: 1}` → row=1, col=2 (1-indexed)
- Column arguments: `{column: 3}` → column=4 (1-indexed)

## Best Practices

### Action Design

1. **Clear Purpose**
   ```yaml
   # ✓ Clear action
   - id: "place_worker"
     uses: "place"
     ui:
       direction: "Place a worker on an empty space"
   
   # ✗ Vague action
   - id: "do_thing"
     uses: "custom"
     ui:
       direction: "Do something"
   ```

2. **Consistent Naming**
   ```yaml
   # ✓ Consistent pattern
   - id: "draw_card"
   - id: "play_card"
   - id: "discard_card"
   
   # ✗ Inconsistent
   - id: "draw"
   - id: "card_play"
   - id: "discarding"
   ```

3. **Informative Logs**
   ```yaml
   # ✓ Informative
   logTemplate: "{player} moved knight from ({fromRow}, {fromCol}) to ({toRow}, {toCol})"
   
   # ✗ Too vague
   logTemplate: "{player} did an action"
   ```

### Performance Considerations

1. **Minimize Cascade Effects**
   ```yaml
   # ✓ Controlled triggers
   triggers:
     - "check_win"
     - "update_score"
   
   # ✗ Potential infinite loop
   triggers:
     - "trigger_more_actions"
     - "which_trigger_more"
   ```

2. **Batch Operations**
   ```yaml
   # ✓ Single verb for multiple operations
   - verb: "cards.deal"
     args:
       count: 7
       players: ["p1", "p2", "p3"]
   
   # ✗ Multiple individual operations
   - verb: "draw"
     args: {to: "p1", count: 7}
   - verb: "draw"
     args: {to: "p2", count: 7}
   ```

### Common Patterns

1. **Turn End Sequence**
   ```yaml
   - id: "end_turn"
     uses: "nextTurn"
     triggers:
       - "draw_card"
       - "reset_action_points"
       - "check_phase_end"
   ```

2. **Resource Generation**
   ```yaml
   - id: "collect_income"
     uses: "generateResources"
     auto: true
     conditions:
       - type: "phase_is"
         phase: "income"
   ```

3. **Combat Resolution**
   ```yaml
   - id: "resolve_combat"
     uses: "combat"
     effects:
       - verb: "damage"
         args:
           target: "{defender}"
           amount: "{attacker.strength}"
     triggers:
       - "check_destroyed"
       - "award_experience"
   ```

## Debugging Actions

### Common Issues

1. **Action Not Available**
   - Check conditions are met
   - Verify action map generation
   - Ensure proper turn/phase

2. **Wrong Coordinates in Log**
   - Verify argument names match template
   - Check path format for location extraction
   - Ensure 1-indexing conversion

3. **Verb Not Found**
   - Confirm verb is implemented in server
   - Check spelling and case
   - Verify custom hooks are loaded

### Testing Actions

1. **Unit Test Individual Verbs**
   ```rust
   #[test]
   fn test_place_with_gravity() {
       let result = apply_verb(&mut state, "placeWithGravity", &args);
       assert!(result.is_ok());
       // Verify entity at bottom row
   }
   ```

2. **Integration Test Action Chains**
   ```rust
   #[test]
   fn test_complete_turn() {
       // Place piece
       // Trigger win check
       // Verify game ends
   }
   ```

3. **Manual Testing**
   - Try edge cases (full columns, board edges)
   - Test invalid moves
   - Verify game log messages

## Related Documentation

- [Developing Games: Zones](./developing-games-zones.md) - Zone types and special mechanics
- [Game Implementation Guide](./game-implementation-guide.md) - Full implementation process
- [Game Log Parameters](./game-log-parameters.md) - Log template details
- [State Structure](./state-structure.md) - Understanding game state

Actions are the heart of your game's interactivity. Well-designed actions make games intuitive and enjoyable!

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
- id: "placeMark"         # ✓ Valid (note: no underscore for tic-tac-toe)
- id: "draw_cards"        # ✓ Valid
- id: "move_piece"        # ✓ Valid
```

#### `uses` (string, required)
The verb that executes the action. Can be built-in or custom.

```yaml
uses: "place"             # Built-in verb
uses: "moveEntity"        # Built-in verb
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
  logTemplate: "{player} placed their mark at ({row}, {col})"
  
  # CRITICAL: Action names must match client expectations exactly
  # Tests and client code expect specific action names like:
  # - placeMark (NOT placeMarker)
  # - selectRank, selectPlayer
  # - dropDisc, movePiece
  # Mismatched names cause action map failures
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
- id: "placeMark"
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

#### `dealToAllPlayers`
Deals cards to all players in the game. This verb automatically iterates through all connected players and deals cards according to the specified mode.

```yaml
- id: "dealCards"
  uses: "dealToAllPlayers"
  ui:
    logTemplate: "Dealing cards to all players"
```

**Arguments:**
- `from` (string): Source zone path (e.g., "/zones/pool" or "/zones/deck")
- `toZonePattern` (string): Destination zone pattern with `{player}` placeholder (e.g., "/zones/hand_{player}")
- `count` (number): Number of cards to deal per round
- `mode` (string, optional): Dealing mode - "fixed" or "dealAll" (defaults to "fixed")

**Modes:**
- `"fixed"`: Each player gets exactly `count` cards (original behavior)
- `"dealAll"`: Deal `count` cards to each player in rotation until the deck is empty

**Examples:**
```yaml
# Fixed mode - each player gets exactly 7 cards
- verb: "dealToAllPlayers"
  args:
    from: "/zones/pool"
    toZonePattern: "/zones/hand_{player}"
    count: 7
    mode: "fixed"  # optional, this is the default

# Deal all mode - deal entire deck
- verb: "dealToAllPlayers"
  args:
    from: "/zones/deck"
    toZonePattern: "/zones/hand_{player}"
    count: 1  # deal 1 card at a time
    mode: "dealAll"  # continues until deck is empty

# Deal all mode with multiple cards per round
- verb: "dealToAllPlayers"
  args:
    from: "/zones/deck"
    toZonePattern: "/zones/hand_{player}"
    count: 5  # deal 5 cards at a time to each player
    mode: "dealAll"
```

The `dealAll` mode is particularly useful for games like War and Old Maid where the entire deck needs to be distributed among players, handling uneven divisions automatically (e.g., 52 cards among 3 players results in 18, 17, 17 cards).

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

#### `formMelds`
Validates and forms melds (sets or runs) from selected cards. This verb handles the complex logic of meld formation for rummy-style games.

```yaml
- id: "makeMeld"
  uses: "formMelds"
  ui:
    direction: "Select cards to form a meld"
    logTemplate: "{player} formed a meld"
```

**Arguments:**
- `fromZone` (string): Source zone containing the cards (e.g., "/zones/hand_{player}")
- `toZone` (string): Destination zone for formed melds (e.g., "/zones/melds_{player}")
- `selectedCards` (array): Array of card indices to form into a meld
- `meldType` (string, optional): Type of meld - "set" or "run" (auto-detects if not specified)

**Example:**
```yaml
# Form a specific type of meld
- verb: "formMelds"
  args:
    fromZone: "/zones/hand_p1"
    toZone: "/zones/melds_p1"
    selectedCards: [0, 1, 2]  # indices of cards in hand
    meldType: "set"  # require it to be a set

# Auto-detect meld type
- verb: "formMelds"
  args:
    fromZone: "/zones/hand_{player}"
    toZone: "/zones/melds_{player}"
    selectedCards: [3, 4, 5, 6]  # will validate as set or run
```

**Validation Rules:**
- **Sets**: 3+ cards of the same rank (e.g., 7♠, 7♥, 7♣)
- **Runs**: 3+ consecutive cards of the same suit (e.g., 5♦, 6♦, 7♦)

The verb automatically validates the selected cards form a valid meld before moving them. Invalid selections will return an error.

**Note**: For complex games requiring multiple meld types or player-controlled meld selection, this should be implemented as a multi-step action allowing iterative meld formation.

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

**Important Note**: The `nextTurn` verb correctly uses the actual number of connected players from `state.players.length`, not the manifest's maximum player count. This ensures it works correctly for games with variable player counts (e.g., a 2-4 player game with only 3 players connected).

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
- Sets `gameStatus.state` to `"ended"`
- Sets `gameStatus.winner` to winning player
- Sets `gameStatus.tie` to `true` if board full with no winner

**Pattern Matching:**
The `entity` parameter supports template variables. For example:
- `"mark_{player}"` will match `"mark_p1"` when evaluated for player p1
- `"disc_{player}"` will match `"disc_p2"` when evaluated for player p2

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

### Control Flow Verbs

#### `conditionalAction`
Executes different actions based on a condition evaluation.

```yaml
- id: "checkWinOrAdvance"
  uses: "conditionalAction"
  auto: true
  with:
    condition:
      condition: "grid.lineOfMarks"
      with:
        zone: "/zones/board"
        entity: "mark_{player}"
        lineLength: 3
        directions: ["horizontal", "vertical", "diagonal"]
    then:  # Actions to execute if condition is true
      - action: "endGame"
    else:  # Actions to execute if condition is false
      - action: "advanceTurn"
```

**Arguments:**
- `condition` (object): Condition to evaluate (see Conditions documentation)
- `then` (array): Actions to execute if condition is true
- `else` (array): Actions to execute if condition is false

**Note:** The verb also supports `ifTrue`/`ifFalse` as aliases for `then`/`else`:
```yaml
with:
  condition: { ... }
  ifTrue: [ ... ]   # Same as "then"
  ifFalse: [ ... ]  # Same as "else"
```

### Phase Management Verbs

#### `setPhase`
Transitions the game to a different phase.

```yaml
- id: "endGame"
  uses: "setPhase"
  auto: true
  with:
    phaseSet: "game"  # The phase set to modify
    phase: "end"      # The phase within that set
```

**Arguments:**
- `phaseSet` (string): The phase set to modify (e.g., "game", "turn", "round")
- `phase` (string): The specific phase to set within that phase set

**Important:** Do NOT use dot notation like `phase: "game.end"`. Always specify `phaseSet` and `phase` separately:
```yaml
# ❌ INCORRECT - Don't do this:
with:
  phase: "game.end"

# ✅ CORRECT - Do this instead:
with:
  phaseSet: "game"
  phase: "end"
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
l```

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

### Multi-Step Actions

Multi-step actions allow players to make sequential decisions before executing a complex action. This is essential for games that require selecting multiple targets, confirming actions, or making dependent choices.

#### Basic Multi-Step Structure

```yaml
- id: "movePiece"
  isMultiStep: true
  cancellable: true  # Players can cancel mid-action
  confirmBeforeFinalizing: true  # Show confirmation before executing
  stateStore: ["selectedPiece", "destination"]  # Variables to track
  
  # Define the sequential steps
  steps:
    - id: "selectPiece"
      as: "bf.selectEntity"
      with:
        zone: "/zones/board"
        entityFilter: "piece_{player}"
      ui:
        direction: "Select a piece to move"
      store: "selectedPiece"  # Store selection for later use
    
    - id: "selectDestination" 
      as: "bf.selectLocation"
      with:
        zone: "/zones/board"
        emptyOnly: true
      ui:
        direction: "Select where to move {selectedPiece}"
      store: "destination"
  
  # The final action to execute
  result:
    as: "moveEntity"
    with:
      from: "{selectedPiece.location}"
      to: "{destination}"
    ui:
      logTemplate: "{player} moved from {selectedPiece.location} to {destination}"
  
  # Optional confirmation prompt
  ui:
    confirmationPrompt: "Move piece from {selectedPiece.location} to {destination}?"
```

#### Multi-Step Components

**1. Metadata Fields:**
- `isMultiStep: true` - Marks this as a multi-step action
- `cancellable: true/false` - Whether players can cancel during the action
- `confirmBeforeFinalizing: true/false` - Show confirmation before executing result
- `stateStore: []` - List of variable names that will be stored during steps

**2. Steps Array:**
Each step defines a user interaction:
```yaml
steps:
  - id: "stepId"  # Unique identifier for this step
    as: "verbName"  # The selection verb to use
    with: {}  # Arguments for the verb
    ui:
      direction: "Instruction for player"
    store: "variableName"  # Where to store the result
    when: []  # Optional conditions for this step
```

**3. Result Action:**
The final action executed after all steps complete:
```yaml
result:
  as: "verbName"  # The verb to execute
  with: {}  # Arguments (can use stored variables)
  ui:
    logTemplate: "What appears in game log"
```

#### Multi-Step Actions in Action Maps

**IMPORTANT**: Multi-step actions appear in action maps with a special format:
- Path: `_multiStep_{actionId}` (e.g., `_multiStep_movePiece`)
- Type: `"multiStep"` in the action map entry
- Single entry per multi-step action (not individual entries for each possible selection)

Example action map for a multi-step move action:
```json
{
  "p1": {
    "_multiStep_movePiece": {
      "action": "movePiece",
      "type": "multiStep",
      "direction": "Move one of your pieces",
      "args": {}
    }
  }
}
```

This is different from regular actions which have specific target paths like `/zones/board/cells/0/0`.

#### Available Selection Verbs

**`bf.selectEntity`** - Select an entity from a zone
```yaml
as: "bf.selectEntity"
with:
  zone: "/zones/board"  # Zone to select from
  entityFilter: "piece_{player}"  # Filter pattern
  minCount: 1  # Minimum selections (default: 1)
  maxCount: 3  # Maximum selections (default: 1)
```

**`bf.selectLocation`** - Select a location/cell
```yaml
as: "bf.selectLocation"
with:
  zone: "/zones/board"  # Zone to select from
  emptyOnly: true  # Only empty locations
  adjacentTo: "{selectedPiece.location}"  # Must be adjacent
```

**`bf.selectCard`** - Select cards from hand
```yaml
as: "bf.selectCard"
with:
  zone: "/zones/hand_{player}"
  minCount: 1
  maxCount: 3
  property: "suit"  # Optional: filter by property
  value: "hearts"   # Optional: property value
```

**`bf.selectChoice`** - Present options to choose from
```yaml
as: "bf.selectChoice"
with:
  options:
    - id: "rock"
      label: "Rock"
      description: "Beats scissors"
    - id: "paper"
      label: "Paper"  
      description: "Beats rock"
    - id: "scissors"
      label: "Scissors"
      description: "Beats paper"
```

#### Variable Substitution

Stored variables can be referenced in later steps using template syntax:

**Simple Values:**
```yaml
# If selectedRank stores "7"
direction: "Select all {selectedRank}s from your hand"
# Becomes: "Select all 7s from your hand"
```

**Object Properties:**
```yaml
# If selectedPiece stores { location: "/zones/board/cells/0/0", entity: "piece_p1" }
direction: "Move from {selectedPiece.location}"
# Becomes: "Move from /zones/board/cells/0/0"
```

**Nested Properties:**
```yaml
# Complex object access
with:
  from: "{selectedCard.properties.owner}'s hand"
  rank: "{selectedCard.properties.rank}"
```

**Important Note on Path Formatting:**

When using template substitution in multi-step actions, be aware that:
- **UI text** (directions, prompts, logs) may format paths for readability (e.g., "Move piece to (1, 1)?")
- **Action arguments** preserve the actual paths needed by verbs (e.g., `/zones/board/cells/0/0`)

This distinction is handled automatically by the server. Game developers don't need to worry about it - just use template variables normally and the server will format them appropriately based on context.

#### Conditional Steps

Steps can have conditions that determine if they execute:

```yaml
steps:
  - id: "selectTarget"
    as: "bf.selectEntity"
    when:
      - condition: "zone.count"
        with:
          zone: "/zones/board"
          entity: "enemy_*"
          operator: ">"
          value: 0
    with:
      zone: "/zones/board"
      entityFilter: "enemy_*"
    ui:
      direction: "Select an enemy to attack"
    store: "target"
```

#### Complex Examples

**Trading Cards Between Players:**
```yaml
- id: "tradeCards"
  isMultiStep: true
  cancellable: true
  confirmBeforeFinalizing: true
  stateStore: ["targetPlayer", "offeredCards", "requestedCards"]
  
  steps:
    - id: "selectPlayer"
      as: "bf.selectChoice"
      with:
        options: "{otherPlayers}"  # Dynamically generated
      ui:
        direction: "Select a player to trade with"
      store: "targetPlayer"
    
    - id: "selectOffer"
      as: "bf.selectCard"
      with:
        zone: "/zones/hand_{player}"
        minCount: 1
        maxCount: 5
      ui:
        direction: "Select cards to offer"
      store: "offeredCards"
    
    - id: "selectRequest"
      as: "bf.selectCard"
      with:
        zone: "/zones/hand_{targetPlayer}"
        minCount: 1
        maxCount: 5
      ui:
        direction: "Select cards you want"
      store: "requestedCards"
  
  result:
    as: "performTrade"
    with:
      player1: "{player}"
      player2: "{targetPlayer}"
      give: "{offeredCards}"
      receive: "{requestedCards}"
    ui:
      logTemplate: "{player} traded {offeredCards.length} cards with {targetPlayer}"
  
  ui:
    confirmationPrompt: "Trade {offeredCards.length} cards for {requestedCards.length} cards with {targetPlayer}?"
```

**Combat with Dice Roll:**
```yaml
- id: "complexAttack"
  isMultiStep: true
  cancellable: true
  stateStore: ["attacker", "target", "diceRoll"]
  
  steps:
    - id: "selectAttacker"
      as: "bf.selectEntity"
      with:
        zone: "/zones/battlefield"
        entityFilter: "unit_{player}"
        hasProperty: "canAttack"
        propertyValue: true
      ui:
        direction: "Select your attacking unit"
      store: "attacker"
    
    - id: "selectTarget"
      as: "bf.selectEntity"
      with:
        zone: "/zones/battlefield"
        entityFilter: "unit_*"
        notFilter: "unit_{player}"
        withinRange: "{attacker.range}"
        fromLocation: "{attacker.location}"
      ui:
        direction: "Select target for {attacker.name}"
      store: "target"
    
    - id: "rollDice"
      as: "bf.rollDice"
      with:
        count: "{attacker.attackDice}"
        sides: 6
      ui:
        direction: "Rolling {attacker.attackDice} dice..."
      store: "diceRoll"
  
  result:
    as: "resolveCombat"
    with:
      attacker: "{attacker}"
      defender: "{target}"
      damage: "{diceRoll.total}"
    ui:
      logTemplate: "{player}'s {attacker.name} dealt {diceRoll.total} damage to {target.name}"
```

#### Client-Server Flow

1. **Initiation**: Server sends available multi-step actions in action map
2. **Step Processing**: 
   - Client sends selections for each step
   - Server validates and stores intermediate state
   - Server sends next step or confirmation request
3. **Confirmation**: Client confirms or cancels
4. **Execution**: Server executes result action and sends patches
5. **Cleanup**: Multi-step state is cleared

#### Best Practices

1. **Clear Step Instructions**: Each step should have explicit directions
2. **Meaningful Variable Names**: Use descriptive names for stored values
3. **Validate Early**: Add conditions to steps to prevent invalid states
4. **Handle Cancellation**: Ensure game state remains consistent if cancelled
5. **Confirmation Templates**: Use stored variables to create clear confirmation prompts
6. **Log Everything**: Include comprehensive log templates in the result action

#### Visual Feedback

The client automatically provides visual feedback during multi-step actions:
- Progress indicators showing current step
- Selected entities remain highlighted
- Cancel button availability based on `cancellable` flag
- Confirmation dialogs with formatted prompts
- Connection lines between selected locations (for movement actions)

### Traditional Multi-Step Pattern (Legacy)

For simpler cases, you can still use the traditional pattern:

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

**⚠️ CRITICAL: All automatic actions MUST include log templates!**

Players need to understand what the game is doing automatically. Without proper logging, the game feels opaque and confusing. Every automatic decision must be clearly communicated.

```yaml
# BAD: No logging
- id: "checkPhaseTransition"
  auto: true
  uses: "setPhase"
  with:
    phase: "movement"

# GOOD: Clear logging
- id: "checkPhaseTransition"
  auto: true
  uses: "setPhase"
  with:
    phase: "movement"
  ui:
    logTemplate: "All pieces placed - entering movement phase"

# Example with dynamic content
- id: "formPairs"
  auto: true
  uses: "detectPairs"
  ui:
    logTemplate: "{player} forms a pair of {rank}s"
```

**Common Automatic Actions That Need Logging:**
- `nextTurn` - "Turn passes to {nextPlayer}"
- `checkWin` - "Checking for winning conditions..."
- `setPhase` - "Moving to {phase} phase"
- `formPairs` - "{player} forms a pair of {rank}s"
- `calculateWinner` - "Calculating winner based on scores..."
- `drawCard` - "{player} drew a {rank} of {suit}"
- `transferCards` - "{source} gives {count} cards to {target}"
- `shuffle` - "Shuffling the {zone}..."
- `dealCards` - "Dealing {count} cards to each player"
- Any phase transition - "Phase changing from {oldPhase} to {newPhase}"
- Any automatic win condition - "{winner} wins by {condition}!"
- Any automatic resource generation - "{player} receives {amount} {resource}"

**Warning**: Many existing games are missing log templates for automatic actions, causing silent state transitions that confuse players. Always audit your automatic actions to ensure they have appropriate logging!

Complete example with conditions:

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
  ui:
    logTemplate: "All pieces placed - moving to movement phase"

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
  ui:
    logTemplate: "Market refilled with {count} new cards"
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
    "action": "placeMark",
    "direction": "Place your mark here"
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
    verb: "canPerformSpecialMove"
    args:
      checkType: "castling"
```

## Game Log Templates

Good game logs enhance the player experience by clearly communicating what happened.

### Critical: Player References ⚠️

**NEVER use hardcoded player names in log templates!** Always use `p1`, `p2`, etc. when referencing specific players. The server automatically replaces these with actual player names.

```yaml
# ✗ WRONG - Never hardcode names
logTemplate: "Player 1 draws a card"
logTemplate: "Alice wins the game"

# ✓ CORRECT - Use player IDs
logTemplate: "p1 draws a card"        # Shows: "Alice draws a card"
logTemplate: "{player} wins the game" # Shows: "Bob wins the game"
```

### Template Syntax

- `{player}` - Replaced with current player name
- `{nextPlayer}` - Replaced with next player name  
- `p1`, `p2`, `p3`, `p4` - Replaced with specific player names
- `{row}`, `{col}` - Grid coordinates (1-indexed for display)
- `{column}` - Column number (1-indexed)
- `{from}`, `{to}` - Movement locations
- `{count}` - Numeric values
- Any action argument can be referenced

### Examples

```yaml
# Simple placement
logTemplate: "{player} placed a mark"

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

## Animation Support

### Server Animation Hints

The Bluefelt engine supports providing animation hints to clients through the `_animation` metadata field in patches. This allows the server to suggest how state changes should be visually presented, while keeping clients generic and game-agnostic.

#### Basic Animation Hint Structure

When a verb generates a patch, it can include animation metadata:

```rust
// In server verb implementation
patches.push(json!({
    "op": "replace",
    "path": format!("/zones/board/cells/{}/{}", row, col),
    "value": { "entity": entity_id },
    "_animation": {
        "type": "entity_spawn",
        "duration": 600,
        // Additional metadata specific to animation type
    }
}));
```

#### Animation Types and Metadata

**1. Entity Spawn**
Used when new entities appear in the game:
```json
{
    "_animation": {
        "type": "entity_spawn",
        "duration": 400,
        "style": "fade_in"  // or "pop_in", "slide_in"
    }
}
```

**2. Entity Movement**
For entities moving between locations:
```json
{
    "_animation": {
        "type": "entity_movement",
        "duration": 600,
        "fromPosition": { "row": 0, "col": 2 },
        "toPosition": { "row": 3, "col": 2 },
        "style": "slide"  // or "arc", "teleport"
    }
}
```

**3. Gravity Drop (Connect 4 Style)**
Special case for pieces that fall with gravity:
```json
{
    "_animation": {
        "type": "entity_spawn",
        "duration": 600,
        "isGravityDrop": true,
        "fromPosition": { "row": 0, "col": 3 },
        "toPosition": { "row": 5, "col": 3 }
    }
}
```

**4. Card Operations**
For card game animations:
```json
{
    "_animation": {
        "type": "card_deal",
        "duration": 300,
        "dealerPosition": "deck",
        "targetPosition": "hand"
    }
}
```

#### Implementing Animation Hints in Verbs

When creating custom verbs or modifying existing ones, consider adding animation hints:

```rust
// Example: placeWithGravity verb
impl Verb for PlaceWithGravity {
    fn apply(&self, state: &mut GameState, args: &Map<String, Value>) -> Result<Vec<Value>> {
        // ... placement logic ...
        
        // Create patch with animation hint
        patches.push(json!({
            "op": "replace",
            "path": format!("/zones/{}/cells/{}/{}", zone_id, row, column),
            "value": { "entity": entity_id },
            "_animation": {
                "type": "entity_spawn",
                "duration": 600,
                "isGravityDrop": true,
                "fromPosition": { "row": 0, "col": column },
                "toPosition": { "row": row, "col": column }
            }
        }));
        
        Ok(patches)
    }
}
```

#### Client Processing

Clients that support animations will:
1. Detect the `_animation` field in patches
2. Extract animation metadata
3. Apply the patch to update state
4. Trigger appropriate visual animations based on the hints

Clients that don't support animations simply ignore the `_animation` field and apply patches normally.

#### Best Practices for Animation Hints

1. **Keep It Generic**: Animation hints should describe what happened, not how to animate it
2. **Include Timing**: Always provide duration suggestions
3. **Position Data**: Include position information for movement animations
4. **Optional Enhancement**: Animations should enhance, not be required for gameplay
5. **Consistent Types**: Use standard animation type names across your game

### Animation Support Through Discrete Patches

To enable smooth client-side animations, complex actions should generate multiple patches rather than a single atomic update:

### Wrong Approach (No Animation Possible) ❌
```yaml
# This verb does everything at once
- id: "askForCards"
  uses: "transferMatchingCards"  # Custom verb that does it all
  with:
    from: "/zones/hand_{target}"
    to: "/zones/hand_{player}"
    rank: "{selectedRank}"
```

### Correct Approach (Animation-Ready) ✅
```yaml
# Step 1: Query what cards to transfer
- id: "findMatchingCards"
  uses: "queryEntities"
  with:
    zone: "/zones/hand_{target}"
    property: "rank"
    value: "{selectedRank}"
    storeAs: "cardsToTransfer"

# Step 2: Remove each card (generates patch)
- id: "takeCard"
  uses: "removeEntity"
  forEach: "{cardsToTransfer}"
  with:
    from: "/zones/hand_{target}"
    entity: "{item}"
  ui:
    logTemplate: "{target} gives a {selectedRank}"

# Step 3: Add to player's hand (generates patch)
- id: "receiveCard"
  uses: "addEntity"
  forEach: "{cardsToTransfer}"
  with:
    to: "/zones/hand_{player}"
    entity: "{item}"

# Step 4: Check for pairs (generates patch if pairs form)
- id: "checkPairs"
  uses: "formPairs"
  with:
    player: "{player}"
  ui:
    logTemplate: "{player} forms a pair of {selectedRank}s"
```

### Benefits of Discrete Patches

1. **Sequential Animation**: Client can animate each step
2. **Visual Clarity**: Players see cards move one by one
3. **Better Feedback**: Each action can have its own log entry
4. **Debugging**: Easier to track what happened when

### Implementation Guidelines

When designing complex actions:
1. **Break into logical steps**: Each visual change = one patch
2. **Order matters**: Remove → Move → Add → Side effects
3. **Test animations**: Verify client receives patches in order
4. **Log each step**: Players should understand the sequence

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

---

# Multi-Step Actions Library

The Bluefelt engine provides a library of pre-built multi-step action patterns that games can use directly. These patterns handle common game mechanics that require multiple player decisions.

## Available Library Actions

### 1. Trade Pattern (`ms.trade`)
Exchange items between players or with the game bank.

**Steps:**
1. Select items to offer
2. Select items to request  
3. Confirm trade

**Usage:**
```yaml
- id: tradeWithBank
  uses: ms.trade
  with:
    minOffer: 1
    maxOffer: 5
    minRequest: 1
    maxRequest: 5
  ui:
    direction: "Trade cards with the bank"
```

### 2. Move with Selection (`ms.moveWithSelection`)
Select a piece, then choose where to move it.

**Steps:**
1. Select piece to move
2. Select destination
3. Optional confirmation

**Usage:**
```yaml
- id: movePiece
  uses: ms.moveWithSelection
  with:
    pieceFilter: "myPieces"
    allowedDestinations: "emptySpaces"
  ui:
    direction: "Move one of your pieces"
```

### 3. Card Exchange (`ms.cardExchange`)
Exchange cards with another source.

**Steps:**
1. Select cards to give
2. Select cards to receive
3. Confirm exchange

**Required Parameters:**
- `minGive`: Minimum cards to give
- `maxGive`: Maximum cards to give
- `exchangeSource`: Where to exchange with (e.g., "deck", "discard", "hand_p2")

**Usage:**
```yaml
- id: exchangeWithDeck
  uses: ms.cardExchange
  with:
    minGive: 2
    maxGive: 3
    exchangeSource: "deck"
  ui:
    direction: "Exchange cards with the deck"
```

### 4. Resource Conversion (`ms.convertResources`)
Transform resources into other types.

**Steps:**
1. Select resources to convert
2. Choose conversion option
3. Confirm conversion

**Usage:**
```yaml
- id: convertResources
  uses: ms.convertResources
  with:
    conversionRates:
      wood: { stone: 2, gold: 5 }
      stone: { wood: 2, gold: 3 }
  ui:
    direction: "Convert your resources"
```

### 5. Auction Bid (`ms.placeBid`)
Place bids on auction items.

**Steps:**
1. Select item to bid on
2. Enter bid amount
3. Confirm bid

**Optional Parameters:**
- `bidIncrement`: Minimum bid increment (default: 1)

**Usage:**
```yaml
- id: bidOnItem
  uses: ms.placeBid  
  with:
    bidIncrement: 5
  ui:
    direction: "Place a bid on an auction item"
```

### 6. Complex Placement (`ms.complexPlacement`)
Place items with multiple configuration options.

**Steps:**
1. Choose what to place
2. Select location
3. Choose orientation (if applicable)
4. Confirm placement

**Usage:**
```yaml
- id: placeBuilding
  uses: ms.complexPlacement
  with:
    availableTypes: ["house", "factory", "farm"]
    requiresOrientation: true
  ui:
    direction: "Place a building on the board"
```

### 7. Multi-Target Selection (`ms.multiTarget`)
Select multiple targets in an area.

**Steps:**
1. Select origin point
2. Select affected targets
3. Confirm targets

**Required Parameters:**
- `range`: Maximum distance from origin
- `maxTargets`: Maximum number of targets

**Usage:**
```yaml
- id: areaEffect
  uses: ms.multiTarget
  with:
    range: 2
    maxTargets: 4
  ui:
    direction: "Select targets for area effect"
```

### 8. Sequential Choices (`ms.sequentialChoice`)
Make a series of related decisions.

**Steps:**
1. First choice
2. Second choice (based on first)
3. Optional third choice

**Required Parameters:**
- `prompt1`: First choice prompt
- `choices1`: First choice options
- `prompt2`: Second choice prompt  
- `choiceMapping`: How first choice affects second

**Usage:**
```yaml
- id: complexDecision
  uses: ms.sequentialChoice
  with:
    prompt1: "Choose action type"
    choices1: ["attack", "defend", "special"]
    prompt2: "Choose specific action"
    choiceMapping:
      attack: ["sword", "bow", "magic"]
      defend: ["shield", "dodge", "counter"]
      special: ["heal", "boost", "teleport"]
  ui:
    direction: "Make your move"
```

## Integration with Games

### Basic Usage

To use a library action, reference it with the `uses` field:

```yaml
- id: myTradeAction
  uses: ms.trade
  with:
    # Parameters specific to your game
  when:
    # Your conditions
  ui:
    direction: "Trade with other players"
```

### Customization

Library actions can be customized through:

1. **Parameters**: Configure behavior through the `with` field
2. **Conditions**: Add game-specific conditions in the `when` field
3. **UI**: Customize prompts and directions
4. **Post-Actions**: Add `then` actions to trigger after completion

### Example: Three Men's Morris Movement

```yaml
# Using the library for piece movement
- id: movePiece
  uses: ms.moveWithSelection
  when:
    - condition: player.isActor
    - condition: phase.is
      with:
        phase: "movement"
  then:
    - action: checkForWin
    - action: advanceTurn
  ui:
    direction: "Move one of your pieces"
    logTemplate: "{player} moved a piece"
```

## Best Practices

1. **Use Library Actions When Possible**: Prefer library actions over custom implementations for common patterns

2. **Provide Clear UI**: Always include helpful directions and prompts

3. **Validate Parameters**: Ensure all required parameters are provided

4. **Consider Cancellation**: Most library actions support cancellation - consider if this fits your game

5. **Test Multi-Step Flows**: Thoroughly test the entire flow, including edge cases like cancellation

## Implementation Notes

- Library actions are processed server-side with full state access
- Each step maintains temporary state accessible via `{stored_data.stepId}`
- Cancellation returns the game to the state before the action started
- UI components automatically handle multi-step progress indicators

## Future Library Additions

The library is designed to be extensible. Future additions may include:
- Draft patterns (simultaneous hidden selection)
- Negotiation patterns (offer/counter-offer)
- Quest patterns (multi-stage objectives)
- Combat patterns (attack/defend sequences)

---

## Related Documentation

- [Developing Games: Zones](./developing-games-zones.md) - Zone types and special mechanics
- [Game Implementation Guide](./game-implementation-guide.md) - Full implementation process
- [Game Log Parameters](./game-log-parameters.md) - Log template details
- [State Structure](./state-structure.md) - Understanding game state

Actions are the heart of your game's interactivity. Well-designed actions make games intuitive and enjoyable!

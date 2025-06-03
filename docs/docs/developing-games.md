# Developing Games: Overview

Bluefelt is a platform for creating turn-based multiplayer games using declarative YAML files. This guide covers everything you need to know about designing, implementing, and deploying games on the Bluefelt platform.

> **📘 Implementation Guide**: For a detailed step-by-step process on implementing new games, see the [Game Implementation Guide](./game-implementation-guide.md). The implementation guide is designed for AI assistants, while this guide is tailored for human developers.

## Overview

Games in Bluefelt are defined using a collection of YAML files that describe:
- **Game metadata** (name, players, version)
- **Entities** (pieces, cards, tokens)
- **Zones** (game board areas, hands, decks)
- **Actions** (what players can do)
- **Phases** (game flow and turn structure)

The Bluefelt engine handles state management, networking, and rule enforcement automatically, allowing you to focus on game design rather than technical implementation.

## Game Structure

Every game is organized in a versioned directory structure:

```
games/
└── your-game/
    └── 1.0/
        ├── manifest.yaml    # Game metadata and setup
        ├── entities.yaml    # Game pieces and components
        ├── zones.yaml      # Game areas and containers
        ├── actions.yaml    # Player actions
        ├── phases.yaml     # Game flow (optional)
        ├── events.yaml     # Triggered effects (optional)
        ├── victory.yaml    # Win conditions (optional)
        ├── computed.yaml   # Dynamic calculations (optional)
        ├── hooks.wasm      # Custom logic (optional)
        └── variants/       # Game variants (optional)
            ├── variant-1.yaml
            └── variant-2.yaml
```

### Required Files

- **`manifest.yaml`** - Game identification and metadata
- **`entities.yaml`** - All game components (pieces, cards, tokens)
- **`zones.yaml`** - All game areas (board, hands, decks)
- **`actions.yaml`** - All possible player actions

### Optional Files

- **`phases.yaml`** - Multi-phase game flow
- **`events.yaml`** - Automatic triggers based on game state
- **`victory.yaml`** - Win conditions and scoring rules
- **`computed.yaml`** - Dynamic calculations and formulas
- **`hooks.wasm`** - Custom WebAssembly logic for complex rules
- **`variants/`** - Alternative game modes and setups

## Creating Your First Game

Let's walk through creating a simple game step by step.

### 1. Manifest (manifest.yaml)

The manifest defines your game's basic information:

```yaml
# Required fields
gameId: "my-awesome-game"
version: "1.0"
specVersion: 1
metadata:
  name: "My Awesome Game"
  description: "A fantastic turn-based strategy game"
  author: "Your Name"
  players:
    min: 2
    max: 4

# Optional metadata for better discoverability
tags: ["strategy", "abstract", "family"]
estimatedTime: 30  # Minutes for typical game
complexity: "medium"  # easy | medium | hard | expert
age: "10+"
language: ["none"]  # or ["english"] if text required

# Advanced turn structure (optional)
turnStructure:
  type: "sequential"  # sequential | simultaneous | realtime
  order: "clockwise"  # clockwise | counterclockwise | custom | bid
  phases:
    - id: "main"
      name: "Main Phase"
      description: "Play cards and take actions"
      actions: ["playCard", "drawCard", "pass"]

# Initial setup commands (optional)
setup:
  - shuffle: "deck"
  - deal:
      from: "deck"
      to: "hand_{player}"
      count: 7
```

**Required Fields:**
- `gameId` - Unique identifier for your game
- `version` - Your game's version (string format)
- `specVersion` - Bluefelt spec version (currently 1)
- `metadata.name` - Display name
- `metadata.description` - Brief description
- `metadata.author` - Your name
- `metadata.players.min/max` - Player count range (1-8)

## Game Initialization

### Understanding Initial State

When a game starts, Bluefelt automatically creates the initial game state based on your configuration files. Here's what gets initialized:

#### Automatic Initialization
The engine automatically sets up:
```json
{
  "zones": {},          // Created from zones.yaml
  "players": [          // Created based on player count
    {"id": "p1"},
    {"id": "p2"}
  ],
  "turn": 0,            // Always starts at 0
  "tick": 0,            // Internal counter
  "currentPlayer": "p1", // Always starts with p1
  "gameStatus": {       // Default playing state
    "state": "playing",
    "winner": null,
    "tie": false
  },
  "phases": {}          // Created from phases.yaml if present
}
```

#### Zone Initialization
Zones are created based on their type:
- **Grid zones**: Initialize with empty cells based on rows/cols
- **List zones**: Start as empty arrays
- **Deck zones**: Populated with entities if specified
- **Resource zones**: Start with initial values

#### Entity Creation
Entities with `{player}` templates are created for each player:
```yaml
# This in entities.yaml:
- id: "score_{player}"
  props:
    value: 0

# Creates:
# - score_p1 with value: 0
# - score_p2 with value: 0
```

### Custom Setup Actions

For complex initialization (shuffling decks, dealing cards, etc.), use setup actions in your manifest.yaml:

```yaml
# In manifest.yaml
setup:
  - shuffle: "deck"
  - deal:
      from: "deck"
      to: "hand_{player}"
      count: 7
```

**Note**: The `setup` field in manifest.yaml is currently limited. Most games handle complex setup through initial phase actions.

### 2. Entities (entities.yaml)

Entities are the game components - pieces, cards, tokens, etc.:

```yaml
# Simple token
- id: mark_{player}
  props:
    value: "{player}"
  ui:
    tokenType: "{player}"

# Score counter
- id: score_{player}
  type: counter
  props:
    value: 0
    min: 0
    max: 100
  ui:
    display: number

# Standard deck of cards
- type: standardDeck
  id: deck
```

**Core Properties:**
- `id` - Unique identifier (supports `{player}` template)
- `type` - Optional: token, counter, deck, dice, tile, boolean
- `props` - Game-specific properties
- `ui` - Visual hints (tokenType, display, icon, color)

### 3. Zones (zones.yaml)

Zones define where entities can exist:

```yaml
# Grid zone for game board
- id: "board"
  type: "grid"
  gridProps:
    rows: 3
    cols: 3
  contents: "empty"
  
# List zone for player hand
- id: "hand_{player}"
  type: "list"
  visibility: "owner"  # Only owner can see
  listProps:
    maxSize: 7
    ordered: true
  contents: "empty"

# Deck zone with auto-shuffle
- id: "deck"
  type: "deck"
  deckProps:
    shuffle: true
    faceDown: true
    reshuffleDiscardWhenEmpty: true
  contents: "standardDeck"

# Track zone for scoring
- id: "score_track"
  type: "track"
  trackProps:
    length: 100
    start: 0
```

**Zone Types:**
- `grid` - 2D grid (chess board, tic-tac-toe)
- `list` - Ordered/unordered collection (hand, supply)
- `deck` - Shuffleable stack of cards
- `track` - Linear progression (scoring, turn order)
- `bag` - Random draw container
- `slot` - Single item container
- `resource` - Numeric counter
- `graph` - Network of connected nodes
- `network` - Player-built connections

**Visibility Options:**
- `"all"` - Everyone sees everything
- `"owner"` - Only owner sees contents
- `"none"` - Nobody sees contents
- `"active"` - Only active player

**Special Features:**
- `{player}` replacement creates per-player zones
- Built-in `contents` like `"standardDeck"` for card games
- Zone-specific properties (gridProps, deckProps, etc.)

### 4. Actions (actions.yaml)

Actions define what players can do:

```yaml
# Simple placement action
- id: "place_piece"
  uses: "place"
  ui:
    direction: "Choose where to place your piece"
    logTemplate: "{player} placed at ({row}, {col})"
  conditions:
    - type: "zone_empty"
      zone: "{target}"
  
# Complex action with costs and effects
- id: "build_road"
  uses: "placeToken"
  with:
    from: "supply_{player}"
    entity: "road_{player}"
  costs:
    resources:
      wood: 1
      stone: 1
  constraints:
    - builtin: "hasResources"
      params: { wood: 1, stone: 1 }
    - builtin: "connected"
      params: { to: "network_{player}" }
  sideEffects:
    - grantResources:
        to: "{player}"
        resources: { points: 1 }
  ui:
    direction: "Build a road (costs 1 wood, 1 stone)"
    icon: "road"
    highlight: "validBuildLocations"

# Action with triggers
- id: "place_and_check"
  uses: "place"
  triggers:
    - action: "checkForWin"
      hook: "checkForWin"
      auto: true
```

**Action Properties:**
- `uses` - Built-in verb or custom behavior
- `with` - Parameters for the action
- `ui.direction` - Instructions shown to players
- `ui.logTemplate` - Game log message template (supports `{player}`, `{row}`, `{col}`)
- `conditions` - When the action is available
- `constraints` - More complex availability rules
- `costs` - Resources or items consumed
- `sideEffects` - Additional effects that happen
- `triggers` - Actions that fire after this one

### 5. Phases (phases.yaml) - Optional

Phases organize game flow into distinct states:

```yaml
- id: game
  phases:
    - id: setup
      initial: true
      enterActions:
        - transitionToPhase: game.play
    
    - id: play
      possibleActions: [placeMarker, attack]
      ui:
        display: "Playing"
    
    - id: end
      ui:
        display: "Game Over"
```

**Key Concepts:**
- Phase sets operate as independent state machines
- `enterActions` - Actions that execute automatically when entering
- `possibleActions` - Limits which actions are available
- Phase transitions are triggered by actions, not defined in phases

## Client-Server Interaction

### Action Map System

Starting with Bluefelt v0.2, the server provides an action map that directly maps locations to available actions. This makes client implementation much simpler.

#### How It Works

The server sends available actions as a map of locations to action details:

```json
{
  "actionMap": {
    "p1": {
      "/zones/board/0/0": {"action": "place", "direction": "Choose a cell"},
      "/zones/board/0/1": {"action": "place", "direction": "Choose a cell"},
      "/zones/hand_p1/2": {"action": "playCard", "direction": "Play this card"}
    }
  }
}
```

#### Location Path Format

- **Grid zones**: `/zones/{zoneId}/{row}/{col}` (e.g., `/zones/board/0/1`)
- **List/deck zones**: `/zones/{zoneId}/{index}` (e.g., `/zones/hand_p1/2`)
- **Whole zones**: `/zones/{zoneId}` (e.g., `/zones/deck` for draw actions)

#### Benefits

1. **Direct Mapping**: Click handlers just check `actionMap[location]`
2. **No Complex Logic**: No need to filter or search through actions
3. **Consistent Format**: Same paths work everywhere
4. **Platform Agnostic**: Works for web, mobile, VR, etc.

## Built-in Verbs

Bluefelt provides several built-in verbs for common game actions. Understanding these verbs is crucial for implementing games efficiently.

### Entity Placement and Movement

#### `place`
Places an entity at a specific grid location.
```yaml
uses: "place"
# The location and entity are determined by the action context
# Typically used with grid zones for placing pieces
```

#### `placeWithGravity`
Places an entity that automatically falls to the lowest available position (Connect 4 style).
```yaml
uses: "placeWithGravity"
# Entity falls down in the specified column
# Returns error if column is full
```

#### `moveEntity`
Moves entities between zones or locations within a zone.
```yaml
uses: "moveEntity"
with:
  from: "/zones/source/0"      # Source location
  to: "/zones/destination/1"   # Target location
  entity: "piece_p1"          # Optional: specific entity to move
```

#### `draw`
Draws entities (typically cards) from one zone to another.
```yaml
uses: "draw"
with:
  from: "/zones/deck"         # Source zone (usually a deck)
  to: "/zones/hand_p1"        # Target zone (usually a hand)
  count: 5                    # Number to draw (default: 1)
```

### Win Condition Checking

#### `grid.lineOfMarks`
Checks for winning line patterns on grid zones. Highly configurable for different games.
```yaml
uses: "grid.lineOfMarks"
with:
  zone: "/zones/board"         # Grid zone to check
  entity: "mark_{player}"      # Entity pattern to match (supports {player} template)
  lineLength: 3                # Length of line to win (3 for tic-tac-toe, 4 for Connect-4, 5 for Gomoku)
  directions: ["horizontal", "vertical", "diagonal"]  # Which directions to check
```

**Behavior:**
- Automatically sets `gameStatus.state` to "ended" when a win is detected
- Sets `gameStatus.winner` to the winning player
- Detects ties when the board is full with no winner
- Works with any grid size and line length

### Turn and Phase Management

#### `nextTurn`
Advances the game to the next player's turn.
```yaml
uses: "nextTurn"
# No parameters needed
# Automatically updates:
# - turn (increments by 1)
# - currentPlayer (cycles through players)
# - tick (internal counter)
```

#### `setPhase`
Changes the current phase within a phase set.
```yaml
uses: "setPhase"
with:
  phaseSet: "game"            # Which phase set to update
  phase: "combat"             # New phase to enter
```

### Important Verb Details

#### Arguments and Context
- Most verbs get their primary arguments from the action context (like target location)
- The `with` field provides additional parameters
- Entity selection often comes from the UI interaction

#### Error Handling
Verbs return errors for invalid operations:
- Placing on occupied spaces
- Moving non-existent entities  
- Drawing from empty decks
- Invalid locations

#### Patch Generation
All verbs generate JSON patches that:
- Update the authoritative game state
- Are broadcast to all connected clients
- Can be applied incrementally

### Extending with New Verbs

If your game needs mechanics not covered by built-in verbs:

1. **First check if existing verbs can work** with different parameters
2. **Consider if the mechanic is reusable** across multiple games
3. **Propose a new generic verb** that could benefit other developers
4. **Only use WebAssembly hooks** as a last resort for truly unique mechanics

See the Implementation Planning section for the process of proposing new verbs.

## Shorthand Syntax

Bluefelt supports several shorthand features to reduce boilerplate:

### Player Replacement

Use `{player}` in entity IDs and zone names:

```yaml
# Creates red_piece_p1, red_piece_p2, etc.
- id: "red_piece_{player}"
  type: "piece"

# Creates hand_p1, hand_p2, etc.  
- id: "hand_{player}"
  type: "list"
```

### Standard Deck

For card games, use the built-in standard deck:

```yaml
# In entities.yaml
- type: "standardDeck"  # Generates 52 playing cards

# In zones.yaml
- id: "draw_pile"
  type: "deck"
  contents: "standardDeck"  # Pre-filled with all cards
```

### Card Game Actions

Built-in card game shortcuts:

```yaml
- id: "deal_cards"
  uses: "cards.deal"
  with:
    count: 7
    to: "eachPlayer"
    from: "draw_pile"

- id: "reveal_card"
  uses: "cards.reveal"
  with:
    from: "draw_pile"
    to: "discard_pile"
```

## Events (events.yaml)

Events trigger automatically based on game state changes:

```yaml
# Score when pieces reach end of track
- id: "reach_end_bonus"
  trigger:
    on: "pieceMove"
    zone: "scoring_track"
  condition: "position == track.length - 1"
  effects:
    - grantResources:
        to: "{triggeringPlayer}"
        resources: { points: 10 }
    - announce:
        message: "{player} reached the end and scored 10 points!"

# Refill market at turn end
- id: "refill_market"
  trigger:
    on: "turnEnd"
  condition: "count(market) < 5"
  effects:
    - draw:
        from: "supply"
        to: "market"
        count: "5 - count(market)"
```

**Common Triggers:**
- `phaseStart/End` - Phase transitions
- `turnStart/End` - Turn boundaries  
- `cardPlayed` - When cards enter play
- `pieceMove` - Entity movement
- `resourceGain/Spent` - Economy changes
- `zoneEmpty/Full` - Container state

## Victory Conditions (victory.yaml)

Define how players win and scoring works:

```yaml
victory:
  type: "highest_score"  # or "first_to", "last_standing", "objective"
  description: "Player with the most points after 10 rounds wins"
  
  # End game triggers
  triggers:
    endOfRound:
      - "round == 10"
    immediate:
      - "any_player.score >= 50"
  
  # Scoring breakdown
  scoring:
    components:
      - id: "territories"
        name: "Controlled Territories"
        formula: "count(zones.*.controlledBy == player)"
        multiplier: 3
      - id: "resources"
        name: "Remaining Resources"
        formula: "sum(resources.*.value)"
        visible: false  # Hidden until game end
    
    finalScore:
      formula: "sum(components.*.value)"
  
  # Tiebreakers
  tiebreakers:
    - highest: "resources.gold"
    - most: "territories"
    - lowest: "turnOrder"  # Earlier in turn order wins
```

## Advanced Features

### Custom Conditions

Define when actions are available:

```yaml
conditions:
  - type: "zone_empty"
    zone: "{target}"
  - type: "current_player_owns"
    entity: "{source}"
  - type: "phase_equals" 
    phase: "playing"
  - type: "hasResources"
    resources: { gold: 5 }
  - type: "connected"
    from: "{source}"
    to: "{target}"
    via: "roads"
```

### Action Triggers

Chain actions together:

```yaml
- id: "place_and_check"
  uses: "place"
  triggers:
    - action: "check_win_condition"
      condition: "always"
    - action: "grant_bonus"
      condition: "placedInCorner"
    - action: "advance_turn"
      condition: "!gameEnded"
```

### Game Variants

Support different game modes:

```yaml
# In manifest.yaml
variants:
  - id: "beginner"
    name: "Beginner Mode"
    description: "Simplified rules for new players"
    file: "variants/beginner.yaml"
    default: true
  - id: "expert"
    name: "Expert Mode"
    description: "Additional complexity for experienced players"
    file: "variants/expert.yaml"

# In variants/expert.yaml
modify:
  zones:
    - id: "board"
      gridProps:
        rows: 5  # Larger board
        cols: 5
  actions:
    - id: "teleport"  # New action for expert mode
      uses: "moveEntity"
      costs:
        resources: { energy: 3 }
```

### Hidden Information

For games with secrets or deduction:

```yaml
# Secret role cards
- id: "role_{player}"
  type: "role_card"
  visibility: "owner"  # Only owning player can see
  
# Hidden victory conditions
- id: "secret_objective_{player}"
  type: "objective"
  visibility:
    contents: "owner"
    exists: "all"  # Everyone knows you have one
```

### Custom Hooks (Advanced)

For complex game logic, write WebAssembly hooks:

```rust
// Rust hook example
#[no_mangle]
pub extern "C" fn calculate_market_price(state: *const u8) -> *const u8 {
    // Complex economic calculations
}
```

Then reference in actions:
```yaml
- id: "buy_from_market"
  hook: "calculate_market_price"
  auto: false  # Player-triggered

## Game Examples

### Tic-Tac-Toe

A complete tic-tac-toe implementation:

```yaml
# manifest.yaml
gameId: "tic-tac-toe"
version: "1.0"
specVersion: 1
metadata:
  name: "Tic-Tac-Toe"
  description: "Classic 3-in-a-row game"
  author: "Traditional"
  players: 
    min: 2
    max: 2

# entities.yaml  
- id: "mark_{player}"
  props: 
    value: "{player}"
  ui:
    tokenType: "{player}"

# zones.yaml
- id: "board"
  type: "grid"
  gridProps:
    rows: 3
    cols: 3
  contents: "empty"

# actions.yaml
- id: "place"
  uses: "place"
  ui:
    direction: "Choose a cell"
    logTemplate: "{player} placed their mark at ({row}, {col})"
  triggers:
    - action: "checkForWin"
      hook: "checkForWin"
      auto: true
```

### Card Game

A basic card game structure:

```yaml
# entities.yaml
- type: "standardDeck"

# zones.yaml
- id: "draw_pile"
  type: "deck"
  contents: "standardDeck"
  deckProps:
    shuffle: true

- id: "hand_{player}"
  type: "list"
  contents: "empty"

# actions.yaml
- id: "deal_initial"
  uses: "cards.deal"
  with:
    count: 7
    to: "eachPlayer"
    from: "draw_pile"
```

## Testing Your Game

### Validation

Use the CLI to validate your game files:

```bash
cd cli
cargo run -- validate ../games/your-game/1.0/
```

### Building

Build your game into deployable bundles:

```bash
# Build single game
cargo run -- build ../games/your-game/1.0/

# Build all games
cargo run -- build-all
```

### Local Testing

Start the server and test in browser:

```bash
# Start server (loads from bundles/)
cd server && cargo run

# Start client
cd clients/react && pnpm dev

# Open http://localhost:5173
```

## Best Practices

### Game Design

1. **Start Simple** - Begin with basic mechanics
2. **Use Built-ins** - Leverage existing verbs and shortcuts
3. **Clear Actions** - Write descriptive `ui.direction` text
4. **Test Early** - Validate frequently during development

### File Organization

1. **Consistent Naming** - Use clear, descriptive IDs
2. **Logical Grouping** - Group related entities/actions
3. **Version Control** - Use semantic versioning
4. **Documentation** - Comment complex rules

### Performance

1. **Efficient Zones** - Avoid overly complex zone structures
2. **Minimal Entities** - Only define what you need
3. **Smart Conditions** - Use efficient condition checks
4. **Batch Actions** - Group related actions when possible

## Common Patterns

### Grid Games
- Use `grid` zones for boards
- `place` actions for piece placement
- Grid coordinates for movement rules

### Card Games  
- `standardDeck` for playing cards
- `deck` zones with shuffling
- `cards.deal` for distribution
- `list` zones for hands

### Turn-Based Strategy
- Phase management for complex turns
- Resource tracking with entity properties
- Area control with zone ownership

### Real-Time Elements
- Action triggers for immediate responses
- State validation with conditions
- Automatic actions with `auto: true`

## Troubleshooting

### Common Issues

**Validation Errors:**
- Check required manifest fields
- Verify entity/zone ID references
- Ensure action `uses` values are valid

**Runtime Errors:**
- Validate zone paths in actions
- Check entity existence before references
- Verify player count constraints

**UI Issues:**
- Provide clear `ui.direction` text
- Test action availability conditions
- Verify entity visual properties

### Debug Tools

1. **Server Logs** - Check console output during gameplay
2. **State Inspector** - Use browser dev tools to examine game state
3. **CLI Validation** - Run validation before deployment
4. **Test Games** - Create minimal test cases

## Complete Schema Reference

For detailed information about all available fields and options:

### Modernized Field Names

Bluefelt v0.2 uses cleaner field names while maintaining backwards compatibility:

#### Zone Definitions
- Use `type` instead of `shape` for zone types
- Use `cols`/`rows` instead of `width`/`height` for grid properties

#### Action Definitions  
- Use `uses` instead of `builtin` or `implementation` for the engine function
- Use `with` instead of `params` for action parameters

### All Zone Properties

Each zone type supports specific properties:

- **Grid**: `rows`, `cols`, `hexagonal`, `wrapping`, `diagonals`, `coordinates`
- **List**: `ordered`, `maxSize`, `unique`, `handLimit`, `sortBy`
- **Deck**: `shuffle`, `faceDown`, `reshuffleDiscardWhenEmpty`, `drawTo`, `discardTo`
- **Track**: `length`, `circular`, `bidirectional`, `start`, `rewards`, `labels`
- **Bag**: `opaque`, `returnable`, `autoRefill`, `refillFrom`

### Common Action Constraints

Built-in constraints you can use:

- `hasResources` - Player has enough resources
- `hasCards` - Cards available in hand
- `isActivePlayer` - It's the player's turn
- `inPhase` - Correct game phase
- `hasSpace` - Room in target zone
- `connected` - Network path exists
- `range` - Within movement distance
- `uniquePerTurn` - Can only do once per turn

### Effect Types

Common effects for actions and events:

- `grantResources` - Give resources to players
- `moveToken` - Advance on tracks
- `drawCards` - Draw from decks
- `revealInformation` - Show hidden data
- `modifyEntity` - Change entity properties
- `triggerEvent` - Fire another event
- `createEntity` - Spawn new items
- `announce` - Show message to all

## Learning from Existing Games

Before implementing your game, study existing implementations:

### Games to Reference:
- **Tic-tac-toe** - Simplest grid placement and win detection
- **Connect-4** - Gravity mechanics and vertical win patterns
- **Gin Rummy** - Complex phases and card game patterns
- **Stone Age** - Worker placement and resource management
- **Checkers** - Piece movement and capture mechanics

### What to Look For:
1. **Action patterns** - How similar mechanics are implemented
2. **Phase usage** - When phases add value vs complexity
3. **Zone structures** - Effective ways to organize game areas
4. **Entity design** - Reusable entity patterns
5. **UI conventions** - Consistent user experience patterns

## Next Steps

Once you've created your first game:

1. **Deploy** - Build bundles and test on live server
2. **Iterate** - Gather feedback and refine gameplay
3. **Advanced Features** - Add phases, custom hooks
4. **Documentation** - Write player guides and rules
5. **Community** - Share your game with other developers

The Bluefelt platform provides a powerful foundation for turn-based game development. Start with simple mechanics and gradually add complexity as you become familiar with the system.

> **Need More Details?** See the [Game Implementation Guide](./game-implementation-guide.md) for step-by-step implementation instructions, or consult the individual topic guides for deep dives into specific features.

## Important Discovery: Conditional Logic in Actions

The action system supports conditional logic that enables sophisticated game mechanics:

```yaml
# Actions can check game state and make decisions
- id: automaticAction
  auto: true
  when:
    - condition: "zone.count"  # Check pieces on board
    - condition: "phase.is"    # Check current phase
  then:
    - action: "setPhase"       # Change game phase
    - action: "otherAction"    # Trigger other effects
```

This pattern allows you to implement:
- **Phase transitions based on game state** (e.g., "switch to movement phase after all pieces placed")
- **Dynamic rule changes** based on board conditions
- **Complex win conditions** beyond simple patterns
- **Milestone-based progression** without hardcoding

The key is thinking of actions as your game's logic engine, not just player interactions!
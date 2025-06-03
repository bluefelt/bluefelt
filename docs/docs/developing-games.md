# Developing Games: Overview

Bluefelt is a platform for creating turn-based multiplayer games using declarative YAML files. This guide covers everything you need to know about designing, implementing, and deploying games on the Bluefelt platform.

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
        ├── manifest.yaml    # Game metadata
        ├── entities.yaml    # Game pieces and components
        ├── zones.yaml      # Game areas and containers
        ├── actions.yaml    # Player actions
        ├── phases.yaml     # Game flow (optional)
        └── hooks.wasm      # Custom logic (optional)
```

### Required Files

- **`manifest.yaml`** - Game identification and metadata
- **`entities.yaml`** - All game components (pieces, cards, tokens)
- **`zones.yaml`** - All game areas (board, hands, decks)
- **`actions.yaml`** - All possible player actions

### Optional Files

- **`phases.yaml`** - Multi-phase game flow
- **`hooks.wasm`** - Custom WebAssembly logic for complex rules

## Creating Your First Game

Let's walk through creating a simple game step by step.

### 1. Manifest (manifest.yaml)

The manifest defines your game's basic information:

```yaml
gameId: "my-awesome-game"
version: 1.0
specVersion: 1
metadata:
  name: "My Awesome Game"
  description: "A fantastic turn-based strategy game"
  author: "Your Name"
  players:
    min: 2
    max: 4
```

**Required Fields:**
- `gameId` - Unique identifier for your game
- `version` - Your game's version
- `specVersion` - Bluefelt spec version
- `metadata.name` - Display name
- `metadata.description` - Brief description
- `metadata.author` - Your name
- `metadata.players.min/max` - Player count range

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
- id: "board"
  type: "grid"
  rows: 3
  cols: 3
  contents: "empty"

- id: "supply_{player}"
  type: "list"
  contents:
    entity: "red_piece"
    count: 10
```

**Zone Types:**
- `grid` - 2D grid (chess board, tic-tac-toe)
- `list` - Ordered collection (hand, deck)
- `deck` - Shuffleable collection with special properties

**Special Features:**
- `{player}` replacement creates per-player zones
- Built-in `contents` like `"standardDeck"` for card games
- Automatic shuffling for deck zones

### 4. Actions (actions.yaml)

Actions define what players can do:

```yaml
- id: "place_piece"
  uses: "place"
  ui:
    direction: "Choose where to place your piece"
  conditions:
    - type: "zone_empty"
      zone: "{target}"
  
- id: "move_piece"
  uses: "moveEntity"
  ui:
    direction: "Move a piece to an adjacent cell"
  conditions:
    - type: "zone_contains_player_entity"
      zone: "{source}"
```

**Action Properties:**
- `uses` - Built-in verb or custom behavior
- `ui.direction` - Instructions shown to players
- `conditions` - When the action is available

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

## Built-in Verbs

Bluefelt provides several built-in verbs for common game actions:

### Core Verbs

- **`place`** - Place an entity at a grid location
  ```yaml
  uses: "place"
  # Allows placing entities on grid zones
  ```

- **`moveEntity`** - Move entities between zones/locations
  ```yaml
  uses: "moveEntity" 
  # Move from one zone to another
  ```

- **`draw`** - Draw from deck to hand/zone
  ```yaml
  uses: "draw"
  # Draw cards from deck
  ```

### Turn Management

- **`nextTurn`** - Advance to next player
- **`setPhase`** - Change game phase

### Preset Actions

- **`presets.grid.move`** - Grid-based movement
- **`presets.turn.advance`** - Standard turn advancement
- **`presets.zone.reset`** - Reset zone contents

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
```

### Action Triggers

Chain actions together:

```yaml
- id: "place_and_check"
  uses: "place"
  triggers:
    - "check_win_condition"
    - "advance_turn"
```

### Custom Hooks (Advanced)

For complex game logic, write WebAssembly hooks:

```rust
// Rust hook example
#[no_mangle]
pub extern "C" fn check_win_condition(state: *const u8) -> *const u8 {
    // Custom win condition logic
}
```

## Game Examples

### Tic-Tac-Toe

A complete tic-tac-toe implementation:

```yaml
# manifest.yaml
gameId: "tic-tac-toe"
metadata:
  players: { min: 2, max: 2 }

# entities.yaml  
- id: "x_token"
  props: { player: "p1" }
- id: "o_token"
  props: { player: "p2" }

# zones.yaml
- id: "board"
  type: "grid"
  rows: 3
  cols: 3
  contents: "empty"

# actions.yaml
- id: "place_mark"
  uses: "place"
  ui:
    direction: "Choose a cell"
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

## Next Steps

Once you've created your first game:

1. **Deploy** - Build bundles and test on live server
2. **Iterate** - Gather feedback and refine gameplay
3. **Advanced Features** - Add phases, custom hooks
4. **Documentation** - Write player guides and rules
5. **Community** - Share your game with other developers

The Bluefelt platform provides a powerful foundation for turn-based game development. Start with simple mechanics and gradually add complexity as you become familiar with the system.
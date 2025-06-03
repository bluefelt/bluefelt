# Developing Games: Entities

Entities represent all game components in Bluefelt - pieces, cards, tokens, counters, and more. They are defined in `entities.yaml` and provide the building blocks for your game.

## Overview

Entities can represent:
- **Tokens** - Game pieces, markers, workers
- **Cards** - Playing cards, action cards
- **Counters** - Score trackers, resource counts
- **Dice** - Random number generators
- **Tiles** - Board tiles, buildings
- **Booleans** - True/false state flags

Each entity has a unique ID and optional properties that define its behavior and appearance.

## Basic Entity Structure

### Minimal Entity

```yaml
- id: mark_{player}
  props:
    value: "{player}"
```

### Typical Entity

```yaml
- id: score_{player}
  name: "{player}'s Score"
  type: counter
  props:
    value: 0
    min: 0
    max: 100
  ui:
    display: number
```

### Card Entity

```yaml
- id: card_hearts_ace
  type: card
  props:
    suit: hearts
    rank: A
    value: 14
  ui:
    glyph: "🂱"
```

## Core Properties

### Required Fields

#### `id` (string, required)
Unique identifier for the entity. Supports `{player}` template substitution.

```yaml
- id: piece_{player}      # Creates piece_p1, piece_p2, etc.
- id: deck                # Shared entity
- id: score_{player}      # Player-specific scores
```

### Optional Fields

#### `name` (string, optional)
Human-readable name, supports template substitution.

```yaml
name: "{player}'s Score"
name: "Red Worker"
```

#### `type` (string, optional)
Entity type that affects behavior:

```yaml
type: token      # Basic game piece
type: counter    # Numeric value with min/max
type: deck       # Card collection
type: dice       # Random number generator
type: tile       # Board tile or building
type: boolean    # True/false flag
```

#### `quantity` (number, optional)
Number of instances to create (default: 1).

```yaml
quantity: 6      # Create 6 dice
quantity: 10     # Create 10 workers
```

#### `props` (object, optional)
Game-specific properties. Common patterns:

```yaml
# Basic properties
props:
  value: "{player}"       # Template substitution
  player: "{player}"      # Owner
  type: regular           # Subtype
  
# Counter properties
props:
  value: 0                # Current value
  min: 0                  # Minimum allowed
  max: 999                # Maximum allowed
  
# Card properties  
props:
  suit: hearts            # Card suit
  rank: A                 # Card rank
  value: 14               # Numeric value
  
# Dice properties
props:
  sides: 6                # Number of sides
  
# Tile properties
props:
  cost: 3                 # Resource cost
  points: 5               # Victory points
  production: wood        # What it produces
```

#### `ui` (object, optional)
Visual representation hints:

```yaml
ui:
  # Token rendering
  tokenType: "{player}"    # Shape/style (circle, x, p1, p2)
  
  # Display types
  display: number          # For counters
  display: dice            # For dice
  display: tile            # For tiles
  display: boolean         # For true/false
  display: meeple          # For workers
  
  # Icons and colors
  icon: apple              # Icon name
  color: "{player.color}"  # Color (supports templates)
  
  # Card glyphs
  glyph: "♠"              # Unicode character
```

#### `options` (object, optional)
Configuration for special entity types:

```yaml
# Deck options
options:
  cardType: cardType.playingCard
  jokers: 0
  
# Standard deck shorthand
- type: standardDeck
  id: deck
  options:
    cardType: playing_card
```

## Player Templates

The `{player}` template creates player-specific entities:

```yaml
# Basic player entity
- id: piece_{player}
  props:
    player: "{player}"
  ui:
    tokenType: "{player}"

# Player score counter
- id: score_{player}
  name: "{player}'s Score"
  type: counter
  props:
    value: 0
    min: 0
    max: 100

# Player-specific tokens
- id: worker_{player}
  props:
    player: "{player}"
  ui:
    tokenType: circle
    color: "{player.color}"
```

**Template Rules:**
- `{player}` expands to `p1`, `p2`, etc.
- Works in `id`, `name`, `props`, and `ui` fields
- Number of players determined by manifest

## Standard Deck

For card games, use the built-in deck shorthand:

```yaml
# Simple deck
- type: standardDeck
  id: deck

# Deck with options
- id: deck
  type: deck
  options:
    cardType: cardType.playingCard
    jokers: 0
```

This generates 52 cards with:
- Suits: hearts, diamonds, clubs, spades
- Ranks: A, 2-10, J, Q, K
- Values: Appropriate numeric values
- Glyphs: Unicode suit symbols

## Entity Types in Detail

### Token
Basic game pieces:

```yaml
- id: mark_{player}
  type: token
  props:
    value: "{player}"
  ui:
    tokenType: "{player}"  # "x" for p1, "o" for p2, etc.

- id: worker_{player}
  type: token
  props:
    player: "{player}"
  ui:
    tokenType: circle
    color: "{player.color}"
```

### Counter
Numeric values with ranges:

```yaml
- id: score_{player}
  type: counter
  props:
    value: 0
    min: 0
    max: 999
  ui:
    display: number

- id: resources_{player}
  type: counter
  props:
    value: 10      # Starting value
    min: 0         # Can't go negative
    max: 50        # Resource cap
```

### Dice
Random number generators:

```yaml
- id: dice
  type: dice
  quantity: 6
  props:
    sides: 6
  ui:
    display: dice
```

### Tile
Board tiles and buildings:

```yaml
- id: hut_3_resources
  type: tile
  quantity: 5
  props:
    cost: 3
    points: 3
  ui:
    display: tile
    icon: hut
```

### Boolean
True/false flags:

```yaml
- id: playerHasKnocked
  type: boolean
  props:
    value: false
  ui:
    display: boolean
```

## Common Entity Patterns

### Game Pieces

```yaml
# Simple piece
- id: piece_{player}
  props:
    player: "{player}"
  ui:
    tokenType: "{player}"

# Checkers piece
- id: disc_{player}
  props:
    player: "{player}"
    type: regular    # regular or king
  ui:
    tokenType: circle
    color: "{player.color}"

# Chess-like piece
- id: king_{player}
  props:
    player: "{player}"
    type: king
    value: 0
  ui:
    glyph: "♔"      # Unicode chess king
```

### Cards

```yaml
# Individual card
- id: card_hearts_ace
  type: card
  props:
    suit: hearts
    rank: A
    value: 14
  ui:
    glyph: "🂱"

# Use standardDeck for full deck
- type: standardDeck
  id: deck
```

### Resources and Scoring

```yaml
# Score counter
- id: score_{player}
  name: "{player}'s Score"
  type: counter
  props:
    value: 0
    min: 0
    max: 999
  ui:
    display: number

# Resource tokens (Stone Age style)
- id: food_{player}
  type: counter
  props:
    value: 12
    min: 0
  ui:
    display: icon
    icon: apple

- id: wood_{player}
  type: counter
  props:
    value: 0
    min: 0
  ui:
    display: icon
    icon: tree
```

### Workers and Meeples

```yaml
# Basic worker
- id: person_{player}
  quantity: 10
  props:
    player: "{player}"
  ui:
    tokenType: circle
    color: "{player.color}"
    display: meeple

# Specialized workers
- id: farmer_{player}
  props:
    player: "{player}"
    type: farmer
  ui:
    tokenType: circle
    color: green
    icon: wheat
```

## UI Properties Reference

### Token Types
```yaml
ui:
  tokenType: circle      # Round token
  tokenType: x           # X marker
  tokenType: o           # O marker  
  tokenType: "{player}"  # Player-specific (p1, p2, etc.)
```

### Display Modes
```yaml
ui:
  display: number        # Show as number (counters)
  display: dice          # Show as dice
  display: tile          # Show as tile
  display: boolean       # Show as checkbox/flag
  display: meeple        # Show as meeple shape
  display: icon          # Show with icon
```

### Icons and Colors
```yaml
ui:
  # Named icons (Stone Age examples)
  icon: apple            # Food
  icon: tree             # Wood
  icon: brick            # Brick
  icon: rock             # Stone
  icon: coin             # Gold
  icon: hut              # Building
  
  # Colors
  color: "{player.color}" # Player's color
  color: red             # Named color
  color: "#ff0000"       # Hex color
```

### Card Glyphs
```yaml
ui:
  # Playing card suits
  glyph: "♠"            # Spades
  glyph: "♥"            # Hearts  
  glyph: "♦"            # Diamonds
  glyph: "♣"            # Clubs
  
  # Chess pieces
  glyph: "♔"            # King
  glyph: "♕"            # Queen
  glyph: "♖"            # Rook
  glyph: "♗"            # Bishop
  glyph: "♘"            # Knight
  glyph: "♙"            # Pawn
```

## Real Game Examples

### Tic-Tac-Toe
```yaml
- id: mark_{player}
  props:
    value: "{player}"
  ui:
    tokenType: "{player}"  # "x" or "o"
```

### Gin Rummy
```yaml
- type: standardDeck
  id: deck
  options:
    cardType: playing_card

- id: score_{player}
  name: "{player}'s score"
  type: counter
  props:
    value: 0
    min: 0
  ui:
    display: number

- id: playerHasKnocked
  type: boolean
  props:
    value: false
```

### Stone Age
```yaml
# Resources
- id: food_{player}
  type: counter
  props:
    value: 12
    min: 0
  ui:
    display: icon
    icon: apple

# Workers
- id: person_{player}
  quantity: 10
  props:
    player: "{player}"
  ui:
    tokenType: circle
    color: "{player.color}"
    display: meeple

# Dice
- id: dice
  type: dice
  quantity: 6
  props:
    sides: 6
  ui:
    display: dice

# Building tiles
- id: hut_3_resources
  type: tile
  quantity: 5
  props:
    cost: 3
    points: 3
  ui:
    display: tile
```

### Checkers
```yaml
- id: disc_{player}
  quantity: 12
  props:
    player: "{player}"
    type: regular
  ui:
    tokenType: circle
    color: "{player.color}"
```


## Best Practices

### 1. Use Player Templates
Instead of creating separate entities for each player, use `{player}`:

```yaml
# Good - Single definition
- id: score_{player}
  type: counter
  props:
    value: 0

# Avoid - Repetitive
- id: score_p1
  type: counter
  props:
    value: 0
- id: score_p2
  type: counter
  props:
    value: 0
```

### 2. Choose Appropriate Types
- Use `counter` for numeric values that change
- Use `token` for game pieces
- Use `boolean` for true/false states
- Use `dice` for random elements
- Use `standardDeck` for playing cards

### 3. Keep Props Simple
Only include properties your game actually uses:

```yaml
# Good - Only what's needed
props:
  player: "{player}"
  value: 0

# Avoid - Unused properties
props:
  player: "{player}"
  value: 0
  unused1: null
  unused2: "default"
```

### 4. Use Standard UI Patterns
- `tokenType: "{player}"` for player-specific shapes
- `color: "{player.color}"` for player colors
- `display: number` for scores
- `display: icon` with named icons for resources

## Summary

Entities in Bluefelt are flexible and simple. The key points:

1. **Required**: Only `id` is required
2. **Templates**: Use `{player}` for player-specific entities
3. **Types**: token, counter, deck, dice, tile, boolean
4. **Props**: Flexible object for game-specific data
5. **UI**: Simple hints for rendering (tokenType, display, icon, color, glyph)

For more examples, examine the entity definitions in the included games.
# Developing Games: UI

Bluefelt follows a "Layer 1 First" philosophy - games are automatically playable with auto-generated UI based on game state, with optional visual enhancements added progressively.

## Overview

The Bluefelt UI system provides:
- **Auto-generated UI** - Every game is immediately playable without custom code
- **Entity representation** - Token types and glyphs for game pieces
- **Zone layouts** - Automatic arrangement of game areas
- **Interactive actions** - Click-to-play interface generated from game rules
- **Progressive enhancement** - Add visual polish without changing gameplay

## Entity UI Properties

Entities can specify how they appear in the game:

```yaml
- id: "x_piece"
  type: "piece"
  ui:
    tokenType: "x"      # Use a named token visual
    
- id: "white_king"
  type: "piece"
  ui:
    glyph: "♔"         # Use a Unicode character
    
- id: "ace_spades"
  type: "card"
  ui:
    cardType: "AS"     # Standard playing card notation
```

The client prioritizes these properties in order:
1. `ui.tokenType` - References pre-defined token visuals (e.g., "x", "o", "circle", "p1", "p2")
2. `ui.glyph` - Any text or Unicode character to display
3. Falls back to displaying the entity's value if neither is specified

## Zone UI Properties

Zones can specify layout and display properties:

```yaml
- id: "board"
  shape: "grid"
  rows: 3
  cols: 3
  ui:
    rotateForPlayer: true    # Rotate 180° for player 2
    
- id: "player_hand"
  shape: "list"
  ui:
    layout: "fan"           # Display cards in a fan
    showCount: true         # Show number of cards
    player: 1               # Associate with specific player
    
- id: "deck"
  shape: "list"
  ui:
    layout: "stack"         # Stack cards on top of each other
    showTop: true          # Show the top card face
```

### Available Zone Layouts

For list-shaped zones:
- `fan` - Cards spread in an arc (typical for hands)
- `spread` - Cards overlapping horizontally
- `stack` - Cards stacked vertically
- `row` - Items in a horizontal row
- Default behavior based on zone name (e.g., zones named "hand" default to fan layout)

## Action UI Properties

Actions provide instructions to players:

```yaml
- id: "place_piece"
  uses: "place"
  ui:
    direction: "Choose an empty space"    # Shown when action is available
    
- id: "draw_card"
  uses: "draw"
  ui:
    direction: "Click to draw a card"
    logTemplate: "{player} draws a card"  # For game log (future feature)
```

The `ui.direction` property is the primary UI element for actions - it tells players what to do when an action is available.

## How the Auto-Generated UI Works

### The Action Map System

The server provides an action map that directly connects UI locations to available actions:

```json
{
  "/zones/board/0/0": {
    "action": "place_piece",
    "direction": "Choose an empty space"
  },
  "/zones/board/0/1": {
    "action": "place_piece", 
    "direction": "Choose an empty space"
  },
  "/zones/deck": {
    "action": "draw_card",
    "direction": "Click to draw"
  }
}
```

The client uses this map to:
1. Make zones interactive where actions are available
2. Show action directions on hover
3. Execute actions when clicked

### Automatic Zone Rendering

Zones are rendered based on their shape and properties:

**Grid zones** → HTML table with clickable cells
**List zones** → Card displays with appropriate layouts
**Single zones** → Container for single items

The client automatically handles:
- Highlighting interactive areas
- Showing/hiding entities based on visibility rules
- Updating displays when state changes

## Zone Groups

You can organize zones into logical groups in your manifest:

```yaml
# In manifest.yaml
zoneGroups:
  - title: "Game Board"
    zones: ["board"]
    
  - title: "Player Areas"
    zones: ["player1_hand", "player2_hand"]
    
  - title: "Shared Areas" 
    zones: ["deck", "discard"]
```

Zone groups help organize complex games with many zones into clear sections.

## Examples

### Tic-Tac-Toe UI

A minimal UI definition:

```yaml
# entities.yaml
- id: "x_piece"
  type: "piece"
  ui:
    tokenType: "x"
    
- id: "o_piece" 
  type: "piece"
  ui:
    tokenType: "o"

# zones.yaml
- id: "board"
  shape: "grid"
  rows: 3
  cols: 3
  ui:
    rotateForPlayer: true

# actions.yaml
- id: "place_piece"
  uses: "place"
  ui:
    direction: "Choose an empty space"
```

### Card Game UI

A more complex example with card zones:

```yaml
# zones.yaml
- id: "player1_hand"
  shape: "list"
  ui:
    layout: "fan"
    player: 1
    showCount: true
    
- id: "deck"
  shape: "list"
  ui:
    layout: "stack"
    showTop: false
    
- id: "discard"
  shape: "list"
  ui:
    layout: "stack"
    showTop: true

# manifest.yaml
zoneGroups:
  - title: "Community Cards"
    zones: ["deck", "discard"]
  - title: "Player Hands"
    zones: ["player1_hand", "player2_hand"]
```

## Future Enhancements

In alignment with Bluefelt's "Layer 1 First" philosophy, future UI enhancements will be added progressively:

- **Asset support** - Add images for cards and tokens while maintaining text fallbacks
- **Animation hints** - Smooth transitions derived from state changes
- **Theme customization** - Player-selected color schemes
- **Accessibility modes** - High contrast, larger text, screen reader improvements

The core principle remains: every game must be fully playable with the auto-generated UI, with visual enhancements being purely optional layers on top.
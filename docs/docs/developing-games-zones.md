# Developing Games: Zones

Zones are the logical containers that define where entities can exist and how they're organized within your game. This comprehensive reference covers zone design following Bluefelt's separation of logic and presentation.

## Architecture Overview

Bluefelt uses a **presentation-separated architecture** where:
- **Zone definitions** contain only logical properties (server-parsed)
- **Visual presentation** is handled by optional client files (client-parsed)
- **All clients** can fallback to generic rendering without presentation data

### File Structure

```
game-name/
├── zones.yaml           # Logic only (server reads)
├── ui/
│   └── skins.yaml       # Presentation hints (client reads)
└── assets/
    └── board.png        # Art assets (client loads)
```

## Zone Logic vs. Presentation

Zones in Bluefelt represent:
- **Game Boards** - Chess boards, tic-tac-toe grids, hex maps
- **Player Areas** - Hands, tableau, personal boards
- **Shared Spaces** - Draw piles, discard piles, market areas
- **Storage Areas** - Supply pools, banks, reserves
- **Virtual Spaces** - Score tracks, turn order, game phases

Each zone has logical properties (shape, size, behavior) that determine gameplay rules, while visual properties are handled separately.

## Zone Structure

### Minimal Zone (Logic Only)

```yaml
- id: "board"
  shape: "grid"
  shapeMeta:
    rows: 3
    cols: 3
```

### Complete Zone with Logical Properties

```yaml
- id: "board"
  shape: "grid"
  shapeMeta:
    rows: 8
    cols: 8
  contents: 
    entity: "chess_piece"
    pattern: "chess_starting"
  visibility: "public"
  owner: null
  behavior:
    allowedEntities: ["chess_piece", "marker"]
    maxEntitiesPerCell: 1
    entityFilter: "owned_by_current_player"
```

### Optional Presentation (ui/skins.yaml)

```yaml
skins:
  board:
    # Visual styling hints for clients that support them
    style:
      cellSize: 64
      gridLines: true
      coordinates: true
      theme: "checkered"
    # Optional: atlas-based rendering
    atlas: "chess/board.png"
    rect: { x: 0, y: 0, w: 512, h: 512 }
    mapping:
      "grid(0,0)": { u: 32, v: 32 }
      "grid(7,7)": { u: 480, v: 480 }
```

## Core Properties

### Required Fields

#### `id` (string, required)
Unique identifier for the zone.

```yaml
- id: "main_board"        # ✓ Valid
- id: "hand_p1"           # ✓ Valid
- id: "supply-wood"       # ✓ Valid (hyphens allowed)
- id: "market_row_1"      # ✓ Valid (numbers allowed)
```

**Naming Conventions:**
- Use descriptive, lowercase names
- Separate words with underscores
- Include location/purpose in name
- Use `{player}` for player-specific zones

#### `shape` (string, required)
Defines the zone's spatial organization and behavior.

```yaml
# Grid-based zones (2D arrays)
shape: "grid"         # Chess board, tic-tac-toe, tile grids

# Linear zones (1D arrays)  
shape: "list"         # Player hands, sequences, rows
shape: "stack"        # Shuffleable stacks, draw piles

# Special zones
shape: "pool"         # Unordered collections, supply areas
shape: "track"        # Scoring tracks, turn order
shape: "choice"       # Choice/selection zones
```

### Type-Specific Required Fields

#### Grid Zones

```yaml
- id: "game_board"
  shape: "grid"
  shapeMeta:
    rows: 8         # Number of rows (required)
    cols: 8         # Number of columns (required)
```

#### List Zones

```yaml
- id: "player_hand"
  shape: "list"
  maxSize: 7        # Maximum entities (optional)
  # No rows/cols needed
```

#### Stack Zones (formerly "deck")

```yaml
- id: "draw_pile"
  shape: "stack"
  shapeMeta:
    shuffle: true   # Auto-shuffle when needed
  visibility: "count"  # Show count but not contents
```

### Optional Core Fields

#### `contents` (object/string, optional)
Defines initial zone contents.

```yaml
# Empty zone
contents: "empty"

# Single entity type with count
contents:
  entity: "chess_pawn"
  count: 8

# Multiple entity types
contents:
  - entity: "red_piece"
    count: 12
  - entity: "blue_piece" 
    count: 12

# Pattern-based setup
contents:
  pattern: "chess_starting_position"
  
# Standard deck
contents: "standardDeck"

# Custom arrangement
contents:
  layout: "custom"
  positions:
    - row: 0
      col: 0
      entity: "white_rook"
    - row: 0
      col: 1
      entity: "white_knight"
```

## Zone Types Deep Dive

### Grid Zones

Two-dimensional arrays perfect for boards and maps.

```yaml
- id: "chess_board"
  type: "grid"
  rows: 8
  cols: 8
  contents:
    pattern: "chess_starting"
  ui:
    cellSize: 64
    theme: "checkered"
    alternatingColors: ["#f0d9b5", "#b58863"]
    coordinates: true
    coordinateStyle: "chess"  # a1-h8 notation
  behavior:
    maxEntitiesPerCell: 1
    allowOverlap: false
```

**Grid-Specific Properties:**

```yaml
# Size and dimensions
rows: 8                    # Required: number of rows
cols: 8                    # Required: number of columns

# Cell behavior
cellType: "square"         # square, hex, triangle
cellConnections: "orthogonal"  # orthogonal, diagonal, all, hex

# Grid patterns
pattern: "chess"           # Predefined layouts
alternatingPattern: true   # Checkerboard coloring
```

**Common Grid Patterns:**

```yaml
# Chess board
- id: "chess_board"
  type: "grid"
  rows: 8
  cols: 8
  contents: "chess_starting"
  ui:
    theme: "chess"
    coordinates: "chess"    # a1-h8 notation

# Tic-tac-toe
- id: "ttt_board"
  type: "grid" 
  rows: 3
  cols: 3
  contents: "empty"
  ui:
    cellSize: 80
    gridLines: true

# Hex map
- id: "hex_map"
  type: "grid"
  rows: 10
  cols: 10
  cellType: "hex"
  cellConnections: "hex"
  ui:
    theme: "hex_terrain"
```

### List Zones

Ordered linear collections ideal for hands and sequences.

```yaml
- id: "player_hand"
  type: "list"
  maxSize: 7
  contents: "empty"
  ui:
    orientation: "horizontal"
    spacing: 5
    overlap: 0.3
    fanAngle: 15
    sortable: true
  behavior:
    allowedEntities: ["card"]
    sortOrder: "manual"
    autoArrange: false
    insertionPoint: "end"
```

**List-Specific Properties:**

```yaml
# Size constraints
maxSize: 10               # Maximum number of entities
minSize: 0                # Minimum number of entities
dynamicSize: true         # Can grow/shrink as needed

# Ordering
sortOrder: "manual"       # manual, value, type, name
sortDirection: "asc"      # asc, desc
maintainOrder: true       # Preserve insertion order

# Insertion behavior
insertionPoint: "end"     # start, end, position
allowInsertion: true      # Can insert at any position
```

**Common List Patterns:**

```yaml
# Player hand
- id: "hand_{player}"
  type: "list"
  maxSize: 7
  ui:
    orientation: "horizontal"
    fanOut: true
    overlap: 0.3
  behavior:
    sortable: true
    private: true          # Only visible to owner

# Turn order track
- id: "turn_order"
  type: "list"
  maxSize: 4
  ui:
    orientation: "vertical"
    spacing: 10
  behavior:
    sortOrder: "manual"
    public: true

# Market row
- id: "market_cards"
  type: "list"
  maxSize: 5
  contents:
    entity: "market_card"
    count: 5
  ui:
    orientation: "horizontal"
    spacing: 5
```

### Deck Zones

Specialized lists with shuffling and hidden information.

```yaml
- id: "draw_pile"
  type: "deck"
  contents: "standardDeck"
  deckProps:
    shuffle: true
    shuffleOnSetup: true
    shuffleWhenEmpty: false
    faceDown: true
    revealTop: false
  ui:
    cardBack: "classic_blue"
    showCount: true
    stackOffset: 2
  behavior:
    drawFrom: "top"        # top, bottom, random
    discardTo: "bottom"    # top, bottom
```

**Deck-Specific Properties:**

```yaml
deckProps:
  # Shuffling behavior
  shuffle: true              # Can be shuffled
  shuffleOnSetup: true       # Shuffle during game setup
  shuffleWhenEmpty: false    # Reshuffle when empty
  autoShuffle: false         # Shuffle automatically
  
  # Visibility
  faceDown: true             # Cards hidden
  revealTop: false           # Top card visible
  revealCount: 0             # Number of visible cards
  
  # Draw behavior
  drawFrom: "top"            # Where to draw from
  infiniteDraw: false        # Never runs out
  recycleFrom: "discard"     # Source for reshuffling
```

### Pool Zones

Unordered collections for supply areas and banks.

```yaml
- id: "supply_pool"
  type: "pool"
  contents:
    - entity: "gold_coin"
      count: 50
    - entity: "silver_coin"
      count: 100
  ui:
    layout: "scattered"     # scattered, grid, pile
    maxVisibleItems: 20
  behavior:
    infinite: true          # Never runs out
    groupByType: true       # Group identical entities
```

**Pool-Specific Properties:**

```yaml
# Layout behavior
layout: "scattered"        # scattered, grid, pile, sorted
groupByType: true          # Group identical entities
maxVisibleItems: 10        # UI performance optimization

# Supply behavior
infinite: false            # Limited supply
replenishable: true        # Can be refilled
replenishSource: "bank"    # Where to get new entities
```

### Track Zones

Linear tracks for scoring and progression.

```yaml
- id: "score_track"
  type: "track"
  length: 100
  contents: "empty"
  ui:
    orientation: "horizontal"
    showNumbers: true
    startValue: 0
    endValue: 100
    tickMarks: 10
    milestones: [25, 50, 75]
  behavior:
    allowMultiple: true     # Multiple markers per space
    wrapAround: false       # Return to start at end
```

### Single Zones

Containers for exactly one entity.

```yaml
- id: "current_player_marker"
  type: "single"
  contents:
    entity: "first_player_token"
  ui:
    highlight: true
    frame: "golden"
  behavior:
    required: true          # Must always contain entity
    swappable: true         # Can exchange entities
```

## UI and Visual Properties

### Layout and Positioning

```yaml
ui:
  # Zone positioning
  position:
    x: 100
    y: 200
    width: 400
    height: 300
  
  # Alignment within container
  alignment: "center"       # center, top-left, bottom-right
  
  # Padding around zone
  padding:
    top: 10
    right: 15
    bottom: 10
    left: 15
  
  # Margin outside zone
  margin:
    top: 5
    right: 10
    bottom: 5
    left: 10
```

### Visual Styling

```yaml
ui:
  # Colors
  backgroundColor: "#f5f5f5"
  borderColor: "#cccccc"
  highlightColor: "#ffff00"
  
  # Border properties
  borderWidth: 2
  borderStyle: "solid"      # solid, dashed, dotted
  borderRadius: 8
  
  # Background patterns
  backgroundImage: "wood_texture.png"
  backgroundPattern: "checkered"
  
  # Visual effects
  shadow: true
  shadowColor: "#000000"
  shadowOffset: { x: 2, y: 2 }
  shadowBlur: 4
```

### Grid-Specific UI

```yaml
ui:
  # Cell appearance
  cellSize: 64              # Base cell size in pixels
  cellSpacing: 2            # Space between cells
  cellShape: "square"       # square, circle, hex
  
  # Grid lines
  gridLines: true
  gridLineColor: "#999999"
  gridLineWidth: 1
  
  # Coordinates
  coordinates: true
  coordinateStyle: "numeric"  # numeric, chess, custom
  coordinatePosition: "outside"  # inside, outside, both
  
  # Alternating colors
  alternatingColors: ["#ffffff", "#f0f0f0"]
  alternatingPattern: "checkerboard"  # checkerboard, stripes
```

### List-Specific UI

```yaml
ui:
  # Orientation
  orientation: "horizontal"  # horizontal, vertical, circular
  
  # Spacing and overlap
  spacing: 5                # Space between entities
  overlap: 0.3              # Overlap ratio (0-1)
  
  # Fan layout (for cards)
  fanOut: true
  fanAngle: 15              # Degrees of arc
  fanRadius: 200            # Radius of fan curve
  
  # Stacking
  stackDirection: "up"      # up, down, left, right
  stackOffset: 3            # Pixels between stacked items
```

### Interactive Elements

```yaml
ui:
  # Hover effects
  hover:
    backgroundColor: "#e0e0e0"
    borderColor: "#0066cc"
    scale: 1.05
  
  # Selection highlighting
  selection:
    borderColor: "#ff0000"
    borderWidth: 3
    glow: true
  
  # Drop targets
  dropTarget:
    backgroundColor: "#90ee90"
    borderStyle: "dashed"
    animation: "pulse"
  
  # Drag and drop
  dragPreview: true
  dropZones: ["hand", "board"]
  dragConstraints: "horizontal"  # horizontal, vertical, none
```

## Behavioral Properties

### Entity Management

```yaml
behavior:
  # Allowed entity types
  allowedEntities: ["card", "token"]
  restrictedEntities: ["piece"]
  
  # Capacity limits
  maxEntities: 10           # Total entity limit
  maxEntitiesPerCell: 1     # For grid zones
  maxEntitiesPerType: 5     # Limit per entity type
  
  # Entity filters
  entityFilter: "owned_by_player"  # Custom filtering logic
  visibilityFilter: "all"          # all, owner, team, none
```

### Interaction Rules

```yaml
behavior:
  # Player permissions
  accessibleBy: "owner"     # owner, all, team, specific_players
  editableBy: "owner"       # Who can modify contents
  viewableBy: "all"         # Who can see contents
  
  # Interaction types
  allowDrop: true           # Can drop entities here
  allowDrag: true           # Can drag entities from here
  allowSelect: true         # Can select entities
  allowReorder: true        # Can rearrange entities
  
  # Special interactions
  doubleClickAction: "flip" # Action on double-click
  rightClickMenu: true      # Context menu available
```

### Automatic Behaviors

```yaml
behavior:
  # Auto-organization
  autoSort: false           # Automatically sort contents
  autoArrange: true         # Rearrange for optimal display
  autoCleanup: false        # Remove empty spaces
  
  # State changes
  autoShuffle: false        # Shuffle when modified
  autoReveal: false         # Reveal entities when added
  autoFlip: false           # Flip entities when moved here
  
  # Triggers
  onEntityAdded: "update_score"     # Action when entity added
  onEntityRemoved: "check_empty"    # Action when entity removed
  onEmpty: "trigger_reshuffle"      # Action when zone empties
  onFull: "prevent_more_additions"  # Action when zone fills
```

## Advanced Features

### Player-Specific Zones

Use `{player}` in zone IDs to create per-player areas:

```yaml
# Creates hand_p1, hand_p2, etc.
- id: "hand_{player}"
  type: "list"
  maxSize: 7
  ui:
    orientation: "horizontal"
    private: true           # Only visible to owner
  behavior:
    accessibleBy: "owner"
    viewableBy: "owner"

# Creates tableau_p1, tableau_p2, etc.
- id: "tableau_{player}"
  type: "grid"
  rows: 2
  cols: 5
  contents: "empty"
  ui:
    backgroundColor: "{player_color}"
```

### Dynamic Zones

Zones that change during gameplay:

```yaml
- id: "expanding_market"
  type: "list"
  maxSize: 3              # Initial size
  contents:
    entity: "market_card"
    count: 3
  ui:
    dynamicSize: true     # Can grow during game
  behavior:
    expansionTrigger: "turn_start"
    maxExpansion: 8       # Maximum final size
    expansionRate: 1      # Entities added per expansion
```

### Linked Zones

Zones that interact with each other:

```yaml
- id: "draw_pile"
  type: "deck"
  contents: "standardDeck"
  deckProps:
    shuffle: true
    recycleFrom: "discard_pile"  # Linked discard pile
  
- id: "discard_pile"
  type: "deck"
  contents: "empty"
  deckProps:
    faceUp: true
    feedsTo: "draw_pile"    # Feeds back to draw pile
```

### Conditional Zones

Zones with dynamic availability:

```yaml
- id: "bonus_area"
  type: "list"
  maxSize: 3
  ui:
    visibility: "conditional"
  behavior:
    availableWhen:
      - condition: "player_score > 20"
      - condition: "phase == 'advanced'"
    requirements: ["unlock_bonus"]
```

## Zone Patterns and Examples

### Chess Board

```yaml
- id: "chess_board"
  type: "grid"
  rows: 8
  cols: 8
  contents:
    pattern: "chess_starting_position"
  ui:
    theme: "chess"
    cellSize: 60
    coordinates: "chess"
    alternatingColors: ["#f0d9b5", "#b58863"]
    borderColor: "#8b4513"
    borderWidth: 3
  behavior:
    maxEntitiesPerCell: 1
    allowedEntities: ["chess_piece"]
    movementRules: "chess"
  rules:
    placement: "legal_chess_moves"
    capture: "replace_enemy"
```

### Card Game Setup

```yaml
# Draw pile
- id: "draw_pile"
  type: "deck"
  contents: "standardDeck"
  deckProps:
    shuffle: true
    faceDown: true
    shuffleOnSetup: true
  ui:
    cardBack: "blue_back"
    showCount: true

# Player hands
- id: "hand_{player}"
  type: "list"
  maxSize: 7
  contents: "empty"
  ui:
    orientation: "horizontal"
    fanOut: true
    fanAngle: 10
    private: true
  behavior:
    accessibleBy: "owner"
    sortable: true

# Discard pile
- id: "discard_pile"
  type: "deck"
  contents: "empty"
  deckProps:
    faceUp: true
    revealTop: true
  ui:
    cardBack: "none"
    showTopCard: true

# Market area
- id: "market"
  type: "list"
  maxSize: 5
  contents:
    entity: "market_card"
    count: 5
  ui:
    orientation: "horizontal"
    spacing: 10
  behavior:
    publicVisible: true
    refreshOnEmpty: true
```

### Strategy Game Board

```yaml
# Main game board
- id: "main_board"
  type: "grid"
  rows: 12
  cols: 12
  contents:
    pattern: "terrain_random"
  ui:
    cellSize: 48
    theme: "medieval"
    terrainTextures: true
    elevationShading: true
  behavior:
    allowedEntities: ["unit", "building", "resource"]
    terrainEffects: true
    lineOfSight: true

# Resource pools
- id: "supply_{resource}"
  type: "pool"
  contents:
    entity: "{resource}_token"
    count: 30
  ui:
    layout: "pile"
    groupByType: true
  behavior:
    infinite: false
    publicVisible: true

# Player areas
- id: "player_area_{player}"
  type: "grid"
  rows: 3
  cols: 5
  ui:
    backgroundColor: "{player_color}"
    borderColor: "darker_{player_color}"
    title: "Player {player} Area"
  behavior:
    accessibleBy: "owner"
    allowedEntities: ["building", "unit"]
```

### Scoring Track

```yaml
- id: "score_track"
  type: "track"
  length: 100
  contents: "empty"
  ui:
    orientation: "spiral"
    startPosition: "bottom_left"
    direction: "clockwise"
    showNumbers: true
    tickInterval: 5
    majorTicks: [25, 50, 75, 100]
    backgroundColor: "#f5f5dc"
    trackColor: "#8b4513"
  behavior:
    allowMultiple: true
    wrapAround: false
    milestoneEffects: true
```

## Special Zone Mechanics

Zones can support special behaviors through custom verbs and properties. These mechanics add depth to gameplay without requiring game-specific code.

### Gravity Mechanics

For games where entities fall to the lowest available position (like Connect 4):

```yaml
# Zone definition remains standard
- id: "board"
  type: "grid"
  rows: 6
  cols: 7
  contents: "empty"
  ui:
    cellSize: 64
    gridLines: true
```

The gravity behavior is implemented through the `placeWithGravity` verb in actions:

```yaml
# In actions.yaml
- id: "drop_disc"
  uses: "placeWithGravity"
  ui:
    direction: "Choose a column"
    logTemplate: "{player} dropped a disc in column {column}"
  conditions:
    - type: "is_current_player_turn"
```

**How it works:**
- Players interact with columns instead of specific cells
- The server calculates the lowest empty row in the selected column
- Entities "fall" to the bottom-most available position
- Full columns are automatically invalid targets

**Client interaction pattern:**
- Action map generates column-based paths: `/zones/board/columns/3`
- Visual affordances show column drop zones above the board
- The generic client detects and renders these automatically

### Line Detection

The `grid.lineOfMarks` verb provides configurable line detection for win conditions:

```yaml
# Detect 3-in-a-row (tic-tac-toe)
- verb: "grid.lineOfMarks"
  args:
    zone: "/zones/board"
    entity: "mark_{player}"
    lineLength: 3
    directions: ["horizontal", "vertical", "diagonal"]

# Detect 4-in-a-row (Connect 4)
- verb: "grid.lineOfMarks"
  args:
    zone: "/zones/board"
    entity: "disc_{player}"
    lineLength: 4
    directions: ["horizontal", "vertical", "diagonal"]

# Detect 5-in-a-row horizontal/vertical only (Gomoku variant)
- verb: "grid.lineOfMarks"
  args:
    zone: "/zones/board"
    entity: "stone_{player}"
    lineLength: 5
    directions: ["horizontal", "vertical"]
```

**Parameters:**
- `zone`: Path to the grid zone to check
- `entity`: Entity pattern to match (use `{player}` for current player)
- `lineLength`: Number of consecutive entities needed (default: 3)
- `directions`: Array of directions to check

**Automatic actions:**
- Sets game status to ended when line is found
- Updates winner field with the player who formed the line
- Detects tie conditions when board is full with no winner

### Stacking Mechanics

For games where entities can stack in zones:

```yaml
- id: "territory"
  type: "grid"
  rows: 8
  cols: 8
  behavior:
    maxEntitiesPerCell: 5  # Allow stacking up to 5
    stackingOrder: "top"   # New entities go on top
  ui:
    renderMode: "stacked"
    stackOffset: 3         # Pixels between stacked items
```

### Wrap-Around Boards

For games with toroidal or cylindrical topologies:

```yaml
- id: "cylindrical_board"
  type: "grid"
  rows: 10
  cols: 10
  behavior:
    wrapHorizontal: true   # Left edge connects to right
    wrapVertical: false    # Top/bottom don't connect
```

### Dynamic Zone Properties

Zones that change during gameplay:

```yaml
- id: "expanding_board"
  type: "grid"
  rows: 3              # Initial size
  cols: 3
  behavior:
    expandable: true
    maxRows: 10
    maxCols: 10
    expansionTrigger: "phase_change"
```

### Terrain and Movement Costs

For strategy games with varied terrain:

```yaml
- id: "strategy_map"
  type: "grid"
  rows: 20
  cols: 20
  contents:
    pattern: "terrain_map"
  behavior:
    terrainTypes: ["plains", "forest", "mountain", "water"]
    movementCosts:
      plains: 1
      forest: 2
      mountain: 3
      water: null    # Impassable
```

### Visibility and Fog of War

For games with hidden information:

```yaml
- id: "exploration_map"
  type: "grid"
  rows: 30
  cols: 30
  behavior:
    fogOfWar: true
    visibilityRange: 3
    revealedCells: "persistent"  # Once revealed, stay visible
  ui:
    fogColor: "#333333"
    fogOpacity: 0.8
```

### Zone Relationships

Zones that interact with each other:

```yaml
# Deck that refills from discard
- id: "draw_deck"
  type: "deck"
  deckProps:
    shuffleWhenEmpty: true
    refillFrom: "discard_pile"

# Discard that feeds deck
- id: "discard_pile"
  type: "deck"
  deckProps:
    feedsTo: "draw_deck"
    clearOnRefill: true
```

### Custom Zone Behaviors Through Actions

While zones define structure, actions can implement complex behaviors:

```yaml
# Zone remains simple
- id: "auction_house"
  type: "list"
  maxSize: 5

# Action implements the behavior
- id: "bid_on_item"
  uses: "customAuction"
  triggers:
    - "sort_by_bid_value"
    - "award_to_highest_bidder"
```

## Best Practices

### Zone Design Guidelines

1. **Purpose-Driven Design**
   ```yaml
   # ✓ Clear purpose
   - id: "player_hand"      # Obviously for cards in hand
   - id: "battle_field"     # Combat area
   - id: "resource_bank"    # Shared resource storage
   
   # ✗ Vague purpose
   - id: "area_1"           # What is this for?
   - id: "stuff_zone"       # Too generic
   ```

2. **Appropriate Type Selection**
   ```yaml
   # ✓ Correct types
   - id: "chess_board"
     type: "grid"           # 2D positioning needed
   
   - id: "card_hand"
     type: "list"           # Order matters
   
   - id: "resource_supply"
     type: "pool"           # Unordered collection
   
   # ✗ Wrong types
   - id: "chess_board"
     type: "list"           # 2D board as 1D list
   ```

3. **Consistent Sizing**
   ```yaml
   # ✓ Logical sizes
   - id: "standard_hand"
     maxSize: 7             # Common card game hand size
   
   - id: "chess_board"
     rows: 8
     cols: 8                # Standard chess board
   
   # ✗ Arbitrary sizes
   - id: "weird_board"
     rows: 13
     cols: 7                # Unusual dimensions without reason
   ```

### Performance Optimization

1. **Limit Visual Complexity**
   ```yaml
   # ✓ Optimized
   ui:
     maxVisibleItems: 20    # Limit for performance
     virtualScrolling: true # For large lists
     lazyLoading: true      # Load as needed
   
   # ✗ Performance issues
   ui:
     showAllItems: true     # Could be thousands
     realTimeUpdates: true  # Every frame updates
   ```

2. **Use Appropriate Data Structures**
   ```yaml
   # ✓ Efficient for purpose
   - id: "large_supply"
     type: "pool"           # Unordered, efficient lookup
   
   - id: "turn_order"
     type: "list"           # Ordered, need sequence
   
   # ✗ Inefficient choice
   - id: "resource_bank"
     type: "grid"           # Unnecessary 2D for 1D data
   ```

### Accessibility and Usability

1. **Clear Visual Hierarchy**
   ```yaml
   ui:
     # High contrast for important zones
     borderColor: "#000000"
     borderWidth: 3
     
     # Subtle styling for background zones
     backgroundColor: "#f8f8f8"
     borderColor: "#cccccc"
   ```

2. **Responsive Design**
   ```yaml
   ui:
     # Flexible sizing
     cellSize: "responsive"
     minCellSize: 32
     maxCellSize: 128
     
     # Adaptive layout
     orientation: "auto"    # Adjust based on space
     compactMode: true      # Smaller screens
   ```

3. **Keyboard Navigation**
   ```yaml
   behavior:
     keyboardNavigable: true
     tabOrder: 1
     arrowKeyNavigation: true
     enterKeyAction: "select"
     spaceKeyAction: "activate"
   ```

### Common Pitfalls to Avoid

1. **Overcomplicated Zone Hierarchies**
   ```yaml
   # ✗ Too complex
   - id: "super_complex_zone"
     type: "grid"
     subZones:
       - type: "list"
         subZones:
           - type: "single"   # Too many levels
   
   # ✓ Simple and clear
   - id: "game_board"
     type: "grid"
   - id: "player_hand"
     type: "list"
   ```

2. **Inconsistent Player Zone Patterns**
   ```yaml
   # ✗ Inconsistent
   - id: "hand_player1"     # Different naming
   - id: "p2_hand"          # Different format
   
   # ✓ Consistent
   - id: "hand_{player}"    # Uniform pattern
   - id: "tableau_{player}" # Same template
   ```

3. **Missing Capacity Planning**
   ```yaml
   # ✗ No limits
   - id: "unlimited_zone"
     type: "list"
     # Could grow infinitely, cause performance issues
   
   # ✓ Planned capacity
   - id: "managed_zone"
     type: "list"
     maxSize: 20            # Reasonable limit
     warningAt: 15          # User feedback
   ```

This comprehensive guide covers all aspects of zone development in Bluefelt. Use it to create well-structured, efficient game spaces that enhance gameplay while maintaining performance and usability.
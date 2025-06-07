# Presentation Layer Architecture

Bluefelt uses a **presentation-separated architecture** that enables the same game logic to render across multiple client types (2D, VR, AR, mobile) without code changes.

## Core Principles

### 1. Every Zone is First a Logical Container

All zones must define these mandatory logical properties:

| Field | Meaning | Fallback Rendering |
|-------|---------|-------------------|
| `id` | Stable handle used by verbs/diffs | — |
| `shape` | `stack`, `list`, `grid`, `track`, `choice`, etc. | **stack** → simple pile UI<br>**list** → scrollable row<br>**grid** → simple table<br>**track** → numbered cells<br>**choice** → selection interface |
| `shapeMeta` | Size, wrap, rows/cols, constraints | Used by generic renderer & rules validation |

With just these three keys, the server can run, rules can be validated, and **any** client can fall back to vanilla visuals.

### 2. Optional Image Skins

Zones can optionally declare presentation hints in `ui/skins.yaml`:

```yaml
skins:
  board:
    # Visual styling hints
    style:
      cellSize: 100
      gridLines: true
      theme: "medieval"
    
    # Optional: atlas-based rendering
    atlas: "game-name/board.png"     # master board texture
    rect: { x: 1280, y: 260, w: 560, h: 300 }   # UV crop window
    mapping:
      # logical-cell → pixel offset INSIDE the rect
      "grid(0,0)": { u: 24,  v: 18 }
      "grid(0,1)": { u: 102, v: 18 }
      "stackTop":  { u: 460, v: 140 }
```

### 3. Schema Separation Contract

| Field | Stored in game state? | Travels in diffs? | Used by server logic? | Used by client? |
|-------|----------------------|-------------------|---------------------|----------------|
| `id`, `shape`, `shapeMeta` | **Yes** | **Yes** | **Yes** | Yes |
| `ui/skins.*` | No (static bundle) | No | No | **Yes (if supported)** |

**Therefore:**
- A phone client that has no art assets **ignores skins** and still renders the game
- A VR client that reads skins can explode zones into ergonomic 3D layouts
- All clients see the exact same game logic and state

## File Organization

```
game-name/1.0/
├── zones.yaml           # LOGICAL ONLY - server parses
├── actions.yaml         # LOGICAL ONLY - server parses  
├── entities.yaml        # LOGICAL ONLY - server parses
├── ui/
│   ├── skins.yaml       # PRESENTATION - client parses
│   └── layouts.yaml     # OPTIONAL presets - client parses
└── assets/
    ├── board.png        # Art assets - client loads
    └── cards/
        ├── back.png
        └── fronts/
```

## zones.yaml (Logic Only)

Contains pure logical properties that the server needs:

```yaml
# Tic-tac-toe example
- id: board
  shape: grid
  shapeMeta:
    rows: 3
    cols: 3
  contents: empty

# Card game example
- id: hand_{player}
  shape: list
  visibility: owner
  maxSize: 7

- id: deck
  shape: stack
  visibility: count
  contents: standardDeck
  shapeMeta:
    shuffle: true
```

**Key changes from old format:**
- `type` → `shape` (more generic term)
- `gridProps` → `shapeMeta` (unified metadata field)
- All `ui:` blocks removed (moved to skins.yaml)
- `deckProps` → `shapeMeta` (consistent structure)

## ui/skins.yaml (Presentation Hints)

Optional file containing visual styling hints:

```yaml
skins:
  board:
    style:
      cellSize: 100
      gap: 4
      showGrid: true
      gridStyle: "intersections"
      theme: "wood"
    
    # Future: atlas support
    # atlas: "three-mens-morris/board.png"
    # rect: { x: 0, y: 0, w: 300, h: 300 }
    
  hand_{player}:
    layout:
      orientation: "horizontal"
      fanOut: true
      overlap: 0.3
      
  deck:
    layout:
      position: "bottom-right"
      showCount: true
      cardBack: "blue_pattern"
```

## Atlas-Based Rendering (Future)

The system supports atlas-based rendering where a single "master board texture" is sliced into functional regions:

### Atlas Definition
```yaml
skins:
  market:
    atlas: "rummy/board.png"
    rect: { x: 200, y: 100, w: 400, h: 200 }
    mapping:
      "list(0)": { u: 50, v: 50 }   # First market slot
      "list(1)": { u: 150, v: 50 }  # Second market slot
      "list(2)": { u: 250, v: 50 }  # etc...
```

### Client Implementation
- **Flat clients**: Render the entire atlas as a flat board image
- **VR clients**: Extract each rect onto separate floating quads positioned in 3D
- **Mobile clients**: Scale and crop regions for optimal viewing

This ensures that every client literally shows the same pixels, even if rearranged spatially.

## Migration Guide

### Updating Existing Games

1. **Remove UI blocks from zones.yaml**:
   ```yaml
   # OLD
   - id: board
     type: grid
     rows: 3
     cols: 3
     ui:
       cellSize: 100
       gridLines: true
   
   # NEW
   - id: board
     shape: grid
     shapeMeta:
       rows: 3
       cols: 3
   ```

2. **Create ui/skins.yaml** with extracted UI properties:
   ```yaml
   skins:
     board:
       style:
         cellSize: 100
         gridLines: true
   ```

3. **Update field names**:
   - `type` → `shape`
   - `gridProps` → `shapeMeta`
   - `deckProps` → `shapeMeta`

### Server Changes Required

- Server must ignore `ui/` directory entirely
- Bundle building should include ui files for client distribution
- Existing zone parsing logic needs field name updates

### Client Changes Required

- Load skins.yaml at game startup (optional)
- Fall back to generic rendering when skins unavailable
- Use skin hints for enhanced visual presentation

## Benefits

1. **Universal Compatibility**: Same game works on any client type
2. **Progressive Enhancement**: Basic functionality without art assets
3. **Artist Freedom**: Visual design independent of game logic
4. **Client Flexibility**: Each client can choose how to interpret hints
5. **Performance**: Clients can optimize rendering without server changes
6. **Future-Proof**: New client types work automatically

## Round-Trip Example

1. **Server diff**: `"/zones/discard": ["Q♠"]`
2. **Generic client**: Draws Q♠ in a simple list cell
3. **Enhanced client**: 
   - Finds `discard` skin in skins.yaml
   - Locates atlas rect for discard pile
   - Draws Q♠ at UV offset within that rect
   - Optionally positions in 3D space for VR

Same logic, different presentation, zero extra server work.
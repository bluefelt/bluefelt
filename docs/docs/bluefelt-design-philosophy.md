# Bluefelt Design Philosophy

Bluefelt uses a **presentation-separated architecture** that enables the same game logic to render across multiple client types (2D, VR, AR, mobile) without code changes. This document covers both the technical architecture and the visual design principles for creating intuitive, accessible game interfaces.

## Core Principles

### 1. Separation of Game Logic from Presentation

Bluefelt formalizes the separation between **Layer 1** (abstract game mechanics) and **Layer 2** (sensory presentation). The key insight is that while cardboard and wooden cubes were optimal for physical tabletops, digital mediums enable entirely new presentation metaphors that can better serve the underlying game mechanics.

**The Fundamental Question**: If we free Layer 2 from cardboard, what new wrapper can let Layer 1 blossom into a pure expression optimized for each medium?

### 2. Cross-Platform Entity Representation

At its core, any game entity (traditionally a "card") is just a way to represent data. A playing card has rank and suit, but more complex entities have many more data fields. Rather than limiting ourselves to rectangular cards, we can choose representations that optimize for:

- **Information density**: How much data fits per pixel/cubic centimeter
- **Interaction efficiency**: Faster physical gestures for common actions  
- **Thematic immersion**: Representations that enhance the game's narrative

### 3. Device-Optimized Presentations

The same entity can render differently across platforms while maintaining logical equivalence:
- **Mobile**: Compact icons with progressive disclosure (tap to expand)
- **Desktop**: Hover tooltips and detailed panels
- **VR**: Spatial 3D objects with physical manipulation
- **Audio**: Spoken descriptions for accessibility

### 4. Every Zone is First a Logical Container

All zones must define these mandatory logical properties:

| Field | Meaning | Fallback Rendering |
|-------|---------|-------------------|
| `id` | Stable handle used by verbs/diffs | — |
| `shape` | `stack`, `list`, `grid`, `track`, `choice`, etc. | **stack** → simple pile UI<br />**list** → scrollable row<br />**grid** → simple table<br />**track** → numbered cells<br />**choice** → selection interface |
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

## Entity Representation Metaphors

### Choosing the Right Visual Form

Before selecting a representation, analyze the entity's **information structure** and **interaction patterns**:

| Dimension | Design Impact |
|-----------|---------------|
| **Data Complexity** | 1-2 stats → coins/chips; 8+ stats → expandable panels |
| **Stacking Needs** | Chips and cubes stack; dominoes and boards don't |
| **Hidden/Public Info** | Objects with easy "flip" (coins, rotating discs) handle secrecy |
| **Interaction Verbs** | Dice invite rolling; radial dials invite twisting |
| **Device Context** | Mobile rewards dense icons; VR rewards spatial objects |

### 2D Presentation Metaphors

| Metaphor | Best For | UX Advantages |
|----------|----------|---------------|
| **Domino/Tile Strip** | 2-3 primary stats, adjacency matters | Alignment becomes meaningful; flatter stacking |
| **Coin/Poker Chip** | 1-2 stats, high counts, currency | Fast drag & throw; color/rim/center = 3 data channels |
| **Dial/Wheel** | Frequently changing values | Direct manipulation; readable from any angle |
| **Icon + Pop-over** | Referenced but rarely inspected | Saves space; hover reveals details |
| **Mini-tableau Row** | Entities with sub-slots/upgrades | Keeps ecosystem together in scroll areas |
| **Color-coded Chip Stack** | Quantity + category display | Height = quantity; color = type; single glance |

### 3D/Spatial Metaphors

| Metaphor | When to Use | VR/AR Benefits |
|----------|-------------|----------------|
| **Rotatable 3D Token** | Hidden facets, dramatic reveals | User-performed reveals; great rotation animation |
| **Holographic Info Orb** | Complex stats, one-at-a-time tracking | Gaze-based expansion; perfect for VR/AR |
| **Translucent Cubes** | Additive values, color categories | Vertical compactness; lighting shows totals |
| **Mini-dashboard Tablet** | Entities with child zones | Grabbable/zoomable in VR; collapsible on mobile |
| **Projected Board Glyph** | Ephemeral effects/auras | No collision; duration via fade/pulse |

### Design Heuristics

1. **Surface Area First**: Start with smallest legible representation; upgrade only if players repeatedly need details
2. **Progressive Disclosure**: Icon during normal play → rich display on inspection
3. **Color/Shape for Category**: Visual parsing beats text labels; reserve text for inspector view
4. **Obvious Interactions**: Rotation hints, expansion chevrons, grip patterns
5. **Device Affordances**: 
   - Mobile: thumb-sized targets, minimal rotation
   - Desktop: hover states, drag-select regions
   - VR: physical gestures, depth cues

### Implementation in Bluefelt

Because the server only ships JSON diffs, clients choose representations independently:

```yaml
# Server defines logical entity
entities:
  - id: resource_token
    data: { type: "wood", count: 3 }
    
# Client A: renders as chip stack (height = count, color = type)
# Client B: renders as "3🌳" text icon
# VR Client: renders as physical wooden cubes
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

---

# Visual Affordances Design Guide

Visual affordances provide clear, intuitive indicators of how players can interact with the game. This section explains how to design and implement effective affordances for different game mechanics within Bluefelt's presentation-separated architecture.

## Affordance Principles

1. **Discoverability**: Players should immediately understand what's interactive
2. **Feedback**: Clear response to user actions (hover, click, selection)
3. **Consistency**: Similar interactions should look and behave similarly
4. **Accessibility**: Work across devices, input methods, and abilities
5. **Performance**: Smooth, responsive interactions

## Interaction Patterns

### Cell-Based Interactions (Tic-Tac-Toe)

**Pattern**: Direct cell clicks on a grid

**Visual Cues**:
- Hover highlighting on valid cells
- Cursor changes to pointer on interactive cells
- Subtle border changes to indicate selection

**Implementation**:
```typescript
// BoardCell component shows hover states
<div 
  className={`cell ${isClickable ? 'clickable' : 'disabled'}`}
  onClick={isClickable ? handleClick : undefined}
>
  {content}
</div>
```

**CSS**:
```css
.cell.clickable {
  cursor: pointer;
  transition: background-color 0.2s;
}

.cell.clickable:hover {
  background-color: rgba(59, 130, 246, 0.1);
  border-color: #3b82f6;
}
```

### Column-Based Interactions (Connect 4)

**Pattern**: Click column headers to drop pieces with gravity

**Visual Cues**:
- Column drop zones above the board
- Down arrows (↓) indicating drop direction
- Blue background for valid columns
- Gray background for full/invalid columns

### Zone-Based Interactions (Card Games)

**Pattern**: Click zones or specific cards within zones

**Visual Cues**:
- Highlighted zone borders for valid drop targets
- Card selection indicators
- Drag and drop visual feedback

### Multi-Step Interactions (Checkers)

**Pattern**: Select piece, then select destination

**Visual Cues**:
- Selected piece highlighting
- Valid move indicators on target squares
- Path visualization for complex moves

## Generic Component Architecture

### Action Map Detection

Components automatically detect interaction patterns from the action map:

```typescript
// BoardZone detects column actions
const columnActions = Object.keys(actionMap || {})
  .filter(path => path.includes(`/zones/${zoneId}/columns/`))
  .map(path => {
    const match = path.match(/\/zones\/[^/]+\/columns\/(\d+)/);
    return match ? parseInt(match[1]) : -1;
  })
  .filter(col => col >= 0);
```

### Conditional Rendering

Affordances appear based on available actions:

```typescript
// Only show column drop zones when column actions exist
{columnActions.length > 0 && (
  <ColumnDropZones 
    columns={cols} 
    clickableColumns={columnActions}
    onColumnClick={handleColumnClick}
    isMyTurn={isMyTurn}
    cellSize={cellSize}
  />
)}
```

### State-Driven Feedback

Visual states reflect game conditions:

```typescript
const cellState = {
  isEmpty: cell === null,
  isClickable: isMyTurn && actionMap?.[location],
  isSelected: selection?.row === row && selection?.col === col,
  isValidTarget: validMoves?.includes(location)
};
```

## Accessibility Considerations

### Keyboard Navigation

```typescript
// Handle keyboard events for grid navigation
const handleKeyDown = (event: React.KeyboardEvent) => {
  switch (event.key) {
    case 'ArrowUp': moveFocus(row - 1, col); break;
    case 'ArrowDown': moveFocus(row + 1, col); break;
    case 'ArrowLeft': moveFocus(row, col - 1); break;
    case 'ArrowRight': moveFocus(row, col + 1); break;
    case 'Enter':
    case ' ': handleCellClick(row, col); break;
  }
};
```

### Screen Reader Support

```typescript
// Descriptive labels for game state
<div
  role="button"
  tabIndex={isClickable ? 0 : -1}
  aria-label={`${isEmpty ? 'Empty' : 'Occupied'} cell at row ${row + 1}, column ${col + 1}`}
  aria-disabled={!isClickable}
  onKeyDown={handleKeyDown}
>
```

### Color and Contrast

- Don't rely solely on color for information
- Ensure sufficient contrast ratios (4.5:1 minimum)
- Use icons, shapes, and text alongside color

### Touch Targets

- Minimum 44×44px touch targets on mobile
- Adequate spacing between interactive elements
- Consider thumb reach zones on larger screens

## Device-Specific Considerations

### Mobile Optimizations

**Touch Interactions**:
```typescript
// Handle both click and touch events
const handleInteraction = useCallback((event: React.MouseEvent | React.TouchEvent) => {
  event.preventDefault();
  if (isClickable) {
    handleAction();
  }
}, [isClickable, handleAction]);

<div
  onClick={handleInteraction}
  onTouchEnd={handleInteraction}
  className="touch-target"
>
```

**Responsive Sizing**:
```typescript
// Adjust cell size based on screen size
const cellSize = useMemo(() => {
  const isMobile = window.innerWidth < 768;
  const baseSize = isMobile ? 48 : 60;
  return Math.max(baseSize, Math.min(100, availableWidth / cols));
}, [cols, availableWidth]);
```

### Desktop Enhancements

**Hover States**:
```css
@media (hover: hover) {
  .interactive:hover {
    transform: scale(1.05);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }
}
```

**Cursor Feedback**:
```css
.clickable { cursor: pointer; }
.draggable { cursor: grab; }
.dragging { cursor: grabbing; }
.disabled { cursor: not-allowed; }
```

## Animation and Transitions

### Smooth State Changes

```css
.cell {
  transition: all 0.2s ease-in-out;
}

.cell.selected {
  transform: scale(1.1);
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}
```

### Piece Movement

```css
@keyframes dropPiece {
  from {
    transform: translateY(-100px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.dropping-piece {
  animation: dropPiece 0.5s ease-out;
}
```

## Best Practices

### Do's ✅

- ✅ Use consistent interaction patterns across games
- ✅ Provide immediate visual feedback for actions
- ✅ Test on multiple devices and input methods
- ✅ Include keyboard and screen reader support
- ✅ Use subtle animations to enhance understanding
- ✅ Make touch targets appropriately sized
- ✅ Handle error states gracefully
- ✅ Follow platform conventions (hover on desktop, touch on mobile)

### Don'ts ❌

- ❌ Rely solely on color to convey information
- ❌ Make interactive elements too small on mobile
- ❌ Use overly complex or distracting animations
- ❌ Ignore keyboard accessibility
- ❌ Create inconsistent interaction patterns
- ❌ Skip error state design
- ❌ Assume all users have perfect motor control
- ❌ Forget to test with real users

## Performance Considerations

### Efficient Rendering

```typescript
// Memoize expensive calculations
const cellStates = useMemo(() => {
  return boardData.map((row, rowIndex) => 
    row.map((cell, colIndex) => ({
      isEmpty: cell === null,
      isClickable: calculateClickable(rowIndex, colIndex),
      isHighlighted: calculateHighlight(rowIndex, colIndex)
    }))
  );
}, [boardData, actionMap, selection]);
```

### Minimize Redraws

```typescript
// Use React.memo for expensive components
const OptimizedCell = React.memo(({ 
  cell, 
  isClickable, 
  onClick 
}: CellProps) => {
  // Component implementation
}, (prevProps, nextProps) => {
  // Custom comparison for performance
  return prevProps.cell === nextProps.cell && 
         prevProps.isClickable === nextProps.isClickable;
});
```

Remember: Great visual affordances make games feel intuitive and enjoyable. Players should never have to guess how to interact with your game!
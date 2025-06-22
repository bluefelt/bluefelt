# Zone Hierarchy System

This document describes the four-tier spatial organization system for Bluefelt zones, which provides a structured way to organize game UI elements based on their function and importance.

## Overview

The zone hierarchy system categorizes all game zones into four tiers based on their spatial and functional relationships:

1. **Hand Zones** - Player-owned entities, private information
2. **Tactical Zones** - Direct interaction spaces, immediate gameplay
3. **Strategic Zones** - Big picture view, game state overview
4. **Ambient Zones** - Meta-information, UI chrome

## Tier Definitions

### Hand Zones (Tier 1)
**Purpose**: Player-specific zones containing owned entities that are typically private or semi-private.

**Characteristics**:
- Always accessible to the owning player
- Often hidden from other players (visibility: owner)
- Primary interaction point for player actions
- Located closest to the player's viewport edge

**Examples**:
- Card hands
- Personal resource pools
- Private scoring areas
- Player-specific tokens

**Layout Hints**:
- Position: bottom (for current player), top/sides (for opponents)
- Display: fanned cards, organized tokens
- Priority: Highest visibility and accessibility

### Tactical Zones (Tier 2)
**Purpose**: Shared gameplay spaces where most game actions occur.

**Characteristics**:
- Central to gameplay
- Visible to all players
- Contains entities that can be interacted with
- Represents the immediate game state

**Examples**:
- Game boards
- Card playing areas
- Shared resource pools
- Battle zones
- Worker placement spots

**Layout Hints**:
- Position: center of play area
- Display: clear grid/layout structure
- Priority: Maximum screen space

### Strategic Zones (Tier 3)
**Purpose**: Information displays showing game state overview and progress.

**Characteristics**:
- Read-only or minimal interaction
- Shows computed/aggregated information
- Helps with planning and strategy
- Often uses view zones

**Examples**:
- Score tracks
- Victory point displays
- Resource summaries
- Turn order indicators
- Phase/round tracking

**Layout Hints**:
- Position: edges of play area, above tactical zones
- Display: charts, tracks, summaries
- Priority: Visible but not obstructive

### Ambient Zones (Tier 4)
**Purpose**: Meta-game information and UI chrome.

**Characteristics**:
- Not part of core gameplay
- Provides context and history
- System-level information
- Can be hidden/minimized

**Examples**:
- Game log
- Chat
- Timer displays
- Connection status
- Settings

**Layout Hints**:
- Position: corners, collapsible panels
- Display: scrollable lists, small indicators
- Priority: Lowest, can be hidden

## Zone Definition Schema

To support the hierarchy system, zones will include a `tier` property:

```yaml
- id: "player_hand"
  name: "Your Hand"
  shape: list
  tier: hand  # Required: hand | tactical | strategic | ambient
  visibility: owner
  layoutHints:
    position: bottom
    display: fan
    maxWidth: 80%

- id: "board"
  name: "Game Board"  
  shape: grid
  tier: tactical
  shapeMeta:
    rows: 8
    cols: 8
  layoutHints:
    position: center
    display: grid
    aspectRatio: "1:1"

- id: "score_track"
  name: "Victory Points"
  shape: view
  tier: strategic
  viewType: track
  layoutHints:
    position: top
    display: horizontal-track
    height: 10%

- id: "game_log"
  name: "History"
  shape: view
  tier: ambient
  viewType: log
  layoutHints:
    position: right
    display: scrollable-list
    collapsible: true
    width: 20%
```

## Layout System Integration

The tier system works with layout hints to provide responsive game UIs:

### Layout Regions
```
+------------------[Strategic]------------------+
|                                               |
|  +--------+  [Tactical Center]  +--------+    |
|  |        |                     |        |    |
|  |  [A]   |    Game Board       |  [A]   |    |
|  |  [M]   |    Play Areas       |  [M]   |    |
|  |  [B]   |                     |  [B]   |    |
|  |  [I]   |                     |  [I]   |    |
|  |  [E]   |                     |  [E]   |    |
|  |  [N]   |                     |  [N]   |    |
|  |  [T]   |                     |  [T]   |    |
|  +--------+                     +--------+    |
|                                               |
+-------------------[Hand]---------------------+
```

### Responsive Behavior
- **Mobile**: Ambient zones collapse, hand zones overlay
- **Tablet**: Side ambient zones visible, strategic zones compact
- **Desktop**: All zones visible with optimal spacing

## Server Implementation

The server will:
1. Validate that all zones have a tier assignment
2. Include tier in zone metadata sent to clients
3. Provide default layout hints based on tier
4. Sort zones by tier for consistent rendering order

## Client Implementation

The client will:
1. Use tier to determine initial positioning
2. Apply responsive layout rules based on screen size
3. Allow tier-based filtering/hiding
4. Respect tier priority for touch/click targets

## Implementation Requirements

All zones must include a tier assignment:
1. Add the `tier` property to zone definitions
2. Choose from: `hand`, `tactical`, `strategic`, or `ambient`
3. Use layout hints to guide responsive behavior

## Benefits

1. **Consistent Layout**: All games follow similar spatial organization
2. **Responsive Design**: Tier-based rules for different screen sizes
3. **Accessibility**: Predictable location of similar elements
4. **Framework Support**: Client can provide tier-based components
5. **Designer Guidance**: Clear categories for zone purposes
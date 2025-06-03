# Bluefelt Game Developer Guide

## Table of Contents
1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
3. [Complete Schema Reference](#complete-schema-reference)
   - [Manifest Schema](#manifest-schema)
   - [Zones Schema](#zones-schema)
   - [Entities Schema](#entities-schema)
   - [Actions Schema](#actions-schema)
   - [Events Schema](#events-schema)
   - [Victory Schema](#victory-schema)
   - [Computed Schema](#computed-schema)
4. [Advanced Features](#advanced-features)
   - [Hidden Information](#hidden-information)
   - [Deduction Mechanics](#deduction-mechanics)
   - [App Integration](#app-integration)
   - [Simultaneous Actions](#simultaneous-actions)
   - [Asymmetric Information](#asymmetric-information)
5. [Game Examples](#game-examples)
   - [Ark Nova](#ark-nova)
   - [Age of Steam](#age-of-steam)
   - [Alchemists](#alchemists)

## Introduction

Bluefelt is a declarative game engine that allows you to create complex tabletop games using only YAML configuration files. This guide provides complete documentation of the schema.

## Core Concepts

### Game Bundle Structure
```
games/
└── your-game-name/
    └── 1.0/
        ├── manifest.yaml       # Game metadata and setup
        ├── zones.yaml          # Board areas and containers
        ├── entities.yaml       # Game pieces and components
        ├── actions.yaml        # Player actions
        ├── events.yaml         # Triggered effects
        ├── victory.yaml        # Win conditions
        ├── computed.yaml       # Dynamic calculations
        └── variants/           # Optional game variants
            ├── variant-1.yaml
            └── variant-2.yaml
```

## Complete Schema Reference

### Manifest Schema

The manifest.yaml file defines game metadata, setup, and turn structure.

```yaml
# Required fields
gameId: string                  # Unique game identifier (e.g., "tic-tac-toe")
version: string                 # Game version (e.g., "1.0")
specVersion: integer            # Bluefelt spec version (currently 1)
metadata:                       # Required metadata object
  name: string                  # Display name of the game
  description: string           # Brief game description
  author: string                # Game designer/creator
  players:
    min: integer                # Minimum players (1-8)
    max: integer                # Maximum players (1-8)

# Optional metadata
author: string                  # Original designer(s)
tags: [string]                  # Searchable tags
estimatedTime: integer          # Minutes for typical game
complexity: string              # "easy" | "medium" | "hard" | "expert"
age: string                     # Minimum age (e.g., "10+")
language: [string]              # Required languages for gameplay

# Game variants
variants:                       # Optional alternate game modes
  - id: string                  # Unique variant identifier
    name: string                # Display name
    description: string         # What makes this variant different
    default: boolean            # Is this the default variant?
    file: string                # Path to variant YAML file

# Variant selection UI
variantSelection:
  prompt: string                # Text shown to players
  display: string               # "list" | "grid" | "dropdown"
  showDescription: boolean      # Show variant descriptions?
  allowCustom: boolean          # Can players mix variants?

# Initial game setup
setup:                          # Sequential setup instructions
  - action: params              # Each step executed in order
  
# Common setup actions:
  - shuffle: zone_id            # Shuffle a deck/bag
  - deal:                       # Deal cards/tiles
      from: zone_id
      to: zone_id | zone_{player}
      count: integer | all
  - grantResources:             # Give starting resources
      to: player_id | all
      resources: {type: amount}
  - placeMarker:                # Place token on track
      track: zone_id
      position: integer
      marker: entity_id
  - forEach:                    # Repeat for each player
      players: all | list
    do: [actions]
  - setupGoodsCubes:            # Game-specific setup
      distribution: method
  - randomizeTurnOrder:         # Set random turn order
      track: zone_id

# Turn structure
turnStructure:
  type: string                  # "sequential" | "simultaneous" | "realtime"
  order: string                 # "clockwise" | "counterclockwise" | "custom" | "bid"
  rounds: integer | unlimited   # Number of rounds before game end
  timer: integer                # Seconds per turn (realtime only)
  
  phases:                       # List of turn phases
    - id: string                # Unique phase identifier
      name: string              # Display name
      description: string       # Help text
      mandatory: boolean        # Must this phase happen?
      optional: boolean         # Can players choose to do it?
      auto: boolean             # Does it happen automatically?
      simultaneous: boolean     # All players act at once?
      order: string             # "turn_order" | "reverse" | "custom"
      actions: [action_id]      # Available actions in this phase
      maxActions: integer | formula  # How many actions allowed
      timer: integer            # Phase timer in seconds
      skipIf: condition         # Skip phase if condition met

# Advanced features
hiddenInformation:             # For deduction games
  enableAppIntegration: boolean # Use companion app?
  secretsGeneration: string     # How secrets are determined
  
simultaneousReveal:             # For programming games
  phases: [phase_id]            # Which phases use this
  revealOrder: string           # Order of revelation
  
asymmetricSetup:               # For asymmetric games
  method: string                # How roles are assigned
  roles: [role_id]              # Available roles
```

### Zones Schema

Zones represent all containers and areas in the game.

```yaml
- id: string                    # Unique identifier (required)
  type: string                  # Zone type (required) 
  # Note: 'shape' is also supported for backwards compatibility
  # Valid types:
  # "list"     - Ordered/unordered collection
  # "deck"     - Shuffleable stack of cards
  # "grid"     - 2D grid of cells
  # "track"    - Linear progression track
  # "bag"      - Random draw container
  # "graph"    - Network of connected nodes
  # "network"  - Player-built connections
  # "slot"     - Single item container
  # "resource" - Numeric counter
  # "reference"- Static data lookup
  
  # Common properties
  visibility: string | object   # Who can see contents
  # Simple visibility:
  # "all"      - Everyone sees everything
  # "none"     - Nobody sees anything
  # "owner"    - Only owner sees
  # "active"   - Only active player
  # "team"     - Only team members
  # Complex visibility:
  visibility:
    contents: string            # Who sees actual items
    count: string               # Who sees number of items
    top: string                 # Who sees top card (deck)
    properties: [string]        # Which properties visible
    
  accessibility: string         # Who can interact
  # "all"      - Anyone can use
  # "owner"    - Only owner
  # "active"   - Current player only
  # "none"     - Nobody (display only)
  
  variant_specific: boolean     # Defined by game variant?
  ui:                           # Rendering hints
    position: string            # "hand" | "board" | "sidebar" | "overlay"
    layout: string              # "row" | "column" | "grid" | "fan" | "stack"
    group: string               # Visual grouping identifier
    hidden: boolean             # Hide from UI entirely?
    
  # List-specific properties
  listProps:
    ordered: boolean            # Does order matter?
    maxSize: integer | infinite # Maximum items
    minSize: integer            # Minimum items
    unique: boolean             # No duplicates allowed?
    handLimit: integer          # Trigger discard if exceeded
    sortBy: string              # Auto-sort by property
    temporary: boolean          # Clear each turn?
    
  # Deck-specific properties  
  deckProps:
    shuffle: boolean            # Shuffle on creation?
    faceDown: boolean           # Cards face down?
    reshuffleDiscardWhenEmpty: boolean  # Auto-reshuffle?
    revealTop: integer          # Show N top cards
    drawTo: zone_id             # Default draw destination
    discardTo: zone_id          # Default discard destination
    
  # Grid-specific properties
  gridProps:
    cols: integer               # Grid columns (width also supported)
    rows: integer               # Grid rows (height also supported)
    hexagonal: boolean          # Hex grid?
    wrapping: boolean           # Edges connect?
    diagonals: boolean          # Diagonal movement?
    buildPattern: string        # "anywhere" | "connected" | "adjacent"
    coordinates: string         # "numeric" | "chess" | "hex"
    
  # Track-specific properties
  trackProps:
    length: integer             # Number of positions
    circular: boolean           # Does it loop?
    bidirectional: boolean      # Can move backwards?
    start: integer              # Starting position index
    rewards: map                # Position-based rewards
      position: {resource: amount}
    labels: map                 # Custom position labels
      position: string
    branches: [track_id]        # Connecting tracks
    
  # Bag-specific properties
  bagProps:
    opaque: boolean             # Can't see contents?
    returnable: boolean         # Can put items back?
    autoRefill: boolean         # Refill when empty?
    refillFrom: zone_id         # Source for refill
    
  # Graph-specific properties
  graphProps:
    nodes: [node_definition]    # List of nodes
    edges: [edge_definition]    # Connections
    directed: boolean           # One-way edges?
    weighted: boolean           # Edges have cost?
    
  # Network-specific properties  
  networkProps:
    type: string                # Network type
    owner: player_id            # Network owner
    color: string               # Display color
    maxLinks: integer           # Maximum connections
    linkCost: formula           # Cost per link
    constraints: [string]       # Building rules
    
  # Slot-specific properties
  slotProps:
    capacity: integer           # Usually 1
    acceptTypes: [entity_type]  # What can go here
    locked: boolean             # Can't remove once placed?
    secret: boolean             # Hidden selection?
    
  # Resource-specific properties
  resourceProps:
    type: string                # Resource name
    min: integer                # Minimum value
    max: integer                # Maximum value
    startValue: integer         # Initial value
    trackable: boolean          # Show on UI?
    allowNegative: boolean      # Can go below 0?
    
  # Initial contents
  contents: string | array      # Starting items
  # "empty"    - Starts empty
  # "preset"   - Use contents array
  # Array format:
  contents:
    - entity: entity_id         # What entity
      count: integer | infinite # How many
      at: position              # Grid position
      properties: map           # Override properties
      
  # State triggers
  triggers:                     # Actions on state change
    onEmpty: [action]           # When last item removed
    onFull: [action]            # When capacity reached
    onAdd: [action]             # When item added
    onRemove: [action]          # When item removed
    onShuffle: [action]         # When shuffled
    
  # Validation
  validPlacements:              # What can be placed
    - type: entity_type
      conditions: [condition]
      
  # Computed properties
  dynamicSize: formula          # Size based on game state
  dynamicVisibility: formula    # Visibility changes
```

### Entities Schema

Entities are all game components: pieces, cards, tokens, counters, etc.

```yaml
- id: string                    # Unique identifier (required)
  name: string                  # Display name (optional)
  type: string                  # Entity type (optional)
  # Types: token, counter, deck, dice, tile, boolean
  quantity: integer             # Number to create (default: 1)
  
  # Properties define entity data
  props:                        # Game-specific properties
    # Common patterns
    value: any                  # Entity value
    player: "{player}"          # Owner (supports templates)
    
    # Counter properties
    min: integer                # Minimum value
    max: integer                # Maximum value
    
    # Card properties  
    suit: string                # Card suit
    rank: string                # Card rank
    
    # Dice properties
    sides: integer              # Number of sides
    
    # Any custom properties
    [key]: any                  # Your game data
    
  # UI representation
  ui:
    # Token rendering
    tokenType: string           # Shape/style
    # Values: circle, x, o, "{player}", etc.
    
    # Display modes
    display: string             # How to render
    # Values: number, dice, tile, boolean, meeple, icon
    
    # Visual properties
    color: string               # Color (supports "{player.color}")
    icon: string                # Icon name
    glyph: string               # Unicode character
    
  # Special options
  options:                      # For deck entities
    cardType: string            # Card type definition
    jokers: integer             # Number of jokers
```

**Shorthand:**
```yaml
# Standard 52-card deck
- type: standardDeck
  id: deck
  
# Player entities using templates
- id: score_{player}
  type: counter
  props:
    value: 0
    min: 0
```

### Actions Schema

Actions define all possible player actions.

```yaml
- id: string                    # Unique identifier (required)
  uses: string                  # Engine function to use 
  # Note: 'implementation' and 'builtin' also supported for backwards compatibility
  # Common functions:
  # "moveEntity"    - Move between zones
  # "playCard"      - Play from hand
  # "draw"          - Draw cards/tokens
  # "discard"       - Discard items
  # "placeToken"    - Put token on board
  # "payResources"  - Spend resources
  # "rollDice"      - Random generation
  # "auction"       - Bidding mechanics
  # "trade"         - Exchange items
  # "select"        - Choose option
  # "activate"      - Use ability
  # "attack"        - Combat action
  
  # When available
  trigger: string               # Phase or condition
  phase: phase_id               # Specific phase only
  priority: integer             # Resolution order
  
  # Requirements
  constraints:                  # When action is valid
    - builtin: string           # Constraint type
      params: map               # Constraint parameters
  # Common constraints:
  # "hasResources"    - Enough resources
  # "hasCards"        - Cards in hand
  # "isActivePlayer"  - Your turn
  # "inPhase"         - Correct phase
  # "hasSpace"        - Room available
  # "connected"       - Network path exists
  # "range"           - Within distance
  # "uniquePerTurn"   - Once per turn
  
  # Parameters
  with:                         # Action configuration
  # Note: 'params' also supported for backwards compatibility
    from: zone_id               # Source zone
    to: zone_id                 # Destination zone
    count: integer | "all"      # How many
    filter: condition           # Which items valid
    target: selector            # What to affect
    
  # Costs
  costs:                        # What player pays
    resources: map              # Resource costs
    actions: integer            # Action points
    items: [entity_id]          # Specific items
    formula: string             # Computed cost
    
  # Effects
  sideEffects:                  # What happens
    - effect_definition         # Ordered list
  # Common effects:
  # "grantResources"  - Give resources  
  # "moveToken"       - Advance on track
  # "drawCards"       - Draw items
  # "revealInformation" - Show hidden info
  # "modifyEntity"    - Change properties
  # "triggerEvent"    - Fire event
  # "createEntity"    - Spawn new item
  
  # Choices
  choices:                      # Player decisions
    - id: string
      description: string
      constraints: [condition]
      do: [effect]
      
  # UI hints
  ui:
    prompt: string              # What to ask player
    help: string                # Detailed help
    confirm: boolean            # Require confirmation?
    highlight: selector         # What to highlight
    preview: boolean            # Show result preview?
    icon: string                # Button icon
    
  # Timing
  timing:
    instant: boolean            # Resolves immediately?
    interruptible: boolean      # Can be responded to?
    duration: string            # "instant" | "turn" | "permanent"
    
  # Validation
  minTargets: integer           # Minimum selections
  maxTargets: integer           # Maximum selections
  optional: boolean             # Can choose not to do it?
  repeatable: boolean           # Can do multiple times?
```

### Events Schema

Events trigger automatically based on game state.

```yaml
- id: string                    # Unique identifier (required)
  
  # Trigger conditions
  trigger:
    on: string                  # What causes this event
    # Common triggers:
    # "phaseStart"      - Phase begins
    # "phaseEnd"        - Phase ends  
    # "turnStart"       - Turn begins
    # "turnEnd"         - Turn ends
    # "cardPlayed"      - Card enters play
    # "pieceMove"       - Entity moves
    # "resourceGain"    - Resources gained
    # "resourceSpent"   - Resources spent
    # "zoneEmpty"       - Zone emptied
    # "zoneFull"        - Zone at capacity
    # "scoreThreshold"  - Score reached
    # "entityCreated"   - New entity made
    # "entityDestroyed" - Entity removed
    # "connection"      - Network built
    # "combat"          - Battle occurs
    
    # Additional filters
    phase: phase_id             # Only in specific phase
    zone: zone_id               # Only for specific zone
    entity: entity_type         # Only for entity type
    player: player_selector     # Only for players
    
  # Conditions for triggering
  condition: condition_definition  # Must be true to fire
  
  # What happens
  effects:                      # Ordered list of effects
    - effect_definition
    
  # Common effect types:
  # "announce"        - Show message
  # "grantBonus"      - Give rewards
  # "applyPenalty"    - Impose cost
  # "modifyState"     - Change game state
  # "triggerAction"   - Force action
  # "preventAction"   - Block action
  # "revealInfo"      - Show hidden data
  
  # Control flow
  priority: integer             # Resolution order
  mandatory: boolean            # Must resolve?
  cancelable: boolean           # Can be prevented?
  unique: boolean               # Only fires once?
  
  # Scope
  affects: player_selector      # Who is affected
  # "triggering"  - Player who caused it
  # "all"         - Every player
  # "others"      - All except triggering
  # "owner"       - Owner of component
  # "active"      - Current player
  
  # Conditional effects
  if:
    condition: condition_def
    then: [effect]
    else: [effect]              # Optional else clause
```

### Victory Schema

Victory conditions and scoring.

```yaml
# Victory type
victory:
  type: string                  # How winner determined
  # "highest_score"   - Most points wins
  # "lowest_score"    - Least points wins
  # "first_to"        - First to threshold
  # "last_standing"   - Elimination
  # "objective"       - Complete goal
  # "vote"            - Players decide
  
  description: string           # Explain victory
  
  # End game triggers
  triggers:
    immediate: [condition]      # Game ends instantly
    endOfTurn: [condition]      # End after turn
    endOfRound: [condition]     # End after round
    
  # Scoring (if applicable)  
  scoring:
    # Score components
    components:
      - id: string
        name: string
        source: string          # Where value comes from
        formula: string         # How to calculate
        multiplier: number      # Scale factor
        max: number             # Cap value
        visible: boolean        # Show during game?
        
    # Categories
    categories:
      - id: string
        name: string
        components: [id]        # Which components
        bonus: formula          # Category bonus
        
    # Final calculation
    finalScore:
      formula: string           # How to combine
      rounding: string          # "up" | "down" | "nearest"
      
  # Objectives (if applicable)
  objectives:
    - id: string
      name: string  
      description: string
      condition: condition_def
      points: integer           # Reward if completed
      unique: boolean           # Only one player can claim?
      
  # Tiebreakers
  tiebreakers:
    - highest: property         # Most of property wins
    - lowest: property          # Least wins
    - most: entity_type         # Count entities
    - fewest: entity_type       # Reverse count
    - custom: formula           # Complex tiebreak
    
  # Special conditions
  special:
    - id: string
      name: string
      condition: condition_def
      result: string            # What happens
      overrides: boolean        # Ignores normal victory?
      
  # Display
  display:
    trackScore: boolean         # Show running score?
    hideUntilEnd: boolean       # Secret scoring?
    components: boolean         # Show breakdown?
    leaderboard: boolean        # Rank players?
```

### Computed Schema

Dynamic calculations based on game state.

```yaml
- id: string                    # Unique identifier (required)
  description: string           # What this computes
  
  # Calculation
  formula: string               # The computation
  # Can use:
  # - Basic math: +, -, *, /, %, ^
  # - Comparisons: <, >, <=, >=, ==, !=
  # - Logic: &&, ||, !
  # - Conditionals: condition ? true : false
  # - Functions: sum(), count(), min(), max(), etc.
  # - Game state: zones, entities, properties
  
  # Context required
  context: [string]             # Required parameters
  # "player"      - Which player
  # "entity"      - Which entity
  # "zone"        - Which zone
  # "position"    - Board position
  # Custom parameters for formula
  
  # Caching
  cache: boolean                # Cache result?
  cacheInvalidation: [string]   # What invalidates cache
  
  # Return type
  returns: string               # Data type returned
  # "integer"     - Whole number
  # "float"       - Decimal number  
  # "boolean"     - True/false
  # "string"      - Text
  # "entity"      - Entity reference
  # "zone"        - Zone reference
  # "list"        - Array of items
  # "map"         - Key-value pairs
  
  # Visibility
  visible: player_selector      # Who can see result
  
  # Examples of formulas:
  # "count(hand_{player})"    - Cards in hand
  # "sum(resources.*.value)"  - Total resource value
  # "distance(a, b) <= range" - Within range
  # "hasPath(from, to, network_{player})" - Connected
```

## Client-Server Interaction

### Action Map System

Starting with Bluefelt v0.2, the server provides an action map instead of a possibleVerbs array. This change simplifies client implementation and ensures consistency across different client platforms.

#### What Changed

**Before (possibleActions array):**
```json
{
  "possibleActions": {
    "p1": [
      {
        "action": "place",
        "direction": "Choose a cell",
        "validOptions": [
          {"zone": "board", "row": 0, "col": 0},
          {"zone": "board", "row": 0, "col": 1}
        ]
      }
    ]
  }
}
```

**Now (actionMap):**
```json
{
  "actionMap": {
    "p1": {
      "/zones/board/0/0": {"action": "place", "direction": "Choose a cell"},
      "/zones/board/0/1": {"action": "place", "direction": "Choose a cell"}
    }
  }
}
```

#### Benefits for Client Developers

1. **Direct Mapping**: Clients can directly check if a location has an action without iterating through arrays
2. **Simplified Logic**: Click handlers just look up `actionMap[location]` instead of filtering possibleVerbs
3. **Consistent Paths**: All locations use the same path format: `/zones/{zoneId}/{index}` or `/zones/{zoneId}/{row}/{col}`
4. **Platform Agnostic**: The same action map works for web, mobile, VR, or any other client platform

#### Location Path Format

- **Grid zones**: `/zones/{zoneId}/{row}/{col}` (e.g., `/zones/board/0/1`)
- **List/deck zones**: `/zones/{zoneId}/{index}` (e.g., `/zones/hand_p1/2`)
- **Whole zones**: `/zones/{zoneId}` (e.g., `/zones/deck` for draw actions)

### Modernized Field Names

Bluefelt v0.2 introduces cleaner field names while maintaining backwards compatibility:

#### Zone Definitions
- Use `type` instead of `shape` for zone types
- Use `cols`/`rows` instead of `width`/`height` for grid properties

#### Action Definitions  
- Use `uses` instead of `builtin` or `implementation` for the engine function
- Use `with` instead of `params` for action parameters

Examples:
```yaml
# Modern zone definition
- id: board
  type: grid  # Previously: shape: grid
  gridProps:
    cols: 8   # Previously: width: 8
    rows: 8   # Previously: height: 8

# Modern action definition
- id: place_mark
  uses: grid.move  # Previously: builtin: moveEntity
  with:            # Previously: params:
    source: marks_{actor}
    target:
      zone: board
```

## Advanced Features

### Hidden Information

For games with deduction, hidden roles, or secret objectives:

```yaml
# Secret generation
secrets:
  - id: string
    type: string                # "role" | "objective" | "information"
    distribution: string        # "random" | "draft" | "auction"
    visibility: string          # Who can see
    revealCondition: condition  # When revealed
    
# Hidden zones
hiddenZones:
  - id: secret_info_{player}
    visibility:
      owner: full
      others: none
    contents: generated         # Created during setup
    
# Deduction tokens
deductionTokens:
  - id: clue_token
    represents: hidden_property
    placeable: boolean          # Can place on board
    shareable: boolean          # Can show others
```

### Deduction Mechanics

For games like Alchemists or Cryptid:

```yaml
# Logical deduction
deduction:
  # Define hidden properties
  hiddenProperties:
    - id: ingredient_nature
      values: [positive, negative, neutral]
      distribution: unique      # Each different
      
  # Testing mechanics
  experiments:
    - id: test_potion
      inputs: [ingredient1, ingredient2]
      reveals: partial_result   # Not full info
      cost: resources
      
  # Recording information
  notebook:
    - id: player_notebook_{player}
      tracks: experiments
      visibility: owner
      shareable: selective      # Can show parts
```

### App Integration

For games requiring companion apps:

```yaml
# App connection
appIntegration:
  required: boolean             # Must use app?
  optional: boolean             # Can use app?
  features: [string]            # What app provides
  # "randomizer"    - Complex randomization
  # "calculator"    - Scoring computation
  # "referee"       - Rule enforcement
  # "narrator"      - Story/event reading
  # "timer"         - Time tracking
  # "secret_keeper" - Hidden information
  
  # Communication
  protocol: string              # How to connect
  syncData: [string]            # What to sync
  
  # Fallback
  withoutApp:                   # If app not used
    alternative: string         # How to play
    components: [string]        # Extra needed
```

### Simultaneous Actions

For programming or real-time games:

```yaml
# Simultaneous planning
simultaneousPlanning:
  # Action selection
  selection:
    secret: boolean             # Hidden selection
    limited: boolean            # Limited options
    ordered: boolean            # Order matters
    
  # Reveal phase
  reveal:
    order: string               # "simultaneous" | "priority"
    conflicts: string           # How to resolve
    
  # Resolution
  resolution:
    order: formula              # Based on choices
    interrupts: boolean         # Can interrupt
```

### Asymmetric Information

For games where players know different things:

```yaml
# Information distribution
informationSets:
  - id: string
    knownTo: player_selector
    hiddenFrom: player_selector
    revealWhen: condition
    
# Asymmetric setup
asymmetricSetup:
  - role: string
    startingInfo: [info_id]
    hiddenInfo: [info_id]
    specialRules: [rule_id]
    
# Communication limits
communication:
  allowed: boolean
  restrictions: [string]        # What can't say
  verification: string          # How to verify truth
```

## Game Examples

### Ark Nova
[Previous Ark Nova content]

### Age of Steam  
[Previous Age of Steam content]

### Alchemists
[Full implementation follows...]
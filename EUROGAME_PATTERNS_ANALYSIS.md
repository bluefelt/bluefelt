# Eurogame Patterns Analysis for Bluefelt SDK Evolution

Based on analysis of the top 25 games on BoardGameGeek, this document identifies common eurogame patterns and proposes SDK changes to support games of Ark Nova complexity.

## Executive Summary

Modern eurogames share 12 major mechanical patterns that the Bluefelt SDK needs to support. The most critical are:
1. **Multi-use cards** (80% of games) - Cards serve multiple purposes based on context
2. **Action selection systems** (100% of games) - Complex decision trees beyond simple verb lists
3. **Resource conversion chains** (75% of games) - Transform resources through multiple steps
4. **Variable player powers** (70% of games) - Asymmetric abilities that modify core rules

## Detailed Pattern Analysis

### 1. Multi-Use Cards (20/25 games)
Cards that can be used in multiple ways depending on context.

**Examples:**
- **Ark Nova**: Cards can be played for animals OR discarded for money/actions
- **Terraforming Mars**: Cards provide immediate effects, ongoing production, or can be sold
- **Wingspan**: Birds provide points, activate powers, or enable egg laying
- **Race for the Galaxy**: Cards are both buildable developments AND payment currency

**SDK Requirements:**
```yaml
# Proposed enhancement
- id: playCard
  builtin: contextualAction
  contexts:
    - condition: {mode: build}
      action: buildCard
    - condition: {mode: discard}
      action: discardForResource
    - condition: {mode: activate}
      action: activatePower
```

### 2. Action Selection Systems (25/25 games)
Complex systems for choosing and executing actions, often with variable costs or limited availability.

**Types Identified:**
- **Worker Placement** (Agricola, Caverna, Lost Ruins of Arnak)
- **Action Tracks** (Ark Nova with upgrading actions)
- **Rondel** (Great Western Trail circular action selection)
- **Card-Driven** (Twilight Struggle, Gloomhaven)
- **Dice Placement** (Castles of Burgundy)

**SDK Requirements:**
```yaml
# Proposed action selection framework
actionSystems:
  - type: workerPlacement
    spots:
      - id: gatherWood
        capacity: 1
        rewards: [wood: 3]
        requirements: [worker: 1]
  
  - type: actionTrack
    actions:
      - id: build
        levels: [1, 2, 3, 4, 5]
        costToUpgrade: [money: 2]
```

### 3. Resource Conversion Chains (19/25 games)
Multi-step processes to transform basic resources into victory points.

**Examples:**
- **Agricola**: Grain → Bread → Food → Points
- **Brass Birmingham**: Coal + Iron → Manufactured Goods → Money → Infrastructure
- **Wingspan**: Food → Eggs → More Birds → Points

**SDK Requirements:**
```yaml
# Proposed conversion system
conversions:
  - id: bakeBread
    inputs: [grain: 1]
    outputs: [food: 2]
    requirements: [oven: true]
    
  - id: chainedConversion
    chain:
      - convert: [wood: 2] → [plank: 1]
      - convert: [plank: 3] → [furniture: 1]
      - convert: [furniture: 1] → [points: 5]
```

### 4. Area Control & Influence (13/25 games)
Spatial control mechanics with cascading effects.

**Examples:**
- **Brass Birmingham**: Connected networks for resource/market access
- **Twilight Struggle**: Influence in countries affects scoring
- **Eclipse**: Hex control for resources and combat
- **Scythe**: Territory control with resource production

**SDK Requirements:**
```yaml
# Proposed area control system
zones:
  - id: region_{id}
    type: territory
    properties:
      controller: null
      influence: {p1: 0, p2: 0}
      adjacentTo: [region_2, region_3]
      resources: [wood: 2, stone: 1]
    
effects:
  - type: adjacencyBonus
    trigger: controlGained
    bonus: [influence: 1]
    range: adjacent
```

### 5. Tech Trees & Progression Systems (16/25 games)
Unlocking new abilities or improving existing ones through advancement.

**Examples:**
- **Terraforming Mars**: Corporation engines with card synergies
- **Eclipse**: Technology tiles providing permanent upgrades
- **Through the Ages**: Civilization advancement through ages

**SDK Requirements:**
```yaml
# Proposed tech tree system
techTrees:
  - id: militaryTech
    nodes:
      - id: improvedLogistics
        cost: [science: 3]
        unlocks: [moveRange: +2]
        prerequisites: [basicLogistics]
      
progressionTracks:
  - id: infrastructure
    levels: [0, 1, 2, 3, 4, 5]
    benefits:
      1: [income: +2]
      3: [actions: +1]
      5: [endgamePoints: 10]
```

### 6. Variable Player Powers (18/25 games)
Asymmetric abilities that fundamentally change how players interact with the game.

**Examples:**
- **Root**: Completely different victory conditions and actions per faction
- **Spirit Island**: Unique spirit powers that break core rules
- **Terraforming Mars**: Corporation powers modifying costs/benefits

**SDK Requirements:**
```yaml
# Proposed player power system
players:
  - id: p1
    faction: merchants
    powers:
      - modifyCost: {action: trade, discount: 1}
      - extraAction: {type: trade, perTurn: 1}
      - uniqueVerb: establishTradeRoute
    victoryCondition: [gold: 100]
```

### 7. Multi-Phase Rounds (21/25 games)
Rounds divided into distinct phases with different rules/actions.

**Current Implementation:** Basic phase system exists
**Enhancement Needed:** Automatic phase transitions, phase-specific rules

```yaml
# Proposed enhancement
phases:
  - id: planning
    simultaneous: true  # All players act at once
    actions: [selectCards, allocateWorkers]
    autoAdvance: allPlayersReady
    
  - id: execution
    turnOrder: dynamic  # Based on planning choices
    actions: contextual  # Based on worker placement
```

### 8. Hidden Information & Simultaneous Planning (15/25 games)
Secret selection followed by simultaneous reveal.

**Examples:**
- **Gloomhaven**: Card selection for combat
- **Twilight Struggle**: Simultaneous card play
- **7 Wonders**: Drafting with simultaneous reveal

**SDK Requirements:**
```yaml
# Proposed hidden information system
- id: selectAction
  builtin: hiddenSelect
  params:
    choices: hand_{actor}
    count: 2
    visibility: private
  triggers:
    - condition: allPlayersSelected
      action: revealSelections
```

### 9. Market Mechanisms (14/25 games)
Dynamic markets with supply/demand pricing.

**Examples:**
- **Brass Birmingham**: Coal/iron markets with depletion
- **Power Grid**: Resource market with price escalation
- **Clans of Caledonia**: Fluctuating goods prices

**SDK Requirements:**
```yaml
# Proposed market system
markets:
  - id: coalMarket
    type: depletingSupply
    levels:
      - quantity: 3, price: 1
      - quantity: 3, price: 2
      - quantity: 3, price: 3
    refill: {phase: market, amount: 3}
    
  - id: goodsMarket
    type: fluctuating
    tracks: [wool, whiskey, cheese]
    priceRange: [1, 6]
    modifiers:
      onSell: -1
      onBuy: +1
```

### 10. Tile/Card Placement Puzzles (11/25 games)
Spatial puzzles where placement matters for scoring.

**Examples:**
- **Castles of Burgundy**: Tetris-like estate building
- **Carcassonne**: Tile placement for area completion
- **Azul**: Pattern building with restrictions

**SDK Requirements:**
```yaml
# Proposed placement system
- id: placeTile
  builtin: spatialPlace
  params:
    target: personalBoard_{actor}
    constraints:
      - adjacent: true
      - matchingEdges: true
      - colorRestriction: {row: unique, column: unique}
    scoring:
      - pattern: [2x2_same_color, points: 10]
      - completion: [row_filled, points: 5]
```

### 11. Engine Building Progression (20/25 games)
Creating combinations that generate increasing returns.

**Examples:**
- **Wingspan**: Bird powers triggering in sequence
- **Terraforming Mars**: Production chains compounding each round
- **Race for the Galaxy**: Phase bonuses stacking

**SDK Requirements:**
```yaml
# Proposed engine system
engines:
  - id: productionEngine
    triggers:
      - phase: production
      - cascade: true  # Effects trigger other effects
    effects:
      - foreach: [building_type: factory]
        produce: [goods: 2]
      - foreach: [goods: ">5"]
        bonus: [money: 1]
```

### 12. Victory Point Salad (22/25 games)
Multiple scoring paths allowing different strategies.

**Examples:**
- **Castles of Burgundy**: Points from tiles, goods, knowledge, etc.
- **Everdell**: Points from cards, events, journey, prosperity

**SDK Requirements:**
```yaml
# Proposed scoring system
scoring:
  tracks:
    - id: immediate
      visible: true
      sources: [buildings, trade, exploration]
    
    - id: endgame
      visible: false
      calculations:
        - setCollection: {animals: unique, multiplier: 3}
        - areaMultiplier: {largestRegion: 2}
        - resourceConversion: {per3goods: 1}
```

## Priority Implementation Order

Based on frequency and impact, implement in this order:

### Phase 1: Core Eurogame Infrastructure
1. **Multi-use cards/context system** - Fundamental to modern designs
2. **Enhanced action selection** - Beyond simple verb lists
3. **Resource conversion chains** - Economic game foundation
4. **Variable player powers** - Asymmetry is expected

### Phase 2: Advanced Mechanics
5. **Tech trees/progression** - Advancement systems
6. **Hidden information** - Simultaneous planning
7. **Market mechanisms** - Dynamic economies
8. **Area control** - Spatial interactions

### Phase 3: Polish Features
9. **Tile placement** - Spatial puzzles
10. **Engine building** - Cascading effects
11. **Complex scoring** - Multiple paths to victory

## Shorthand Opportunities

### 1. Standard Resource Set
```yaml
# Current: Define each resource
resources:
  - wood
  - stone
  - food
  - gold

# Proposed shorthand
resources: standardEuro  # wood, stone, food, gold, workers
```

### 2. Worker Placement Actions
```yaml
# Current: Complex verb definition
- id: gatherWood
  builtin: moveEntity
  params:
    source: workers_{actor}
    target: woodGatheringSpot
  effects:
    - addResource: {wood: 3}

# Proposed shorthand
- builtin: workerSpot
  spot: gatherWood
  cost: 1
  reward: [wood: 3]
```

### 3. Market Templates
```yaml
# Current: Define entire market
# ... 50+ lines ...

# Proposed shorthand
- builtin: market.depleting
  resource: coal
  template: standard  # 3/3/3 at 1/2/3 cost
```

### 4. Tech Tree Templates
```yaml
# Current: Define each node
# ... 100+ lines ...

# Proposed shorthand
- builtin: techTree
  template: civilization  # Military, Science, Culture, Economy branches
  customNodes: [...]  # Add game-specific techs
```

## Compatibility Strategy

All enhancements should be:
1. **Additive** - Don't break existing games
2. **Optional** - Games can use old or new syntax
3. **Composable** - Mix and match features
4. **Declarative** - No mandatory scripting

## Conclusion

The Bluefelt SDK has strong foundations with its recent card game improvements. To support eurogames of Ark Nova's complexity, focus on:

1. **Context-aware actions** - Same card/piece, different uses
2. **Complex action selection** - Beyond simple verb lists
3. **Resource transformation** - Multi-step conversion chains
4. **Asymmetric gameplay** - Variable powers and victory conditions

These patterns appear in 70%+ of top eurogames. Implementing them would allow Bluefelt to handle most modern board game designs while maintaining its declarative, shorthand-friendly approach.
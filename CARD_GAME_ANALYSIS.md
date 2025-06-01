# Comprehensive Card Game Analysis

## Games Analyzed

1. **Solitaire (Klondike)** - Single-player tableau building
2. **Poker** - Betting and hand ranking
3. **Blackjack** - Target score with dealer
4. **Gin Rummy** - Meld formation with deadwood
5. **Hearts** - Trick-taking avoidance
6. **Crazy Eights** - Shedding with wildcards
7. **Go Fish** - Set collection
8. **Cribbage** - Points via combinations, uses board
9. **Euchre** - Trick-taking with trump
10. **Bridge/Whist** - Partnership trick-taking

## Common Patterns Identified

### 1. Zone Types (Beyond Current Implementation)

#### New Zone Patterns
- **Tableau** (Solitaire): Multi-column layout with face-up/down cards
- **Foundation** (Solitaire): Build-up zones by suit
- **Stock/Waste** (Solitaire): Draw pile with discard, recyclable
- **Crib** (Cribbage): Separate hand for dealer
- **Trick Zone** (Hearts, Euchre, Bridge): Temporary collection area

#### Parameterized Zone Properties
```yaml
zones:
  - id: tableau_{column}
    type: stack
    stackProps:
      faceUp: bottom  # or "top", "all", "none"
      buildDirection: down  # or "up"
      buildBy: alternatingColor  # or "suit", "rank"
      startingCards: {column}  # dynamic count
      
  - id: foundation_{suit}
    type: stack
    stackProps:
      acceptOnly: {suit}
      buildDirection: up
      startFrom: A
      endAt: K
```

### 2. Turn Structures

#### Pattern: Multi-Phase Turns
Many games have complex turn structures:

**Gin Rummy**: Draw → Meld/Arrange → Discard → Knock/Gin
**Cribbage**: Deal → Discard to Crib → Play → Show → Score
**Hearts**: Pass Cards → Play Tricks → Score

#### Proposed Shorthand
```yaml
turnStructure:
  phases:
    - draw:
        from: [drawPile, discardPile]  # choice
        count: 1
    - action:
        optional: true
        verbs: [meld, layoff]
    - discard:
        count: 1
        required: true
    - endTurn:
        options: [knock, continue]
```

### 3. Scoring Mechanisms

#### Pattern Types
1. **Point Avoidance** (Hearts): Minimize points
2. **Target Score** (Blackjack): Get close without exceeding
3. **Combination Scoring** (Cribbage): Points for patterns
4. **Meld Scoring** (Gin Rummy): Deadwood difference
5. **Trick Counting** (Euchre): Win majority

#### Proposed Scoring Builtin
```yaml
scoring:
  type: avoidance  # or "target", "combination", "difference"
  params:
    avoidanceCards:
      - suit: hearts
        points: 1
      - card: Q♠
        points: 13
    specialCases:
      - name: "shootTheMoon"
        condition: "hasAll(hearts) && has(Q♠)"
        effect: "opponents.score += 26"
```

### 4. Card Movement Patterns

#### New Movement Types
1. **Build** (Solitaire): Stack with specific rules
2. **Meld** (Rummy): Form groups, keep visible
3. **Trick Play** (Hearts): All play to center, winner takes
4. **Pass** (Hearts): Simultaneous exchange

#### Proposed Builtin
```yaml
- id: passCards
  builtin: simultaneousExchange
  params:
    count: 3
    direction: left  # or "right", "across", "none"
    
- id: playToTrick
  builtin: trickPlay
  params:
    leadPlayer: {trickWinner}
    mustFollowSuit: true
    scoring: avoidance
```

### 5. Game End Conditions

#### Pattern Types
1. **Empty Hand** (Crazy Eights, Gin)
2. **Score Threshold** (Hearts, Cribbage)
3. **All Cards Placed** (Solitaire)
4. **Fixed Rounds** (Euchre)
5. **Target Achieved** (Blackjack)

#### Proposed Conditions
```yaml
endConditions:
  - type: scoreThreshold
    params:
      threshold: 100
      comparison: "gte"  # or "lte" for Hearts
  - type: zonesComplete
    params:
      zones: [foundation_*]
      requirement: "full"
  - type: roundCount
    params:
      rounds: 5
```

### 6. Card Validation Patterns

#### Common Validations
1. **Sequence Building** (Solitaire, Rummy)
2. **Set Formation** (Rummy, Go Fish)
3. **Suit Following** (Trick games)
4. **Rank Matching** (Crazy Eights)
5. **Trump Rules** (Euchre, Bridge)

#### Proposed Constraint System
```yaml
constraints:
  buildSequence:
    by: [rank, alternatingColor]
    direction: descending
    gapless: true
    
  formMeld:
    types: [set, run]
    minSize: 3
    
  followSuit:
    required: true
    exceptions: [noCardsInSuit]
```

## Proposed SDK Enhancements

### 1. Zone Templates
```yaml
zoneTemplates:
  solitaireTableau:
    columns: 7
    distribution: ascending  # 1, 2, 3... cards
    faceUp: top
    
  trickTaking:
    playerHands: true
    trickZone: true
    
  rummy:
    drawPile: true
    discardPile: true
    playerHands: true
    meldZones: perPlayer
```

### 2. Turn Pattern Library
```yaml
turnPatterns:
  drawDiscardPlay:
    inherit: true
    
  trickTaking:
    leadPlayer: dynamic
    allPlayOne: true
    
  phased:
    phases: [deal, exchange, play, score]
```

### 3. Scoring Engine
```yaml
scoring:
  builtin: combinationScorer
  combinations:
    - pattern: "15"  # Cribbage
      points: 2
    - pattern: "pair"
      points: 2
    - pattern: "run"
      points: length
```

### 4. Meta-Rules System
```yaml
metaRules:
  simultaneousReveal: true  # For games like Blackjack
  hiddenInformation: [dealerCard]
  turnTimer: 30
  allowUndo: false
```

## Implementation Priority

### High Priority (Multiple Games)
1. **Trick-taking mechanics** (Hearts, Euchre, Bridge, Whist)
2. **Build zones** (Solitaire, some patience games)
3. **Meld validation** (Rummy, Gin, Canasta variants)
4. **Score tracking** (Almost all games)
5. **Multi-phase turns** (Many games)

### Medium Priority (2-3 Games)
1. **Pass cards phase** (Hearts, some Rummy variants)
2. **Target scores** (Blackjack, 21 variants)
3. **Trump mechanics** (Euchre, Bridge)
4. **Combination scoring** (Cribbage, Poker)

### Low Priority (Single Game)
1. **Cribbage board** (Only Cribbage)
2. **Betting** (Poker variants)
3. **Stock recycling** (Specific Solitaire rules)

## Parameterization Opportunities

### 1. Dynamic Dealing
```yaml
deal:
  count: 
    formula: "players <= 2 ? 10 : 7"  # Gin Rummy
    perRound: [3, 2, 2]  # Cribbage
```

### 2. Flexible Validation
```yaml
validation:
  cardMatch:
    matchBy: [rank, suit]
    wildcards: [{rank: "8"}]  # Crazy Eights
    
  sequence:
    acePosition: low  # or "high", "both"
    wrapping: false  # K-A-2 allowed?
```

### 3. Conditional Rules
```yaml
rules:
  breakingHearts:
    condition: "heartPlayed || onlyHearts"
    
  mustFollowSuit:
    condition: "hasSuit(leadSuit)"
    exceptions: ["firstTrick && hasOnlyHearts"]
```

## Recommended SDK Architecture Changes

### 1. Introduce Game Categories
```yaml
gameType: trickTaking  # or "shedding", "collecting", "building"
# This would auto-include relevant patterns
```

### 2. Verb Composition
```yaml
verbs:
  - id: playerTurn
    compose:
      - draw
      - optionalMeld
      - mandatoryDiscard
      - checkWin
```

### 3. State Machines for Complex Flows
```yaml
gameMachine:
  states:
    dealing:
      on: 
        DEALT: playing
    playing:
      on:
        KNOCK: scoring
        GIN: gameEnd
    scoring:
      on:
        SCORED: dealing
```

### 4. Built-in AI Patterns
```yaml
ai:
  difficulty: medium
  strategies:
    - avoidHighCards  # Hearts
    - collectSets     # Rummy
    - optimizeMelds   # Gin
```
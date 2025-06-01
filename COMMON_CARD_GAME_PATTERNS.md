# Common Card Game Patterns

After implementing Crazy Eights and Rummy with the SDK shorthand syntax, several common patterns emerge that could be further abstracted.

## 1. Core Components (Already Implemented)

### Standard Deck
- **Pattern**: `type: standardDeck`
- **Usage**: Both games use a standard 52-card deck
- **Benefit**: Eliminates 400+ lines of card definitions

### Deal Cards to Players
- **Pattern**: `uses: cards.deal` with `to: eachPlayer`
- **Usage**: Initial card distribution in setup phase
- **Variations**: 
  - Crazy Eights: 5 cards per player
  - Rummy: 10 cards (2 players) or 7 cards (3-4 players)

### Reveal/Flip Cards
- **Pattern**: `uses: cards.reveal`
- **Usage**: Flipping first card to discard pile
- **Common**: Both games start with one face-up card

## 2. Zone Patterns

### Common Zones
Every card game typically has:
1. **Draw Pile** (deck, face-down)
2. **Discard Pile** (face-up)
3. **Player Hands** (private to each player)

### Zone Properties
```yaml
# Draw pile pattern
- id: drawPile
  shape: deck
  deckProps:
    shuffle: true
    faceDown: true
  visibility: count
  contents: standardDeck

# Discard pile pattern  
- id: discardPile
  shape: deck
  deckProps:
    faceDown: false
    revealTop: 1  # or "all" for games like Rummy
  visibility: all

# Player hand pattern
- id: hand_{player}
  shape: list
  visibility: owner
  ui:
    position: hand
    layout: fan
```

## 3. Turn Structure Patterns

### Draw-Discard Pattern
Common in many card games:
1. Draw a card (from deck or discard)
2. Perform actions (play cards, meld, etc.)
3. Discard a card
4. Advance turn

### Turn Gating
- **Pattern**: Enable/disable actions based on turn phase
- **Example**: In Rummy, discard is only enabled after drawing

## 4. Win Conditions

### Empty Hand
- **Pattern**: `uses: zone.isEmpty` condition
- **Games**: Crazy Eights, Rummy (with variations)

### Point Threshold
- **Pattern**: Score calculation followed by threshold check
- **Games**: Rummy variants, Canasta

## 5. Proposed Additional Shorthands

### Zone Templates
```yaml
zones:
  templates:
    - standardCardGame:
        drawPile: true
        discardPile: true
        playerHands: true
```

### Turn Templates
```yaml
turns:
  pattern: drawDiscardPlay
  options:
    drawFrom: [drawPile, discardPile]
    mustDiscard: true
```

### Dynamic Dealing
```yaml
- id: dealCards
  uses: cards.deal
  with:
    count:
      conditional:
        - if: players == 2
          then: 10
        - else: 7
```

### Meld Validation
```yaml
constraints:
  meld:
    minSize: 3
    types: [set, run]  # set: same rank, run: consecutive same suit
```

## 6. Common Action Patterns

### Draw Cards
```yaml
- id: draw
  uses: entity.move
  with:
    source: drawPile
    target: hand_{actor}
    count: 1
```

### Play/Discard Cards
```yaml
- id: playCard
  uses: entity.move
  with:
    source: hand_{actor}
    target: discardPile
```

### Check Win Condition
```yaml
- id: checkWin
  auto: true
  conditions:
    - uses: zone.isEmpty
      with:
        zone: hand_{actor}
```

## 7. State Management Patterns

### Phase Transitions
- Setup → Play is universal
- Some games add: Play → Scoring → End

### Turn Order
- Simple rotation: most card games
- Complex: based on played cards (Uno reverse)

## 8. Recommendations for SDK

1. **Card Game Template**: Pre-configured zones and basic actions
2. **Conditional Dealing**: Built-in support for player-count-based dealing
3. **Meld/Set Detection**: Built-in validators for common patterns
4. **Score Tracking**: Standardized scoring zones and calculations
5. **Turn Patterns**: Pre-built draw-play-discard flows
6. **Card Visibility**: Shortcuts for "show top N cards" patterns

## 9. Code Reduction Impact

Using current shorthands:
- **Crazy Eights**: 680 → 163 lines (76% reduction)
- **Rummy**: ~200 lines (estimated 80% reduction from verbose form)

With proposed enhancements:
- Could reduce to ~50-75 lines per game
- Most logic would be in custom validation hooks

## 10. Hook Requirements

Both games revealed needs for custom hooks:
- **Crazy Eights**: `matchingCard` validation (rank or suit match, 8s wild)
- **Rummy**: `validMeld`, `validLayoff`, `calculateRummyScores`

These could become parameterized actions:
```yaml
constraints:
  cardMatch:
    matchBy: [rank, suit]
    wild: "8"
```
# SDK Improvements Implemented

Based on analysis of the top 10 card games, the following SDK improvements have been implemented to benefit all games.

## 1. Scoring System

### Actions Added:
- `score.update` - Add, subtract, or set player scores
- `score.check` - Check if score meets threshold with flexible comparisons

### Usage Example:
```yaml
# Update score
- id: scorePoints
  uses: score.update
  with:
    player: "{actor}"  # or specific player like "p1"
    amount: 10
    operation: add  # or "subtract", "set"

# Check win condition
- id: checkForWin
  uses: score.check
  auto: true
  with:
    player: "{actor}"
    threshold: 100
    comparison: gte  # gte, gt, lte, lt, eq
    result:
      gameWin: player  # or gameLose: player
```

## 2. Phase System (Already Implemented)

### Features:
- Automatic phase transitions
- Phase-specific actions
- Initial phase triggers

### Usage Example:
```yaml
# manifest.yaml
phases:
  - id: setup
    name: "Setup"
    auto: true
    then:
      - dealCards
  - id: play
    name: "Play"

setup:
  initialPhase: setup

# actions.yaml
- id: dealCards
  uses: cards.deal
  phase: setup  # Only available in setup phase
  auto: true
  with:
    count: 7
    to: eachPlayer
    from: drawPile
```

## 3. Enhanced Card Validation

### Constraint System:
The `moveEntity` builtin now supports card constraint validation

### Current Implementation:
```yaml
- id: playCard
  uses: entity.move
  phase: play
  with:
    source: hand_{actor}
    target:
      zone: discardPile
      constraints:
        matchingCard: true  # Validates rank/suit match, 8s wild
```

### Extensible for Future Games:
```yaml
constraints:
  matchBy: [rank, suit, color]
  wildcards: ["8", "Joker"]
  sequence: ascending
  buildBy: alternatingColor
```

## 4. Zone Empty Condition

### Action Added:
- `zone.isEmpty` - Check if a zone has no cards/entities

### Usage Example:
```yaml
- id: checkWin
  auto: true
  conditions:
    - uses: zone.isEmpty
      with:
        zone: hand_{actor}
      result:
        - gameWin: actor
```

## 5. Card Counting

### Action Added:
- `cards.count` - Count cards in a zone

### Usage Example:
```yaml
- id: countHandSize
  uses: cards.count
  with:
    zone: hand_{actor}
    storeAs: handSize  # Optional: store in temp variable
```

## 6. Flexible Result Handling

### Conditions now support both formats:
```yaml
# Array format
result:
  - gameWin: actor
  - trigger: someAction

# String format
result: gameWin
```

## 7. Standard Deck Shorthand (Already Implemented)

### Features:
- `type: standardDeck` - Generates 52 cards
- `contents: standardDeck` - Fills zone with card IDs
- `uses: cards.deal` - Deals to all players
- `uses: cards.reveal` - Flips cards

## Benefits for Existing Games

### Tic-Tac-Toe
- Can now track scores across multiple rounds
- Phase system can handle match setup

### Connect 4
- Score tracking for tournament play
- Phase transitions for setup/play/scoring

### Checkers
- Count pieces for scoring
- Phase system for different game stages

### Reversi
- Track scores (piece count)
- End game when no moves available

### All Games
- Consistent phase management
- Standardized win/lose conditions
- Score persistence across rounds
- Better constraint validation

## Example: Implementing Hearts with New Features

```yaml
# manifest.yaml
phases:
  - id: passing
    name: "Pass Cards"
    then:
      - passCards
  - id: playing
    name: "Play Tricks"
  - id: scoring
    name: "Score Round"

# actions.yaml
- id: playToTrick
  uses: entity.move
  phase: playing
  with:
    source: hand_{actor}
    target: trick
    constraints:
      followSuit: true
      
- id: scoreHearts
  uses: score.update
  phase: scoring
  auto: true
  with:
    player: "{trickWinner}"
    amount: "{heartCount}"
    operation: add
    
- id: checkGameEnd
  uses: score.check
  auto: true
  with:
    player: "{anyPlayer}"
    threshold: 100
    comparison: gte
    result:
      gameLose: player
```

## Migration Guide

### For Existing Games:
1. No breaking changes - all improvements are additive
2. Can gradually adopt new features
3. Consistent modern syntax

### Best Practices:
1. Use phases for complex turn structures
2. Implement scoring for competitive games
3. Use constraints for card validation
4. Leverage conditions for win states
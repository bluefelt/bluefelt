# SDK Card Game Shorthands

This document proposes convenience methods for common card game patterns in the Bluefelt SDK.

## Problem

Currently, defining a standard 52-card deck requires ~450 lines of YAML to define each individual card entity. Similarly, dealing cards requires verbose action definitions for each card dealt to each player.

## Proposed Solution

### 1. Standard Deck Generation

#### Option A: Entity-level shorthand
```yaml
# entities.yaml
- type: standardDeck
  id: deck
  options:
    includeJokers: false  # optional, default: false
    cardType: playing_card  # optional, default: playing_card
```

This would automatically generate 52 card entities with IDs like:
- `card_hearts_a`, `card_hearts_2`, ... `card_hearts_k`
- `card_diamonds_a`, `card_diamonds_2`, ... `card_diamonds_k`
- `card_clubs_a`, `card_clubs_2`, ... `card_clubs_k`
- `card_spades_a`, `card_spades_2`, ... `card_spades_k`

Each card would have properties:
```yaml
type: card
props:
  suit: hearts|diamonds|clubs|spades
  rank: A|2-10|J|Q|K
  value: 1-13
ui:
  cardType: playing_card
```

#### Option B: Zone-level shorthand
```yaml
# zones.yaml
- id: drawPile
  name: "Draw Pile"
  shape: deck
  contents: standardDeck  # Auto-generates and places 52 cards
  deckProps:
    shuffle: true
    faceDown: true
```

### 2. Dealing Cards Shorthand

Replace verbose individual deal actions with:

```yaml
# actions.yaml
- id: dealCards
  uses: cards.deal
  phase: setup
  auto: true
  with:
    count: 5  # number of cards per player
    to: eachPlayer  # special keyword
    from: drawPile
```

This would expand internally to deal the specified number of cards to each player's hand zone.

### 3. Dynamic Card Counts

Support conditional dealing based on player count:

```yaml
- id: dealCards
  uses: cards.deal
  phase: setup
  auto: true
  with:
    count: 
      if: players == 2
      then: 7
      else: 5
    to: eachPlayer
    from: drawPile
```

### 4. Common Card Operations

#### Reveal/Flip Cards
```yaml
- id: revealTopCard
  uses: cards.reveal
  with:
    from: drawPile
    to: discardPile
    count: 1
```

#### Draw Multiple Cards
```yaml
- id: drawPenalty
  uses: cards.draw
  with:
    count: 2
    to: hand_{actor}
    from: drawPile
```

### 5. Enhanced Setup Section

For even more concise game definitions:

```yaml
# manifest.yaml
setup:
  deck: standard  # Creates standard deck in drawPile zone
  shuffle: true
  deal:
    count: 5
    to: eachPlayer
  reveal: 1  # Flips one card to discardPile
```

## Benefits

1. **Reduced Verbosity**: A standard card game setup goes from ~500 lines to ~20 lines
2. **Less Error-Prone**: No risk of typos in 52 card definitions
3. **Faster Development**: Common patterns become one-liners
4. **Maintainability**: Changes to card structure only need updates in one place
5. **Readability**: Game logic is clearer without boilerplate

## Implementation Notes

- The SDK would need to recognize these special keywords during game parsing
- Entity generation would happen before game initialization
- Generated entities would be indistinguishable from manually defined ones at runtime
- This approach could extend to other common game components (dice, tokens, etc.)

## Example: Crazy Eights with Shorthands

Before: ~680 lines
After: ~163 lines

## Implementation Status

✅ **Implemented in Server** (server/src/shorthand.rs):
- `type: standardDeck` - Generates 52 cards automatically
- `contents: standardDeck` - Fills zone with all card IDs
- `uses: cards.deal` with `to: eachPlayer` - Deals to all players
- `uses: cards.reveal` - Simplified card reveal

## Real-World Usage

Two games have been implemented using these shorthands:
1. **Crazy Eights** (`games/crazy-eights/1.0/`)
2. **Rummy** (`games/rummy/1.0/`)

See [COMMON_CARD_GAME_PATTERNS.md](./COMMON_CARD_GAME_PATTERNS.md) for patterns discovered during implementation.

```yaml
# entities.yaml
- type: standardDeck
  id: deck

# zones.yaml
- id: drawPile
  name: "Draw Pile"
  shape: deck
  contents: standardDeck
  deckProps:
    shuffle: true
    faceDown: true
  visibility: count
  ui:
    position: board
    layout: stack

- id: discardPile
  name: "Discard Pile"
  shape: deck
  visibility: all
  ui:
    position: board
    layout: stack

# Player hands...
- id: hand_p1
  name: "{p1}'s Hand"
  shape: list
  visibility: owner
  ui:
    position: hand
    layout: fan

# actions.yaml
- id: setup
  uses: cards.deal
  phase: setup
  auto: true
  with:
    count: 5
    to: eachPlayer
    from: drawPile
  then:
    - revealFirstCard

- id: revealFirstCard
  uses: cards.reveal
  auto: true
  with:
    from: drawPile
    to: discardPile
    count: 1
  then:
    - startGame
```
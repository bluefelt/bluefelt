# Card Game Common Patterns Analysis

## Overview
After analyzing 10 card games (including Go Fish), several common patterns emerge that suggest opportunities for reusable built-in functionality.

## Common Patterns Identified

### 1. Trick-Taking Mechanics
**Games**: Hearts, Oh Hell
**Pattern**: 
- Players play one card each in turn
- Must follow suit if possible
- Highest card of led suit wins (unless trumped)
- Winner leads next trick

**Suggested Verb**: `playTrick`
```yaml
uses: playTrick
with:
  leadPlayer: "{currentTrickLeader}"
  followSuitRequired: true
  trumpSuit: "{trumpSuit}"  # optional
```

### 2. Meld Formation
**Games**: Gin Rummy, Contract Rummy
**Pattern**:
- Sets: Multiple cards of same rank
- Runs/Sequences: Consecutive cards of same suit
- Validation of complete melds

**Suggested Verbs**: 
- `validateMeld`: Check if cards form valid meld
- `layDownMelds`: Place melds on table
- `addToMeld`: Add cards to existing melds

### 3. Card Matching/Shedding
**Games**: Crazy Eights, Old Maid, Cheat
**Pattern**:
- Match by rank or suit
- Remove pairs/matches from hand
- Play cards to central pile

**Suggested Verbs**:
- `matchCard`: Check if card matches criteria
- `removePairs`: Auto-remove pairs from hand
- `shedCard`: Play card to discard with validation

### 4. Bidding/Prediction
**Games**: Oh Hell, GOPS
**Pattern**:
- Players make predictions/bids
- Simultaneous or sequential bidding
- Bid validation and constraints

**Suggested Verb**: `makeBid`
```yaml
uses: makeBid
with:
  bidType: "tricks"  # or "cards", "points"
  constraints: "{hookRule}"  # optional
  simultaneous: false
```

### 5. Drawing and Dealing Patterns
**Common to all games**
**Patterns**:
- Deal specific number to each player
- Draw from deck or discard
- Automatic reshuffling when deck empty

**Enhanced Verbs**:
- Extend `draw` with auto-reshuffle option
- Add `dealRound` for complex dealing patterns
- Add `drawOrDiscard` for Rummy-style games

### 6. Scoring Systems
**Multiple patterns across games**
**Types**:
- Penalty points (Hearts)
- Exact bid bonuses (Oh Hell)
- Card values (Gin Rummy)
- Round-based accumulation

**Suggested Verb**: `calculateScore`
```yaml
uses: calculateScore
with:
  method: "penalty"  # or "exact-bid", "card-value", "meld-value"
  params: {...}      # game-specific parameters
```

### 7. Turn Order Variations
**Patterns**:
- Clockwise progression
- Winner of last trick leads
- Based on cards played

**Enhancement**: Add to `nextTurn`
```yaml
uses: nextTurn
with:
  order: "winner-leads"  # or "clockwise", "counterclockwise"
  basedOn: "lastTrick"   # optional context
```

### 8. Challenge/Doubt Mechanics
**Games**: Cheat, (potentially Gin Rummy knocking)
**Pattern**:
- Player makes claim
- Others can challenge
- Resolution reveals truth

**Suggested Verb**: `makeChallenge`
```yaml
uses: makeChallenge
with:
  target: "{lastAction}"
  penalty: "takeDiscardPile"
```

## Recommended Implementations

### Priority 1 - High Reuse Value
1. **Enhanced draw mechanics** - Every game needs this
2. **Trick-taking system** - Complete subsystem for multiple games
3. **Meld validation** - Complex logic worth centralizing
4. **Match checking** - Simple but very common

### Priority 2 - Medium Reuse Value
5. **Bidding system** - Several games use this
6. **Scoring calculator** - Reduce duplicate logic
7. **Turn order variants** - More flexibility

### Priority 3 - Game-Specific
8. **Challenge system** - Only a few games
9. **Special card effects** - Varies too much

## Implementation Notes

### State Management Needs
- `lastTrick`: For trick-taking games
- `currentBids`: For bidding games
- `melds`: For rummy games
- `claimedRank`: For bluffing games

### UI Enhancements Needed
- Simultaneous card selection (GOPS, passing in Hearts)
- Hidden bid entry (Oh Hell)
- Meld arrangement interface (Rummy games)
- Challenge button (Cheat)

### Complex Logic Requiring Hooks
- Shooting the Moon detection (Hearts)
- Contract validation (Contract Rummy)
- Gin/Big Gin detection (Gin Rummy)
- War resolution (War)

## Next Steps
1. Implement high-priority verbs before creating games
2. Design consistent state structure for similar games
3. Create reusable UI components for common patterns
4. Build games using these new primitives
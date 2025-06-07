# Exploding Kittens - Bluefelt Implementation Analysis

## 1. Core Rules Summary

**Objective**: Be the last player alive by avoiding Exploding Kitten cards or defusing them when drawn.

**Key Mechanics**:
- Players take turns playing cards then drawing from deck
- Drawing an Exploding Kitten eliminates you unless you have a Defuse card
- Defuse cards let you secretly place the Exploding Kitten back in the deck
- Various action cards provide strategic options (Skip, Attack, See Future, Shuffle, Nope)
- Cat cards collected in pairs allow stealing from opponents
- No hand size limits

**Turn Structure**:
1. Play 0 or more cards (optional)
2. Draw 1 card (mandatory, ends turn)
3. If Exploding Kitten drawn: must Defuse or be eliminated

## 2. Mapping to Bluefelt Architecture

### Entities
```yaml
entities:
  # Card types
  - id: exploding_kitten
    name: "Exploding Kitten"
    type: card
    
  - id: defuse
    name: "Defuse"
    type: card
    
  - id: skip
    name: "Skip" 
    type: card
    
  - id: attack
    name: "Attack"
    type: card
    
  - id: see_future
    name: "See the Future"
    type: card
    
  - id: shuffle
    name: "Shuffle"
    type: card
    
  - id: nope
    name: "Nope"
    type: card
    
  # Cat cards (various types for pairing)
  - id: cat_taco
    name: "Tacocat"
    type: card
    
  - id: cat_beard
    name: "Beard Cat"
    type: card
    
  - id: cat_rainbow
    name: "Rainbow Cat"
    type: card
    
  # Player status markers
  - id: player_alive_{player}
    name: "{player} Status"
    type: marker
```

### Zones
```yaml
zones:
  - id: draw_pile
    type: deck
    visibility: hidden
    
  - id: discard_pile
    type: deck
    visibility: visible
    
  - id: hand_{player}
    type: list
    visibility: owner  # Only owner can see
    
  - id: player_status
    type: list
    visibility: visible  # Track who's alive/eliminated
    
  - id: peek_zone
    type: list
    visibility: owner  # For "See the Future" card
```

### Actions
```yaml
actions:
  - id: play_card
    conditions:
      - isActor: true
      - phase.is: player_turn
      - entity.owner: {player}
    
  - id: draw_card
    conditions:
      - isActor: true
      - phase.is: player_turn
      - custom: "hasNotDrawnThisTurn"
    
  - id: defuse_kitten
    conditions:
      - custom: "drewExplodingKitten"
      - custom: "hasDefuseCard"
    
  - id: insert_kitten
    conditions:
      - phase.is: defusing
      - isActor: true
    
  - id: steal_card
    conditions:
      - custom: "playedMatchingPair"
      - isActor: true
    
  - id: nope_action
    conditions:
      - custom: "actionInProgress"
      - custom: "hasNopeCard"
```

### Phases
```yaml
phases:
  - id: setup
  - id: player_turn
  - id: defusing  # When inserting Exploding Kitten back
  - id: peeking   # See the Future active
  - id: stealing  # Choosing card to steal
  - id: game_over
```

## 3. Engine Capabilities Analysis

### What the Engine CAN Do:
✅ **Turn-based gameplay** - Core requirement met
✅ **Deck zones with shuffling** - Draw pile management
✅ **Hidden/visible zones** - Hand visibility, deck hiding
✅ **Draw verb** - Moving cards from deck to hand
✅ **Phase transitions** - Managing game states
✅ **Player elimination tracking** - Via markers in status zone
✅ **Basic conditions** - isActor, phase.is, entity.owner
✅ **Triggered actions** - Can chain card effects

### What the Engine CANNOT Currently Do:

#### 🚫 **Instant Response Actions (Nope Cards)**
- **Gap**: No support for interrupting/canceling actions in progress
- **Need**: Reaction window system for playing Nope cards
- **Pattern**: Similar to counterspells in Magic or instant-speed responses

#### 🚫 **Secret Placement/Insertion**
- **Gap**: No mechanism for secretly placing cards at specific deck positions
- **Need**: Private deck manipulation with position selection
- **Pattern**: Unique to Exploding Kittens' Defuse mechanic

#### 🚫 **Conditional Draw Results**
- **Gap**: Cannot trigger different outcomes based on what card was drawn
- **Need**: Draw verb that can branch based on card type
- **Pattern**: Drawing = potential elimination is core to game

#### 🚫 **Peeking at Deck (See the Future)**
- **Gap**: No way to view top X cards of deck privately
- **Need**: Temporary visibility of deck subset
- **Pattern**: Information advantage cards common in many games

#### 🚫 **Force Multiple Turns (Attack Card)**
- **Gap**: nextTurn always advances to next player
- **Need**: Ability to give same player multiple consecutive turns
- **Pattern**: "Take another turn" effects in various games

#### 🚫 **Variable Card Effects**
- **Gap**: Actions are static, can't have different effects per card type
- **Need**: Card-specific action definitions or effect mapping
- **Pattern**: Each card type has unique effect

#### 🚫 **Pair Detection for Cat Cards**
- **Gap**: No built-in matching/set collection detection
- **Need**: Condition to check for pairs in hand
- **Pattern**: Set collection mechanic (rummy-style)

#### 🚫 **Random Target Selection (Stealing)**
- **Gap**: No way to randomly select from opponent's hand
- **Need**: Random selection from hidden zones
- **Pattern**: Blind stealing/drawing from opponents

## 4. Reusable Patterns & Commonalities

### Patterns Seen in Other Games:
1. **Deck Management** - Similar to Gin Rummy, Go Fish
2. **Hidden Information** - Like all card games implemented
3. **Player Elimination** - New pattern, but could apply to elimination tournaments
4. **Set Collection** - Similar to Gin Rummy's melds, Go Fish's books

### Novel Patterns in Exploding Kittens:
1. **Russian Roulette Mechanic** - Draw-to-eliminate is unique
2. **Defuse & Reinsert** - Secret deck manipulation unprecedented
3. **Instant Responses** - Nope cards require new interrupt system
4. **Forced Multi-turns** - Attack cards break normal turn flow

### Engine Extensions Needed (Priority Order):
1. **Conditional Draw Handling** - Essential for core mechanic
2. **Interrupt/Response System** - For Nope cards
3. **Secret Deck Manipulation** - For Defuse placement
4. **Deck Peeking** - For See the Future
5. **Multi-turn Assignment** - For Attack cards
6. **Pair Detection** - For Cat card combos
7. **Random Selection** - For stealing

## 5. Implementation Feasibility

**Current Status**: NOT IMPLEMENTABLE without significant engine extensions

**Critical Blockers**:
1. No way to handle Exploding Kitten draws (core mechanic)
2. No interrupt system for Nope cards
3. No secret deck insertion for Defuse

**Partial Implementation Possible**:
- Could implement simplified version without Nope cards
- Could use markers/phases to track some states
- Basic card play and turn structure feasible

**Recommendation**: This game exposes several gaps in the engine that would benefit many card games if addressed, particularly:
- Conditional draw handling
- Interrupt/response windows  
- Secret zone manipulation
- Information peeking mechanics
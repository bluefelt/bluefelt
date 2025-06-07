# Sushi Go Analysis for Bluefelt Implementation

## 1. Core Rules Summary

**Game Overview:**
- 2-5 players
- 3 rounds of card drafting
- Players simultaneously pick cards and pass hands
- Score points by collecting sets of sushi cards
- Highest total score after 3 rounds wins

**Key Mechanics:**
- **Simultaneous card selection**: All players pick a card at the same time
- **Hand passing**: Pass remaining cards to adjacent player (left in rounds 1&3, right in round 2)
- **Set collection**: Score based on combinations (tempura pairs, sashimi triples, etc.)
- **Round scoring**: Most cards score at end of round, pudding scores at game end
- **Special actions**: Chopsticks allow taking 2 cards in one turn

## 2. Mapping to Bluefelt Architecture

### Entities
```yaml
# Card entities would include:
- nigiri_egg (1 point)
- nigiri_salmon (2 points)  
- nigiri_squid (3 points)
- wasabi (multiplier for nigiri)
- tempura (5 points per pair)
- sashimi (10 points per triple)
- maki_1, maki_2, maki_3 (position-based scoring)
- dumpling (escalating points)
- pudding (end-game scoring)
- chopsticks (action card)
```

### Zones
```yaml
# Required zones:
- deck (face-down draw pile)
- player_hand_p1, player_hand_p2, etc. (private hands)
- played_p1, played_p2, etc. (public played cards)
- selected_p1, selected_p2, etc. (temporarily hidden selections)
- pudding_p1, pudding_p2, etc. (persists between rounds)
```

### Actions
```yaml
# Core actions needed:
- deal_cards (setup)
- select_card (pick from hand)
- reveal_selections (simultaneous reveal)
- pass_hand (rotate hands)
- use_chopsticks (take 2 cards)
- score_round
- score_game
```

### Phases
```yaml
# Game flow:
- setup
- selection (all players pick)
- reveal (show selections)
- pass (rotate hands)
- score_round (after all cards played)
- next_round (rounds 2-3)
- final_scoring (pudding + winner)
```

## 3. Engine Capabilities Analysis

### ✅ What the Engine CAN Do:
- Turn-based structure (though Sushi Go needs simultaneous)
- Multiple zones for cards
- Basic move/place actions
- Phase transitions
- Triggered actions (for scoring)
- Count-based conditions
- Entity ownership tracking

### ❌ What the Engine CANNOT Do:

#### 1. **Simultaneous Actions** (CRITICAL GAP)
- Engine is strictly turn-based
- Sushi Go requires all players to select cards simultaneously
- No support for hidden simultaneous selection followed by reveal

#### 2. **Hand Rotation/Passing** (MAJOR GAP)
- No verb to pass entire zone contents between players
- Would need custom logic to transfer all cards from one player's hand to another's
- Direction changes by round (left→right→left)

#### 3. **Complex Scoring Logic** (SIGNIFICANT GAP)
- **Position-based scoring**: Maki rolls award points to 1st/2nd place
- **Set collection**: Tempura (pairs), Sashimi (triples)
- **Escalating values**: Dumplings worth more as you collect more
- **Negative scoring**: Least pudding loses points
- **Card interactions**: Wasabi multiplies next nigiri played

#### 4. **Persistent Cross-Round State** (GAP)
- Pudding cards persist between rounds while others are discarded
- No clear mechanism for selective persistence

#### 5. **Dynamic Card Effects** (GAP)
- Chopsticks allows taking 2 cards instead of 1
- Requires modifying action constraints mid-game

#### 6. **Private Information Management** (PARTIAL GAP)
- Hands are private but selections need to be hidden until reveal
- Engine has zones but needs better visibility controls

## 4. Reusable Patterns

### From Other Games:
- **Card zones**: Similar to Gin Rummy's hand management
- **Set collection**: Like Gin Rummy's melds but simpler
- **Multi-round structure**: Similar to trick-taking games
- **Scoring combinations**: Pattern matching like Poker hands

### New Patterns Sushi Go Would Introduce:
1. **Simultaneous action resolution**
2. **Zone rotation between players**
3. **Position-based scoring (1st, 2nd, etc.)**
4. **Cross-round persistence**
5. **Dynamic action modification**

## 5. Implementation Recommendations

### Engine Extensions Needed:
1. **Simultaneous Actions**
   - New phase type: `simultaneous_action`
   - Hidden selection + reveal mechanism
   - All players must act before proceeding

2. **Zone Rotation Verb**
   ```yaml
   - verb: rotateZones
     zones: [player_hand_p1, player_hand_p2, ...]
     direction: left/right
   ```

3. **Advanced Scoring Verbs**
   ```yaml
   - verb: scoreByPosition
     zone: played_{player}
     entity: maki_*
     positions: [{rank: 1, points: 6}, {rank: 2, points: 3}]
   
   - verb: scoreBySet
     zone: played_{player}
     entity: tempura
     setSize: 2
     pointsPerSet: 5
   ```

4. **Conditional Persistence**
   ```yaml
   - verb: moveSelectively
     from: played_{player}
     to: discard
     except: pudding
   ```

### Workaround Strategies:
- **For simultaneous play**: Could implement as rapid turns with UI hiding selections
- **For hand passing**: Chain of moveEntity actions (inefficient but possible)
- **For scoring**: Custom WASM hooks for complex logic
- **For persistence**: Separate pudding zones that don't clear

## 6. Complexity Assessment

**Implementation Difficulty: HIGH**

Sushi Go pushes beyond current engine capabilities in several fundamental ways:
- Simultaneous play is core to the game experience
- Scoring requires positional comparisons across players
- Hand rotation is essential mechanic
- Multiple interacting scoring rules

**Recommendation**: Sushi Go would require significant engine enhancements to implement properly. The simultaneous action system alone would be a major addition. Consider implementing simpler card games first to build toward these capabilities incrementally.
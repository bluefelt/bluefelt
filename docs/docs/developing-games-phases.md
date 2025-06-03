# Developing Games: Phases

Phases organize your game into distinct states, each with its own available actions and automatic behaviors. Bluefelt uses a state machine approach where multiple phase sets can run independently and simultaneously.

## Overview

Phases in Bluefelt work as independent state machines that:
- Control which actions are available to players at any given time
- Execute automatic actions when entering phases
- Manage game flow without hardcoding transitions
- Support multiple simultaneous phase tracks (game phases, round phases, turn phases, etc.)

Key principles:
- Each phase set operates independently as its own state machine
- Phase transitions are triggered by actions, not defined in the phase itself
- Phases define what happens when you enter them and what actions are possible
- Multiple phase sets can be active simultaneously

## Phase Structure

### Basic Phase Set

```yaml
- id: game
  phases:
    - id: setup
      initial: true
      enterActions:
        - transitionToPhase: game.play
    
    - id: play
      possibleActions: [placeMarker, moveUnit]
      
    - id: end
      ui:
        display: "Game Over"
```

### Real Example: Gin Rummy Phases

```yaml
# Game-level phases
- id: game
  phases:
    - id: setup
      initial: true
      enterActions:
        - transitionToPhase: game.rounds

    - id: rounds
      enterActions:
        - transitionToPhase: round.deal
      ui:
        display: "Game {iteration}"

    - id: endScoring
      enterActions:
        - calculateFinalScores
      ui:
        display: "Calculating final scores..."

    - id: end
      ui:
        display: "Game Over"

# Round-level phases
- id: round
  phases:
    - id: null
      initial: true

    - id: deal
      enterActions:
        - dealCards
        - revealFirstDiscardPileCard
        - transitionToPhase: round.play
      ui:
        display: "Dealing..."

    - id: play
      enterActions:
        - transitionToPhase: turn.player
    
    - id: scoring
      enterActions:
        - calculateScores
      ui:
        display: "Scoring Round {iteration}"
    
    - id: checkEnd
      enterActions:
        - checkGameEnd
      ui:
        display: "Checking scores..."

# Turn phases
- id: turn
  phases:
    - id: null
      initial: true
    - id: player
      enterActions:
        - transitionToPhase: playerTurn.draw
      ui:
          display: "{actor}'s Turn"

# Player turn sub-phases
- id: playerTurn
  phases:
    - id: null
      initial: true
    - id: draw
      possibleActions: [drawFromDeck, drawFromDiscard]
      ui:
        prompt: "Draw a card from the deck or discard pile"
    - id: discard
      possibleActions: [discard, knock]
      ui:
        prompt: "Discard a card to end your turn (or knock if you have 10 or fewer deadwood points)"
```

## Core Properties

### Phase Set Structure

Each phase set has:
- `id` - Unique identifier for the phase set
- `phases` - Array of phases within this set

### Phase Properties

Each phase can have:

#### `id` (string, required)
Unique identifier within the phase set. Can be `null` for placeholder/initial phases.

#### `initial` (boolean, optional)
Marks the starting phase for this phase set. Only one phase per set should have this.

#### `enterActions` (array, optional)
Actions that execute automatically when entering this phase. Common uses:
- Transition to another phase: `transitionToPhase: phaseSet.phaseId`
- Execute game actions: `dealCards`, `calculateScores`, etc.

```yaml
enterActions:
  - dealCards
  - revealFirstDiscardPileCard
  - transitionToPhase: round.play
```

#### `possibleActions` (array, optional)
Actions that players can perform during this phase. Only these actions will be available.

```yaml
possibleActions: [drawFromDeck, drawFromDiscard]
```

#### `ui` (object, optional)
Display properties for the phase:
- `display` - Text shown during the phase (supports placeholders like `{actor}`, `{iteration}`)
- `prompt` - Instruction text for players

```yaml
ui:
  display: "Round {iteration}"
  prompt: "Draw a card from the deck or discard pile"
```

## How Phases Work

### Phase Transitions

Phase transitions are NOT defined in the phase itself. Instead, they are triggered by actions using `transitionToPhase`:

```yaml
# In actions.yaml
- id: drawFromDeck
  uses: entity.move
  with:
    source: drawPile
    target: hand_{actor}
  then:
    - transitionToPhase: playerTurn.discard  # Move to discard phase

# Special case: transitionToPhase as a standalone action
- id: discard
  uses: entity.move
  with:
    source: hand_{actor}
    target: discardPile
  then:
    - action: advanceTurn
    - action: transitionToPhase
      to: turn.player  # Alternative syntax
```

### Multiple Simultaneous Phase Sets

Games can have multiple phase sets active at once. For example, gin rummy has:
- `game` phases (setup, rounds, endScoring, end)
- `round` phases (null, deal, play, scoring, checkEnd)
- `turn` phases (null, player)
- `playerTurn` phases (null, draw, discard)

Each operates independently and can be in different states simultaneously.

### The "null" Phase

Many phase sets start with a `null` phase as a placeholder:

```yaml
- id: round
  phases:
    - id: null
      initial: true  # Starting state
    - id: deal
      # ... actual phase content
```

This allows phase sets to exist without being "active" until needed.

## Common Phase Patterns

### Simple Turn Structure

```yaml
- id: turn
  phases:
    - id: waiting
      initial: true
    
    - id: active
      possibleActions: [move, attack, endTurn]
      ui:
        display: "{actor}'s turn"
```

### Round-Based Games

```yaml
# Separate phase sets for game and rounds
- id: game
  phases:
    - id: setup
      initial: true
      enterActions:
        - setupGame
        - transitionToPhase: round.start

    - id: playing
      # Main game continues here

- id: round
  phases:
    - id: start
      enterActions:
        - dealCards
        - transitionToPhase: round.playerTurns
    
    - id: playerTurns
      # Players take turns
    
    - id: scoring
      enterActions:
        - calculateRoundScores
        - transitionToPhase: round.start  # Next round
```

### Action Selection Phases

```yaml
- id: actionPhase
  phases:
    - id: planning
      possibleActions: [selectAction1, selectAction2, selectAction3]
      ui:
        prompt: "Choose your action for this turn"
    
    - id: resolution
      enterActions:
        - resolveSelectedActions
        - transitionToPhase: actionPhase.planning
```

## Best Practices

### 1. Keep Phases Simple
Each phase should have a single, clear purpose. Use `enterActions` for automatic behaviors and `possibleActions` to limit player choices.

### 2. Use Multiple Phase Sets
Instead of complex branching logic, use separate phase sets that operate independently:
- Game-level phases (setup, playing, scoring, end)
- Round/turn phases (deal, play, resolve)
- Player-specific phases (draw, action, discard)

### 3. Phase Transitions in Actions
Remember that phases don't define where they go next. Actions trigger phase transitions:

```yaml
# In actions.yaml
- id: endRound
  then:
    - transitionToPhase: round.scoring
```

### 4. The Null Phase Pattern
Use `null` as initial phases for sets that aren't immediately active:

```yaml
- id: combat
  phases:
    - id: null
      initial: true
    - id: attackerDeclaration
      # ... combat phases
```

### 5. UI Feedback
Always provide clear feedback about the current phase:

```yaml
ui:
  display: "Round {iteration}"
  prompt: "Select a card to play"
```

## Integration with Actions

Phases and actions work together:

1. **Phases restrict available actions** via `possibleActions`
2. **Actions trigger phase transitions** via `transitionToPhase`
3. **Phases can auto-execute actions** via `enterActions`

This creates a flexible system where game flow emerges from the interaction between phases and actions, rather than being hardcoded.

## Debugging Tips

1. Use the PhaseTracker component (shown in debug mode) to see all active phases
2. Check that phase transitions use the format `phaseSet.phaseId`
3. Ensure all referenced actions in `enterActions` and `possibleActions` exist
4. Remember that multiple phase sets can be active simultaneously

For more examples, examine the phase definitions in existing games like gin-rummy, tic-tac-toe, or stone-age.
# Selection Pattern Design

## Overview

Many board games require a two-phase interaction pattern:
1. **Selection Phase**: Choose a game piece (non-committal)
2. **Action Phase**: Choose what to do with it (committal)

This pattern appears in Chess, Checkers, Three Men's Morris (movement phase), and many other games.

## Current Limitations

The current engine lacks:
1. **Selection State Management**: No built-in way to store which piece is selected between actions
2. **Cancel Mechanism**: No way to cancel a selection without advancing the turn
3. **Conditional UI Affordances**: Action maps can't show different affordances based on selection state

## Proposed Engine Enhancements

### 1. Selection State Management

Add a built-in selection mechanism to the game state:

```yaml
# In state
selection:
  location: "/zones/board/cells/0/1"  # Where the selected piece is
  entity: "piece_p1"                   # What was selected
  actor: "p1"                          # Who made the selection
```

### 2. New Built-in Actions

#### selectEntity
```yaml
- id: selectPiece
  uses: selectEntity
  with:
    zone: "{target}"  # Click location becomes selection
  when:
    - condition: entity.owner
      with:
        entity: "{entityAtLocation}"
        owner: "{player}"
```

This would:
- Store the selection in state
- NOT advance the turn
- Trigger any `then` actions (like phase transitions)

#### moveSelected
```yaml
- id: moveSelectedPiece
  uses: moveSelected
  with:
    to: "{target}"  # Destination
    # 'from' is implicit from selection state
  when:
    - condition: zone.isEmpty
      with:
        zone: "{target}"
```

#### clearSelection
```yaml
- id: cancelSelection
  uses: clearSelection
  # No parameters needed - clears current selection
```

### 3. Enhanced Action Map Generation

The action map should consider selection state:

```javascript
// When no selection
actionMap = {
  "/zones/board/cells/0/0": {
    action: "selectPiece",
    direction: "Select this piece"
  }
}

// When piece is selected
actionMap = {
  "/zones/board/cells/1/1": {
    action: "moveSelectedPiece", 
    direction: "Move here"
  },
  // Special UI element for cancel
  "/ui/cancel": {
    action: "cancelSelection",
    direction: "Cancel selection"
  }
}
```

### 4. UI Enhancements

#### Selection Highlighting
When a piece is selected, the UI should:
- Highlight the selected piece
- Show valid destinations with different affordances
- Display a cancel button/option

#### Phase-Based Prompts
```yaml
phases:
  - id: selection
    ui:
      prompt: "Select a piece to move"
      showCancel: false  # No cancel in selection phase
  
  - id: movement  
    ui:
      prompt: "Move to a highlighted space (or cancel)"
      showCancel: true   # Show cancel option
```

## Implementation Example: Three Men's Morris

```yaml
# phases.yaml
- id: game
  phases:
    - id: placement
      possibleActions: [placeToken]
      ui:
        prompt: "Place a piece"
    
    - id: selection
      possibleActions: [selectPiece]
      ui:
        prompt: "Select a piece to move"
        
    - id: movement
      possibleActions: [moveSelectedPiece, cancelSelection]
      ui:
        prompt: "Move your piece or cancel"
        showCancel: true

# actions.yaml
- id: selectPiece
  uses: selectEntity
  when:
    - condition: player.isActor
    - condition: entity.owner
      with:
        entity: "{entityAtLocation}"
        owner: "{player}"
  then:
    - action: setPhase
      with:
        phaseSet: "game"
        phase: "movement"

- id: moveSelectedPiece
  uses: moveSelected
  with:
    to: "{target}"
  when:
    - condition: zone.isEmpty
      with:
        zone: "{target}"
    - condition: isAdjacent  # New condition type
      with:
        from: "{selection.location}"
        to: "{target}"
  then:
    - action: clearSelection
    - action: checkForWin
    - action: advanceTurn
    - action: setPhase
      with:
        phaseSet: "game"
        phase: "selection"

- id: cancelSelection
  uses: clearSelection
  then:
    - action: setPhase
      with:
        phaseSet: "game"
        phase: "selection"
```

## Benefits

1. **Reusable Pattern**: Many games can use this selection mechanism
2. **Better UX**: Clear visual feedback and ability to change mind
3. **Cleaner Game Definitions**: No need for complex workarounds
4. **Consistent Behavior**: All games handle selection the same way

## Migration Path

1. Add selection state to engine
2. Implement new verbs (selectEntity, moveSelected, clearSelection)
3. Update action map generation to consider selection state
4. Add UI support for cancel actions
5. Document the pattern for game developers

## Alternative Approaches

### Approach 1: Multi-Step Actions
Instead of phases, support actions with multiple steps:

```yaml
- id: movePiece
  steps:
    - type: select
      prompt: "Select a piece"
      condition: entity.owner
    - type: target
      prompt: "Choose destination"
      condition: zone.isEmpty
```

### Approach 2: Stateful Actions
Actions that maintain state across invocations:

```yaml
- id: movePiece
  uses: statefulMove
  states:
    - selecting: "Choose a piece"
    - moving: "Choose destination"
    - complete: "Move complete"
```

## Conclusion

The selection pattern is fundamental to many board games. Adding proper engine support will:
- Simplify game development
- Improve player experience
- Enable more complex games

This enhancement would make Bluefelt suitable for a much wider range of board games while maintaining its declarative, YAML-based approach.
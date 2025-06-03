# Three Men's Morris Implementation

## Overview

Three Men's Morris has been implemented as a demonstration of the Bluefelt engine's enhanced condition system. The game properly enforces placement limits and phase transitions.

## What's Implemented

### 1. Placement Phase ✅
- Players alternate placing pieces on a 3x3 grid
- Each player can place exactly 3 pieces (enforced by `zone.count` condition)
- Automatic phase transition when both players have placed 3 pieces
- Win detection during placement (3 in a row)

### 2. Movement Phase ⚠️
- Phase transition works correctly
- Movement actions are not yet implemented due to engine limitations
- Requires selection pattern (select piece → move to destination)

## Technical Implementation

### Key Features Used

1. **Advanced Conditions**:
   ```yaml
   # Limit placement to 3 pieces per player
   - condition: zone.count
     with:
       zone: "/zones/board"
       entity: "piece_{player}"
       operator: "<"
       value: 3
   ```

2. **Automatic Phase Transitions**:
   ```yaml
   - id: checkPhaseTransition
     uses: setPhase
     auto: true
     when:
       - condition: zone.count
         with:
           zone: "/zones/board"
           entity: "piece_p1"
           operator: "=="
           value: 3
       - condition: zone.count
         with:
           zone: "/zones/board"
           entity: "piece_p2"
           operator: "=="
           value: 3
   ```

3. **Phase-Based Actions**:
   - Actions are restricted to specific phases using `phase.is` condition
   - UI prompts change based on current phase

### Current Limitations

1. **No Movement Implementation**: The movement phase requires:
   - Selection state (which piece to move)
   - Two-phase interaction (select then move)
   - Cancel mechanism
   - Adjacency validation

2. **No Draw Detection**: The game doesn't detect when no moves are possible

## Testing

The implementation includes comprehensive tests:
- `three_mens_morris_test.rs` - Basic game setup and win conditions
- `three_mens_morris_placement_test.rs` - Placement limits and phase transitions

All tests pass, confirming:
- ✅ Players cannot place more than 3 pieces
- ✅ Phase transitions correctly after 6 pieces placed
- ✅ Win detection works during placement
- ✅ Conditions properly restrict actions

## How to Play (Current Version)

1. **Start Game**: Two players join the lobby
2. **Placement Phase**: 
   - Players take turns clicking empty intersections
   - Each player places exactly 3 pieces
   - If someone gets 3 in a row, they win
3. **Movement Phase**: 
   - Currently displays a message about awaiting engine support
   - Movement requires the selection pattern enhancement

## Future Enhancements

To complete Three Men's Morris, the engine needs:

1. **Selection Pattern Support**:
   ```yaml
   - id: selectPiece
     uses: selectEntity
     when:
       - condition: entity.owner
         with:
           entity: "{entityAtLocation}"
           owner: "{player}"
   
   - id: moveSelectedPiece
     uses: moveSelected
     when:
       - condition: adjacentTo
         with:
           from: "{selection.location}"
           to: "{target}"
   ```

2. **Adjacency Validation**: 
   - New condition to check if moves are to adjacent spaces only
   
3. **Draw Detection**:
   - Check if current player has any valid moves

## Conclusion

The Three Men's Morris implementation successfully demonstrates:
- ✅ Advanced condition system (`zone.count`, `phase.is`)
- ✅ Automatic phase transitions
- ✅ Proper game rule enforcement
- ⚠️ Need for selection pattern support

This implementation proves the engine can handle complex game logic declaratively, while also highlighting the next steps needed for full board game support.
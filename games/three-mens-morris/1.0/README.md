# Three Men's Morris Implementation

This implementation demonstrates how to create a game with phase-like behavior using the current Bluefelt engine.

## Game Overview

Three Men's Morris is a strategy game where:
1. Players take turns placing 3 pieces each on a 3x3 grid
2. After all pieces are placed, players move their pieces to adjacent spaces
3. First player to get 3 in a row wins

## Implementation Approach

### Enhanced Engine Capabilities

The engine now supports:
- Basic conditions: `zone.isEmpty`, `player.isActor`
- **NEW: Complex conditions**: `zone.count`, `entity.owner`, `phase.is`, `resource.value`
- Phase system with `possibleActions` restrictions
- Automatic actions with `auto: true`
- Conditional action availability based on game state

### Implementation Strategy

This implementation demonstrates a complete Three Men's Morris game:

1. **Placement Phase**: Players alternate placing pieces (max 3 each)
   - Uses `zone.count` to limit pieces to 3 per player
   - Automatically transitions to movement phase when both have 3 pieces
   
2. **Movement Phase**: Players move pieces to adjacent empty spaces
   - Currently limited by engine - moveEntity needs from/to coordinates
   - Full implementation requires selection pattern (see design docs)

### Key Conditions Used

```yaml
# Limit placement to 3 pieces per player
- condition: zone.count
  with:
    zone: "/zones/board"
    entity: "piece_{player}"
    operator: "<"
    value: 3

# Transition to movement when both players have 3 pieces
- condition: zone.count
  with:
    zone: "/zones/board"
    entity: "piece_p1"
    operator: "=="
    value: 3
```

### Current Limitations

The movement phase requires the selection pattern (select piece → choose destination) which needs:
- Selection state management
- Two-phase interaction with cancel support
- Enhanced UI affordances

See `selection-pattern-design.md` for the proposed solution.

## Files

- `manifest.yaml` - Game metadata
- `entities.yaml` - Player pieces
- `zones.yaml` - 3x3 game board
- `actions.yaml` - Place and move actions
- `phases.yaml` - Game flow phases
- `RULES.md` - Complete game rules

## Testing

The game includes comprehensive tests in `server/tests/three_mens_morris_test.rs`:
- Setup verification
- Action existence
- Gameplay flow
- Win conditions
- Movement mechanics

## Key Learnings

1. **Conditional actions**: The new `zone.count` condition enables state-based action availability
2. **Implicit phases**: Complex phase behavior can emerge from simple conditions
3. **Clean design**: Enhanced conditions reduce the need for complex phase management
4. **Future ready**: This approach scales well as more conditions are added
# Engine Enhancement: Advanced Condition System

## Overview

The Bluefelt engine now supports a comprehensive condition evaluation system that enables sophisticated game rules and state-based action availability.

## Implemented Conditions

### 1. zone.isEmpty
Checks if a zone or cell is empty.

```yaml
when:
  - condition: zone.isEmpty
    with:
      zone: "{target}"  # Can use template variables
```

### 2. player.isActor
Validates that the current player is the one taking the action.

```yaml
when:
  - condition: player.isActor
```

### 3. zone.count
Counts entities in a zone with flexible filtering and comparison.

```yaml
when:
  - condition: zone.count
    with:
      zone: "/zones/board"
      entity: "piece_{player}"  # Optional: filter by entity type
      operator: ">="          # ==, !=, >, <, >=, <=
      value: 3
```

Features:
- Works with both grid zones (counts non-null cells) and list zones
- Supports entity pattern matching (e.g., `piece_{player}`)
- Multiple comparison operators

### 4. entity.owner
Checks if an entity belongs to a specific player.

```yaml
when:
  - condition: entity.owner
    with:
      entity: "{selected}"
      owner: "{player}"
```

### 5. phase.is
Validates the current phase of a phase set.

```yaml
when:
  - condition: phase.is
    with:
      phaseSet: "game"
      phase: "placement"
```

### 6. resource.value
Checks resource zone values.

```yaml
when:
  - condition: resource.value
    with:
      resource: "gold_{player}"
      operator: ">="
      value: 10
```

## Implementation Details

### Architecture
- Centralized condition evaluation in `src/conditions.rs`
- Modular design allows easy addition of new conditions
- Consistent error handling and reporting
- Template variable substitution (e.g., `{player}`, `{target}`)

### Integration Points
1. **Action Validation**: Conditions are evaluated before action execution
2. **Action Map Generation**: Only shows actions with met conditions
3. **Automatic Actions**: Can trigger based on state conditions

### Testing
Comprehensive test suite validates all condition types:
- Empty zone detection
- Player validation
- Entity counting with various operators
- Ownership checks
- Phase state verification
- Resource value comparisons

## Usage Examples

### Three Men's Morris
Demonstrates conditional piece placement:

```yaml
- id: placeToken
  when:
    - condition: zone.isEmpty
      with:
        zone: "{target}"
    - condition: player.isActor
    - condition: zone.count
      with:
        zone: "/zones/stock_{player}"
        operator: ">"
        value: 0
```

### Chess (Future)
Could implement complex rules:

```yaml
- id: castle
  when:
    - condition: entity.owner
      with:
        entity: "{king}"
        owner: "{player}"
    - condition: zone.count
      with:
        zone: "/zones/board"
        entity: "king_{player}_moved"
        operator: "=="
        value: 0
```

## Benefits

1. **Declarative Rules**: Game logic expressed clearly in YAML
2. **Implicit Phases**: Complex phase behavior emerges from conditions
3. **Reduced Complexity**: Less need for WebAssembly hooks
4. **Better Validation**: Actions validated before execution
5. **Dynamic UI**: Action availability updates automatically

## Future Enhancements

Potential additions:
- `path.isClear` - Check if movement path is unobstructed
- `entity.hasProperty` - Check entity properties
- `game.turn` - Turn number comparisons
- `logical.and/or/not` - Combine conditions
- `zone.adjacent` - Check adjacent zones

## Migration Guide

To use the new conditions:

1. Update action definitions with `when` conditions
2. Remove workarounds that checked conditions in hooks
3. Simplify phase management using state-based conditions
4. Test thoroughly - conditions prevent invalid actions

## Conclusion

The enhanced condition system makes Bluefelt more powerful while keeping game definitions clean and maintainable. It bridges the gap between simple declarative rules and complex game logic.
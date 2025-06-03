# Three Men's Morris Implementation: Lessons Learned

This document captures insights from implementing Three Men's Morris, highlighting both successes and challenges to improve future game development.

## Implementation Summary

Three Men's Morris was successfully implemented as a two-phase abstract strategy game using only declarative YAML configuration - no custom code required.

### What Worked Well

1. **Phase-Based Actions**: The engine's conditional action system elegantly handled the placement → movement phase transition
2. **Built-in Verbs**: All game mechanics were achievable with existing verbs (place, selectEntity, moveSelected, etc.)
3. **Automatic Win Detection**: The `grid.lineOfMarks` verb worked perfectly for 3-in-a-row detection
4. **Selection Pattern**: The two-step movement (select → move) using built-in verbs was clean

### Implementation Challenges

#### 1. Log Message Placeholders
**Issue**: Game logs showed unsubstituted placeholders: `"ben moved a piece to ({row}, {col})"`

**Root Cause**: The `moveSelected` verb only provides `{target}` (full path), not parsed `{row}` and `{col}` values

**Solution**: Simplified log template to `"{player} moved a piece"`

**Lesson**: Check available placeholders for each verb before writing log templates

#### 2. Client-Side Patch Errors  
**Issue**: Browser console showed `OPERATION_PATH_UNRESOLVABLE` errors when selecting pieces

**Root Cause**: Initial game state had empty `selection: {}` object, but server sent patches to `/game/selection/p1` which didn't exist

**Solution**: Modified engine to pre-initialize selection paths for each player:
```rust
// In state.rs
if let Some(selection) = initial_state.get_mut("selection").and_then(|s| s.as_object_mut()) {
    for player in &players {
        if let Some(player_id) = player.get("id").and_then(|id| id.as_str()) {
            selection.insert(player_id.to_string(), json!(null));
        }
    }
}
```

**Lesson**: Initial state structure must anticipate all possible update paths

## Key Takeaways

### 1. Engine Capabilities Are Sufficient
The entire game was implemented without custom code, validating the declarative approach. Complex logic like phase transitions was achieved through clever use of conditions and automatic actions.

### 2. Documentation Gaps Matter
Missing information about verb placeholders and state structure requirements caused unnecessary debugging. Better verb documentation would have prevented both issues.

### 3. State Management Is Critical
How the initial state is structured directly impacts whether client updates work. Pre-initializing expected paths prevents runtime errors.

### 4. Testing Reveals Hidden Issues
Both problems only appeared during actual gameplay, not during initial implementation. Early and thorough testing is essential.

## Recommendations for Future Games

### Before Starting
1. **Review similar games** for patterns and state structure
2. **Check verb documentation** for available placeholders
3. **Plan state structure** including all paths that might be updated

### During Development
1. **Test early** with actual client interaction
2. **Check browser console** for patch errors
3. **Verify log messages** show correctly

### For Engine Improvements
1. **Better verb documentation** listing all available placeholders
2. **State structure validator** to catch missing paths
3. **More informative error messages** for patch failures

## Reusable Patterns

### Phase Transition Pattern
```yaml
# Automatic check after each placement
- id: checkPhaseTransition
  uses: setPhase
  auto: true
  with:
    phaseSet: "game"
    phase: "movement"
  when:
    - condition: zone.count
      with:
        zone: "/zones/board"
        entity: "piece_p1"
        value: 3
    - condition: zone.count
      with:
        zone: "/zones/board"
        entity: "piece_p2"
        value: 3
```

### Selection-Based Movement Pattern
```yaml
# Step 1: Select piece
- id: selectPiece
  uses: selectEntity
  when:
    - condition: entity.owner
      with:
        entity: "{entityAtLocation}"
        owner: "{player}"

# Step 2: Move selected piece
- id: moveSelectedPiece
  uses: moveSelected
  when:
    - condition: entity.selected
    - condition: zone.isEmpty
      with:
        zone: "{target}"
```

## Conclusion

Three Men's Morris demonstrated that Bluefelt's declarative approach works well for abstract strategy games. The issues encountered were related to documentation and state initialization rather than fundamental limitations. Each game implementation helps identify areas for platform improvement, making future games easier to develop.
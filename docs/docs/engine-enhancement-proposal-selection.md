# Engine Enhancement Proposal: Selection and Cancel Pattern

## Executive Summary

This proposal outlines engine enhancements to support a common board game pattern: selecting a piece, then choosing an action (with the ability to cancel). This pattern is essential for games like Chess, Checkers, and Three Men's Morris.

## Problem Statement

Currently, the Bluefelt engine lacks:

1. **Persistent Selection State**: No way to store which piece is selected between user clicks
2. **Cancel Mechanism**: No way to undo a selection without consuming a turn
3. **Selection-Aware UI**: Action maps can't show different affordances based on selection state
4. **Non-Committal Actions**: All actions advance the game state

## Proposed Solution

### 1. Built-in Selection State

Add a `selection` property to the game state:

```json
{
  "selection": {
    "location": "/zones/board/cells/0/1",
    "entity": "piece_p1",
    "actor": "p1",
    "timestamp": 1234567890
  }
}
```

### 2. New Verb: selectEntity

```rust
fn apply_select_entity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let location = args["location"].as_str().ok_or("Missing location")?;
    let actor = args["actor"].as_str().ok_or("Missing actor")?;
    
    // Get entity at location
    let entity = get_cell_value(state, location)?;
    
    // Store selection
    state["selection"] = json!({
        "location": location,
        "entity": entity["entity"],
        "actor": actor
    });
    
    // Return patch
    Ok(vec![json!({
        "op": "replace",
        "path": "/game/selection",
        "value": state["selection"]
    })])
}
```

Key features:
- Does NOT advance turn
- Stores selection for use by subsequent actions
- Can be called multiple times (changing selection)

### 3. Enhanced moveEntity Verb

Update `moveEntity` to support selection-based movement:

```rust
fn apply_move_entity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    // Support both explicit from/to and selection-based movement
    let from_path = if args["from"].is_string() {
        args["from"].as_str().unwrap()
    } else if let Some(selection) = state.get("selection") {
        selection["location"].as_str().ok_or("No selection location")?
    } else {
        return Err("No 'from' location or selection".to_string());
    };
    
    let to_path = args["to"].as_str().ok_or("Missing 'to' location")?;
    
    // Perform move...
    
    // Clear selection after successful move
    state["selection"] = json!(null);
}
```

### 4. New Verb: clearSelection

```rust
fn apply_clear_selection(state: &mut Value, _args: &Value) -> Result<Vec<Value>, String> {
    state["selection"] = json!(null);
    
    Ok(vec![json!({
        "op": "replace",
        "path": "/game/selection",
        "value": null
    })])
}
```

### 5. Enhanced Action Map Generation

Update `compute_action_map` in lobby.rs:

```rust
// When computing action map, consider selection state
if let Some(selection) = state.get("selection") {
    // Show movement options
    for valid_destination in get_valid_moves(selection["location"]) {
        action_map.insert(valid_destination, json!({
            "action": "moveSelectedPiece",
            "direction": "Move here"
        }));
    }
    
    // Add cancel option
    action_map.insert("/ui/actions/cancel", json!({
        "action": "cancelSelection",
        "direction": "Cancel selection",
        "uiElement": "button"
    }));
} else {
    // Show selection options
    for piece_location in get_player_pieces(current_player) {
        action_map.insert(piece_location, json!({
            "action": "selectPiece",
            "direction": "Select this piece"
        }));
    }
}
```

### 6. UI Protocol Extensions

Add support for non-spatial UI elements:

```json
{
  "ui": {
    "actionMap": {
      "/zones/board/cells/0/0": {
        "action": "selectPiece",
        "direction": "Select"
      },
      "/ui/actions/cancel": {
        "action": "cancelSelection",
        "direction": "Cancel",
        "uiElement": "button",
        "position": "bottom-bar"
      }
    }
  }
}
```

## Implementation Example

### Three Men's Morris with Selection

```yaml
# actions.yaml
- id: selectPiece
  uses: selectEntity
  ui:
    direction: "Select this piece"
  when:
    - condition: player.isActor
    - condition: entity.owner
      with:
        entity: "{entityAtLocation}"
        owner: "{player}"
    - condition: phase.is
      with:
        phaseSet: "game"
        phase: "movement"

- id: moveSelectedPiece
  uses: moveEntity
  with:
    to: "{target}"
    # 'from' comes from selection state
  ui:
    direction: "Move here"
  when:
    - condition: zone.isEmpty
      with:
        zone: "{target}"
    - condition: selection.exists
    - condition: selection.owner
      with:
        owner: "{player}"
  then:
    - action: checkForWin
    - action: advanceTurn

- id: cancelSelection
  uses: clearSelection
  ui:
    direction: "Cancel selection"
    uiElement: "button"
  when:
    - condition: selection.exists
    - condition: selection.owner
      with:
        owner: "{player}"
```

## Benefits

1. **Natural Interaction**: Matches player expectations from physical board games
2. **Error Prevention**: Players can correct mis-clicks
3. **Visual Feedback**: Selected pieces can be highlighted
4. **Reusable Pattern**: Many games can use this mechanism

## Client Implementation

The React client would need updates to:

1. **Render Selection State**: Highlight selected pieces
2. **Show Cancel UI**: Display cancel button when selection exists
3. **Handle Non-Spatial Actions**: Support UI element actions like cancel

```typescript
// In GameView component
const renderActionButtons = () => {
  const cancelAction = actionMap['/ui/actions/cancel'];
  if (cancelAction) {
    return (
      <Button onClick={() => handleAction(cancelAction)}>
        {cancelAction.direction}
      </Button>
    );
  }
};

// Highlight selected piece
const isSelected = (location: string) => {
  return gameState.selection?.location === location;
};
```

## Migration Path

1. **Phase 1**: Implement selection state and new verbs
2. **Phase 2**: Update action map generation
3. **Phase 3**: Add client UI support
4. **Phase 4**: Update existing games to use selection pattern

## Alternative Considerations

### WebAssembly Hooks
Could implement selection in WASM, but this would:
- Be game-specific, not reusable
- Require complex state management
- Not integrate well with action system

### Multi-Step Actions
Could make actions have multiple steps, but:
- Would complicate action definition
- Wouldn't support cancel easily
- Would be a larger architectural change

## Conclusion

The selection pattern is fundamental to many board games. Adding proper engine support will:
- Enable more game types
- Improve player experience  
- Maintain Bluefelt's declarative approach

This enhancement represents a natural evolution of the engine's capabilities while staying true to its design principles.
# Generic Gravity Support Architecture

## Problem
The current Connect 4 implementation violates the principle of a generic client by adding game-specific components.

## Better Solution: Generic Zone Behaviors

### 1. Zone Metadata Approach

Extend zone definitions to include behavior metadata:

```yaml
# zones.yaml
- id: board
  type: grid
  gridProps:
    rows: 6
    cols: 7
  contents: empty
  behavior:
    type: gravity
    direction: down
    clickTarget: column  # Click targets whole column, not individual cells
```

### 2. Server-Side: Enhanced Action Processing

Add a generic `placeWithGravity` verb:

```rust
// In verbs.rs
fn apply_place_with_gravity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args["zone"].as_str().ok_or("Missing 'zone' path")?;
    let column = args["column"].as_u64().ok_or("Missing 'column' index")? as usize;
    let entity = args["entity"].as_str().ok_or("Missing 'entity' id")?;
    
    // Get zone metadata to understand gravity behavior
    let zone = get_zone_mut(state, zone_path)?;
    let cells = zone["cells"].as_array_mut().ok_or("Zone is not a grid")?;
    
    // Find lowest empty row in specified column
    let mut target_row = None;
    for (row_idx, row) in cells.iter().enumerate().rev() {
        if let Some(cell) = row.as_array().and_then(|r| r.get(column)) {
            if cell.is_null() {
                target_row = Some(row_idx);
                break;
            }
        }
    }
    
    let row = target_row.ok_or("Column is full")?;
    
    // Place entity and return patches
    let entity_value = json!({"entity": entity});
    cells[row].as_array_mut().unwrap()[column] = entity_value.clone();
    
    Ok(vec![
        json!({
            "op": "replace",
            "path": format!("/game{}/cells/{}/{}", zone_path, row, column),
            "value": entity_value
        }),
        // Animation metadata for client
        json!({
            "op": "add",
            "path": "/ui/lastAnimation",
            "value": {
                "type": "gravity",
                "zone": zone_path,
                "column": column,
                "targetRow": row,
                "entity": entity
            }
        })
    ])
}
```

### 3. Game Definition Update

```yaml
# actions.yaml
- id: dropDisc
  uses: placeWithGravity
  ui:
    direction: "Click a column to drop your disc"
    targetType: column  # New: indicates this action targets columns
  when:
    - condition: column.hasSpace
      with:
        zone: "/zones/board"
        column: "{targetColumn}"
    - condition: player.isActor
  with:
    zone: "/zones/board"
    column: "{targetColumn}"
    entity: disc_{player}
  then:
    - action: checkWin
    - action: advanceTurn
```

### 4. Client-Side: Generic Zone Behavior Handler

Enhance the existing Board component to handle different zone behaviors:

```tsx
// Enhanced Board.tsx
export default function Board({ zones, zoneMetadata, ... }: BoardProps) {
  
  const getZoneBehavior = (zoneId: string) => {
    const meta = zoneMetadata?.find(z => z.id === zoneId);
    return meta?.behavior || { type: 'standard' };
  };

  const handleZoneClick = (zoneId: string, row: number, col: number) => {
    const behavior = getZoneBehavior(zoneId);
    
    if (behavior.type === 'gravity' && behavior.clickTarget === 'column') {
      // For gravity zones, convert cell click to column action
      const columnAction = actionMap[`/zones/${zoneId}/columns/${col}`];
      if (columnAction) {
        // Trigger column-based action instead of cell action
        onColumnAction?.(zoneId, col);
        return;
      }
    }
    
    // Standard cell click behavior
    onCellClick?.(row, col);
  };

  const renderZone = (zoneId: string, zoneData: any) => {
    const behavior = getZoneBehavior(zoneId);
    const isGravityZone = behavior.type === 'gravity';
    
    return (
      <div className={`zone-${behavior.type}`}>
        {isGravityZone && (
          <ColumnDropIndicators 
            zoneId={zoneId}
            cols={zoneData.cells[0]?.length || 0}
            actionMap={actionMap}
            currentPlayer={currentPlayer}
            onColumnClick={(col) => handleZoneClick(zoneId, -1, col)}
          />
        )}
        <GridDisplay 
          cells={zoneData.cells}
          onCellClick={(row, col) => handleZoneClick(zoneId, row, col)}
          behavior={behavior}
        />
      </div>
    );
  };
}
```

### 5. Generic Animation System

```tsx
// useZoneAnimations.ts
export function useZoneAnimations(lastAnimation: any) {
  const [activeAnimation, setActiveAnimation] = useState(null);
  
  useEffect(() => {
    if (lastAnimation?.type === 'gravity') {
      setActiveAnimation({
        type: 'drop',
        zone: lastAnimation.zone,
        column: lastAnimation.column,
        targetRow: lastAnimation.targetRow,
        entity: lastAnimation.entity,
        duration: lastAnimation.targetRow * 100 // 100ms per row
      });
      
      setTimeout(() => setActiveAnimation(null), lastAnimation.targetRow * 100);
    }
  }, [lastAnimation]);
  
  return activeAnimation;
}
```

## Benefits of This Approach

### ✅ Generic Client
- No game-specific components
- All behavior driven by zone metadata
- Reusable for any gravity-based game (Tetris, Puyo Puyo, etc.)

### ✅ Declarative Configuration
- Games specify behavior in YAML
- No client code changes needed for new gravity games
- Maintains Bluefelt's declarative philosophy

### ✅ Server Validation
- Server properly validates gravity mechanics
- Prevents cheating/invalid moves
- Consistent game state

### ✅ Rich Animations
- Generic animation system
- Server provides animation hints
- Client renders smooth transitions

## Implementation Steps

1. **Phase 1**: Add `behavior` support to zone metadata
2. **Phase 2**: Implement `placeWithGravity` verb on server
3. **Phase 3**: Update Board component to handle zone behaviors
4. **Phase 4**: Add generic animation system
5. **Phase 5**: Update Connect 4 to use new architecture
6. **Phase 6**: Remove game-specific ConnectFourBoard component

This approach maintains the core principle that **games are defined by data, not code**, while still enabling rich interactive experiences like gravity mechanics.
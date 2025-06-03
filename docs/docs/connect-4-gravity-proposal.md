# Connect 4 Gravity Implementation Proposal

## Overview
Implement proper Connect 4 gravity mechanics where players click on a column and the disc animates down to the lowest available position.

## Server-Side Changes

### 1. New Verb: `placeWithGravity`

```rust
// In verbs.rs
fn apply_place_with_gravity(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let column_path = args["column"].as_str().ok_or("Missing 'column' path")?;
    let entity = args["entity"].as_str().ok_or("Missing 'entity' id")?;
    
    // Extract column index from path like "/zones/board/columns/3"
    let col_index = extract_column_index(column_path)?;
    
    // Find the lowest empty row in this column
    let board = get_zone_mut(state, "/zones/board")?;
    let cells = board["cells"].as_array().ok_or("Board cells not found")?;
    
    let mut target_row = None;
    for (row_idx, row) in cells.iter().enumerate().rev() {
        if row[col_index].is_null() {
            target_row = Some(row_idx);
            break;
        }
    }
    
    let row = target_row.ok_or("Column is full")?;
    
    // Place the entity at the calculated position
    let target_path = format!("/zones/board/cells/{}/{}", row, col_index);
    let entity_value = json!({"entity": entity});
    set_cell_value(state, &target_path, entity_value.clone())?;
    
    Ok(vec![
        json!({
            "op": "replace",
            "path": format!("/game{}", target_path),
            "value": entity_value
        }),
        // Include animation hint for client
        json!({
            "op": "add",
            "path": "/ui/lastMove",
            "value": {
                "type": "dropAnimation",
                "column": col_index,
                "fromRow": 0,
                "toRow": row,
                "entity": entity
            }
        })
    ])
}
```

### 2. Action Map Generation Update

The server should generate column-based action targets:

```rust
// In action map generation
for col in 0..board_width {
    // Check if column has space
    if column_has_space(board, col) {
        action_map.insert(
            format!("/zones/board/columns/{}", col),
            json!({
                "action": "dropDisc",
                "direction": "Drop disc in this column"
            })
        );
    }
}
```

## Client-Side Changes

### 1. Column Click Detection

Create a new component for Connect 4 boards:

```tsx
// ConnectFourBoard.tsx
export default function ConnectFourBoard({ 
  board, 
  onColumnClick, 
  lastMove,
  actionMap 
}: Props) {
  const [animatingDisc, setAnimatingDisc] = useState<AnimationState | null>(null);
  
  // Handle column clicks
  const handleColumnClick = (col: number) => {
    const columnAction = actionMap[`/zones/board/columns/${col}`];
    if (columnAction) {
      onColumnClick(col);
    }
  };
  
  // Render drop zones above the board
  return (
    <div className="connect-four-board">
      {/* Drop zone indicators */}
      <div className="drop-zones">
        {Array.from({ length: 7 }, (_, col) => (
          <div
            key={col}
            className={`drop-zone ${actionMap[`/zones/board/columns/${col}`] ? 'active' : ''}`}
            onClick={() => handleColumnClick(col)}
            onMouseEnter={() => setHoveredColumn(col)}
            onMouseLeave={() => setHoveredColumn(null)}
          >
            {hoveredColumn === col && (
              <div className="drop-preview">
                <Disc color={currentPlayerColor} />
              </div>
            )}
          </div>
        ))}
      </div>
      
      {/* Game board */}
      <div className="board-grid">
        {board.map((row, rowIdx) => (
          <div key={rowIdx} className="board-row">
            {row.map((cell, colIdx) => (
              <BoardCell 
                key={`${rowIdx}-${colIdx}`}
                entity={cell?.entity}
                isAnimating={animatingDisc?.row === rowIdx && animatingDisc?.col === colIdx}
              />
            ))}
          </div>
        ))}
      </div>
      
      {/* Animating disc overlay */}
      {animatingDisc && (
        <AnimatingDisc
          column={animatingDisc.column}
          fromRow={animatingDisc.fromRow}
          toRow={animatingDisc.toRow}
          entity={animatingDisc.entity}
          onComplete={() => setAnimatingDisc(null)}
        />
      )}
    </div>
  );
}
```

### 2. Drop Animation Component

```tsx
// AnimatingDisc.tsx
export default function AnimatingDisc({ 
  column, 
  fromRow, 
  toRow, 
  entity,
  onComplete 
}: Props) {
  useEffect(() => {
    const duration = (toRow - fromRow) * 100; // 100ms per row
    const timer = setTimeout(onComplete, duration);
    return () => clearTimeout(timer);
  }, []);
  
  return (
    <div 
      className="animating-disc"
      style={{
        '--from-row': fromRow,
        '--to-row': toRow,
        '--column': column,
        '--duration': `${(toRow - fromRow) * 100}ms`
      }}
    >
      <Disc entity={entity} />
    </div>
  );
}
```

### 3. CSS for Animations

```css
.connect-four-board {
  position: relative;
}

.drop-zones {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  height: 60px;
  margin-bottom: 10px;
}

.drop-zone {
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s;
}

.drop-zone.active:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.drop-preview {
  opacity: 0.5;
  animation: bounce 0.5s ease-in-out infinite;
}

@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-10px); }
}

.animating-disc {
  position: absolute;
  animation: drop var(--duration) cubic-bezier(0.25, 0.46, 0.45, 0.94);
  pointer-events: none;
}

@keyframes drop {
  from {
    transform: translate(
      calc(var(--column) * 100% + 50%), 
      calc(var(--from-row) * 100% + 50%)
    );
  }
  to {
    transform: translate(
      calc(var(--column) * 100% + 50%), 
      calc(var(--to-row) * 100% + 50%)
    );
  }
}
```

## Game Definition Update

```yaml
# actions.yaml
- id: dropDisc
  uses: placeWithGravity  # New verb
  ui:
    direction: "Click a column to drop your disc"
    logTemplate: "{player} dropped a disc in column {col}"
  when:
    - condition: column.hasSpace  # New condition type
      with:
        column: "{target}"
    - condition: player.isActor
  with:
    entity: disc_{player}
  then:
    - action: checkWin
    - action: advanceTurn
```

## Alternative Approach: Client-Side Calculation

If modifying the server is not immediately feasible, we could:

1. Have the client calculate the target row when a column is clicked
2. Send the standard `place` action with the calculated cell
3. Still show the drop animation on the client side

```tsx
const handleColumnClick = (col: number) => {
  // Find lowest empty row
  let targetRow = -1;
  for (let row = board.length - 1; row >= 0; row--) {
    if (!board[row][col]) {
      targetRow = row;
      break;
    }
  }
  
  if (targetRow >= 0) {
    // Show animation
    setAnimatingDisc({ 
      column: col, 
      fromRow: -1, 
      toRow: targetRow,
      entity: currentPlayerDisc
    });
    
    // Send action after animation starts
    setTimeout(() => {
      onCellClick(targetRow, col);
    }, 50);
  }
};
```

## Benefits of Server-Side Approach

1. **Consistency**: Server validates the move is to the correct row
2. **Simplicity**: Actions target columns, not cells
3. **Reusability**: Other games (like Puissance 4 variants) can use the same verb
4. **Animation Hints**: Server can provide animation metadata in patches

## Implementation Priority

1. Start with client-side calculation for immediate improvement
2. Add drop preview on hover
3. Implement drop animation
4. Later: Add server-side `placeWithGravity` verb for proper validation

This approach provides the expected Connect 4 UX while maintaining the declarative nature of the Bluefelt platform.
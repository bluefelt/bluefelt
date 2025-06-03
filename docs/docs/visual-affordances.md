# Visual Affordances

Visual affordances provide clear, intuitive indicators of how players can interact with the game. This guide explains how to design and implement effective affordances for different game mechanics.

## Overview

Visual affordances answer the fundamental question: "What can I click on?" They guide player interactions through visual cues, making games intuitive and accessible.

### Core Principles

1. **Discoverability**: Players should immediately understand what's interactive
2. **Feedback**: Clear response to user actions (hover, click, selection)
3. **Consistency**: Similar interactions should look and behave similarly
4. **Accessibility**: Work across devices, input methods, and abilities
5. **Performance**: Smooth, responsive interactions

## Interaction Patterns

### Cell-Based Interactions (Tic-Tac-Toe)

**Pattern**: Direct cell clicks on a grid

**Visual Cues**:
- Hover highlighting on valid cells
- Cursor changes to pointer on interactive cells
- Subtle border changes to indicate selection

**Implementation**:
```typescript
// BoardCell component shows hover states
<div 
  className={`cell ${isClickable ? 'clickable' : 'disabled'}`}
  onClick={isClickable ? handleClick : undefined}
>
  {content}
</div>
```

**CSS**:
```css
.cell.clickable {
  cursor: pointer;
  transition: background-color 0.2s;
}

.cell.clickable:hover {
  background-color: rgba(59, 130, 246, 0.1);
  border-color: #3b82f6;
}
```

### Column-Based Interactions (Connect 4)

**Pattern**: Click column headers to drop pieces with gravity

**Visual Cues**:
- Column drop zones above the board
- Down arrows (↓) indicating drop direction
- Blue background for valid columns
- Gray background for full/invalid columns

**Implementation**:
```typescript
// BoardZone detects column actions and renders drop zones
{columnActions.length > 0 && (
  <div className="grid gap-0 mb-2" style={{gridTemplateColumns: `repeat(${cols}, ${cellSize}px)`}}>
    {Array.from({ length: cols }, (_, colIndex) => {
      const isClickableColumn = columnActions.includes(colIndex);
      return (
        <div
          className={`column-drop-zone ${isClickableColumn && isMyTurn 
            ? 'clickable' 
            : 'disabled'}`}
          onClick={() => isClickableColumn && handleColumnClick(colIndex)}
        >
          {isClickableColumn && isMyTurn && <span>↓</span>}
        </div>
      );
    })}
  </div>
)}
```

### Zone-Based Interactions (Card Games)

**Pattern**: Click zones or specific cards within zones

**Visual Cues**:
- Highlighted zone borders for valid drop targets
- Card selection indicators
- Drag and drop visual feedback

### Multi-Step Interactions (Checkers)

**Pattern**: Select piece, then select destination

**Visual Cues**:
- Selected piece highlighting
- Valid move indicators on target squares
- Path visualization for complex moves

## Generic Component Architecture

### Action Map Detection

Components automatically detect interaction patterns from the action map:

```typescript
// BoardZone detects column actions
const columnActions = Object.keys(actionMap || {})
  .filter(path => path.includes(`/zones/${zoneId}/columns/`))
  .map(path => {
    const match = path.match(/\/zones\/[^/]+\/columns\/(\d+)/);
    return match ? parseInt(match[1]) : -1;
  })
  .filter(col => col >= 0);
```

### Conditional Rendering

Affordances appear based on available actions:

```typescript
// Only show column drop zones when column actions exist
{columnActions.length > 0 && (
  <ColumnDropZones 
    columns={cols} 
    clickableColumns={columnActions}
    onColumnClick={handleColumnClick}
    isMyTurn={isMyTurn}
    cellSize={cellSize}
  />
)}
```

### State-Driven Feedback

Visual states reflect game conditions:

```typescript
const cellState = {
  isEmpty: cell === null,
  isClickable: isMyTurn && actionMap?.[location],
  isSelected: selection?.row === row && selection?.col === col,
  isValidTarget: validMoves?.includes(location)
};
```

## Accessibility Considerations

### Keyboard Navigation

```typescript
// Handle keyboard events for grid navigation
const handleKeyDown = (event: React.KeyboardEvent) => {
  switch (event.key) {
    case 'ArrowUp': moveFocus(row - 1, col); break;
    case 'ArrowDown': moveFocus(row + 1, col); break;
    case 'ArrowLeft': moveFocus(row, col - 1); break;
    case 'ArrowRight': moveFocus(row, col + 1); break;
    case 'Enter':
    case ' ': handleCellClick(row, col); break;
  }
};
```

### Screen Reader Support

```typescript
// Descriptive labels for game state
<div
  role="button"
  tabIndex={isClickable ? 0 : -1}
  aria-label={`${isEmpty ? 'Empty' : 'Occupied'} cell at row ${row + 1}, column ${col + 1}`}
  aria-disabled={!isClickable}
  onKeyDown={handleKeyDown}
>
```

### Color and Contrast

- Don't rely solely on color for information
- Ensure sufficient contrast ratios (4.5:1 minimum)
- Use icons, shapes, and text alongside color

### Touch Targets

- Minimum 44×44px touch targets on mobile
- Adequate spacing between interactive elements
- Consider thumb reach zones on larger screens

## Device-Specific Considerations

### Mobile Optimizations

**Touch Interactions**:
```typescript
// Handle both click and touch events
const handleInteraction = useCallback((event: React.MouseEvent | React.TouchEvent) => {
  event.preventDefault();
  if (isClickable) {
    handleAction();
  }
}, [isClickable, handleAction]);

<div
  onClick={handleInteraction}
  onTouchEnd={handleInteraction}
  className="touch-target"
>
```

**Responsive Sizing**:
```typescript
// Adjust cell size based on screen size
const cellSize = useMemo(() => {
  const isMobile = window.innerWidth < 768;
  const baseSize = isMobile ? 48 : 60;
  return Math.max(baseSize, Math.min(100, availableWidth / cols));
}, [cols, availableWidth]);
```

### Desktop Enhancements

**Hover States**:
```css
@media (hover: hover) {
  .interactive:hover {
    transform: scale(1.05);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }
}
```

**Cursor Feedback**:
```css
.clickable { cursor: pointer; }
.draggable { cursor: grab; }
.dragging { cursor: grabbing; }
.disabled { cursor: not-allowed; }
```

## Animation and Transitions

### Smooth State Changes

```css
.cell {
  transition: all 0.2s ease-in-out;
}

.cell.selected {
  transform: scale(1.1);
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}
```

### Piece Movement

```css
@keyframes dropPiece {
  from {
    transform: translateY(-100px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.dropping-piece {
  animation: dropPiece 0.5s ease-out;
}
```

### Loading States

```typescript
// Show loading state during action processing
const [isProcessing, setIsProcessing] = useState(false);

const handleAction = async () => {
  setIsProcessing(true);
  try {
    await sendAction();
  } finally {
    setIsProcessing(false);
  }
};
```

## Game-Specific Patterns

### Tic-Tac-Toe
- **Pattern**: Simple cell clicks
- **Affordances**: Hover highlighting, cursor changes
- **Feedback**: Immediate mark placement

### Connect 4
- **Pattern**: Column drops with gravity
- **Affordances**: Column drop zones, down arrows
- **Feedback**: Piece animation falling to position

### Checkers
- **Pattern**: Piece selection → move destination
- **Affordances**: Selected piece highlight, valid move indicators
- **Feedback**: Path preview, capture indicators

### Card Games
- **Pattern**: Card selection, zone targeting
- **Affordances**: Card highlighting, zone borders
- **Feedback**: Drag preview, drop zones

## Implementation Patterns

### Action Map Integration

```typescript
// Generic component detects available actions
const detectInteractionPattern = (actionMap: Record<string, any>, zoneId: string) => {
  const patterns = {
    cellBased: Object.keys(actionMap).some(path => path.includes('/cells/')),
    columnBased: Object.keys(actionMap).some(path => path.includes('/columns/')),
    zoneBased: Object.keys(actionMap).some(path => path === `/zones/${zoneId}`),
    indexBased: Object.keys(actionMap).some(path => path.match(/\/zones\/[^/]+\/\d+$/))
  };
  
  return patterns;
};
```

### Progressive Enhancement

```typescript
// Start with basic functionality, enhance with advanced features
const EnhancedCell = ({ basic, enhanced }: CellProps) => {
  const [isAdvancedSupported] = useAdvancedFeatures();
  
  return (
    <div className="cell">
      {basic.content}
      {isAdvancedSupported && enhanced.animations}
      {isAdvancedSupported && enhanced.sounds}
    </div>
  );
};
```

### Error State Handling

```typescript
// Handle and display interaction errors gracefully
const [error, setError] = useState<string | null>(null);

const handleAction = async () => {
  try {
    await performAction();
    setError(null);
  } catch (err) {
    setError('Invalid move. Please try again.');
    // Show visual feedback for error
    showErrorHighlight();
  }
};
```

## Testing Visual Affordances

### Manual Testing Checklist

- [ ] Hover states work on desktop
- [ ] Touch targets work on mobile
- [ ] Keyboard navigation is functional
- [ ] Screen reader announcements are clear
- [ ] Error states provide helpful feedback
- [ ] Animations are smooth and purposeful
- [ ] Loading states appear when appropriate

### Automated Testing

```typescript
describe('Visual Affordances', () => {
  it('should show hover states on interactive elements', () => {
    render(<GameBoard />);
    const cell = screen.getByLabelText(/empty cell/i);
    
    fireEvent.mouseEnter(cell);
    expect(cell).toHaveClass('hoverable');
  });

  it('should handle keyboard navigation', () => {
    render(<GameBoard />);
    const firstCell = screen.getAllByRole('button')[0];
    
    firstCell.focus();
    fireEvent.keyDown(firstCell, { key: 'ArrowRight' });
    
    const secondCell = screen.getAllByRole('button')[1];
    expect(secondCell).toHaveFocus();
  });
});
```

### Performance Testing

```typescript
// Test that affordances don't impact performance
const testRenderPerformance = () => {
  const start = performance.now();
  
  render(<ComplexGameBoard />);
  
  const end = performance.now();
  expect(end - start).toBeLessThan(100); // 100ms threshold
};
```

## Best Practices

### Do's ✅

- ✅ Use consistent interaction patterns across games
- ✅ Provide immediate visual feedback for actions
- ✅ Test on multiple devices and input methods
- ✅ Include keyboard and screen reader support
- ✅ Use subtle animations to enhance understanding
- ✅ Make touch targets appropriately sized
- ✅ Handle error states gracefully
- ✅ Follow platform conventions (hover on desktop, touch on mobile)

### Don'ts ❌

- ❌ Rely solely on color to convey information
- ❌ Make interactive elements too small on mobile
- ❌ Use overly complex or distracting animations
- ❌ Ignore keyboard accessibility
- ❌ Create inconsistent interaction patterns
- ❌ Skip error state design
- ❌ Assume all users have perfect motor control
- ❌ Forget to test with real users

## Performance Considerations

### Efficient Rendering

```typescript
// Memoize expensive calculations
const cellStates = useMemo(() => {
  return boardData.map((row, rowIndex) => 
    row.map((cell, colIndex) => ({
      isEmpty: cell === null,
      isClickable: calculateClickable(rowIndex, colIndex),
      isHighlighted: calculateHighlight(rowIndex, colIndex)
    }))
  );
}, [boardData, actionMap, selection]);
```

### Minimize Redraws

```typescript
// Use React.memo for expensive components
const OptimizedCell = React.memo(({ 
  cell, 
  isClickable, 
  onClick 
}: CellProps) => {
  // Component implementation
}, (prevProps, nextProps) => {
  // Custom comparison for performance
  return prevProps.cell === nextProps.cell && 
         prevProps.isClickable === nextProps.isClickable;
});
```

## Related Documentation

- [Game Implementation Guide](./game-implementation-guide.md) - Overall development process
- [Testing Guide](./testing-guide.md) - Testing UI interactions
- [State Structure](./state-structure.md) - Understanding game state

Remember: Great visual affordances make games feel intuitive and enjoyable. Players should never have to guess how to interact with your game!
import BoardZone from './BoardZone';

interface BoardProps {
  zones: any;
  entityDefinitions?: any[];
  onCellClick?: (row: number, col: number) => void;
  isMyTurn?: boolean;
  zoneMetadata?: any[];
  playerNames?: string[];
  actionMap?: Record<string, any>;
  currentPlayerActionMap?: Record<string, any>;
  selection?: any;
}

export default function Board({ 
  zones, 
  entityDefinitions, 
  onCellClick,
  isMyTurn = false,
  zoneMetadata,
  playerNames,
  actionMap,
  currentPlayerActionMap,
  selection
}: BoardProps) {
  console.log('[Board] Component props:', {
    zones: zones ? Object.keys(zones) : 'null',
    zoneMetadata,
    isMyTurn,
    actionMap: actionMap ? Object.keys(actionMap).length : 0
  });
  
  if (!zones) {
    return null;
  }
  
  // Enhanced cell click handler that supports both cell and column actions
  const handleCellClick = (zoneKey: string, row: number, col: number) => {
    // Check if there's a column-based action for this column
    const columnPath = `/zones/${zoneKey}/columns/${col}`;
    const columnAction = currentPlayerActionMap?.[columnPath] || actionMap?.[columnPath];
    
    if (columnAction) {
      // This is a column-based action (like gravity) - trigger the column action
      // We need to pass the column information to the action handler
      if (onCellClick) {
        // Store the column info in a way the action handler can access
        // For now, we'll use a special row value to indicate this is a column action
        onCellClick(-1, col); // -1 indicates column action
      }
      return;
    }
    
    // Standard cell click
    onCellClick?.(row, col);
  };
  
  // Find all grid-shaped zones (2D arrays)
  const gridZones: Array<{key: string, data: any[][]}> = [];
  Object.entries(zones).forEach(([key, value]) => {
    // Check if it's a 2D array and not a marks zone
    if (Array.isArray(value) && 
        value.length > 0 && 
        Array.isArray(value[0]) &&
        !key.includes('marks')) {
      gridZones.push({ key, data: value as any[][] });
    }
  });
  
  console.log('[Board] Grid zones found:', gridZones.map(gz => ({ key: gz.key, rows: gz.data.length, cols: gz.data[0]?.length })));
  
  if (gridZones.length === 0) {
    console.log('[Board] No grid zones found, returning null');
    return null;
  }

  console.log('[Board] Rendering board with zones:', gridZones.map(gz => gz.key));

  return (
    <div className={`w-full ${gridZones.length > 1 ? 'grid gap-6 lg:grid-cols-2' : ''}`} data-testid="board-container">
      {gridZones.map(({ key: zoneId, data: boardData }) => (
        <BoardZone
          key={zoneId}
          zoneId={zoneId}
          boardData={boardData}
          isMyTurn={isMyTurn}
          onCellClick={(row, col) => handleCellClick(zoneId, row, col)}
          entityDefinitions={entityDefinitions}
          zoneMetadata={zoneMetadata}
          isSingleZone={gridZones.length === 1}
          playerNames={playerNames}
          actionMap={currentPlayerActionMap || actionMap}
          selection={selection}
        />
      ))}
    </div>
  );
}
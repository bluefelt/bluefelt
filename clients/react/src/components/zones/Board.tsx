import BoardZone from './BoardZone';

interface BoardProps {
  zones: any;
  entityDefinitions?: any[];
  onCellClick?: (row: number, col: number) => void;
  isMyTurn?: boolean;
  zoneMetadata?: any[];
  playerNames?: string[];
  actionMap?: Record<string, any>;
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
  selection
}: BoardProps) {
  if (!zones) {
    return null;
  }
  
  // Enhanced cell click handler that supports both cell and column actions
  const handleCellClick = (zoneKey: string, row: number, col: number) => {
    // Check if there's a column-based action for this column
    const columnPath = `/zones/${zoneKey}/columns/${col}`;
    const columnAction = actionMap?.[columnPath];
    
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
  
  if (gridZones.length === 0) {
    return null;
  }

  return (
    <div className={`w-full ${gridZones.length > 1 ? 'grid gap-6 lg:grid-cols-2' : ''}`}>
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
          actionMap={actionMap}
          selection={selection}
        />
      ))}
    </div>
  );
}
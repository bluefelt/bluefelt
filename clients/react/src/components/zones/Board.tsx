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
          onCellClick={onCellClick}
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
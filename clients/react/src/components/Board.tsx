import React, { useRef, useEffect, useLayoutEffect, useState } from 'react';

interface BoardProps {
  zones: any;
  playerId: string;
  entityDefinitions?: any[];
  currentPlayer?: string;
  onCellClick?: (row: number, col: number) => void;
  isMyTurn?: boolean;
  zoneMetadata?: any[]; // Array of zone definitions from manifest
}

interface ZoneProps {
  zoneId: string;
  boardData: any[][];
  isMyTurn: boolean;
  onCellClick?: (row: number, col: number) => void;
  entityDefinitions?: any[];
  zoneMetadata?: any[];
  isSingleZone?: boolean;
}

// Map player marks to colors
const getMarkColor = (cell: string) => {
  if (cell === 'mark_p1') return '#FF1493'; // Pink
  if (cell === 'mark_p2') return '#FFD700'; // Gold
  return '#888';
};

function Zone({ zoneId, boardData, isMyTurn, onCellClick, entityDefinitions, zoneMetadata, isSingleZone = false }: ZoneProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [cellSize, setCellSize] = useState(60); // Start with a reasonable default
  const lastContainerWidth = useRef<number>(0);
  
  const rows = boardData.length;
  const cols = boardData[0].length;
  const maxCellSize = 100;
  const minCellSize = 40;
  
  // Use layout effect for initial size calculation
  useLayoutEffect(() => {
    const updateCellSize = () => {
      if (!containerRef.current) return;
      
      const container = containerRef.current;
      const parentElement = container.parentElement;
      if (!parentElement) return;
      
      // Use parent element width to avoid feedback loops
      const parentWidth = parentElement.clientWidth;
      
      // Total padding and margins to account for:
      // - Container padding: 24px each side (p-6)
      // - Board black background padding: 16px each side (p-4) 
      // - Additional inner padding: 16px each side (p-4)
      // - Grid border: 2px each side
      // - Cell borders: 1px between each cell
      const containerPadding = 48; // 24 * 2
      const boardPadding = 32; // 16 * 2
      const innerPadding = 32; // 16 * 2
      const gridBorder = 4; // 2 * 2
      const cellBorders = (cols - 1) * 1; // 1px border between cells
      const totalPadding = containerPadding + boardPadding + innerPadding + gridBorder + cellBorders;
      
      // Calculate available width based on parent
      const availableWidth = Math.max(0, parentWidth - totalPadding);
      
      // Height calculation
      const viewportHeight = window.innerHeight;
      const titleAndPadding = 200; // Approximate space for header, title, padding
      const availableHeight = Math.max(0, viewportHeight - titleAndPadding);
      
      // Calculate the maximum cell size that would fit
      const maxCellWidth = Math.floor(availableWidth / cols);
      const maxCellHeight = Math.floor(availableHeight / rows);
      const optimalCellSize = Math.min(maxCellWidth, maxCellHeight);
      
      // Apply constraints
      const newCellSize = Math.max(minCellSize, Math.min(maxCellSize, optimalCellSize));
      setCellSize(newCellSize);
    };
    
    // Calculate immediately
    updateCellSize();
  }, [rows, cols, minCellSize, maxCellSize]);
  
  // Separate effect for window resize
  useEffect(() => {
    const handleResize = () => {
      if (!containerRef.current) return;
      
      const container = containerRef.current;
      const parentElement = container.parentElement;
      if (!parentElement) return;
      
      const parentWidth = parentElement.clientWidth;
      
      // Only update if the width has changed significantly
      if (Math.abs(parentWidth - lastContainerWidth.current) < 20) {
        return;
      }
      lastContainerWidth.current = parentWidth;
      
      // Recalculate sizes
      const containerPadding = 48;
      const boardPadding = 32;
      const innerPadding = 32;
      const gridBorder = 4;
      const cellBorders = (cols - 1) * 1;
      const totalPadding = containerPadding + boardPadding + innerPadding + gridBorder + cellBorders;
      
      const availableWidth = Math.max(0, parentWidth - totalPadding);
      const viewportHeight = window.innerHeight;
      const titleAndPadding = 200;
      const availableHeight = Math.max(0, viewportHeight - titleAndPadding);
      
      const maxCellWidth = Math.floor(availableWidth / cols);
      const maxCellHeight = Math.floor(availableHeight / rows);
      const optimalCellSize = Math.min(maxCellWidth, maxCellHeight);
      
      const newCellSize = Math.max(minCellSize, Math.min(maxCellSize, optimalCellSize));
      
      setCellSize(prevSize => {
        if (Math.abs(prevSize - newCellSize) < 5) {
          return prevSize;
        }
        return newCellSize;
      });
    };
    
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [rows, cols, minCellSize, maxCellSize]);
  
  // Calculate font size based on cell size
  const fontSize = cellSize >= 80 ? 'text-5xl' : cellSize >= 60 ? 'text-4xl' : 'text-2xl';
  
  // Get glyph from entity definitions or fallback
  const getGlyph = (cell: string) => {
    const entity = entityDefinitions?.find(e => e.id === cell);
    return entity?.ui?.glyph || (cell === 'mark_p1' ? 'X' : 'O');
  };
  
  // Handle cell clicks for this specific zone
  const handleCellClick = (row: number, col: number) => {
    if (isMyTurn && boardData[row][col] === null && onCellClick) {
      // For tic-tac-toe games, we typically interact with the first/main board
      if (zoneId === 'board' || zoneId === 'da-board') {
        onCellClick(row, col);
      }
    }
  };

  // Get zone name from metadata if available
  const zoneInfo = zoneMetadata?.find(z => z.id === zoneId);
  let zoneName = zoneInfo?.name || zoneId;
  
  // Temporary mapping until server is restarted with zone metadata
  if (!zoneInfo?.name) {
    const knownZoneNames: Record<string, string> = {
      'board': 'Board',
      'da-board': 'Da Board',
      'useless-board': 'Da Useless Board'
    };
    zoneName = knownZoneNames[zoneId] || zoneId;
  }
  
  return (
    <div 
      ref={containerRef} 
      className="bg-gray-800 p-6 rounded-lg flex flex-col h-full min-h-[250px] w-full"
    >
      <h2 className="text-xl font-semibold mb-4 text-white">
        {zoneName}
      </h2>
      <div className="flex-1 flex items-center justify-center overflow-auto p-4">
        <div className="bg-black p-4 rounded">
          <div 
            className="grid gap-0 border-2 border-white"
            style={{
              gridTemplateColumns: `repeat(${cols}, ${cellSize}px)`,
              gridTemplateRows: `repeat(${rows}, ${cellSize}px)`,
              width: 'max-content',
              height: 'max-content'
            }}
          >
            {boardData.map((row, rowIndex) =>
              row.map((cell: any, colIndex: number) => {
                const isEmpty = cell === null;
                const isClickable = isMyTurn && isEmpty && (zoneId === 'board' || zoneId === 'da-board');
                
                return (
                  <div
                    key={`${rowIndex}-${colIndex}`}
                    className="relative border border-gray-700"
                    style={{ width: `${cellSize}px`, height: `${cellSize}px` }}
                  >
                    {isEmpty ? (
                      <div 
                        className={`w-full h-full flex items-center justify-center ${
                          isClickable ? 'cursor-pointer' : 'cursor-not-allowed'
                        }`}
                        style={{
                          backgroundImage: `radial-gradient(circle, #4a5568 1px, transparent 1px)`,
                          backgroundSize: '10px 10px',
                          backgroundColor: '#1a202c'
                        }}
                        onClick={() => handleCellClick(rowIndex, colIndex)}
                        onMouseEnter={(e) => {
                          if (isClickable) {
                            e.currentTarget.style.backgroundColor = 'rgba(75, 85, 99, 0.5)';
                          }
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.backgroundColor = '#1a202c';
                        }}
                      />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center bg-black">
                        <span 
                          className={`${fontSize} font-bold`}
                          style={{ color: getMarkColor(cell) }}
                        >
                          {getGlyph(cell)}
                        </span>
                      </div>
                    )}
                  </div>
                );
              })
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default function Board({ 
  zones, 
  playerId, 
  entityDefinitions, 
  currentPlayer,
  onCellClick,
  isMyTurn = false,
  zoneMetadata
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
        <Zone
          key={zoneId}
          zoneId={zoneId}
          boardData={boardData}
          isMyTurn={isMyTurn}
          onCellClick={onCellClick}
          entityDefinitions={entityDefinitions}
          zoneMetadata={zoneMetadata}
          isSingleZone={gridZones.length === 1}
        />
      ))}
    </div>
  );
}
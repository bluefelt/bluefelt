import React, { useRef, useEffect, useLayoutEffect, useState } from 'react';

interface BoardProps {
  zones: any;
  playerId: string;
  entityDefinitions?: any[];
  currentPlayer?: string;
  onCellClick?: (row: number, col: number) => void;
  isMyTurn?: boolean;
  zoneMetadata?: any[]; // Array of zone definitions from manifest
  playerNames?: string[]; // Array of player names
  possibleVerbs?: any[]; // Array of possible verbs for current player
}

interface ZoneProps {
  zoneId: string;
  boardData: any[][];
  isMyTurn: boolean;
  onCellClick?: (row: number, col: number) => void;
  entityDefinitions?: any[];
  zoneMetadata?: any[];
  isSingleZone?: boolean;
  playerNames?: string[];
  possibleVerbs?: any[];
}

import { usePlayer } from '../context/PlayerContext';
import { getColorById, getPlayerColor, PLAYER_COLORS } from '../config/colors';

// Get mark color based on player colors
const useMarkColor = () => {
  const { player } = usePlayer();
  
  return (cell: string, playerNames?: string[]) => {
    if (!playerNames) return '#888';
    
    // Find which player this entity belongs to - matches mark_p1, chip_p1, etc.
    const match = cell.match(/_p(\d+)$/);
    if (!match) return '#888';
    
    const entityPlayerIndex = parseInt(match[1]) - 1;
    
    if (player) {
      const myPlayerIndex = playerNames.findIndex(name => name === player.username);
      
      if (myPlayerIndex !== -1) {
        // Current player is in the game - use their color preference
        const color = getPlayerColor(entityPlayerIndex, player.color, myPlayerIndex);
        return color.hex;
      }
    }
    
    // Spectator or player not in game - assign colors based on player index
    // Use a consistent color assignment for spectators
    const colorIndex = entityPlayerIndex % PLAYER_COLORS.length;
    return PLAYER_COLORS[colorIndex].hex;
  };
};

function Zone({ zoneId, boardData, isMyTurn, onCellClick, entityDefinitions, zoneMetadata, isSingleZone = false, playerNames, possibleVerbs }: ZoneProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [cellSize, setCellSize] = useState(60); // Start with a reasonable default
  const lastContainerWidth = useRef<number>(0);
  const getMarkColor = useMarkColor();
  const { player } = usePlayer();
  
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
  
  // Get glyph or token info from entity definitions
  const getEntityDisplay = (cell: string) => {
    const entity = entityDefinitions?.find(e => e.id === cell);
    if (entity?.ui?.tokenType) {
      return { type: 'token', tokenType: entity.ui.tokenType };
    }
    return { type: 'glyph', glyph: entity?.ui?.glyph || (cell === 'mark_p1' ? 'X' : 'O') };
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
                
                // Check if this cell is a valid move based on possibleVerbs
                let isClickable = false;
                if (isMyTurn && isEmpty && (zoneId === 'board' || zoneId === 'da-board')) {
                  if (possibleVerbs !== undefined) {
                    // If possibleVerbs is provided (even if empty), use it
                    isClickable = possibleVerbs.some(verb => 
                      verb.validOptions?.some(opt => 
                        opt.zone === zoneId && opt.row === rowIndex && opt.col === colIndex
                      )
                    );
                  } else {
                    // Only fall back to old logic if possibleVerbs is not provided at all
                    isClickable = true;
                  }
                }
                
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
                          backgroundImage: isClickable && player ? 
                            `radial-gradient(circle, ${getColorById(player.color).hex} 1px, transparent 1px)` : 
                            'none',
                          backgroundSize: '10px 10px',
                          backgroundColor: isClickable ? '#1a202c' : '#000000'
                        }}
                        onClick={() => handleCellClick(rowIndex, colIndex)}
                        onMouseEnter={(e) => {
                          if (isClickable) {
                            e.currentTarget.style.backgroundColor = 'rgba(75, 85, 99, 0.5)';
                          }
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.backgroundColor = isClickable ? '#1a202c' : '#000000';
                        }}
                      />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center bg-black">
                        {(() => {
                          const display = getEntityDisplay(cell);
                          const color = getMarkColor(cell, playerNames);
                          
                          if (display.type === 'token') {
                            // Render SVG token
                            const tokenSize = cellSize * 0.525; // 52.5% of cell size (reduced by 25%)
                            return (
                              <div 
                                style={{ 
                                  width: `${tokenSize}px`, 
                                  height: `${tokenSize}px`,
                                  backgroundColor: color,
                                  maskImage: `url(/tokens/token_${display.tokenType}.svg)`,
                                  maskSize: 'contain',
                                  maskRepeat: 'no-repeat',
                                  maskPosition: 'center',
                                  WebkitMaskImage: `url(/tokens/token_${display.tokenType}.svg)`,
                                  WebkitMaskSize: 'contain',
                                  WebkitMaskRepeat: 'no-repeat',
                                  WebkitMaskPosition: 'center'
                                }}
                              />
                            );
                          } else {
                            // Render text glyph
                            return (
                              <span 
                                className={`${fontSize} font-bold`}
                                style={{ color }}
                              >
                                {display.glyph}
                              </span>
                            );
                          }
                        })()}
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
  zoneMetadata,
  playerNames,
  possibleVerbs
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
          playerNames={playerNames}
          possibleVerbs={possibleVerbs}
        />
      ))}
    </div>
  );
}
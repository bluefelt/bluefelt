import { useRef, useEffect, useLayoutEffect, useState } from 'react';
import { usePlayer } from '../../context/PlayerContext';
import { useMarkColor } from '../../hooks/useMarkColor';
import BoardCell from './BoardCell';

interface BoardZoneProps {
  zoneId: string;
  boardData: any[][];
  isMyTurn: boolean;
  onCellClick?: (row: number, col: number) => void;
  entityDefinitions?: any[];
  zoneMetadata?: any[];
  isSingleZone?: boolean;
  playerNames?: string[];
  actionMap?: Record<string, any>;
  selection?: any;
}

export default function BoardZone({
  zoneId,
  boardData,
  isMyTurn,
  onCellClick,
  entityDefinitions,
  zoneMetadata,
  playerNames,
  actionMap,
  selection
}: BoardZoneProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  
  // Detect if this zone has column-based actions
  const columnActions = Object.keys(actionMap || {})
    .filter(path => path.includes(`/zones/${zoneId}/columns/`))
    .map(path => {
      const match = path.match(/\/zones\/[^/]+\/columns\/(\d+)/);
      return match ? parseInt(match[1]) : -1;
    })
    .filter(col => col >= 0);
  const [cellSize, setCellSize] = useState(60);
  const lastContainerWidth = useRef<number>(0);
  const getMarkColor = useMarkColor();
  const { player } = usePlayer();
  
  const rows = boardData.length;
  const cols = boardData[0].length;
  const maxCellSize = 100;
  const isTouchDevice = 'ontouchstart' in window || navigator.maxTouchPoints > 0;
  const minCellSize = isTouchDevice ? 48 : 40;
  
  // Get zone info from metadata
  const zoneInfo = zoneMetadata?.find(z => z.id === zoneId);
  
  // Check if board should be rotated for player 2
  const shouldRotate = zoneInfo?.ui?.rotateForPlayer && player && playerNames && 
    playerNames.findIndex(name => name === player?.username) === 1;
  
  // Use layout effect for initial size calculation
  useLayoutEffect(() => {
    const updateCellSize = () => {
      if (!containerRef.current) return;
      
      const container = containerRef.current;
      const parentElement = container.parentElement;
      if (!parentElement) return;
      
      const parentWidth = parentElement.clientWidth;
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
      
      let optimalCellSize;
      if (maxCellWidth < minCellSize) {
        optimalCellSize = Math.min(minCellSize, maxCellHeight);
      } else {
        optimalCellSize = Math.min(maxCellWidth, maxCellHeight);
      }
      
      const newCellSize = Math.max(minCellSize, Math.min(maxCellSize, optimalCellSize));
      setCellSize(newCellSize);
    };
    
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
      
      if (Math.abs(parentWidth - lastContainerWidth.current) < 20) {
        return;
      }
      lastContainerWidth.current = parentWidth;
      
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
      
      let optimalCellSize;
      if (maxCellWidth < minCellSize) {
        optimalCellSize = Math.min(minCellSize, maxCellHeight);
      } else {
        optimalCellSize = Math.min(maxCellWidth, maxCellHeight);
      }
      
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
  
  // Get glyph or token info from entity definitions
  const getEntityDisplay = (cell: any) => {
    let entityId: string;
    if (typeof cell === 'string') {
      entityId = cell;
    } else if (cell && typeof cell === 'object' && cell.entity) {
      entityId = cell.entity;
    } else {
      return { type: 'glyph', glyph: '?' };
    }
    
    const entity = entityDefinitions?.find(e => e.id === entityId);
    if (entity?.ui?.tokenType) {
      return { type: 'token', tokenType: entity.ui.tokenType };
    }
    return { 
      type: 'glyph', 
      glyph: entity?.ui?.glyph || (entityId === 'stone_p1' ? 'X' : entityId === 'stone_p2' ? 'O' : (entityId === 'mark_p1' ? 'X' : 'O')) 
    };
  };
  
  // Handle cell clicks for this specific zone
  const handleCellClick = (row: number, col: number) => {
    if (isMyTurn && onCellClick) {
      onCellClick(row, col);
    }
  };

  // Get zone name from metadata if available
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
  
  // Check if zone uses checkerboard pattern
  const useCheckerPattern = zoneInfo?.ui?.checkerPattern || false;
  
  return (
    <div 
      ref={containerRef} 
      className="bg-gray-800 p-6 rounded-lg flex flex-col h-full min-h-[250px] w-full"
    >
      <h2 className="text-xl font-semibold mb-4 text-white">
        {zoneName}
      </h2>
      <div className="flex-1 overflow-auto p-4 scrollbar-thin scrollbar-thumb-gray-600 scrollbar-track-gray-800" 
           style={{ 
             WebkitOverflowScrolling: 'touch'
           }}>
          <div className="bg-black p-4 rounded inline-block">
          {/* Column drop zones for gravity-based games */}
          {columnActions.length > 0 && (
            <div 
              className="grid gap-0 mb-2"
              style={{
                gridTemplateColumns: `repeat(${cols}, ${cellSize}px)`,
                width: 'max-content',
                height: '40px'
              }}
            >
              {Array.from({ length: cols }, (_, colIndex) => {
                const isClickableColumn = columnActions.includes(colIndex);
                const columnPath = `/zones/${zoneId}/columns/${colIndex}`;
                const columnAction = actionMap?.[columnPath];
                
                return (
                  <div
                    key={`column-${colIndex}`}
                    className={`
                      border border-gray-600 flex items-center justify-center text-xs font-medium
                      transition-all duration-200 relative overflow-hidden
                      ${isClickableColumn && isMyTurn 
                        ? 'bg-blue-600 hover:bg-blue-500 cursor-pointer text-white' 
                        : 'bg-gray-700 text-gray-400 cursor-not-allowed'
                      }
                    `}
                    onClick={() => {
                      if (isClickableColumn && isMyTurn && onCellClick) {
                        onCellClick(-1, colIndex); // -1 indicates column action
                      }
                    }}
                    title={columnAction?.direction || `Column ${colIndex + 1}`}
                    style={{
                      width: `${cellSize}px`,
                      height: '40px'
                    }}
                  >
                    {isClickableColumn && isMyTurn && (
                      <>
                        <span className="relative z-10">↓</span>
                        <div className="absolute inset-0 bg-gradient-to-b from-blue-400 to-blue-600 opacity-20"></div>
                      </>
                    )}
                    {!isClickableColumn && (
                      <span className="text-gray-500">{colIndex + 1}</span>
                    )}
                  </div>
                );
              })}
            </div>
          )}
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
              row.map((_cell: any, colIndex: number) => {
                // Get the actual cell data considering rotation
                const actualRow = shouldRotate ? (rows - 1 - rowIndex) : rowIndex;
                const actualCol = shouldRotate ? (cols - 1 - colIndex) : colIndex;
                const actualCell = boardData[actualRow][actualCol];
                
                const isEmpty = actualCell === null;
                
                // Check if this cell is selected
                const isSelected = selection && 
                  selection.zone === zoneId && 
                  selection.row === actualRow && 
                  selection.col === actualCol;
                
                // Check if this cell is a valid move based on actionMap
                let isClickable = false;
                if (isMyTurn) {
                  if (actionMap !== undefined) {
                    const location = `/zones/${zoneId}/cells/${actualRow}/${actualCol}`;
                    isClickable = location in actionMap;
                    if (!isClickable) {
                      const altLocation = `/zones/${zoneId}/${actualRow}/${actualCol}`;
                      isClickable = altLocation in actionMap;
                    }
                  } else if (isEmpty && (zoneId === 'board' || zoneId === 'da-board')) {
                    isClickable = true;
                  }
                }
                
                const isDarkSquare = (rowIndex + colIndex) % 2 === 1;
                
                return (
                  <BoardCell
                    key={`${rowIndex}-${colIndex}`}
                    cell={actualCell}
                    row={actualRow}
                    col={actualCol}
                    isClickable={isClickable}
                    cellSize={cellSize}
                    isDarkSquare={isDarkSquare}
                    useCheckerPattern={useCheckerPattern}
                    isSelected={isSelected}
                    entityDisplay={getEntityDisplay(actualCell)}
                    markColor={getMarkColor(actualCell, playerNames)}
                    onCellClick={handleCellClick}
                  />
                );
              })
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
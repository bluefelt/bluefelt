import { useRef, useEffect, useLayoutEffect, useState } from 'react';
import { usePlayer } from '../../context/PlayerContext';
import { useMarkColor } from '../../hooks/useMarkColor';
import BoardCell from './BoardCell';
import { useHasAction } from '../ActionIndicator';
import { getCellActionLocation, hasActionAtLocation } from '../../utils/actionMapUtils';
import MultiStepSelectionOverlay from '../MultiStepSelectionOverlay';

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
  playerPreferences?: Record<string, any>;
  multiStepState?: any;
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
  selection,
  playerPreferences,
  multiStepState
}: BoardZoneProps) {
  console.log(`[BoardZone] Rendering zone ${zoneId}, boardData:`, boardData);
  console.log(`[BoardZone] Entity definitions:`, entityDefinitions);
  
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
    console.log(`[BoardZone] getEntityDisplay called with cell:`, cell);
    
    // Handle null cells (empty)
    if (cell === null || cell === undefined) {
      console.log(`[BoardZone] Cell is null/undefined, returning empty`);
      return { type: 'glyph', glyph: '' };
    }
    
    let entityId: string;
    if (typeof cell === 'string') {
      entityId = cell;
      console.log(`[BoardZone] Cell is string, using as entityId: "${entityId}"`);
    } else if (cell && typeof cell === 'object' && cell.entity) {
      entityId = cell.entity;
      console.log(`[BoardZone] Cell is object, extracted entityId: "${entityId}"`);
    } else {
      console.log(`[BoardZone] Cell format not recognized, returning empty`);
      return { type: 'glyph', glyph: '' };
    }
    
    console.log(`[BoardZone] Looking up entity "${entityId}" in entityDefinitions:`, entityDefinitions);
    console.log(`[BoardZone] Available entity IDs:`, entityDefinitions?.map(e => e.id));
    
    const entity = entityDefinitions?.find(e => e.id === entityId);
    console.log(`[BoardZone] Found entity definition:`, entity);
    
    if (entity?.ui?.tokenType) {
      console.log(`[BoardZone] Entity has tokenType, returning token display`);
      return { type: 'token', tokenType: entity.ui.tokenType };
    }
    
    const glyph = entity?.ui?.glyph || entity?.ui?.display || entity?.props?.symbol || '?';
    console.log(`[BoardZone] Returning glyph "${glyph}" for entity "${entityId}"`);
    
    return { 
      type: 'glyph', 
      glyph: glyph
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
      data-testid={`${zoneId}-zone`}
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
            className="grid gap-0 border-2 border-white relative"
            style={{
              gridTemplateColumns: `repeat(${cols}, ${cellSize}px)`,
              gridTemplateRows: `repeat(${rows}, ${cellSize}px)`,
              width: 'max-content',
              height: 'max-content'
            }}
          >
            {/* Multi-step selection overlay */}
            <MultiStepSelectionOverlay
              multiStepState={multiStepState}
              zoneId={zoneId}
              cellSize={cellSize}
              rows={rows}
              cols={cols}
            />
            
            {boardData.map((row, rowIndex) =>
              row.map((_cell: any, colIndex: number) => {
                // Get the actual cell data considering rotation
                const actualRow = shouldRotate ? (rows - 1 - rowIndex) : rowIndex;
                const actualCol = shouldRotate ? (cols - 1 - colIndex) : colIndex;
                const actualCell = boardData[actualRow][actualCol];
                
                const isEmpty = actualCell === null;
                
                // Check if this cell is selected
                // Handle both formats: old format {zone, row, col} and new server format {player: {location, entity}}
                let isSelected = false;
                if (selection) {
                  // Old format (for backwards compatibility with tests)
                  if (selection.zone === zoneId && selection.row === actualRow && selection.col === actualCol) {
                    isSelected = true;
                  } else {
                    // New server format: selection[playerId] = {location, entity}
                    // Check all players' selections for the current cell location
                    const currentLocation = `/zones/${zoneId}/cells/${actualRow}/${actualCol}`;
                    const altLocation = `/zones/${zoneId}/${actualRow}/${actualCol}`;
                    
                    for (const [playerId, playerSelection] of Object.entries(selection)) {
                      if (playerSelection && typeof playerSelection === 'object' && 'location' in playerSelection) {
                        const selectionLocation = (playerSelection as any).location;
                        if (selectionLocation === currentLocation || selectionLocation === altLocation) {
                          isSelected = true;
                          break;
                        }
                      }
                    }
                  }
                }
                
                // Check if this cell is a valid move based on actionMap or multiStepState
                let isClickable = false;
                if (isMyTurn) {
                  const location = `/zones/${zoneId}/cells/${actualRow}/${actualCol}`;
                  const altLocation = `/zones/${zoneId}/${actualRow}/${actualCol}`;
                  
                  // Check regular action map
                  if (actionMap !== undefined) {
                    isClickable = location in actionMap || altLocation in actionMap;
                  }
                  
                  // Also check multi-step action map if in multi-step mode
                  if (!isClickable && multiStepState?.stepActionMap) {
                    isClickable = location in multiStepState.stepActionMap || altLocation in multiStepState.stepActionMap;
                    if (isClickable) {
                      console.log(`[BoardZone] Cell (${actualRow},${actualCol}) is clickable via multi-step action map`);
                    }
                  }
                  
                  if (isClickable) {
                    console.log(`[BoardZone] Cell (${actualRow},${actualCol}) is clickable via location: ${location}`);
                  }
                }
                
                const isDarkSquare = (rowIndex + colIndex) % 2 === 1;
                
                // Check for multi-step action
                const cellLocation = `/zones/${zoneId}/cells/${actualRow}/${actualCol}`;
                const { hasAction, isMultiStepAction, multiStepIndicatorState, stepNumber } = useHasAction(
                  cellLocation,
                  actionMap || {},
                  Boolean(multiStepState),
                  multiStepState
                );
                
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
                    hasAction={hasAction}
                    isMultiStepAction={isMultiStepAction}
                    multiStepIndicatorState={multiStepIndicatorState}
                    stepNumber={stepNumber}
                    onCellClick={handleCellClick}
                    zoneId={zoneId}
                    playerPreferences={playerPreferences}
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
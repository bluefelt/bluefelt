import { useMemo } from 'react';
import { usePlayer } from '../../context/PlayerContext';
import { getColorById } from '../../config/colors';
import { useAnimationsEnabled } from '../../context/AnimationContext';
import TokenDisplay from '../TokenDisplay';
import { ActionIndicator } from '../ActionIndicator';

export interface BoardCellProps {
  cell: any;
  row: number;
  col: number;
  isClickable: boolean;
  cellSize: number;
  isDarkSquare: boolean;
  useCheckerPattern: boolean;
  isSelected: boolean;
  entityDisplay: { type: 'token' | 'glyph'; glyph?: string; tokenType?: string };
  markColor: string;
  onCellClick: (row: number, col: number) => void;
  zoneId?: string; // Add zone ID for animation targeting
  playerPreferences?: Record<string, any>;
  hasAction?: boolean;
  isMultiStepAction?: boolean;
  multiStepIndicatorState?: 'available' | 'current_step' | 'next_step' | 'selected' | 'confirmed';
  stepNumber?: number;
}

export default function BoardCell({
  cell,
  row,
  col,
  isClickable,
  cellSize,
  isDarkSquare,
  useCheckerPattern,
  isSelected,
  entityDisplay,
  markColor,
  onCellClick,
  zoneId = 'board',
  playerPreferences,
  hasAction = false,
  isMultiStepAction = false,
  multiStepIndicatorState = 'available',
  stepNumber
}: BoardCellProps) {
  const { player } = usePlayer();
  const animationsEnabled = useAnimationsEnabled();
  const isEmpty = cell === null;
  
  // Calculate cell background color
  const cellBgColor = useMemo(() => {
    if (useCheckerPattern) {
      return isDarkSquare ? '#000000' : '#1a1a1a';
    }
    return '#000000';
  }, [useCheckerPattern, isDarkSquare]);

  // Enhanced border styling for multi-step actions
  const getBorderClasses = () => {
    if (isSelected) {
      if (isMultiStepAction) {
        switch (multiStepIndicatorState) {
          case 'selected':
            return 'border-green-400 border-3 shadow-lg shadow-green-400/30';
          case 'confirmed':
            return 'border-green-500 border-3 shadow-lg shadow-green-500/40';
          case 'current_step':
            return 'border-yellow-400 border-3 shadow-lg shadow-yellow-400/40 animate-pulse';
          default:
            return 'border-yellow-400 border-2';
        }
      }
      return 'border-yellow-400 border-2';
    }
    
    // Special borders for multi-step action states
    if (isMultiStepAction && hasAction) {
      switch (multiStepIndicatorState) {
        case 'current_step':
          return 'border-yellow-300 border-2 border-dashed';
        case 'next_step':
          return 'border-blue-300 border border-dashed';
        default:
          return 'border-gray-700';
      }
    }
    
    return 'border-gray-700';
  };

  const handleClick = () => {
    if (isClickable) {
      onCellClick(row, col);
    }
  };

  // Calculate font size based on cell size
  const fontSize = cellSize >= 80 ? 'text-5xl' : cellSize >= 60 ? 'text-4xl' : 'text-2xl';

  return (
    <div
      className={`relative border ${getBorderClasses()}`}
      style={{ width: `${cellSize}px`, height: `${cellSize}px` }}
      data-testid={`cell-${row}-${col}`}
      data-zone={zoneId}
      data-row={row}
      data-col={col}
      data-game-container={row === 0 && col === 0 ? 'true' : undefined}
    >
      <ActionIndicator 
        hasAction={hasAction} 
        isMultiStep={isMultiStepAction}
        multiStepState={multiStepIndicatorState}
        stepNumber={stepNumber}
      />
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
            backgroundColor: isClickable ? 
              (useCheckerPattern ? (isDarkSquare ? '#0a0a0a' : '#2a2a2a') : '#1a202c') : 
              cellBgColor
          }}
          onClick={handleClick}
          role={isClickable ? "button" : undefined}
          tabIndex={isClickable ? 0 : undefined}
          onMouseEnter={(e) => {
            if (isClickable) {
              e.currentTarget.style.backgroundColor = 'rgba(75, 85, 99, 0.5)';
            }
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = isClickable ? 
              (useCheckerPattern ? (isDarkSquare ? '#0a0a0a' : '#2a2a2a') : '#1a202c') : 
              cellBgColor;
          }}
        />
      ) : (
        <div 
          className={`w-full h-full flex items-center justify-center relative ${
            isClickable ? 'cursor-pointer' : ''
          }`}
          style={{ 
            backgroundColor: cellBgColor,
            backgroundImage: isClickable && player && !isSelected ? 
              `radial-gradient(circle, ${getColorById(player.color).hex} 1px, transparent 1px)` : 
              'none',
            backgroundSize: '10px 10px',
          }}
          onClick={handleClick}
          role={isClickable ? "button" : undefined}
          tabIndex={isClickable ? 0 : undefined}
        >
          <div 
            className="absolute inset-0 flex items-center justify-center pointer-events-none"
            data-entity-display="true"
            data-zone={zoneId}
            data-row={row}
            data-col={col}
            style={{
              // Ensure the entity display is isolated from the cell background
              zIndex: 10,
              isolation: 'isolate'
            }}
          >
            {entityDisplay.type === 'token' ? (
              <TokenDisplay 
                tokenType={entityDisplay.tokenType!} 
                cellSize={cellSize} 
                color={markColor}
                entityId={cell?.entity}
                playerPreferences={playerPreferences}
              />
            ) : (
              <span 
                className={`${fontSize} font-bold relative`}
                style={{ color: markColor }}
              >
                {entityDisplay.glyph}
              </span>
            )}
          </div>
        </div>
      )}
    </div>
  );
}


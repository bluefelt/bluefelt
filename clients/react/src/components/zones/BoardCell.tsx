import { useMemo } from 'react';
import { usePlayer } from '../../context/PlayerContext';
import { getColorById } from '../../config/colors';

interface BoardCellProps {
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
  onCellClick
}: BoardCellProps) {
  const { player } = usePlayer();
  const isEmpty = cell === null;
  
  // Calculate cell background color
  const cellBgColor = useMemo(() => {
    if (useCheckerPattern) {
      return isDarkSquare ? '#000000' : '#1a1a1a';
    }
    return '#000000';
  }, [useCheckerPattern, isDarkSquare]);

  const handleClick = () => {
    if (isClickable) {
      onCellClick(row, col);
    }
  };

  // Calculate font size based on cell size
  const fontSize = cellSize >= 80 ? 'text-5xl' : cellSize >= 60 ? 'text-4xl' : 'text-2xl';

  return (
    <div
      className={`relative border ${isSelected ? 'border-yellow-400 border-2' : 'border-gray-700'}`}
      style={{ width: `${cellSize}px`, height: `${cellSize}px` }}
      data-testid={`cell-${row}-${col}`}
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
          {entityDisplay.type === 'token' ? (
            <TokenDisplay tokenType={entityDisplay.tokenType!} cellSize={cellSize} color={markColor} />
          ) : (
            <span 
              className={`${fontSize} font-bold relative z-10`}
              style={{ color: markColor }}
            >
              {entityDisplay.glyph}
            </span>
          )}
        </div>
      )}
    </div>
  );
}

// Helper component for token display
function TokenDisplay({ tokenType, cellSize, color }: { tokenType: string; cellSize: number; color: string }) {
  const tokenSize = cellSize * 0.7;
  
  // Helper function to calculate hue rotation for color filters
  const getHueRotation = (targetColor: string): number => {
    const hex = targetColor.replace('#', '');
    const r = parseInt(hex.substr(0, 2), 16) / 255;
    const g = parseInt(hex.substr(2, 2), 16) / 255;
    const b = parseInt(hex.substr(4, 2), 16) / 255;
    
    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    const diff = max - min;
    
    let h = 0;
    if (diff !== 0) {
      switch (max) {
        case r: h = ((g - b) / diff + (g < b ? 6 : 0)) / 6; break;
        case g: h = ((b - r) / diff + 2) / 6; break;
        case b: h = ((r - g) / diff + 4) / 6; break;
      }
    }
    
    const targetHue = h * 360;
    const baseHue = 320; // Base hue for the token SVGs
    
    let rotation = targetHue - baseHue;
    while (rotation > 180) rotation -= 360;
    while (rotation < -180) rotation += 360;
    
    return rotation;
  };

  return (
    <div 
      className="relative z-10"
      style={{ 
        width: `${tokenSize}px`, 
        height: `${tokenSize}px`,
        backgroundImage: `url(/tokens/token_${tokenType}.svg)`,
        backgroundSize: 'contain',
        backgroundRepeat: 'no-repeat',
        backgroundPosition: 'center',
        filter: `hue-rotate(${getHueRotation(color)}deg) saturate(1.2)`
      }}
    />
  );
}
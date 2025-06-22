import React, { useRef, useLayoutEffect, useState } from 'react';
import { usePlayer } from '../../context/PlayerContext';
import { useMarkColor } from '../../hooks/useMarkColor';
import HexCell from './HexCell';

interface HexBoardProps {
  zoneId: string;
  hexData: { [key: string]: any }; // hex cells keyed by "q,r" coordinates
  layout: 'flat' | 'pointy';
  isMyTurn: boolean;
  onCellClick?: (row: number, col: number) => void;
  onHexClick?: (q: number, r: number) => void;
  entityDefinitions?: any[];
  zoneMetadata?: any[];
  playerNames?: string[];
  actionMap?: Record<string, any>;
  selection?: any;
}

// Helper functions for hex coordinate conversion and layout
function parseHexCoord(coordString: string): { q: number; r: number } {
  const parts = coordString.split(',');
  return { q: parseInt(parts[0]), r: parseInt(parts[1]) };
}

function hexToPixel(q: number, r: number, size: number, layout: 'flat' | 'pointy'): { x: number; y: number } {
  const sqrt3 = Math.sqrt(3);
  
  if (layout === 'flat') {
    const x = size * (3/2) * q;
    const y = size * sqrt3 * (r + q/2);
    return { x, y };
  } else {
    const x = size * sqrt3 * (q + r/2);
    const y = size * (3/2) * r;
    return { x, y };
  }
}

function getBoundingBox(hexData: { [key: string]: any }): { minQ: number; maxQ: number; minR: number; maxR: number } {
  const coords = Object.keys(hexData).map(parseHexCoord);
  
  if (coords.length === 0) {
    return { minQ: 0, maxQ: 0, minR: 0, maxR: 0 };
  }
  
  return {
    minQ: Math.min(...coords.map(c => c.q)),
    maxQ: Math.max(...coords.map(c => c.q)),
    minR: Math.min(...coords.map(c => c.r)),
    maxR: Math.max(...coords.map(c => c.r))
  };
}

export default function HexBoard({
  zoneId,
  hexData,
  layout = 'flat',
  isMyTurn,
  onCellClick,
  onHexClick,
  entityDefinitions,
  zoneMetadata,
  playerNames,
  actionMap,
  selection
}: HexBoardProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [hexSize, setHexSize] = useState(30);
  const getMarkColor = useMarkColor();
  const { player } = usePlayer();
  
  // Get zone info from metadata
  const zoneInfo = zoneMetadata?.find(z => z.id === zoneId);
  
  // Calculate bounding box for all hexes
  const boundingBox = getBoundingBox(hexData);
  const hexCoords = Object.keys(hexData).map(parseHexCoord);
  
  // Calculate layout dimensions
  useLayoutEffect(() => {
    const updateHexSize = () => {
      if (!containerRef.current) return;
      
      const container = containerRef.current;
      const parentElement = container.parentElement;
      if (!parentElement) return;
      
      const parentWidth = parentElement.clientWidth;
      const parentHeight = window.innerHeight * 0.6; // Use 60% of viewport height
      
      // Calculate the pixel extent of all hexes
      let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
      
      hexCoords.forEach(({ q, r }) => {
        const { x, y } = hexToPixel(q, r, 1, layout); // Use size 1 for calculation
        minX = Math.min(minX, x);
        maxX = Math.max(maxX, x);
        minY = Math.min(minY, y);
        maxY = Math.max(maxY, y);
      });
      
      // Add some padding and account for hex radius
      const padding = 100;
      const gridWidth = maxX - minX + 2; // +2 for hex radius on each side
      const gridHeight = maxY - minY + 2;
      
      // Calculate size that fits both width and height constraints
      const maxSizeForWidth = Math.max(20, (parentWidth - padding) / gridWidth);
      const maxSizeForHeight = Math.max(20, (parentHeight - padding) / gridHeight);
      const newSize = Math.min(Math.min(maxSizeForWidth, maxSizeForHeight), 60);
      
      setHexSize(newSize);
    };
    
    updateHexSize();
    window.addEventListener('resize', updateHexSize);
    return () => window.removeEventListener('resize', updateHexSize);
  }, [hexCoords, layout]);
  
  // Calculate SVG dimensions
  const svgPadding = hexSize;
  let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
  
  hexCoords.forEach(({ q, r }) => {
    const { x, y } = hexToPixel(q, r, hexSize, layout);
    minX = Math.min(minX, x);
    maxX = Math.max(maxX, x);
    minY = Math.min(minY, y);
    maxY = Math.max(maxY, y);
  });
  
  const svgWidth = maxX - minX + 2 * svgPadding;
  const svgHeight = maxY - minY + 2 * svgPadding;
  const offsetX = -minX + svgPadding;
  const offsetY = -minY + svgPadding;
  
  // Handle hex click
  const handleHexClick = (q: number, r: number) => {
    console.log(`[HexBoard] Hex clicked: q=${q}, r=${r}`);
    
    // Call hex-specific handler if provided
    if (onHexClick) {
      onHexClick(q, r);
      return;
    }
    
    // Fall back to grid cell click handler
    // For compatibility, we can map hex coordinates to row/col
    // This is a simple mapping - more sophisticated games might need custom logic
    if (onCellClick) {
      onCellClick(r, q); // Map r to row, q to col
    }
  };
  
  // Check if a hex has an action available
  const hasHexAction = (q: number, r: number): boolean => {
    const hexPath = `/zones/${zoneId}/${q},${r}`;
    return actionMap?.[hexPath] !== undefined;
  };
  
  console.log(`[HexBoard] Rendering hex board ${zoneId}:`, {
    layout,
    hexCount: hexCoords.length,
    hexSize,
    svgDimensions: { width: svgWidth, height: svgHeight },
    boundingBox
  });
  
  return (
    <div 
      ref={containerRef}
      className="hex-board-container w-full flex flex-col items-center"
      data-testid={`hex-board-${zoneId}`}
    >
      {/* Zone title */}
      {zoneInfo?.name && (
        <h3 className="text-lg font-semibold mb-4 text-center">
          {zoneInfo.name}
        </h3>
      )}
      
      {/* SVG container for hex grid */}
      <div className="hex-board-svg-container">
        <svg
          width={svgWidth}
          height={svgHeight}
          viewBox={`0 0 ${svgWidth} ${svgHeight}`}
          className="hex-board-svg"
          style={{ maxWidth: '100%', height: 'auto' }}
        >
          {/* Render each hex */}
          {Object.entries(hexData).map(([coordStr, cellData]) => {
            const { q, r } = parseHexCoord(coordStr);
            const { x, y } = hexToPixel(q, r, hexSize, layout);
            const pixelX = x + offsetX;
            const pixelY = y + offsetY;
            
            return (
              <HexCell
                key={coordStr}
                q={q}
                r={r}
                x={pixelX}
                y={pixelY}
                size={hexSize}
                layout={layout}
                cellData={cellData}
                isMyTurn={isMyTurn}
                hasAction={hasHexAction(q, r)}
                entityDefinitions={entityDefinitions}
                onClick={() => handleHexClick(q, r)}
                getMarkColor={getMarkColor}
                selection={selection}
                zoneId={zoneId}
              />
            );
          })}
        </svg>
      </div>
      
      {/* Debug info (only in development) */}
      {process.env.NODE_ENV === 'development' && (
        <div className="text-xs text-gray-500 mt-2 text-center">
          Hex Grid: {hexCoords.length} cells, {layout} layout, size {hexSize.toFixed(1)}px
        </div>
      )}
    </div>
  );
}
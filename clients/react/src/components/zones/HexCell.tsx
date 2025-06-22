import React from 'react';

interface HexCellProps {
  q: number;
  r: number;
  x: number;
  y: number;
  size: number;
  layout: 'flat' | 'pointy';
  cellData: any;
  isMyTurn: boolean;
  hasAction: boolean;
  entityDefinitions?: any[];
  onClick: () => void;
  getMarkColor?: (entityId: string) => string;
  selection?: any;
  zoneId: string;
}

function generateHexPoints(size: number, layout: 'flat' | 'pointy'): string {
  const sqrt3 = Math.sqrt(3);
  const points: { x: number; y: number }[] = [];
  
  // Calculate the 6 corners of the hexagon
  for (let i = 0; i < 6; i++) {
    const angleDeg = layout === 'flat' ? 60 * i : 60 * i + 30;
    const angleRad = (Math.PI / 180) * angleDeg;
    points.push({
      x: size * Math.cos(angleRad),
      y: size * Math.sin(angleRad)
    });
  }
  
  return points.map(p => `${p.x},${p.y}`).join(' ');
}

export default function HexCell({
  q,
  r,
  x,
  y,
  size,
  layout,
  cellData,
  isMyTurn,
  hasAction,
  entityDefinitions,
  onClick,
  getMarkColor,
  selection,
  zoneId
}: HexCellProps) {
  
  // Parse cell data to get entity information
  const entityId = cellData?.entity || (typeof cellData === 'string' ? cellData : null);
  const entity = entityId ? entityDefinitions?.find(e => e.id === entityId) : null;
  
  // Check if this hex is selected
  const coordKey = `${q},${r}`;
  const isSelected = selection && selection[zoneId] && selection[zoneId][coordKey];
  
  // Determine hex colors and styling
  const isEmpty = !cellData || cellData === null;
  const isClickable = isMyTurn && hasAction;
  
  let fillColor = '#f8f9fa'; // Light gray for empty cells
  let strokeColor = '#dee2e6'; // Gray border
  let strokeWidth = 1;
  
  if (isSelected) {
    strokeColor = '#007bff'; // Blue for selected
    strokeWidth = 3;
  } else if (isClickable) {
    strokeColor = '#28a745'; // Green for actionable
    strokeWidth = 2;
  }
  
  // Color for entity marks (like player pieces)
  if (entity && getMarkColor) {
    fillColor = getMarkColor(entity.id);
  } else if (entityId && entity?.color) {
    fillColor = entity.color;
  } else if (!isEmpty) {
    fillColor = '#e9ecef'; // Slightly darker for occupied cells
  }
  
  const hexPoints = generateHexPoints(size * 0.9, layout); // Use 0.9 to create small gaps between hexes
  
  // Handle hover effects
  const handleMouseEnter = (e: React.MouseEvent) => {
    if (isClickable) {
      const polygon = e.currentTarget as SVGPolygonElement;
      polygon.style.opacity = '0.8';
    }
  };
  
  const handleMouseLeave = (e: React.MouseEvent) => {
    const polygon = e.currentTarget as SVGPolygonElement;
    polygon.style.opacity = '1';
  };
  
  return (
    <g 
      className={`hex-cell ${isClickable ? 'cursor-pointer' : ''}`}
      data-testid={`hex-cell-${q}-${r}`}
    >
      {/* Hex background */}
      <polygon
        points={hexPoints}
        fill={fillColor}
        stroke={strokeColor}
        strokeWidth={strokeWidth}
        transform={`translate(${x}, ${y})`}
        onClick={isClickable ? onClick : undefined}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        style={{ 
          transition: 'opacity 0.1s ease',
          filter: isSelected ? 'brightness(1.1)' : 'none'
        }}
      />
      
      {/* Entity content */}
      {entity && (
        <g transform={`translate(${x}, ${y})`}>
          {/* Entity symbol or text */}
          {entity.symbol ? (
            <text
              x="0"
              y="0"
              textAnchor="middle"
              dominantBaseline="central"
              fontSize={Math.min(size * 0.6, 24)}
              fill={entity.textColor || '#000'}
              fontWeight="bold"
              style={{ userSelect: 'none', pointerEvents: 'none' }}
            >
              {entity.symbol}
            </text>
          ) : entity.name ? (
            <text
              x="0"
              y="0"
              textAnchor="middle"
              dominantBaseline="central"
              fontSize={Math.min(size * 0.3, 12)}
              fill={entity.textColor || '#000'}
              style={{ userSelect: 'none', pointerEvents: 'none' }}
            >
              {entity.name.length > 3 ? entity.name.substring(0, 3) : entity.name}
            </text>
          ) : (
            // Default circle for entities without symbols
            <circle
              cx="0"
              cy="0"
              r={size * 0.3}
              fill={entity.color || '#6c757d'}
              stroke="#fff"
              strokeWidth="1"
              style={{ pointerEvents: 'none' }}
            />
          )}
        </g>
      )}
      
      {/* Coordinate labels (only in development mode and for small grids) */}
      {process.env.NODE_ENV === 'development' && size > 40 && (
        <text
          x={x}
          y={y + size * 0.8}
          textAnchor="middle"
          fontSize="8"
          fill="#6c757d"
          style={{ userSelect: 'none', pointerEvents: 'none' }}
        >
          {q},{r}
        </text>
      )}
      
      {/* Action indicator */}
      {isClickable && isEmpty && (
        <circle
          cx={x}
          cy={y}
          r={size * 0.15}
          fill="#28a745"
          opacity="0.6"
          style={{ pointerEvents: 'none' }}
        />
      )}
    </g>
  );
}
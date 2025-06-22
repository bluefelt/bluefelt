import React from 'react';

interface MultiStepSelectionOverlayProps {
  multiStepState?: any;
  zoneId: string;
  cellSize: number;
  rows: number;
  cols: number;
}

export const MultiStepSelectionOverlay: React.FC<MultiStepSelectionOverlayProps> = ({
  multiStepState,
  zoneId,
  cellSize,
  rows,
  cols
}) => {
  if (!multiStepState || !multiStepState.storedData) {
    return null;
  }

  // Extract locations from stored data
  const selectedLocations: Array<{ row: number; col: number; step: string }> = [];
  
  Object.entries(multiStepState.storedData).forEach(([stepKey, value]) => {
    if (typeof value === 'string' && value.includes(`/zones/${zoneId}/`)) {
      // Parse location like "/zones/board/cells/1/2"
      const match = value.match(/\/zones\/[^/]+\/cells\/(\d+)\/(\d+)/);
      if (match) {
        selectedLocations.push({
          row: parseInt(match[1]),
          col: parseInt(match[2]),
          step: stepKey
        });
      }
    } else if (value && typeof value === 'object' && value.location) {
      const match = value.location.match(/\/zones\/[^/]+\/cells\/(\d+)\/(\d+)/);
      if (match) {
        selectedLocations.push({
          row: parseInt(match[1]),
          col: parseInt(match[2]),
          step: stepKey
        });
      }
    }
  });

  if (selectedLocations.length < 2) {
    return null; // Need at least 2 selections to draw connections
  }

  // Calculate SVG dimensions
  const gridWidth = cols * cellSize;
  const gridHeight = rows * cellSize;

  // Create connection paths between selected cells
  const connections = [];
  for (let i = 0; i < selectedLocations.length - 1; i++) {
    const from = selectedLocations[i];
    const to = selectedLocations[i + 1];

    // Calculate center points of cells
    const fromX = (from.col * cellSize) + (cellSize / 2);
    const fromY = (from.row * cellSize) + (cellSize / 2);
    const toX = (to.col * cellSize) + (cellSize / 2);
    const toY = (to.row * cellSize) + (cellSize / 2);

    connections.push({
      id: `${from.step}-to-${to.step}`,
      fromX,
      fromY,
      toX,
      toY,
      stepNumber: i + 1
    });
  }

  return (
    <div 
      className="absolute inset-0 pointer-events-none z-10"
      style={{
        width: gridWidth,
        height: gridHeight
      }}
    >
      <svg
        width={gridWidth}
        height={gridHeight}
        className="absolute inset-0"
        style={{ zIndex: 5 }}
      >
        <defs>
          {/* Gradient for connection lines */}
          <linearGradient id="connectionGradient" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" style={{ stopColor: '#10B981', stopOpacity: 0.8 }} />
            <stop offset="100%" style={{ stopColor: '#059669', stopOpacity: 0.9 }} />
          </linearGradient>
          
          {/* Arrow marker */}
          <defs>
            <marker
              id="arrowhead"
              markerWidth="10"
              markerHeight="7"
              refX="0"
              refY="3.5"
              orient="auto"
            >
              <polygon
                points="0 3.5, 10 0, 10 7"
                fill="#059669"
                opacity="0.9"
              />
            </marker>
          </defs>
        </defs>

        {connections.map((connection) => (
          <g key={connection.id}>
            {/* Main connection line */}
            <line
              x1={connection.fromX}
              y1={connection.fromY}
              x2={connection.toX}
              y2={connection.toY}
              stroke="url(#connectionGradient)"
              strokeWidth="3"
              strokeDasharray="8,4"
              markerEnd="url(#arrowhead)"
              className="animate-pulse"
            />
            
            {/* Glowing shadow line */}
            <line
              x1={connection.fromX}
              y1={connection.fromY}
              x2={connection.toX}
              y2={connection.toY}
              stroke="#10B981"
              strokeWidth="6"
              opacity="0.3"
              strokeDasharray="8,4"
              className="animate-pulse"
              style={{ animationDelay: '0.5s' }}
            />

            {/* Step number indicator at midpoint */}
            <circle
              cx={(connection.fromX + connection.toX) / 2}
              cy={(connection.fromY + connection.toY) / 2}
              r="12"
              fill="#059669"
              stroke="#ffffff"
              strokeWidth="2"
              opacity="0.9"
            />
            <text
              x={(connection.fromX + connection.toX) / 2}
              y={(connection.fromY + connection.toY) / 2}
              textAnchor="middle"
              dominantBaseline="central"
              fontSize="10"
              fontWeight="bold"
              fill="white"
            >
              {connection.stepNumber}
            </text>
          </g>
        ))}

        {/* Selection order indicators on cells */}
        {selectedLocations.map((location, index) => (
          <g key={`selection-${location.step}`}>
            <circle
              cx={(location.col * cellSize) + (cellSize / 2)}
              cy={(location.row * cellSize) + (cellSize / 2)}
              r="8"
              fill="#10B981"
              stroke="#ffffff"
              strokeWidth="2"
              opacity="0.9"
            />
            <text
              x={(location.col * cellSize) + (cellSize / 2)}
              y={(location.row * cellSize) + (cellSize / 2)}
              textAnchor="middle"
              dominantBaseline="central"
              fontSize="10"
              fontWeight="bold"
              fill="white"
            >
              {index + 1}
            </text>
          </g>
        ))}
      </svg>
    </div>
  );
};

export default MultiStepSelectionOverlay;
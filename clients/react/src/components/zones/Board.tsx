import React from 'react';
import type { EntityUI } from '../../types/game-types';
import BoardCell from './BoardCell';

interface BoardProps {
  zone: {
    cells: (string | null)[][];
  };
  zoneId: string;
  getEntityUI: (entityId: string | null) => EntityUI | null;
  onEntityClick: (entityId: string, actionId: string) => void;
  onCellClick: (row: number, col: number) => void;
  tier?: number;
}

export function Board({
  zone,
  zoneId,
  getEntityUI,
  onEntityClick,
  onCellClick,
  tier = 0
}: BoardProps) {
  const { cells } = zone;
  
  return (
    <div className={`board-zone tier-${tier}`} data-zone-id={zoneId}>
      <div className="board-grid">
        {cells.map((row, rowIndex) => (
          <div key={rowIndex} className="board-row">
            {row.map((entity, colIndex) => {
              const entityUI = entity ? getEntityUI(entity) : null;
              const hasInteractions = entityUI?.interactions.length || 0 > 0;
              const interactionStyle = entityUI?.interactions[0]?.style;
              
              return (
                <div
                  key={`${rowIndex}-${colIndex}`}
                  className={`board-cell ${hasInteractions ? 'interactive' : ''} ${interactionStyle?.highlight || ''}`}
                  style={{ cursor: interactionStyle?.cursor || 'default' }}
                  onClick={() => {
                    if (entity && entityUI?.interactions[0]) {
                      onEntityClick(entity, entityUI.interactions[0].actionId);
                    } else {
                      onCellClick(rowIndex, colIndex);
                    }
                  }}
                >
                  {entity && (
                    <BoardCell
                      entity={entity}
                      entityUI={entityUI}
                      position={[rowIndex, colIndex]}
                    />
                  )}
                </div>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}


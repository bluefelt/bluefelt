import React from 'react';
import type { GroupedAction, ActionOption } from '../ws/useLobbyWebSocket';
import type { EntityDefinition } from '../types/messages';
import { buildGlyphMapping, getEntityGlyph } from '../utils/entityUtils';

export type ZoneAction = {
  action: string;
  zone: string;
  row: number;
  col: number;
};

type InteractiveZoneProps = {
  zoneName: string;
  zoneData: (string | null)[][];
  groupedActions: GroupedAction[];
  onAction: (action: ZoneAction) => void;
  isMyTurn: boolean;
  entities?: EntityDefinition[];
};

const cellStyle: React.CSSProperties = {
  width: 60,
  height: 60,
  border: '1px solid #333',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  fontSize: 32,
  transition: 'all 0.2s ease',
};

const clickableCellStyle: React.CSSProperties = {
  ...cellStyle,
  cursor: 'pointer',
  backgroundColor: 'rgba(59, 130, 246, 0.1)',
};

const hoveredCellStyle: React.CSSProperties = {
  ...clickableCellStyle,
  backgroundColor: 'rgba(59, 130, 246, 0.3)',
  transform: 'scale(1.05)',
};

const rowStyle: React.CSSProperties = {
  display: 'flex',
};

interface CellProps {
  value: string | null;
  row: number;
  col: number;
  isClickable: boolean;
  onClick: () => void;
  glyphMapping: Map<string, string>;
}

const Cell = React.memo(function Cell({ value, row, col, isClickable, onClick, glyphMapping }: CellProps) {
  const [isHovered, setIsHovered] = React.useState(false);

  const style = isClickable 
    ? (isHovered ? hoveredCellStyle : clickableCellStyle)
    : cellStyle;

  const handleClick = React.useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    console.log('[Cell] Clicked:', { row, col, isClickable });
    if (isClickable) {
      onClick();
    }
  }, [row, col, isClickable, onClick]);

  return (
    <div 
      style={style}
      onClick={handleClick}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      title={isClickable ? `Click to place at (${row}, ${col})` : undefined}
    >
      {getEntityGlyph(value, glyphMapping)}
    </div>
  );
});

export default function InteractiveZone({ 
  zoneName, 
  zoneData, 
  groupedActions, 
  onAction,
  isMyTurn,
  entities 
}: InteractiveZoneProps) {
  if (!zoneData || !Array.isArray(zoneData)) return null;
  
  // Build glyph mapping from entities
  const glyphMapping = React.useMemo(() => buildGlyphMapping(entities), [entities]);

  // Find all valid options for this zone across all actions
  const validOptionsMap = new Map<string, { action: string; option: ActionOption }>();
  
  groupedActions.forEach(groupedAction => {
    groupedAction.validOptions
      .filter(option => option.zone === zoneName)
      .forEach(option => {
        const key = `${option.row},${option.col}`;
        validOptionsMap.set(key, { action: groupedAction.action, option });
      });
  });

  const handleCellClick = React.useCallback((row: number, col: number) => {
    const key = `${row},${col}`;
    const validOption = validOptionsMap.get(key);
    
    console.log('[InteractiveZone] Cell clicked:', { 
      row, 
      col, 
      key, 
      validOption, 
      isMyTurn,
      validOptionsMapSize: validOptionsMap.size,
      hasOption: validOptionsMap.has(key)
    });
    
    if (validOption) {
      const zoneAction = {
        action: validOption.action,
        zone: zoneName,
        row,
        col
      };
      console.log('[InteractiveZone] Sending action:', zoneAction);
      onAction(zoneAction);
    } else {
      console.log('[InteractiveZone] No valid option found for cell');
    }
  }, [validOptionsMap, zoneName, onAction, isMyTurn]);

  return (
    <div>
      <h4 className="text-sm font-semibold mb-2 text-gray-400">Zone: {zoneName}</h4>
      <div>
        {zoneData.map((row, r) => (
          <div key={r} style={rowStyle}>
            {row.map((cell, c) => {
              const posKey = `${r},${c}`;
              const isClickable = isMyTurn && validOptionsMap.has(posKey);
              
              return (
                <Cell 
                  key={`${r}-${c}-${isClickable}`} 
                  value={cell} 
                  row={r}
                  col={c}
                  isClickable={isClickable}
                  onClick={() => handleCellClick(r, c)}
                  glyphMapping={glyphMapping}
                />
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
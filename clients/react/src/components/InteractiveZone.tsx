import React from 'react';

export type ZoneAction = {
  verb: string;
  zone: string;
  args: { row: number; col: number };
};

type InteractiveZoneProps = {
  zoneName: string;
  zoneData: (string | null)[][];
  possibleActions: ZoneAction[];
  onAction: (action: ZoneAction) => void;
  isMyTurn: boolean;
};

const markToGlyph: Record<string, string> = {
  mark_x: 'X',
  mark_o: 'O',
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
}

const Cell = React.memo(function Cell({ value, row, col, isClickable, onClick }: CellProps) {
  const [isHovered, setIsHovered] = React.useState(false);

  const style = isClickable 
    ? (isHovered ? hoveredCellStyle : clickableCellStyle)
    : cellStyle;

  return (
    <div 
      style={style}
      onClick={isClickable ? onClick : undefined}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      title={isClickable ? `Click to place at (${row}, ${col})` : undefined}
    >
      {value ? markToGlyph[value] ?? value : ''}
    </div>
  );
});

export default function InteractiveZone({ 
  zoneName, 
  zoneData, 
  possibleActions, 
  onAction,
  isMyTurn 
}: InteractiveZoneProps) {
  if (!zoneData || !Array.isArray(zoneData)) return null;

  // Create a map of clickable positions for this zone
  const clickablePositions = new Set<string>();
  possibleActions
    .filter(action => action.zone === zoneName)
    .forEach(action => {
      clickablePositions.add(`${action.args.row},${action.args.col}`);
    });

  const handleCellClick = (row: number, col: number) => {
    const action = possibleActions.find(
      a => a.zone === zoneName && a.args.row === row && a.args.col === col
    );
    if (action) {
      onAction(action);
    }
  };

  return (
    <div>
      <h4 className="text-sm font-semibold mb-2 text-gray-400">Zone: {zoneName}</h4>
      <div>
        {zoneData.map((row, r) => (
          <div key={r} style={rowStyle}>
            {row.map((cell, c) => {
              const posKey = `${r},${c}`;
              const isClickable = isMyTurn && clickablePositions.has(posKey);
              
              return (
                <Cell 
                  key={c} 
                  value={cell} 
                  row={r}
                  col={c}
                  isClickable={isClickable}
                  onClick={() => handleCellClick(r, c)}
                />
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
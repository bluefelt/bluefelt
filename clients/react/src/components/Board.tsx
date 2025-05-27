import React from 'react';

interface BoardProps {
  zones: any;
  playerId: string;
  entityDefinitions?: any[];
  currentPlayer?: string;
  onCellClick?: (row: number, col: number) => void;
  isMyTurn?: boolean;
}

export default function Board({ 
  zones, 
  playerId, 
  entityDefinitions, 
  currentPlayer,
  onCellClick,
  isMyTurn = false
}: BoardProps) {
  const boardData = zones?.board;
  if (!boardData || !Array.isArray(boardData)) return null;

  // Map player marks to colors
  const getMarkColor = (cell: string) => {
    if (cell === 'mark_p1') return '#FF1493'; // Pink
    if (cell === 'mark_p2') return '#FFD700'; // Gold
    return '#888';
  };

  // Get glyph from entity definitions or fallback
  const getGlyph = (cell: string) => {
    const entity = entityDefinitions?.find(e => e.id === cell);
    return entity?.ui?.glyph || (cell === 'mark_p1' ? 'X' : 'O');
  };

  const handleCellClick = (row: number, col: number) => {
    if (isMyTurn && boardData[row][col] === null && onCellClick) {
      onCellClick(row, col);
    }
  };

  return (
    <div className="bg-gray-800 p-6 rounded-lg">
      <h2 className="text-xl font-semibold mb-4 text-white">Board</h2>
      <div className="inline-block bg-black p-4 rounded">
        <div 
          className="grid gap-0 border-2 border-white"
          style={{
            gridTemplateColumns: `repeat(${boardData[0].length}, 1fr)`,
            width: 'fit-content'
          }}
        >
          {boardData.map((row, rowIndex) =>
            row.map((cell: any, colIndex: number) => {
              const isEmpty = cell === null;
              const isClickable = isMyTurn && isEmpty;
              
              return (
                <div
                  key={`${rowIndex}-${colIndex}`}
                  className="relative border border-gray-700"
                  style={{ width: '120px', height: '120px' }}
                >
                  {isEmpty ? (
                    <div 
                      className={`w-full h-full flex items-center justify-center ${
                        isClickable ? 'cursor-pointer' : 'cursor-not-allowed'
                      }`}
                      style={{
                        backgroundImage: `radial-gradient(circle, #4a5568 1px, transparent 1px)`,
                        backgroundSize: '10px 10px',
                        backgroundColor: '#1a202c'
                      }}
                      onClick={() => handleCellClick(rowIndex, colIndex)}
                      onMouseEnter={(e) => {
                        if (isClickable) {
                          e.currentTarget.style.backgroundColor = 'rgba(75, 85, 99, 0.5)';
                        }
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.backgroundColor = '#1a202c';
                      }}
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center bg-black">
                      <span 
                        className="text-6xl font-bold"
                        style={{ color: getMarkColor(cell) }}
                      >
                        {getGlyph(cell)}
                      </span>
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
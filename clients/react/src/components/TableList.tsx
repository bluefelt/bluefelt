import React from 'react';
import type { Table } from '../types/game-types';

interface TableListProps {
  tables: Table[];
  currentPlayerId: string;
  isLobbyArchived?: boolean;
  onCreateTable: (bundleId: string) => void;
  onJoinTable: (tableId: string) => void;
  onClaimSeat: (tableId: string, seatIndex: number) => void;
  onReleaseSeat: (tableId: string, seatIndex: number) => void;
  onSetReady: (tableId: string, ready: boolean) => void;
  onDeleteTable: (tableId: string) => void;
  onViewGame?: (tableId: string) => void;
}

// Available game types
const GAME_TYPES = [
  { id: 'tic-tac-toe', name: 'Tic-Tac-Toe', minPlayers: 2, maxPlayers: 2 },
  { id: 'three-mens-morris', name: 'Three Men\'s Morris', minPlayers: 2, maxPlayers: 2 },
  { id: 'connect-four', name: 'Connect Four', minPlayers: 2, maxPlayers: 2 },
  { id: 'go-fish', name: 'Go Fish', minPlayers: 2, maxPlayers: 4 },
];

export function TableList({
  tables,
  currentPlayerId,
  isLobbyArchived = false,
  onCreateTable,
  onJoinTable,
  onClaimSeat,
  onReleaseSeat,
  onSetReady,
  onDeleteTable,
  onViewGame,
}: TableListProps) {
  const [selectedGameType, setSelectedGameType] = React.useState('tic-tac-toe');
  
  // Check if player is already seated at any table
  const isSeatedAtAnyTable = tables.some(table =>
    table.seats.some(seat => seat?.playerId === currentPlayerId)
  );
  
  const handleCreateTable = () => {
    onCreateTable(selectedGameType);
  };
  
  return (
    <div className="space-y-6">
      {/* Create Table Section */}
      {!isSeatedAtAnyTable && !isLobbyArchived && (
        <div className="card">
          <h3 className="text-lg font-semibold mb-4">Create a Table</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium mb-1">Game Type</label>
              <select
                value={selectedGameType}
                onChange={(e) => setSelectedGameType(e.target.value)}
                className="w-full bg-gray-700 rounded px-3 py-2"
              >
                {GAME_TYPES.map(type => (
                  <option key={type.id} value={type.id}>
                    {type.name} ({type.minPlayers === type.maxPlayers 
                      ? `${type.minPlayers} players` 
                      : `${type.minPlayers}-${type.maxPlayers} players`})
                  </option>
                ))}
              </select>
            </div>
            
            <button
              onClick={handleCreateTable}
              className="btn btn-primary w-full"
            >
              Create Table
            </button>
          </div>
        </div>
      )}
      
      {/* Tables List */}
      <div className="card">
        <h3 className="text-lg font-semibold mb-4">Active Tables</h3>
        {tables.length === 0 ? (
          <p className="text-gray-400">No tables yet. Create one to get started!</p>
        ) : (
          <div className="space-y-4">
            {tables.map((table) => (
              <TableCard
                key={table.id}
                table={table}
                currentPlayerId={currentPlayerId}
                isSeatedAtAnyTable={isSeatedAtAnyTable}
                onJoinTable={onJoinTable}
                onClaimSeat={onClaimSeat}
                onReleaseSeat={onReleaseSeat}
                onSetReady={onSetReady}
                onDeleteTable={onDeleteTable}
                onViewGame={onViewGame}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

interface TableCardProps {
  table: Table;
  currentPlayerId: string;
  isSeatedAtAnyTable: boolean;
  onJoinTable: (tableId: string) => void;
  onClaimSeat: (tableId: string, seatIndex: number) => void;
  onReleaseSeat: (tableId: string, seatIndex: number) => void;
  onSetReady: (tableId: string, ready: boolean) => void;
  onDeleteTable: (tableId: string) => void;
  onViewGame?: (tableId: string) => void;
}

function TableCard({
  table,
  currentPlayerId,
  isSeatedAtAnyTable,
  onJoinTable,
  onClaimSeat,
  onReleaseSeat,
  onSetReady,
  onDeleteTable,
  onViewGame,
}: TableCardProps) {
  const gameType = GAME_TYPES.find(t => t.id === table.bundleId);
  const playerSeatIndex = table.seats.findIndex(seat => seat?.playerId === currentPlayerId);
  const isSeated = playerSeatIndex !== -1;
  const isOwner = table.owner === currentPlayerId;
  const isReady = isSeated && table.readyStates[playerSeatIndex];
  
  // Calculate seated players count
  const seatedCount = table.seats.filter(seat => seat !== null).length;
  const canStart = seatedCount >= table.minPlayers;
  
  // Get status display
  const getStatusDisplay = () => {
    switch (table.status) {
      case 'Open':
        return { text: 'Waiting for Players', className: 'bg-yellow-600' };
      case 'Countdown':
        return { text: 'Starting Soon...', className: 'bg-orange-600' };
      case 'Playing':
        return { text: 'In Progress', className: 'bg-green-600' };
      case 'Finished':
        return { text: 'Finished', className: 'bg-gray-600' };
      default:
        return { text: table.status, className: 'bg-gray-600' };
    }
  };
  
  const status = getStatusDisplay();
  
  return (
    <div className="bg-gray-700 rounded-lg p-4">
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div>
          <h4 className="font-medium">
            {gameType?.name || table.bundleId}
          </h4>
          <div className="text-sm text-gray-400">
            Owner: {table.owner}
            {isOwner && ' (you)'}
          </div>
        </div>
        <span className={`px-2 py-1 rounded text-sm ${status.className}`}>
          {status.text}
        </span>
      </div>
      
      {/* Countdown Timer */}
      {table.status === 'Countdown' && table.countdownEndsAt && (
        <CountdownTimer endsAt={table.countdownEndsAt} />
      )}
      
      {/* Seats */}
      <div className="mb-4">
        <div className="text-sm font-medium mb-2">
          Seats ({seatedCount}/{table.maxPlayers})
        </div>
        <div className="grid grid-cols-2 gap-2">
          {table.seats.map((seat, index) => (
            <SeatSlot
              key={index}
              seat={seat}
              seatIndex={index}
              tableId={table.id}
              isReady={table.readyStates?.[index] || false}
              isCurrentPlayer={seat?.playerId === currentPlayerId}
              canClaim={!isSeatedAtAnyTable && !seat && table.status === 'Open'}
              canRelease={seat?.playerId === currentPlayerId && table.status === 'Open'}
              onClaimSeat={onClaimSeat}
              onReleaseSeat={onReleaseSeat}
            />
          ))}
        </div>
      </div>
      
      {/* Actions */}
      <div className="flex gap-2 flex-wrap">
        {/* View Game button - shown for playing games */}
        {table.status === 'Playing' && onViewGame && (
          <button
            onClick={() => onViewGame(table.id)}
            className="btn btn-sm btn-primary"
          >
            <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
            </svg>
            View Game
          </button>
        )}
        
        {/* Join Table button - shown when not seated at any table */}
        {!isSeatedAtAnyTable && table.status === 'Open' && seatedCount < table.maxPlayers && (
          <button
            onClick={() => onJoinTable(table.id)}
            className="btn btn-sm btn-primary"
          >
            Join Table
          </button>
        )}
        
        {isSeated && table.status === 'Open' && (
          <button
            onClick={() => onSetReady(table.id, !isReady)}
            className={`btn btn-sm ${isReady ? 'btn-secondary' : 'btn-success'}`}
          >
            {isReady ? 'Not Ready' : 'Ready'}
          </button>
        )}
        
        {isOwner && table.status === 'Open' && seatedCount === 0 && (
          <button
            onClick={() => onDeleteTable(table.id)}
            className="btn btn-sm btn-danger"
          >
            Delete Table
          </button>
        )}
        
        {!canStart && table.status === 'Open' && (
          <div className="text-sm text-gray-400 flex items-center">
            Need {table.minPlayers - seatedCount} more player{table.minPlayers - seatedCount !== 1 ? 's' : ''} to start
          </div>
        )}
        
        {table.status === 'Open' && seatedCount >= table.maxPlayers && !isSeated && (
          <div className="text-sm text-red-400 flex items-center">
            Table is full
          </div>
        )}
      </div>
    </div>
  );
}

interface SeatSlotProps {
  seat: { playerId: string; username: string } | null;
  seatIndex: number;
  tableId: string;
  isReady: boolean;
  isCurrentPlayer: boolean;
  canClaim: boolean;
  canRelease: boolean;
  onClaimSeat: (tableId: string, seatIndex: number) => void;
  onReleaseSeat: (tableId: string, seatIndex: number) => void;
}

function SeatSlot({
  seat,
  seatIndex,
  tableId,
  isReady,
  isCurrentPlayer,
  canClaim,
  canRelease,
  onClaimSeat,
  onReleaseSeat,
}: SeatSlotProps) {
  if (!seat) {
    return (
      <div className="bg-gray-800 rounded p-3 text-center">
        <div className="text-sm text-gray-400 mb-1">Seat {seatIndex + 1}</div>
        {canClaim ? (
          <button
            onClick={() => onClaimSeat(tableId, seatIndex)}
            className="btn btn-xs btn-primary"
          >
            Claim Seat
          </button>
        ) : (
          <div className="text-xs text-gray-500">Empty</div>
        )}
      </div>
    );
  }
  
  return (
    <div className={`rounded p-3 ${isCurrentPlayer ? 'bg-blue-900' : 'bg-gray-800'}`}>
      <div className="text-sm text-gray-400 mb-1">Seat {seatIndex + 1}</div>
      <div className="font-medium text-sm">
        {seat.username}
        {isCurrentPlayer && ' (you)'}
      </div>
      {isReady && (
        <div className="text-xs text-green-400 mt-1">✓ Ready</div>
      )}
      {canRelease && (
        <button
          onClick={() => onReleaseSeat(tableId, seatIndex)}
          className="btn btn-xs btn-secondary mt-2"
        >
          Leave Seat
        </button>
      )}
    </div>
  );
}

function CountdownTimer({ endsAt }: { endsAt: number }) {
  const [timeLeft, setTimeLeft] = React.useState(0);
  
  React.useEffect(() => {
    const updateTimer = () => {
      const now = Date.now() / 1000;
      const remaining = Math.max(0, endsAt - now);
      setTimeLeft(Math.ceil(remaining));
    };
    
    updateTimer();
    const interval = setInterval(updateTimer, 100);
    
    return () => clearInterval(interval);
  }, [endsAt]);
  
  return (
    <div className="text-center mb-3">
      <div className="text-2xl font-bold text-orange-400">{timeLeft}</div>
      <div className="text-sm text-gray-400">Game starting in...</div>
    </div>
  );
}
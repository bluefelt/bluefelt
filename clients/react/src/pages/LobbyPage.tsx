import { useParams, useNavigate } from 'react-router-dom';
import { useEffect, useState } from 'react';
import { usePlayer } from '../context/PlayerContext';
import { useLobbyWebSocketContext } from '../context/LobbyWebSocketContext';
import { TableList } from '../components/TableList';
import { LobbyChat } from '../components/LobbyChat';
import ProtectedRoute from '../components/ProtectedRoute';

// Game type definitions
const GAME_TYPES = [
  { id: 'tic-tac-toe', name: 'Tic-Tac-Toe' },
  { id: 'three-mens-morris', name: 'Three Men\'s Morris' },
  { id: 'connect-four', name: 'Connect Four' },
  { id: 'go-fish', name: 'Go Fish' },
];

export default function LobbyPage() {
  const { lobbyId } = useParams<{ lobbyId: string }>();
  const navigate = useNavigate();
  const { player } = usePlayer();
  const [selectedGameType, setSelectedGameType] = useState('tic-tac-toe');
  const [isEditingName, setIsEditingName] = useState(false);
  const [newLobbyName, setNewLobbyName] = useState('');
  
  const {
    lobbyState,
    connected,
    connectionState,
    leaveLobby,
    renameLobby,
    joinLobby,
    createTable,
    joinTable,
    claimSeat,
    releaseSeat,
    setReady,
    sendChatMessage,
    clearTableError,
  } = useLobbyWebSocketContext();

  // Handle errors
  useEffect(() => {
    if (lobbyState.error === 'Lobby does not exist') {
      navigate('/lobbies', {
        state: { message: 'The lobby you tried to join does not exist.' }
      });
    }
  }, [lobbyState.error, navigate]);
  
  // Cancel editing when lobby name changes
  useEffect(() => {
    if (isEditingName && lobbyState.name !== newLobbyName) {
      setIsEditingName(false);
    }
  }, [lobbyState.name]);

  // Remove old game handling - no longer needed

  // Find current user's table if any
  const userTable = lobbyState.tables?.find(table =>
    table.seats.some(seat => seat?.playerId === player?.username)
  );

  const renderLobbyView = () => {
    // Show join button if not already a member
    if (!lobbyState.isJoined) {
      return (
        <div className="card max-w-2xl mx-auto">
          <h1 className="text-2xl font-bold mb-4">{lobbyState.name || 'Loading...'}</h1>
          {lobbyState.archived && (
            <div className="bg-red-500 bg-opacity-10 border border-red-500 rounded p-3 mb-4">
              <p className="text-red-400 font-semibold">This lobby is archived and cannot be joined.</p>
            </div>
          )}
          <div className="text-gray-400 mb-6">
            <p>Lobby ID: {lobbyId}</p>
            {lobbyState.inviteCode && <p>Invite Code: {lobbyState.inviteCode}</p>}
            <p className="mt-2">Members: {lobbyState.members?.filter(m => m.connected).length || 0} online</p>
          </div>
          {!lobbyState.archived && (
            <button
              onClick={joinLobby}
              className="btn btn-primary btn-lg"
            >
              Join Lobby
            </button>
          )}
          <button
            onClick={() => navigate('/lobbies')}
            className="btn btn-secondary btn-lg ml-4"
          >
            Back to Lobbies
          </button>
        </div>
      );
    }
    
    return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
      {/* Left Column - Tables */}
      <div className="lg:col-span-2 space-y-6">
        {/* Lobby Info */}
        <div className="card">
          {lobbyState.archived && (
            <div className="bg-red-500 bg-opacity-10 border border-red-500 rounded p-3 mb-4">
              <p className="text-red-400 font-semibold">This lobby is archived. No new players can join.</p>
            </div>
          )}
          <div className="flex items-center justify-between mb-4">
            {isEditingName && !lobbyState.archived ? (
              <form 
                onSubmit={(e) => {
                  e.preventDefault();
                  if (newLobbyName.trim()) {
                    renameLobby(newLobbyName.trim());
                    setIsEditingName(false);
                  }
                }}
                className="flex items-center gap-2 flex-1"
              >
                <input
                  type="text"
                  value={newLobbyName}
                  onChange={(e) => setNewLobbyName(e.target.value)}
                  className="input flex-1"
                  placeholder="Enter new lobby name"
                  autoFocus
                />
                <button type="submit" className="btn btn-primary btn-sm">
                  Save
                </button>
                <button 
                  type="button" 
                  onClick={() => setIsEditingName(false)}
                  className="btn btn-secondary btn-sm"
                >
                  Cancel
                </button>
              </form>
            ) : (
              <>
                <h1 className="text-2xl font-bold">{lobbyState.name}</h1>
                {lobbyState.owner === player?.username && (
                  <button
                    onClick={() => {
                      setNewLobbyName(lobbyState.name || '');
                      setIsEditingName(true);
                    }}
                    className="btn btn-sm btn-secondary"
                    title="Rename lobby"
                  >
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                    </svg>
                  </button>
                )}
              </>
            )}
          </div>
          <div className="flex gap-4 text-sm text-gray-400 mb-4">
            <span>Lobby ID: {lobbyId}</span>
            {lobbyState.inviteCode && <span>Invite Code: {lobbyState.inviteCode}</span>}
            {lobbyState.owner && <span>Owner: {lobbyState.owner}</span>}
          </div>
          
          {/* Members */}
          <div>
            <h3 className="font-semibold mb-2">Connected Members ({lobbyState.members.filter(m => m.connected).length})</h3>
            <div className="flex flex-wrap gap-2">
              {lobbyState.members
                .filter(m => m.connected)
                .map(member => (
                  <span key={member.username} className="text-sm bg-gray-700 rounded px-2 py-1">
                    {member.username}
                    {member.username === player?.username && ' (you)'}
                  </span>
                ))}
            </div>
          </div>
        </div>
        
        {/* Table Error Display */}
        {lobbyState.tableError && (
          <div className="bg-red-900 border border-red-700 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div>
                <h4 className="font-medium text-red-100">Table Error</h4>
                <p className="text-red-200 text-sm">{lobbyState.tableError}</p>
              </div>
              <button
                onClick={clearTableError}
                className="text-red-300 hover:text-red-100"
              >
                ✕
              </button>
            </div>
          </div>
        )}
        
        {/* Tables */}
        <TableList
          tables={lobbyState.tables || []}
          currentPlayerId={player?.username || ''}
          isLobbyArchived={lobbyState.archived || false}
          onCreateTable={createTable}
          onJoinTable={joinTable}
          onClaimSeat={claimSeat}
          onReleaseSeat={releaseSeat}
          onSetReady={setReady}
          onDeleteTable={(tableId) => {
            // TODO: Implement delete table
            console.log('Delete table:', tableId);
          }}
          onViewGame={handleViewGame}
        />

      </div>

      {/* Right Column - Chat */}
      <div className="lg:col-span-1">
        <div className="lg:sticky lg:top-4">
          <LobbyChat
            messages={lobbyState.recentChat || []}
            currentScope={userTable ? 'table' : 'lobby'}
            currentTableId={userTable?.id}
            currentUsername={player?.username || ''}
            onSendMessage={sendChatMessage}
          />
        </div>
      </div>

      <button
        onClick={() => {
          leaveLobby();
          navigate('/lobbies');
        }}
        className="btn btn-secondary"
      >
        Leave Lobby
      </button>
    </div>
  );
  };

  // Auto-navigate to game view if in an active game (only when not already on game route)
  useEffect(() => {
    if (lobbyState.game && lobbyState.game.state && lobbyState.game.id) {
      const currentPath = window.location.pathname;
      // Use tableId if available, otherwise fall back to game.id
      const tableId = lobbyState.game.tableId || lobbyState.game.id;
      const gamePagePath = `/lobby/${lobbyId}/table/${tableId}`;
      
      // Only navigate if we're not already on the game page or coming from the game page
      const isOnGamePage = currentPath.includes('/table/');
      const justLeftGame = sessionStorage.getItem('leftGameView') === 'true';
      
      if (!isOnGamePage && !justLeftGame) {
        console.log('[LobbyPage] Active game detected, navigating to game view');
        navigate(gamePagePath);
      }
      
      // Clear the flag after checking
      if (justLeftGame) {
        sessionStorage.removeItem('leftGameView');
      }
    }
  }, [lobbyState.game, lobbyId, navigate]);

  // Handle viewing a specific game
  const handleViewGame = (tableId: string) => {
    navigate(`/lobby/${lobbyId}/table/${tableId}`);
  };

  // Render lobby view
  return (
    <ProtectedRoute>
      <div className="container mx-auto px-4 py-8 max-w-4xl">
        {connected ? (
          renderLobbyView()
        ) : (
          <div className="card">
            <p className="text-center text-gray-400">Connecting to lobby...</p>
          </div>
        )}
      </div>
    </ProtectedRoute>
  );
}
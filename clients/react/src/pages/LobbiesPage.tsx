import { useNavigate } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';
import ProtectedRoute from '../components/ProtectedRoute';
import { useWebSocketContext } from '../context/WebSocketContext';

export default function LobbiesPage() {
  const navigate = useNavigate();
  const { player } = usePlayer();
  const { lobbies, createLobby } = useWebSocketContext();

  const handleLobbySelected = (lobbyId: string) => {
    navigate(`/lobby/${lobbyId}`);
  };

  const handleCreateLobby = async () => {
    try {
      const lobby = await createLobby(`${player?.username}'s Lobby`);
      navigate(`/lobby/${lobby.id}`);
    } catch (error) {
      console.error('Failed to create lobby:', error);
    }
  };

  // Sort lobbies by creation time (newest first)
  const sortedLobbies = [...lobbies].sort((a, b) => 
    new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  // Categorize lobbies
  const activeLobbies = sortedLobbies.filter(lobby => 
    lobby.games?.some(g => g.status === 'playing') || 
    lobby.members?.some(m => m.connected)
  );
  
  const emptyLobbies = sortedLobbies.filter(lobby => 
    !lobby.members?.some(m => m.connected) &&
    !lobby.games?.some(g => g.status === 'playing')
  );

  const renderLobbyCard = (lobby: typeof lobbies[0]) => {
    const connectedMembers = lobby.members?.filter(m => m.connected) || [];
    const activeGames = lobby.games?.filter(g => g.status !== 'ended') || [];
    
    return (
      <li
        key={lobby.id}
        onClick={() => handleLobbySelected(lobby.id)}
        className="bg-gray-700 rounded-lg p-4 hover:bg-gray-600 transition-colors cursor-pointer"
      >
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <h3 className="font-medium text-lg text-white">{lobby.name}</h3>
            <span className="text-sm text-gray-400">{lobby.id}</span>
          </div>
          
          <div className="text-sm text-gray-300">
            <div>Connected: {connectedMembers.length} player{connectedMembers.length !== 1 ? 's' : ''}</div>
            {connectedMembers.length > 0 && (
              <div className="text-gray-400">
                {connectedMembers.map(m => m.username).join(', ')}
              </div>
            )}
          </div>
          
          {activeGames.length > 0 && (
            <div className="text-sm">
              <span className="text-yellow-400">
                {activeGames.length} active game{activeGames.length !== 1 ? 's' : ''}
              </span>
            </div>
          )}
          
          <div className="text-xs text-gray-500">
            Created {new Date(lobby.created_at).toLocaleString()}
          </div>
        </div>
      </li>
    );
  };

  return (
    <ProtectedRoute>
      <div className="container mx-auto px-4 py-8 max-w-4xl">
        <div className="space-y-8">
          <div className="card">
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-xl font-semibold">Lobbies</h2>
              <button
                onClick={handleCreateLobby}
                className="btn btn-primary"
              >
                Create Lobby
              </button>
            </div>

            {lobbies.length === 0 ? (
              <p className="text-gray-400 text-center py-8">
                No lobbies exist. Create one to get started!
              </p>
            ) : (
              <div className="space-y-6">
                {activeLobbies.length > 0 && (
                  <div>
                    <h3 className="text-lg font-semibold mb-4 text-gray-300">Active Lobbies</h3>
                    <ul className="space-y-4">
                      {activeLobbies.map(renderLobbyCard)}
                    </ul>
                  </div>
                )}
                
                {emptyLobbies.length > 0 && (
                  <div>
                    <h3 className="text-lg font-semibold mb-4 text-gray-300">Empty Lobbies</h3>
                    <ul className="space-y-4">
                      {emptyLobbies.map(renderLobbyCard)}
                    </ul>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </ProtectedRoute>
  );
}
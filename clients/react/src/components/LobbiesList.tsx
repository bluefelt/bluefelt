import { useNavigate } from 'react-router-dom';
import { useWebSocketContext } from '../context/WebSocketContext';
import { usePlayer } from '../context/PlayerContext';

type Props = {
  onLobbySelected: (lobbyId: string) => void;
};

type LobbyWithDetails = {
  id: string;
  game_id: string;
  game_name?: string; // Server now provides this
  name: string;
  players: string[];
  started: boolean;
  created_at?: number; // Unix timestamp - optional for backward compatibility
  gameName?: string;
  currentTurn?: string;
  gameStatus?: {
    state: 'ended';
    winner?: string;
    tie?: boolean;
  };
};

export default function LobbiesList({ onLobbySelected }: Props) {
  const navigate = useNavigate();
  const { player } = usePlayer();
  const { lobbies, lobbiesWS } = useWebSocketContext();
  // Server now provides game names directly in lobby data - no separate HTTP calls needed

  // Enhanced lobbies with game names (server now provides game_name)
  const lobbiesWithDetails: LobbyWithDetails[] = lobbies.map(lobby => {
    const enhanced = {
      ...lobby,
      gameName: (lobby as any).game_name || lobby.game_id,
    };
    
    // Debug log for completed games
    if (lobby.gameStatus) {
    }
    
    return enhanced;
  });

  // Helper function to format creation date
  const formatCreationDate = (timestamp: number): string => {
    // Handle case where timestamp might be undefined or invalid
    if (!timestamp || isNaN(timestamp)) {
      return 'Unknown';
    }
    
    const date = new Date(timestamp * 1000);
    
    // Check if date is valid
    if (isNaN(date.getTime())) {
      return 'Unknown';
    }
    
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  // Helper function to sort by creation time, handling missing timestamps
  const sortByCreationTime = (a: LobbyWithDetails, b: LobbyWithDetails) => {
    // If both have timestamps, sort by timestamp (newest first)
    if (a.created_at && b.created_at) {
      return b.created_at - a.created_at;
    }
    // If only one has timestamp, prioritize the one with timestamp
    if (a.created_at && !b.created_at) return -1;
    if (!a.created_at && b.created_at) return 1;
    // If neither has timestamp, sort by ID as fallback
    return a.id.localeCompare(b.id);
  };

  // Categorize and sort lobbies by creation time (newest first within each category)
  const waitingLobbies = lobbiesWithDetails
    .filter(lobby => !lobby.started)
    .sort(sortByCreationTime);
  const inProgressLobbies = lobbiesWithDetails
    .filter(lobby => lobby.started && (!lobby.gameStatus || lobby.gameStatus.state !== 'ended'))
    .sort(sortByCreationTime);
  const finishedLobbies = lobbiesWithDetails
    .filter(lobby => lobby.gameStatus && lobby.gameStatus.state === 'ended')
    .sort(sortByCreationTime);

  const renderLobbyCard = (lobby: LobbyWithDetails) => {
    const isFinished = lobby.gameStatus?.state === 'ended';
    const isInProgress = lobby.started && !isFinished;
    
    // Format players with current turn indicator
    const formatPlayers = () => {
      if (lobby.players.length === 0) return 'None';
      
      const playerElements = lobby.players.map(p => 
        p === player?.username ? { name: p, isYou: true } : { name: p, isYou: false }
      );
      
      // If there's a current turn and the game is in progress, add turn indicator
      if (lobby.currentTurn && isInProgress) {
        const isYourTurn = lobby.currentTurn === player?.username;
        const turnIndicator = isYourTurn ? 'your turn' : `${lobby.currentTurn}'s turn`;
        return { players: playerElements, turnIndicator };
      }
      
      return { players: playerElements, turnIndicator: null };
    };

    return (
      <li
        key={lobby.id}
        onClick={() => onLobbySelected(lobby.id)}
        className="bg-gray-700 rounded-lg p-4 hover:bg-gray-600 transition-colors cursor-pointer relative"
      >
        {/* Status Badge - Upper Right Corner */}
        {isFinished && (
          <div className="absolute top-3 right-3 inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-600 text-gray-300">
            {lobby.gameStatus?.tie ? 'Tie Game' : 
             lobby.gameStatus?.winner ? `Winner: ${
               (() => {
                 const winnerMatch = lobby.gameStatus.winner.match(/^p(\d+)$/);
                 const winnerIndex = winnerMatch ? parseInt(winnerMatch[1]) - 1 : -1;
                 return winnerIndex >= 0 && winnerIndex < lobby.players.length 
                   ? lobby.players[winnerIndex] 
                   : lobby.gameStatus.winner;
               })()
             }` : 'Finished'}
          </div>
        )}
        {!lobby.started && (
          <div className="absolute top-3 right-3 inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-yellow-600 text-yellow-100">
            Waiting for Players
          </div>
        )}
        {isInProgress && (
          <div className="absolute top-3 right-3 inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-600 text-green-100">
            In Progress
          </div>
        )}

        <div className="space-y-2 pr-24">
          {/* Game Name and ID */}
          <div className="flex items-center gap-2">
            <h3 className="font-medium text-lg text-white">{lobby.gameName}</h3>
            <span className="text-sm text-gray-400">{lobby.id}</span>
          </div>
          
          {/* Creation Date */}
          <div className="text-sm text-gray-400">
            Created {formatCreationDate(lobby.created_at || 0)}
          </div>
          
          {/* Players */}
          <div className="text-sm">
            <span className="text-gray-300">Players: </span>
            <span className="text-white">
              {(() => {
                const playerData = formatPlayers();
                if (typeof playerData === 'string') {
                  return playerData;
                }
                
                const playerDisplay = playerData.players.map((p, index) => (
                  <span key={p.name}>
                    {index > 0 && ', '}
                    {p.isYou ? <strong>you</strong> : p.name}
                  </span>
                ));
                
                if (playerData.turnIndicator) {
                  return (
                    <>
                      {playerDisplay} — <strong>{playerData.turnIndicator}</strong>
                    </>
                  );
                }
                
                return playerDisplay;
              })()}
            </span>
          </div>
        </div>
      </li>
    );
  };

  const renderSection = (title: string, lobbies: LobbyWithDetails[]) => {
    if (lobbies.length === 0) return null;
    
    return (
      <div className="mb-8">
        <h3 className="text-lg font-semibold mb-4 text-gray-300">{title}</h3>
        <ul className="space-y-4">
          {lobbies.map(renderLobbyCard)}
        </ul>
      </div>
    );
  };

  // Use WebSocket connection state for loading - no separate loading states needed

  return (
    <>
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-semibold">Available Lobbies</h2>
          <button
            onClick={() => navigate('/create-lobby')}
            className="btn btn-primary"
          >
            Create Lobby
          </button>
        </div>

        {lobbies.length === 0 ? (
          <p className="text-gray-400 text-center py-8">
            No active lobbies. Create one to get started!
          </p>
        ) : (
          <div>
            {renderSection('Waiting for Players', waitingLobbies)}
            {renderSection('In Progress', inProgressLobbies)}
            {renderSection('Finished', finishedLobbies)}
          </div>
        )}
      </div>
    </>
  );
}
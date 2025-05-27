import { Link } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';

interface GameHeaderProps {
  lobbyId: string;
  gameId: string;
  gameName: string;
  status: 'waiting' | 'in_progress' | 'finished';
  players: { username: string; isConnected: boolean }[];
  currentPlayer?: string;
}

export default function GameHeader({ lobbyId, gameId, gameName, status, players, currentPlayer }: GameHeaderProps) {
  const { player, logout } = usePlayer();

  // Define player colors
  const playerColors = ['#FF1493', '#FFD700']; // Pink and Gold
  
  const getPlayerColor = (username: string) => {
    const index = players.findIndex(p => p.username === username);
    return playerColors[index] || '#888';
  };

  const getInitials = (username: string) => {
    return username.slice(0, 2).toUpperCase();
  };

  return (
    <div className="bg-gray-800 border-b border-gray-700">
      <div className="container mx-auto px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            {/* Breadcrumb */}
            <Link to="/lobbies" className="text-yellow-500 hover:text-yellow-400">
              Lobbies
            </Link>
            <span className="text-gray-500">›</span>

            {/* Game title */}
            <h1 className="text-xl font-semibold text-white">
              {gameName} <span className="text-gray-500 text-sm">#{gameId}</span>
            </h1>

            {/* Status badge */}
            <span className={`px-3 py-1 rounded text-sm font-medium ${
              status === 'in_progress' 
                ? 'bg-green-900 text-green-300' 
                : status === 'finished'
                ? 'bg-gray-700 text-gray-300'
                : 'bg-yellow-900 text-yellow-300'
            }`}>
              {status === 'in_progress' ? 'In Progress' : status === 'finished' ? 'Finished' : 'Waiting'}
            </span>
          </div>

          {/* Player indicators */}
          <div className="flex items-center space-x-3">
            {players.map((p, i) => (
              <div key={p.username} className="flex items-center space-x-2">
                <div
                  className="w-4 h-4 rounded-sm"
                  style={{ backgroundColor: getPlayerColor(p.username) }}
                />
                <span className={`text-sm ${p.isConnected ? 'text-gray-300' : 'text-gray-500'}`}>
                  {p.username}
                  {p.username === currentPlayer && ' ●'}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
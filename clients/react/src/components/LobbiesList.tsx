import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { getLobbies } from '../api/lobbies';
import { getGames } from '../api/games';
import type { Game } from '../api/games';
import { useWebSocketContext } from '../context/WebSocketContext';
import WebSocketStatus from './WebSocketStatus';
import { usePlayer } from '../context/PlayerContext';

type Props = {
  onLobbySelected: (lobbyId: string) => void;
};

type LobbyWithDetails = {
  id: string;
  game_id: string;
  name: string;
  players: string[];
  started: boolean;
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
  const [games, setGames] = useState<Record<string, Game>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      // Fetch initial lobbies list
      await getLobbies();
      
      // Fetch games
      const gamesList = await getGames();
      const gamesMap: Record<string, Game> = {};
      gamesList.forEach(game => {
        gamesMap[game.id] = game;
      });
      setGames(gamesMap);
      
    } catch (error) {
      console.error('Failed to fetch data:', error);
      setError('Unable to connect to server. Please check if the server is running.');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  // Enhanced lobbies with game names
  const lobbiesWithDetails: LobbyWithDetails[] = lobbies.map(lobby => {
    const enhanced = {
      ...lobby,
      gameName: games[lobby.game_id]?.name || lobby.game_id,
    };
    
    // Debug log for completed games
    if (lobby.gameStatus) {
    }
    
    return enhanced;
  });

  // Categorize lobbies - check gameStatus from the lobby object
  const waitingLobbies = lobbiesWithDetails.filter(lobby => !lobby.started);
  const inProgressLobbies = lobbiesWithDetails.filter(lobby => lobby.started && (!lobby.gameStatus || lobby.gameStatus.state !== 'ended'));
  const finishedLobbies = lobbiesWithDetails.filter(lobby => lobby.gameStatus && lobby.gameStatus.state === 'ended');

  const renderLobbyCard = (lobby: LobbyWithDetails) => {
    const isFinished = lobby.gameStatus?.state === 'ended';
    const isInProgress = lobby.started && !isFinished;
    
    let statusText = 'Waiting for Players';
    let statusColor = 'text-yellow-400';
    
    if (isFinished) {
      if (lobby.gameStatus?.tie) {
        statusText = 'Tie Game';
      } else if (lobby.gameStatus?.winner) {
        // Map winner ID (p1/p2/p3/p4) to player name
        const winnerMatch = lobby.gameStatus.winner.match(/^p(\d+)$/);
        const winnerIndex = winnerMatch ? parseInt(winnerMatch[1]) - 1 : -1;
        const winnerName = winnerIndex >= 0 && winnerIndex < lobby.players.length 
          ? lobby.players[winnerIndex] 
          : lobby.gameStatus.winner;
        statusText = `Winner: ${winnerName}`;
      } else {
        statusText = 'Finished';
      }
      statusColor = 'text-gray-400';
    } else if (isInProgress) {
      statusText = lobby.currentTurn ? `${lobby.currentTurn}'s turn` : 'In Progress';
      statusColor = 'text-green-400';
    }

    return (
      <li
        key={lobby.id}
        onClick={() => onLobbySelected(lobby.id)}
        className="bg-gray-700 rounded-lg p-4 hover:bg-gray-600 transition-colors cursor-pointer"
      >
        <div className="flex justify-between items-start">
          <div className="flex-1">
            <h3 className="font-medium text-lg">{lobby.gameName}</h3>
            <p className="text-xs text-gray-500 mt-1">ID: {lobby.id}</p>
            <div className="mt-2 space-y-1">
              <p className="text-sm text-gray-400">
                Players: {lobby.players.length > 0 
                  ? lobby.players.map(p => p === player?.username ? `${p} (you)` : p).join(', ')
                  : 'None'}
              </p>
              <p className={`text-sm ${statusColor}`}>
                {statusText}
              </p>
            </div>
          </div>
          <div className="ml-4">
            <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
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

  // Show loading state
  if (isLoading) {
    return (
      <>
        <div>
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-xl font-semibold">Available Lobbies</h2>
            <button
              onClick={() => navigate('/create-lobby')}
              className="btn btn-primary"
              disabled
            >
              Create Lobby
            </button>
          </div>
          <div className="text-center py-8">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white mx-auto mb-4"></div>
            <p className="text-gray-400">Loading lobbies...</p>
          </div>
        </div>
        <WebSocketStatus connected={lobbiesWS.connected} state={lobbiesWS.state} />
      </>
    );
  }

  // Show error state
  if (error) {
    return (
      <>
        <div>
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-xl font-semibold">Available Lobbies</h2>
            <button
              onClick={() => navigate('/create-lobby')}
              className="btn btn-primary"
              disabled
            >
              Create Lobby
            </button>
          </div>
          <div className="text-center py-8">
            <div className="bg-red-900 bg-opacity-50 border border-red-600 text-red-200 p-6 rounded-lg">
              <p className="mb-4">{error}</p>
              <button
                onClick={loadData}
                className="btn btn-secondary"
              >
                Retry Connection
              </button>
            </div>
          </div>
        </div>
        <WebSocketStatus connected={lobbiesWS.connected} state={lobbiesWS.state} />
      </>
    );
  }

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
      <WebSocketStatus connected={lobbiesWS.connected} state={lobbiesWS.state} />
    </>
  );
}
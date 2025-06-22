import { useParams, useNavigate } from 'react-router-dom';
import { useEffect, useState } from 'react';
import { usePlayer } from '../context/PlayerContext';
import { useLobbyWebSocketContext } from '../context/LobbyWebSocketContext';
import { GameView } from '../components/GameView';
import ProtectedRoute from '../components/ProtectedRoute';
import type { PlayerInfo } from '../types/game-types';

export default function GamePage() {
  const { lobbyId, tableId } = useParams<{ lobbyId: string; tableId: string }>();
  const navigate = useNavigate();
  const { player } = usePlayer();
  
  const {
    lobbyState,
    connected,
    connectionState,
    sendGameAction,
    requestGameState,
  } = useLobbyWebSocketContext();

  // Get current game from lobby state
  // First check if there's an active game for this table
  const activeTable = lobbyState.tables?.find(t => t.id === tableId && t.status === 'Playing');
  
  // If we have an active table and a game in the lobby state, use it
  // Note: The game might have a different ID than the tableId
  const currentGame = activeTable && lobbyState.game ? lobbyState.game : null;

  // Debug logging
  console.log('[GamePage] Debug info:', {
    tableId,
    activeTable,
    tables: lobbyState.tables,
    lobbyStateGame: lobbyState.game,
    currentGame,
    gameId: lobbyState.game?.id,
    connected,
  });

  // Additional debug for game structure
  if (currentGame) {
    console.log('[GamePage] Game structure:', {
      gameState: currentGame.state,
      gameUI: currentGame.ui,
      zones: currentGame.ui?.zones,
      actionMap: currentGame.ui?.actionMap,
    });
  }

  // Handle game actions from GameView
  const handleEntityClick = (entityId: string, actionId: string) => {
    if (!currentGame) return;
    console.log('[GamePage] Entity click:', { entityId, actionId });
    sendGameAction(currentGame.id, {
      action: actionId,
      entity: entityId,
    });
  };

  const handleZoneClick = (zoneId: string, position?: [number, number], actionId?: string) => {
    if (!currentGame) return;
    console.log('[GamePage] Zone click:', { zoneId, position, actionId });
    
    const actionData: any = {
      action: actionId || 'place',
      zone: zoneId,
    };
    
    if (position) {
      actionData.position = position;
    }
    
    sendGameAction(currentGame.id, actionData);
  };

  // Give some time for the game data to load before redirecting
  const [hasWaited, setHasWaited] = useState(false);
  const [waitTime, setWaitTime] = useState(0);
  const [hasRequestedState, setHasRequestedState] = useState(false);
  
  useEffect(() => {
    // Wait up to 5 seconds for game data
    const interval = setInterval(() => {
      setWaitTime(prev => {
        if (prev >= 5) {
          setHasWaited(true);
          return prev;
        }
        return prev + 1;
      });
    }, 1000);
    
    return () => clearInterval(interval);
  }, []);
  
  // Request game state if table is active but we don't have game data
  useEffect(() => {
    if (connected && tableId && activeTable && activeTable.status === 'Playing' && !currentGame && !hasRequestedState) {
      console.log('[GamePage] Table is active but no game data, requesting state...');
      requestGameState(tableId);
      setHasRequestedState(true);
    }
  }, [connected, tableId, activeTable, currentGame, hasRequestedState, requestGameState]);

  // Only redirect if we've waited and there's definitely no game
  useEffect(() => {
    // Check if the table exists and is either Playing or we have game state
    const tableExists = lobbyState.tables?.some(t => t.id === tableId);
    const shouldStay = tableExists || currentGame || !hasWaited;
    
    if (connected && hasWaited && !shouldStay) {
      console.log('[GamePage] No active game after waiting, redirecting to lobby');
      navigate(`/lobby/${lobbyId}`);
    }
  }, [connected, hasWaited, currentGame, lobbyId, navigate, tableId, lobbyState.tables]);

  if (!connected || !hasWaited) {
    return (
      <div className="container mx-auto px-4 py-8 max-w-6xl">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white mx-auto mb-4"></div>
          <p className="text-gray-300">
            {!connected ? 'Connecting to game...' : 'Loading game data...'}
          </p>
        </div>
      </div>
    );
  }

  if (!currentGame || !currentGame.state) {
    // Check if table exists but game hasn't loaded yet
    if (activeTable && activeTable.status === 'Playing') {
      return (
        <div className="container mx-auto px-4 py-8 max-w-6xl">
          <div className="bg-gray-800 rounded-lg p-6 max-w-md mx-auto">
            <h2 className="text-xl font-bold mb-4">Active Game Session</h2>
            <div className="space-y-2 text-sm">
              <p><span className="text-gray-400">Table:</span> {tableId}</p>
              <p><span className="text-gray-400">Game:</span> {activeTable.bundleId}</p>
              <p><span className="text-gray-400">Status:</span> <span className="text-green-400">Playing</span></p>
              <p><span className="text-gray-400">Players:</span></p>
              <ul className="ml-4">
                {activeTable.seats.map((seat: any, idx: number) => seat && (
                  <li key={idx}>Seat {idx + 1}: {seat.username}</li>
                ))}
              </ul>
            </div>
            
            <div className="mt-6 border-t border-gray-700 pt-4">
              {waitTime < 5 ? (
                <>
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white mx-auto mb-2"></div>
                  <p className="text-sm text-gray-400 text-center">
                    {hasRequestedState ? 'Requesting game state from server...' : 'Reconnecting to game...'} ({5 - waitTime}s)
                  </p>
                </>
              ) : (
                <div className="text-center">
                  <p className="text-yellow-400 mb-4">
                    Unable to restore game state.
                  </p>
                  <p className="text-sm text-gray-400 mb-4">
                    The game is still active. Please return to the lobby and click "View Game" again.
                  </p>
                </div>
              )}
            </div>
            
            <button
              onClick={() => {
                sessionStorage.setItem('leftGameView', 'true');
                navigate(`/lobby/${lobbyId}`);
              }}
              className="w-full mt-4 bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-lg"
            >
              Back to Lobby
            </button>
          </div>
        </div>
      );
    }
    
    return (
      <div className="container mx-auto px-4 py-8 max-w-6xl">
        <div className="text-center">
          <h2 className="text-2xl font-bold text-white mb-4">Game Not Found</h2>
          <p className="text-gray-300 mb-6">The game you're looking for is not active.</p>
          <button
            onClick={() => {
              sessionStorage.setItem('leftGameView', 'true');
              navigate(`/lobby/${lobbyId}`);
            }}
            className="bg-blue-500 hover:bg-blue-600 text-white px-6 py-2 rounded-lg"
          >
            Back to Lobby
          </button>
        </div>
      </div>
    );
  }

  return (
    <ProtectedRoute>
      <div className="container mx-auto px-4 py-8 max-w-6xl">
        {/* Back to Lobby button */}
        <div className="mb-4">
          <button
            onClick={() => {
              sessionStorage.setItem('leftGameView', 'true');
              navigate(`/lobby/${lobbyId}`);
            }}
            className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-lg flex items-center gap-2"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
            Back to Lobby
          </button>
        </div>

        <GameView
          game={{
            id: currentGame.id,
            gameId: currentGame.ui?.gameMetadata?.gameId || 'unknown',
            state: currentGame.state,
            ui: currentGame.ui,
            you: lobbyState.you || 'p1',
            players: currentGame.state?.players?.reduce((acc: Record<string, PlayerInfo>, player: any, idx: number) => {
              const playerId = player.id || `p${idx + 1}`;
              acc[playerId] = {
                id: player.name || playerId,
                name: player.name || playerId,
                connected: true
              };
              return acc;
            }, {}) || {}
          }}
          entityUI={{}}
          onEntityClick={handleEntityClick}
          onZoneClick={handleZoneClick}
        />
      </div>
    </ProtectedRoute>
  );
}
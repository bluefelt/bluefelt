import { useState, useEffect, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';
import { useLobbyWebSocket } from '../ws/useLobbyWebSocket';
import { getLobby } from '../api/lobbies';
import { getPlayerColor } from '../config/colors';
import GameHeader from './GameHeader';
import GameZones from './GameZones';
import GameLog from './GameLog';
import GameResultBanner from './GameResultBanner';
import PhaseDisplay from './PhaseDisplay';
import type { GameManifest } from '../api/games';

interface GameViewProps {
  lobbyId: string;
  onLeave: () => void;
}

interface LogEntry {
  message: string;
  timestamp: string;
  player?: string;
  isYou?: boolean;
}

export default function GameView({ lobbyId }: GameViewProps) {
  const navigate = useNavigate();
  const { player } = usePlayer();
  const { sendMessage, lobbyState, connectionState, joinLobby, leaveLobby, startGame, disconnect } = useLobbyWebSocket(
    lobbyId,
    player!.username,
    false // Don't auto-join, let user decide
  );
  
  const [lobbyInfo, setLobbyInfo] = useState<{
    id: string;
    game_id: string;
    players: string[];
    started: boolean;
    manifest: GameManifest;
  } | null>(null);

  const [gameLog, setGameLog] = useState<LogEntry[]>([]);
  const [loadingError, setLoadingError] = useState(false);

  // Check if player has joined
  const joined = lobbyState.you && lobbyState.you !== "spectator";

  // Handle connection errors and lobby errors
  useEffect(() => {
    if (lobbyState.error) {
      console.error('Lobby error:', lobbyState.error);
      disconnect(); // Stop the WebSocket connection
      navigate('/', { replace: true });
    }
  }, [lobbyState.error, navigate, disconnect]);

  // Handle connection state errors
  useEffect(() => {
    if (connectionState === 'error') {
      console.error('WebSocket connection error');
      // Give it a moment to try reconnecting
      const timeout = setTimeout(() => {
        if (connectionState === 'error') {
          navigate('/', { replace: true });
        }
      }, 3000);
      return () => clearTimeout(timeout);
    }
  }, [connectionState, navigate]);

  // Fetch initial lobby info
  useEffect(() => {
    getLobby(lobbyId)
      .then(setLobbyInfo)
      .catch((error) => {
        console.error('Failed to load lobby:', error);
        setLoadingError(true);
      });
  }, [lobbyId]);

  // Redirect to home on error
  useEffect(() => {
    if (loadingError) {
      disconnect(); // Stop the WebSocket connection
      navigate('/', { replace: true });
    }
  }, [loadingError, navigate, disconnect]);

  // Update lobby info when we receive state updates
  useEffect(() => {
    if (lobbyState.meta && lobbyState.meta.players) {
      setLobbyInfo(prev => {
        if (!prev) return prev;
        return {
          ...prev,
          players: lobbyState.meta?.players || prev.players,
          started: lobbyState.started || false
        };
      });
    }
  }, [lobbyState.meta, lobbyState.started]);

  // Update game log from server
  useEffect(() => {
    if (lobbyState.meta?.gameLog) {
      console.log('[GameView] Updating game log, server log length:', lobbyState.meta.gameLog.length);
      // Reverse the array to show newest entries first
      setGameLog(lobbyState.meta.gameLog.slice().reverse().map((entry: any) => ({
        message: entry.message,
        timestamp: entry.timestamp,
        player: entry.player,
        isYou: entry.player === player?.username
      })));
    }
  }, [lobbyState.meta?.gameLog, player?.username]);

  const isYourTurn = lobbyState.you && 
                     lobbyState.you !== 'spectator' && 
                     lobbyState.state?.turn === lobbyState.you;
  
  // Debug logging
  console.log('[GameView Debug]', {
    you: lobbyState.you,
    turn: lobbyState.state?.turn,
    isYourTurn,
    possibleActions: lobbyState.meta?.possibleActions?.[lobbyState.you || ''],
    players: lobbyState.meta?.players
  });

  const currentPlayerIndex = lobbyState.state?.turn ? parseInt(lobbyState.state.turn.replace('p', '')) - 1 : -1;
  const currentPlayerName = lobbyState.meta?.players?.[currentPlayerIndex] || '';


  // Prepare player data for header
  const players = useMemo(() => {
    return lobbyState.meta?.players?.map((username) => ({
      username,
      isConnected: true // You might want to track actual connection state
    })) || [];
  }, [lobbyState.meta?.players]);

  // Handle board cell clicks
  const handleCellClick = (row: number, col: number) => {
    if (!isYourTurn) return;
    
    // Check if there's an action at this location
    const playerActions = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
    const location = `/zones/board/${row}/${col}`;
    const action = playerActions[location];
    
    if (action) {
      const message = JSON.stringify({
        action: action.action,
        args: { row, col }
      });
      sendMessage(message);
    }
  };

  // Handle card actions
  const handleCardAction = (zoneId: string, cardIndex: number) => {
    if (!isYourTurn) return;
    
    const playerActions = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
    let location: string;
    let action: any;
    
    if (cardIndex === -1) {
      // Zone-level action (e.g., drawing from deck)
      location = `/zones/${zoneId}`;
      action = playerActions[location];
      
      if (action) {
        const message = JSON.stringify({
          action: action.action,
          args: {} // No specific card for zone-level actions
        });
        sendMessage(message);
      }
    } else {
      // Card-specific action
      location = `/zones/${zoneId}/${cardIndex}`;
      action = playerActions[location];
      
      if (action) {
        const message = JSON.stringify({
          action: action.action,
          args: { card: cardIndex }
        });
        sendMessage(message);
      }
    }
  };


  // Check if can start game
  const canStart = lobbyInfo &&
    !lobbyState.started &&
    lobbyInfo.players &&
    lobbyInfo.manifest &&
    lobbyInfo.players.length >= lobbyInfo.manifest.metadata.players.min &&
    lobbyInfo.players.length <= lobbyInfo.manifest.metadata.players.max;

  if (!lobbyInfo || loadingError) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <p className="text-gray-400">Loading game...</p>
        </div>
      </div>
    );
  }
  
  // Ensure all required properties exist
  if (!lobbyInfo.manifest || !lobbyInfo.players) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <p className="text-gray-400">Invalid lobby data</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white flex flex-col">
      {/* Header */}
      <GameHeader
        lobbyId={lobbyId}
        gameId={lobbyId.slice(-10).toUpperCase()}
        gameName={lobbyInfo.manifest.metadata.name}
        status={lobbyState.meta?.gameStatus?.state === 'ended' ? 'finished' : 
                lobbyState.started ? 'in_progress' : 'waiting'}
        players={players}
        currentPlayer={currentPlayerName}
        entityDefinitions={lobbyState.meta?.entities}
        turnPrompt={(() => {
          // First check if there's a phase prompt
          if (lobbyState.meta?.currentPhasePrompt) {
            return lobbyState.meta.currentPhasePrompt;
          }
          
          // Otherwise fall back to action directions
          const actionMap = lobbyState.meta?.actionMap?.[lobbyState.you || ''];
          if (!actionMap) return undefined;
          
          // Get unique directions from all available actions
          const directions = new Set<string>();
          Object.values(actionMap).forEach((action: any) => {
            if (action?.direction) {
              directions.add(action.direction);
            }
          });
          
          // If all actions have the same direction, use that
          // Otherwise, use the first one (could be enhanced to combine them)
          if (directions.size === 1) {
            return Array.from(directions)[0];
          } else if (directions.size > 0) {
            return Array.from(directions)[0];
          }
          return undefined;
        })()}
      />

      {/* Phase display */}
      <PhaseDisplay 
        phaseMessages={lobbyState.meta?.phaseDisplayMessages}
        phaseStates={lobbyState.meta?.phaseStates}
      />

      {/* Main content */}
      <div className="flex-1 flex flex-col">
        {/* Game board or lobby info */}
        <div className="flex-1 container mx-auto px-4 py-6">
          {lobbyState.started ? (
            <>
              {/* Game result banner - shown above the board when game ends */}
              <GameResultBanner
                gameStatus={lobbyState.meta?.gameStatus}
                you={lobbyState.you}
                playerNames={lobbyState.meta?.players}
              />
              
              <GameZones
                zones={lobbyState.state?.zones}
                entityDefinitions={lobbyState.meta?.entities}
                onCellClick={handleCellClick}
                onCardAction={handleCardAction}
                isMyTurn={!!isYourTurn}
                zoneMetadata={(lobbyInfo.manifest as any)?.zones || (lobbyState.meta as any)?.zones}
                playerNames={lobbyState.meta?.players}
                actionMap={lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {}}
                selection={(lobbyState.state as any)?.meta?.selection}
                you={lobbyState.you}
              />
            </>
          ) : (
            /* Pre-game lobby view */
            <div className="max-w-4xl mx-auto">
              <div className="grid gap-6 md:grid-cols-2">
                {/* Lobby Info Card */}
                <div className="bg-gray-800 rounded-lg p-6">
                  <h3 className="text-xl font-semibold mb-4">Lobby Info</h3>
                  <div className="space-y-3 text-gray-300">
                    <p><strong className="text-white">Lobby ID:</strong> {lobbyId}</p>
                    <p><strong className="text-white">Game:</strong> {lobbyInfo.manifest.metadata.name}</p>
                    <p><strong className="text-white">Players:</strong> {lobbyInfo.players.length} / {lobbyInfo.manifest.metadata.players.min === lobbyInfo.manifest.metadata.players.max 
                      ? lobbyInfo.manifest.metadata.players.min 
                      : `${lobbyInfo.manifest.metadata.players.min}-${lobbyInfo.manifest.metadata.players.max}`}</p>
                    <div>
                      <strong className="text-white">Connected Players:</strong>
                      <ul className="mt-2 space-y-1">
                        {lobbyInfo.players.map((playerName, index) => {
                          const myIndex = lobbyInfo.players.findIndex(p => p === player?.username);
                          const color = player ? getPlayerColor(index, player.color, myIndex) : { hex: '#888' };
                          return (
                            <li key={playerName} className="flex items-center space-x-2 ml-4">
                              <div 
                                className="w-3 h-3 rounded-sm"
                                style={{ backgroundColor: color.hex }}
                              />
                              <span>{playerName}</span>
                            </li>
                          );
                        })}
                      </ul>
                    </div>
                  </div>
                  
                  <div className="mt-6 space-y-3">
                    {joined ? (
                      <>
                        <button onClick={leaveLobby} className="w-full btn btn-secondary">
                          Leave Lobby
                        </button>
                        {canStart && (
                          <button onClick={startGame} className="w-full btn btn-primary">
                            Start Game
                          </button>
                        )}
                      </>
                    ) : (
                      <button onClick={joinLobby} className="w-full btn btn-primary">
                        Join Lobby
                      </button>
                    )}
                  </div>
                </div>

                {/* Game Info Card */}
                <div className="bg-gray-800 rounded-lg p-6">
                  <h3 className="text-xl font-semibold mb-4">Game Details</h3>
                  <div className="space-y-3 text-gray-300">
                    <p><strong className="text-white">Author:</strong> {lobbyInfo.manifest.metadata.author}</p>
                    <p><strong className="text-white">Description:</strong> {lobbyInfo.manifest.metadata.description}</p>
                    <p><strong className="text-white">Version:</strong> {lobbyInfo.manifest.version}</p>
                    <p><strong className="text-white">Players Required:</strong> {lobbyInfo.manifest.metadata.players.min === lobbyInfo.manifest.metadata.players.max 
                      ? lobbyInfo.manifest.metadata.players.min 
                      : `${lobbyInfo.manifest.metadata.players.min} - ${lobbyInfo.manifest.metadata.players.max}`}</p>
                  </div>
                  
                  {!canStart && !lobbyState.started && lobbyInfo.players.length > 0 && (
                    <div className="mt-4 p-3 bg-yellow-900 bg-opacity-50 rounded text-yellow-300 text-sm">
                      Waiting for {lobbyInfo.manifest.metadata.players.min - lobbyInfo.players.length} more player{lobbyInfo.manifest.metadata.players.min - lobbyInfo.players.length !== 1 ? 's' : ''} to start
                    </div>
                  )}
                </div>
              </div>
              
              {/* Connection Status */}
              <div className="mt-6 text-center">
                <p className="text-sm text-gray-400">
                  Connection Status: <span className={connectionState === 'connected' ? 'text-green-400' : 'text-yellow-400'}>
                    {connectionState}
                  </span>
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Game log - only show during game */}
        {lobbyState.started && <GameLog entries={gameLog} playerNames={lobbyState.meta?.players} />}
      </div>
    </div>
  );
}
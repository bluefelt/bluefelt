import { useState, useEffect, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';
import { useLobbyWebSocket } from '../ws/useLobbyWebSocket';
import { getLobby } from '../api/lobbies';
import { useGameActions } from '../hooks/useGameActions';
import GameHeader from './GameHeader';
import GameZones from './GameZones';
import GameLog from './GameLog';
import GameResultBanner from './GameResultBanner';
import PhaseDisplay from './PhaseDisplay';
import PhaseTracker from './PhaseTracker';
import GameControls from './GameControls';
import PlayerList from './PlayerList';
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
    false
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

  // Game state calculations
  const joined = lobbyState.you && lobbyState.you !== "spectator";
  const currentPlayer = lobbyState.game?.currentPlayer;
  const isYourTurn = lobbyState.you && 
                     lobbyState.you !== 'spectator' && 
                     currentPlayer === lobbyState.you &&
                     lobbyState.game?.gameStatus?.state !== 'ended';
  
  const currentPlayerIndex = currentPlayer ? parseInt(currentPlayer.replace('p', '')) - 1 : -1;
  const currentPlayerName = lobbyState.ui?.players?.[currentPlayerIndex] || '';

  // Use game actions hook
  const { handleCellClick, handleCardAction, handleZoneAction } = useGameActions({
    isYourTurn,
    lobbyState,
    sendMessage
  });

  // Handle connection errors
  useEffect(() => {
    if (lobbyState.error) {
      if (lobbyState.error === 'Lobby does not exist') {
        disconnect();
        navigate('/', { 
          replace: true,
          state: { message: 'The game lobby was lost (server may have restarted). Please create a new game.' }
        });
      } else {
        disconnect();
        navigate('/', { replace: true });
      }
    }
  }, [lobbyState.error, navigate, disconnect]);

  // Handle connection state errors
  useEffect(() => {
    if (connectionState === 'error') {
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
      .catch(() => setLoadingError(true));
  }, [lobbyId]);

  // Redirect on error
  useEffect(() => {
    if (loadingError) {
      disconnect();
      navigate('/', { replace: true });
    }
  }, [loadingError, disconnect, navigate]);

  // Handle disconnect on unmount
  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  // Update game log from server
  useEffect(() => {
    if (lobbyState.ui?.gameLog) {
      setGameLog(lobbyState.ui.gameLog.slice().reverse().map((entry: any) => ({
        message: entry.message,
        timestamp: entry.timestamp,
        player: entry.player,
        isYou: entry.player === player?.username
      })));
    }
  }, [lobbyState.ui?.gameLog, player?.username]);

  // Handle leaving the game
  const handleLeave = () => {
    if (joined) {
      leaveLobby();
    }
    navigate('/lobbies');
  };

  // Prepare player data for header
  const players = useMemo(() => {
    return lobbyState.ui?.players?.map((username) => ({
      username,
      isConnected: true
    })) || [];
  }, [lobbyState.ui?.players]);

  // Loading state
  if (connectionState === 'connecting' || !lobbyInfo) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white mx-auto mb-4"></div>
          <p className="text-gray-300">Connecting to game...</p>
        </div>
      </div>
    );
  }

  // Error state
  if (connectionState === 'error') {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center text-red-500">
          <p className="text-xl mb-4">Connection error</p>
          <button
            onClick={() => navigate('/')}
            className="px-4 py-2 bg-gray-700 text-white rounded hover:bg-gray-600"
          >
            Return to Home
          </button>
        </div>
      </div>
    );
  }

  const gameStarted = lobbyState.started || lobbyState.game;
  const manifest = lobbyState.ui?.manifest || lobbyInfo?.manifest;
  const canStartGame = players.length >= (manifest?.metadata?.players?.min || 2);
  const gameStatus = lobbyState.game?.gameStatus;

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <div className="max-w-7xl mx-auto px-4 py-8">
        <div className="mb-6">
          <h1 className="text-3xl font-bold">{manifest?.metadata?.name || 'Game'}</h1>
          <p className="text-gray-400 mt-2">
            {manifest?.metadata?.description || 'A fun game to play with friends'}
          </p>
        </div>
        
        <GameControls
          joined={joined}
          isGameStarted={gameStarted}
          canStartGame={canStartGame}
          onJoin={() => joinLobby(player!.username)}
          onStart={startGame}
          onLeave={handleLeave}
          playerUsername={player!.username}
        />

        {!gameStarted && (
          <>
            <PlayerList 
              players={lobbyState.ui?.players || []}
              maxPlayers={manifest?.metadata?.players?.max}
              currentPlayer={player?.username}
            />
            <div className="mb-6 p-4 bg-gray-800 rounded-lg">
              <p className="text-gray-300">
                {players.length < (manifest?.metadata?.players?.min || 2)
                  ? `Waiting for ${(manifest?.metadata?.players?.min || 2) - players.length} more player(s) to join...`
                  : joined
                  ? 'Ready to start! Click "Start Game" when all players have joined.'
                  : 'Click "Join Game" to play!'}
              </p>
            </div>
          </>
        )}

        {gameStatus?.state === 'ended' && (
          <GameResultBanner 
            winner={gameStatus.winner}
            tie={gameStatus.tie}
            playerNames={lobbyState.ui?.players || []}
          />
        )}

        {gameStarted && (
          <>
            <GameHeader
              lobbyId={lobbyId}
              gameId={lobbyId}
              gameName={manifest?.metadata?.name || 'Game'}
              status={gameStatus?.state === 'ended' ? 'finished' : 'in_progress'}
              players={players}
              currentPlayer={currentPlayerName}
              entityDefinitions={lobbyState.ui?.entities}
              turnPrompt={(() => {
                const actionMap = lobbyState.ui?.actionMap?.[lobbyState.you || ''];
                if (actionMap && Object.keys(actionMap).length > 0) {
                  // Get the first action's direction
                  const firstAction = Object.values(actionMap)[0];
                  return firstAction?.direction || undefined;
                }
                return undefined;
              })()}
            />

            {lobbyState.ui?.phases && (
              <div className="mb-6">
                <PhaseTracker
                  phases={lobbyState.ui.phases}
                  currentPhaseId={lobbyState.game?.phases?.game?.current}
                />
                <PhaseDisplay
                  phaseMessages={lobbyState.ui?.phaseMessages}
                  phaseStates={lobbyState.game?.phases}
                />
              </div>
            )}

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              <div className="lg:col-span-2">
                <GameZones
                  zones={lobbyState.game?.zones}
                  entityDefinitions={lobbyState.ui?.entities}
                  onCellClick={handleCellClick}
                  onCardAction={handleCardAction}
                  onAction={handleZoneAction}
                  isMyTurn={isYourTurn}
                  you={lobbyState.you}
                  zoneMetadata={lobbyState.ui?.zones}
                  playerNames={lobbyState.ui?.players}
                  actionMap={lobbyState.ui?.actionMap?.[lobbyState.you || ''] || {}}
                  selection={lobbyState.game?.selection}
                  zoneGroups={lobbyState.ui?.zoneGroups}
                />
              </div>
              <div>
                <GameLog entries={gameLog} />
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
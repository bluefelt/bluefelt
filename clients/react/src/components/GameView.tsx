import { useState, useEffect, useMemo } from 'react';
import { usePlayer } from '../context/PlayerContext';
import { useLobbyWebSocket } from '../ws/useLobbyWebSocket';
import { getLobby } from '../api/lobbies';
import GameHeader from './GameHeader';
import TurnBanner from './TurnBanner';
import Board from './Board';
import GameLog from './GameLog';
import GameEndDisplay from './GameEndDisplay';
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

export default function GameView({ lobbyId, onLeave }: GameViewProps) {
  const { player } = usePlayer();
  const { sendMessage, lobbyState, connectionState } = useLobbyWebSocket(
    lobbyId,
    player!.username,
    true // Auto-join
  );
  
  const [lobbyInfo, setLobbyInfo] = useState<{
    id: string;
    game_id: string;
    players: string[];
    started: boolean;
    manifest: GameManifest;
  } | null>(null);

  const [gameLog, setGameLog] = useState<LogEntry[]>([]);

  // Fetch initial lobby info
  useEffect(() => {
    getLobby(lobbyId).then(setLobbyInfo).catch(() => {});
  }, [lobbyId]);

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

  // Track game actions in the log
  useEffect(() => {
    // Check for game started
    if (lobbyState.started && gameLog.length === 0) {
      const now = new Date();
      const timestamp = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
      setGameLog([{
        message: 'The game has started',
        timestamp
      }]);
    }

    // You can add more sophisticated action tracking here based on state changes
    // For now, let's track turn changes
    if (lobbyState.state?.turn && lobbyState.meta?.players) {
      const currentPlayerIndex = parseInt(lobbyState.state.turn.replace('p', '')) - 1;
      const currentPlayerName = lobbyState.meta.players[currentPlayerIndex];
      
      // This is a placeholder - you'd want to track actual moves
      // by comparing previous and current board states
    }
  }, [lobbyState, gameLog.length]);

  const isYourTurn = lobbyState.you && 
                     lobbyState.you !== 'spectator' && 
                     lobbyState.state?.turn === lobbyState.you;

  const currentPlayerIndex = lobbyState.state?.turn ? parseInt(lobbyState.state.turn.replace('p', '')) - 1 : -1;
  const currentPlayerName = lobbyState.meta?.players?.[currentPlayerIndex] || '';

  // Get action instruction
  const instruction = useMemo(() => {
    if (!isYourTurn) return '';
    const verbs = lobbyState.meta?.possibleVerbs?.[lobbyState.you || ''] || [];
    return verbs[0]?.direction || 'Make your move';
  }, [isYourTurn, lobbyState.meta?.possibleVerbs, lobbyState.you]);

  // Prepare player data for header
  const players = useMemo(() => {
    return lobbyState.meta?.players?.map((username, index) => ({
      username,
      isConnected: true // You might want to track actual connection state
    })) || [];
  }, [lobbyState.meta?.players]);

  // Handle board cell clicks
  const handleCellClick = (row: number, col: number) => {
    if (!isYourTurn) return;
    
    // Get the appropriate verb (usually "place" for tic-tac-toe)
    const verbs = lobbyState.meta?.possibleVerbs?.[lobbyState.you || ''] || [];
    const placeVerb = verbs.find(v => v.verb === 'place') || verbs[0];
    
    if (placeVerb) {
      const message = JSON.stringify({
        verb: placeVerb.verb,
        args: { row, col }
      });
      sendMessage(message);
      
      // Add to game log
      const now = new Date();
      const timestamp = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
      const playerName = player?.username || '';
      
      setGameLog(prev => [{
        message: `You placed a mark on ${row}, ${col}`,
        timestamp,
        player: playerName,
        isYou: true
      }, ...prev].slice(0, 50));
    }
  };

  // Add game end to log
  useEffect(() => {
    if (lobbyState.meta?.gameStatus?.state === 'ended') {
      const now = new Date();
      const timestamp = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
      
      let message = 'The game has ended';
      if (lobbyState.meta.gameStatus.winner) {
        const winnerIndex = parseInt(lobbyState.meta.gameStatus.winner.replace('p', '')) - 1;
        const winnerName = lobbyState.meta.players?.[winnerIndex];
        message = `${winnerName} wins!`;
      } else if (lobbyState.meta.gameStatus.tie) {
        message = "It's a tie!";
      }
      
      setGameLog(prev => {
        // Check if we already added this message
        if (prev.length > 0 && prev[0].message === message) {
          return prev;
        }
        return [{
          message,
          timestamp
        }, ...prev].slice(0, 50);
      });
    }
  }, [lobbyState.meta?.gameStatus, lobbyState.meta?.players]);

  if (!lobbyInfo) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <p className="text-gray-400">Loading game...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white flex flex-col">
      {/* Game ended modal */}
      <GameEndDisplay 
        gameStatus={lobbyState.meta?.gameStatus}
        you={lobbyState.you}
        playerNames={lobbyState.meta?.players}
        onClose={() => {}}
      />

      {/* Header */}
      <GameHeader
        lobbyId={lobbyId}
        gameId={lobbyId.slice(-10).toUpperCase()}
        gameName={lobbyInfo.manifest.metadata.name}
        status={lobbyState.meta?.gameStatus?.state === 'ended' ? 'finished' : 
                lobbyState.started ? 'in_progress' : 'waiting'}
        players={players}
        currentPlayer={lobbyState.state?.turn}
      />

      {/* Turn indicator */}
      {lobbyState.started && lobbyState.meta?.gameStatus?.state !== 'ended' && (
        <TurnBanner
          isYourTurn={!!isYourTurn}
          currentPlayer={currentPlayerName}
          instruction={instruction}
        />
      )}

      {/* Main content */}
      <div className="flex-1 flex flex-col">
        {/* Game board */}
        <div className="flex-1 container mx-auto px-4 py-6">
          {lobbyState.started ? (
            <Board
              zones={lobbyState.state?.zones}
              playerId={player!.username}
              entityDefinitions={lobbyState.meta?.entities}
              currentPlayer={lobbyState.state?.turn}
              onCellClick={handleCellClick}
              isMyTurn={!!isYourTurn}
            />
          ) : (
            <div className="bg-gray-800 p-6 rounded-lg text-center">
              <p className="text-gray-400 mb-4">Waiting for game to start...</p>
              <p className="text-sm text-gray-500">
                {lobbyInfo.players.length} / {lobbyInfo.manifest.metadata.players.min}-{lobbyInfo.manifest.metadata.players.max} players
              </p>
            </div>
          )}
        </div>

        {/* Game log */}
        <GameLog entries={gameLog} />
      </div>
    </div>
  );
}
import { useState, useEffect } from 'react';
import GameZones from './GameZones';
import GameResultBanner from './GameResultBanner';
import PhaseDisplay from './PhaseDisplay';
import { useGameActions } from '../hooks/useGameActions';

interface TestGameViewProps {
  initialState: any;
}

export default function TestGameView({ initialState }: TestGameViewProps) {
  const [lobbyState, setLobbyState] = useState(initialState);
  
  // Update state when initialState prop changes
  useEffect(() => {
    setLobbyState(initialState);
  }, [initialState]);

  // Mock sendMessage for test mode
  const sendMessage = (message: any) => {
    console.log('Test Mode - Would send message:', message);
    // In test mode, we could simulate some server responses here
  };

  // Game state calculations
  const currentPlayer = lobbyState.game?.currentPlayer;
  const isYourTurn = true; // Always allow actions in test mode
  
  const currentPlayerIndex = currentPlayer ? parseInt(currentPlayer.replace('p', '')) - 1 : -1;
  const currentPlayerName = lobbyState.game?.players?.[currentPlayerIndex] || '';

  // Use game actions hook
  const { handleCellClick, handleCardAction, handleZoneAction, handleChoiceSelect } = useGameActions({
    isYourTurn,
    lobbyState,
    sendMessage
  });

  const isGameEnded = lobbyState.meta?.gameStatus?.state === 'ended';

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 flex flex-col gap-4">
        {/* Game Header Info */}
        <div className="bg-white rounded-lg shadow p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <PhaseDisplay phase={lobbyState.game?.phase || 'setup'} />
              <div className="text-sm text-gray-600">
                Current Player: <span className="font-medium">{currentPlayerName || currentPlayer}</span>
              </div>
            </div>
            <div className="text-sm text-gray-500">
              Test Mode
            </div>
          </div>
        </div>

        {/* Game Content */}
        <div className="flex-1 min-h-0">
          {/* Game Result Banner */}
          {isGameEnded && (
            <div className="mb-4">
              <GameResultBanner
                gameStatus={lobbyState.meta?.gameStatus}
                players={lobbyState.game?.players || []}
                you={currentPlayer}
              />
            </div>
          )}

          {/* Game Zones */}
          <GameZones
            zones={lobbyState.zones || {}}
            actionMap={lobbyState.ui?.actionMap || {}}
            selection={lobbyState.game?.selection}
            onCellClick={handleCellClick}
            onCardAction={handleCardAction}
            onZoneAction={handleZoneAction}
            onChoiceSelect={handleChoiceSelect}
            disabled={isGameEnded}
          />
        </div>

        {/* State Inspector */}
        <div className="mt-4">
          <details className="bg-gray-100 rounded-lg p-2">
            <summary className="cursor-pointer font-medium text-sm">State Inspector</summary>
            <pre className="mt-2 text-xs overflow-auto max-h-40">
              {JSON.stringify(lobbyState, null, 2)}
            </pre>
          </details>
        </div>
      </div>
    </div>
  );
}
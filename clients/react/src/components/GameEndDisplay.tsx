import React from 'react';

type GameEndDisplayProps = {
  gameStatus?: {
    state: string;
    winner?: string;
    tie?: boolean;
  };
  you?: string;
  playerNames?: string[];  // Array of usernames in order (p1, p2, etc.)
  onNewGame?: () => void;
  onClose?: () => void;
};

// Helper to get username from actor ID
function getPlayerName(actorId: string, playerNames?: string[]): string {
  if (!playerNames || !actorId.startsWith('p')) return actorId;
  
  const playerIndex = parseInt(actorId.substring(1)) - 1;
  if (playerIndex >= 0 && playerIndex < playerNames.length) {
    return playerNames[playerIndex];
  }
  
  return actorId;
}

export default function GameEndDisplay({ gameStatus, you, playerNames, onNewGame, onClose }: GameEndDisplayProps) {
  const [isVisible, setIsVisible] = React.useState(true);
  
  React.useEffect(() => {
    // Reset visibility when game status changes
    if (gameStatus?.state === 'ended') {
      setIsVisible(true);
    }
  }, [gameStatus?.state]);
  
  if (!gameStatus || gameStatus.state !== 'ended' || !isVisible) {
    return null;
  }

  const isWinner = gameStatus.winner === you;
  const isLoser = gameStatus.winner && !isWinner && you !== 'spectator';
  const isTie = gameStatus.tie;
  const winnerName = gameStatus.winner ? getPlayerName(gameStatus.winner, playerNames) : '';

  const handleClose = () => {
    setIsVisible(false);
    onClose?.();
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-8 max-w-md text-center space-y-4 relative">
        <button
          onClick={handleClose}
          className="absolute top-4 right-4 text-gray-400 hover:text-white transition-colors"
          aria-label="Close"
        >
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
        <h2 className="text-3xl font-bold">
          {isTie ? (
            <span className="text-yellow-400">It's a Tie!</span>
          ) : isWinner ? (
            <span className="text-green-400">You Win!</span>
          ) : isLoser ? (
            <span className="text-red-400">You Lose!</span>
          ) : (
            <span className="text-blue-400">
              {winnerName} Wins!
            </span>
          )}
        </h2>
        
        <p className="text-gray-300">
          {isTie
            ? "The game ended in a draw. Well played!"
            : isWinner
            ? "Congratulations on your victory!"
            : isLoser
            ? "Better luck next time!"
            : `${winnerName} has won the game.`}
        </p>

        <div className="flex gap-3 justify-center mt-4">
          {onNewGame && (
            <button
              onClick={onNewGame}
              className="btn btn-primary"
            >
              New Game
            </button>
          )}
          <button
            onClick={handleClose}
            className="btn btn-secondary"
          >
            View Board
          </button>
        </div>
      </div>
    </div>
  );
}
import React from 'react';

type GameResultBannerProps = {
  gameStatus?: {
    state: string;
    winner?: string;
    tie?: boolean;
  };
  you?: string;
  playerNames?: string[];  // Array of usernames in order (p1, p2, etc.)
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

export default function GameResultBanner({ gameStatus, you, playerNames }: GameResultBannerProps) {
  if (!gameStatus || gameStatus.state !== 'ended') {
    return null;
  }

  const isWinner = gameStatus.winner === you;
  const isLoser = gameStatus.winner && !isWinner && you !== 'spectator';
  const isTie = gameStatus.tie;
  const winnerName = gameStatus.winner ? getPlayerName(gameStatus.winner, playerNames) : '';

  return (
    <div className="mb-6 p-6 bg-gray-800 rounded-lg text-center">
      <div className="text-2xl font-bold mb-2">
        {isTie ? (
          <span className="text-yellow-400">Game Ended in a Tie!</span>
        ) : (
          <span className="text-yellow-400">
            {winnerName} Won!
          </span>
        )}
      </div>
      
      {you !== 'spectator' && (
        <p className="text-lg text-gray-300">
          {isTie
            ? "The game ended in a draw."
            : isWinner
            ? "Congratulations! You won this game."
            : "Better luck next time!"}
        </p>
      )}
    </div>
  );
}
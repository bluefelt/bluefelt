
type GameResultBannerProps = {
  winner?: string | null;
  tie?: boolean;
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

export default function GameResultBanner({ winner, tie, playerNames }: GameResultBannerProps) {
  if (!winner && !tie) {
    return null;
  }

  const winnerName = winner ? getPlayerName(winner, playerNames) : '';

  return (
    <div className="mb-6 p-6 bg-gray-800 rounded-lg text-center">
      <div className="text-2xl font-bold mb-2">
        {tie ? (
          <span className="text-yellow-400">Game Ended in a Tie!</span>
        ) : (
          <span className="text-yellow-400">
            Player {winnerName} wins!
          </span>
        )}
      </div>
    </div>
  );
}
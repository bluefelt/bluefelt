interface TurnBannerProps {
  isYourTurn: boolean;
  currentPlayer: string;
  instruction: string;
}

export default function TurnBanner({ isYourTurn, currentPlayer, instruction }: TurnBannerProps) {
  if (!isYourTurn) {
    return (
      <div className="bg-gray-700 px-4 py-3 text-center">
        <span className="text-gray-300">Waiting for {currentPlayer} to play...</span>
      </div>
    );
  }

  return (
    <div className="bg-yellow-600 text-black px-4 py-3 flex items-center">
      <div className="bg-black text-yellow-600 px-3 py-1 font-bold text-sm mr-4">
        YOUR TURN
      </div>
      <div className="font-medium">
        {instruction}
      </div>
    </div>
  );
}
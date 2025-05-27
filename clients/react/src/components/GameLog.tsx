interface LogEntry {
  message: string;
  timestamp: string;
  player?: string;
  isYou?: boolean;
}

interface GameLogProps {
  entries: LogEntry[];
}

export default function GameLog({ entries }: GameLogProps) {
  // Define player colors matching the header
  const getPlayerColor = (isYou?: boolean) => {
    return isYou ? '#FF1493' : '#FFD700'; // Pink for current player, gold for others
  };

  return (
    <div className="bg-gray-900 border-t border-gray-800">
      <div className="px-4 py-3 border-b border-gray-800">
        <h3 className="text-lg font-medium text-white">Game Log</h3>
      </div>
      <div className="max-h-48 overflow-y-auto game-log-scroll">
        {entries.length === 0 ? (
          <div className="px-4 py-3 text-gray-500 text-sm">
            No actions yet
          </div>
        ) : (
          <div className="divide-y divide-gray-800">
            {entries.map((entry, index) => (
              <div key={index} className="px-4 py-2 flex justify-between items-center text-sm">
                <span 
                  className={entry.player ? 'font-medium' : 'text-gray-400'}
                  style={{ color: entry.player ? getPlayerColor(entry.isYou) : undefined }}
                >
                  {entry.message}
                </span>
                <span className="text-gray-500 text-xs ml-4">
                  {entry.timestamp}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
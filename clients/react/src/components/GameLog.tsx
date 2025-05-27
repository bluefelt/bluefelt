import { usePlayer } from '../context/PlayerContext';
import { getColorById, getPlayerColor } from '../config/colors';

interface LogEntry {
  message: string;
  timestamp: string;
  player?: string;
  isYou?: boolean;
}

interface GameLogProps {
  entries: LogEntry[];
  playerNames?: string[];
}

export default function GameLog({ entries, playerNames }: GameLogProps) {
  const { player } = usePlayer();
  
  // Get player color based on their position
  const getLogPlayerColor = (playerName?: string, isYou?: boolean) => {
    if (!player || !playerName || !playerNames) return undefined;
    
    if (isYou) {
      return getColorById(player.color).hex;
    }
    
    const playerIndex = playerNames.findIndex(name => name === playerName);
    const myPlayerIndex = playerNames.findIndex(name => name === player.username);
    
    if (playerIndex === -1 || myPlayerIndex === -1) return undefined;
    
    const color = getPlayerColor(playerIndex, player.color, myPlayerIndex);
    return color.hex;
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
                  style={{ color: entry.player ? getLogPlayerColor(entry.player, entry.isYou) : undefined }}
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
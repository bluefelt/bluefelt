import { theme } from '../theme/constants';
import { Card } from './ui';

interface PlayerListProps {
  players: string[];
  maxPlayers?: number;
  currentPlayer?: string;
}

export default function PlayerList({ players, maxPlayers, currentPlayer }: PlayerListProps) {
  return (
    <Card className="mb-6">
      <h3 className="text-xl font-semibold mb-4">Players in Lobby</h3>
      <div className="space-y-2">
        {players.map((player, index) => (
          <div
            key={player}
            className={`flex items-center justify-between p-2 rounded ${
              player === currentPlayer ? 'bg-gray-700' : ''
            }`}
          >
            <span className={theme.colors.text.primary}>
              {index + 1}. {player}
              {player === currentPlayer && (
                <span className={`ml-2 ${theme.colors.text.muted}`}>(You)</span>
              )}
            </span>
          </div>
        ))}
        {maxPlayers && players.length < maxPlayers && (
          <div className={`p-2 ${theme.colors.text.muted} italic`}>
            Waiting for {maxPlayers - players.length} more player{maxPlayers - players.length > 1 ? 's' : ''}...
          </div>
        )}
      </div>
    </Card>
  );
}

export type Player = { id: string; mark?: string };

import type { EntityDefinition } from '../types/messages';
import { buildGlyphMapping, getEntityGlyph } from '../utils/entityUtils';

type Props = {
  you?: string;
  turn?: string;
  players?: Player[];
  playerNames?: string[];  // Array of usernames in order (p1, p2, etc.)
  entities?: EntityDefinition[];
  gameStatus?: {
    state: string;
    winner?: string;
    tie?: boolean;
  };
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

export default function TurnIndicator({ you, turn, players, playerNames, entities, gameStatus }: Props) {
  if (!turn || !players) return null;
  
  // Build glyph mapping from entities
  const glyphMapping = buildGlyphMapping(entities);
  
  // Don't show turn indicator if game has ended
  if (gameStatus?.state === 'ended') {
    return (
      <div style={{ marginBottom: 16, fontWeight: "bold" }}>
        {gameStatus.tie ? (
          "Game ended in a tie!"
        ) : gameStatus.winner ? (
          `${getPlayerName(gameStatus.winner, playerNames)} wins!`
        ) : (
          "Game ended!"
        )}
      </div>
    );
  }
  
  const player = players.find(p => p.id === turn);
  const glyph = player?.mark ? getEntityGlyph(player.mark, glyphMapping) : "";
  const turnPlayerName = getPlayerName(turn, playerNames);
  
  const label = you && you === turn
    ? `It is your turn`
    : `It is ${turnPlayerName}'s turn`;
  
  return (
    <div style={{ marginBottom: 16, fontWeight: "bold" }}>
      {label} {glyph && `(${glyph})`}
    </div>
  );
}

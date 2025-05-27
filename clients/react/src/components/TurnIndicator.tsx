import React from "react";

export type Player = { id: string; mark: string };

const markToGlyph: Record<string, string> = {
  mark_x: "X",
  mark_o: "O",
};

type Props = {
  you?: string;
  turn?: string;
  players?: Player[];
  gameStatus?: {
    state: string;
    winner?: string;
    tie?: boolean;
  };
};

export default function TurnIndicator({ you, turn, players, gameStatus }: Props) {
  if (!turn || !players) return null;
  
  // Don't show turn indicator if game has ended
  if (gameStatus?.state === 'ended') {
    return (
      <div style={{ marginBottom: 16, fontWeight: "bold" }}>
        {gameStatus.tie ? (
          "Game ended in a tie!"
        ) : (
          `${gameStatus.winner === 'p1' ? 'Player 1' : 'Player 2'} wins!`
        )}
      </div>
    );
  }
  
  const player = players.find(p => p.id === turn);
  const glyph = player ? markToGlyph[player.mark] ?? player.mark : "";
  const label = you && you === turn
    ? `It is your turn`
    : `It is ${player ? player.id : turn}'s turn`;
  return (
    <div style={{ marginBottom: 16, fontWeight: "bold" }}>
      {label} {glyph && `(${glyph})`}
    </div>
  );
}

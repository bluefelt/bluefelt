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
};

export default function TurnIndicator({ you, turn, players }: Props) {
  if (!turn || !players) return null;
  const player = players.find(p => p.id === turn);
  const glyph = player ? markToGlyph[player.mark] : "";
  const label = you && you === turn
    ? `It is your turn`
    : `It is ${player ? player.id : turn}'s turn`;
  return (
    <div style={{ marginBottom: 16, fontWeight: "bold" }}>
      {label} {glyph && `(${glyph})`}
    </div>
  );
}

import React from "react";

export type Player = { id: string; mark: string };

const markToGlyph: Record<string, string> = {
  mark_x: "X",
  mark_o: "O",
};

type Props = {
  turn?: string;
  players?: Player[];
  you?: string;
};

export default function TurnIndicator({ turn, players, you }: Props) {
  if (!turn || !players) return null;
  const player = players.find(p => p.id === turn);
  const glyph = player ? markToGlyph[player.mark] : "";
  const label = turn === you ? "Your" : `${turn}'s`;
  return (
    <div style={{ marginBottom: 16, fontWeight: "bold" }}>
      {label} turn {glyph && `(${glyph})`}
    </div>
  );
}

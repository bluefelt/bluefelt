import React from "react";

export type BoardProps = {
  board: (string | null)[][];
};

const markToGlyph: Record<string, string> = {
  mark_x: "X",
  mark_o: "O",
};

const cellStyle: React.CSSProperties = {
  width: 60,
  height: 60,
  border: "1px solid #333",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  fontSize: 32,
};

const rowStyle: React.CSSProperties = {
  display: "flex",
};

const Cell = React.memo(function Cell({ value }: { value: string | null }) {
  return <div style={cellStyle}>{value ? markToGlyph[value] ?? value : ""}</div>;
});

export default function Board({ board }: BoardProps) {
  if (!board || !Array.isArray(board)) return null;
  return (
    <div>
      {board.map((row, r) => (
        <div key={r} style={rowStyle}>
          {row.map((cell, c) => (
            <Cell key={c} value={cell} />
          ))}
        </div>
      ))}
    </div>
  );
}

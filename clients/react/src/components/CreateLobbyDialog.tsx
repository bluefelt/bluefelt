import type { Lobby } from "../api/lobbies.ts";
import type { Game } from "../api/games.ts";
import React, { useEffect, useState } from "react";
import { getGames } from "../api/games.ts";
import { createLobby } from "../api/lobbies.ts";

type Props = {
  onCreated: (lobby: Lobby) => void;
  onCancel: () => void;
};

export default function CreateLobbyDialog({ onCreated, onCancel }: Props) {
  const [games, setGames] = useState<Game[]>([]);
  const [selectedGameId, setSelectedGameId] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    getGames().then(setGames);
  }, []);

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedGameId) return;
    setLoading(true);
    try {
      const lobby = await createLobby(selectedGameId);
      onCreated(lobby);
    } catch (err) {
      alert("Failed to create lobby: " + err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={{
      background: "#fff", border: "1px solid #ccc", padding: 16, borderRadius: 8, maxWidth: 300
    }}>
      <form onSubmit={handleCreate}>
        <h3>Create Lobby</h3>
        <select
          value={selectedGameId}
          onChange={e => setSelectedGameId(e.target.value)}
          required
        >
          <option value="">Select a game</option>
          {games.map((game) => (
            <option key={game.id} value={game.id}>{game.name}</option>
          ))}
        </select>
        <br />
        <button type="submit" disabled={!selectedGameId || loading}>
          {loading ? "Creating..." : "Create"}
        </button>
        <button type="button" onClick={onCancel} style={{ marginLeft: 8 }}>
          Cancel
        </button>
      </form>
    </div>
  )
}
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
      alert("Failed to create lobby: " + (err instanceof Error ? err.message : "Unknown error"));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4">
      <div className="card w-full max-w-md">
        <form onSubmit={handleCreate} className="space-y-6">
          <div>
            <h3 className="text-xl font-semibold mb-4">Create Lobby</h3>
            <select
              value={selectedGameId}
              onChange={e => setSelectedGameId(e.target.value)}
              required
              className="input w-full bg-gray-700 text-white"
            >
              <option value="" className="bg-gray-700">Select a game</option>
              {games.map((game) => (
                <option key={game.id} value={game.id} className="bg-gray-700">{game.name}</option>
              ))}
            </select>
          </div>
          <div className="flex gap-3">
            <button
              type="submit"
              disabled={!selectedGameId || loading}
              className="btn btn-primary flex-1 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? "Creating..." : "Create"}
            </button>
            <button
              type="button"
              onClick={onCancel}
              className="btn btn-secondary flex-1"
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
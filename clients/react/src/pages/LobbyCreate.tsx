import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useGames } from '../hooks/useGames';

export default function LobbyCreate() {
  const { data: games, isLoading } = useGames();
  const [gameId, setGameId] = useState('');
  const navigate = useNavigate();

  const create = async () => {
    const res = await fetch('http://localhost:8000/lobbies', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ gameId }),
    });
    const lobby = await res.json();
    navigate(`/lobbies/${lobby.id}`);
  };

  if (isLoading) return <p className="p-4">Loading…</p>;

  return (
    <div className="p-4 space-y-3">
      <select
        value={gameId}
        onChange={e => setGameId(e.target.value)}
        className="border rounded px-2 py-1"
      >
        <option value="">Select a game</option>
        {games?.map(g => (
          <option key={g.id} value={g.id}>
            {g.name}
          </option>
        ))}
      </select>
      <button
        disabled={!gameId}
        onClick={create}
        className="bg-blue-600 text-white rounded px-3 py-1 disabled:opacity-50"
      >
        Create Lobby
      </button>
    </div>
  );
}

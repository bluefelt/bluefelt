import { useLobbies } from '../hooks/useLobbies';

export default function LobbyList() {
  const { data, isLoading } = useLobbies();
  if (isLoading) return <p className="p-4">Loading…</p>;
  const username = localStorage.getItem('username');

  return (
    <div className="p-4 space-y-4 max-w-md mx-auto">
      <h1 className="text-2xl font-semibold">Available Lobbies</h1>
      {username && (
        <p className="text-sm">
          Logged in as <strong>{username}</strong>
        </p>
      )}
      {!username && (
        <a href="/register" className="underline text-blue-600">
          Register
        </a>
      )}
      <a
        href="/create"
        className="block bg-green-700/10 rounded-xl p-3 hover:bg-green-700/20"
      >
        ➕ Create Lobby
      </a>
      <div className="space-y-2">
        {data?.map((l) => (
          <a
            key={l.id}
            href={`/lobbies/${l.id}`}
            className="block bg-blue-700/10 rounded-xl p-3 hover:bg-blue-700/20"
          >
            🎴 {l.game_id} – {l.id.slice(0, 8)}
          </a>
        ))}
      </div>
    </div>
  );
}

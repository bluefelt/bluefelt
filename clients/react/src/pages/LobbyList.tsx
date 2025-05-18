import { useLobbies } from '../hooks/useLobbies';

export default function LobbyList() {
  const { data, isLoading } = useLobbies();
  if (isLoading) return <p className="p-4">Loading…</p>;

  return (
    <div className="p-4 space-y-2">
      {data?.map(l => (
        <a
          key={l.id}
          href={`/lobbies/${l.id}`}
          className="block bg-blue-700/10 rounded-xl p-3 hover:bg-blue-700/20"
        >
          🎴 {l.game_id} – {l.id.slice(0, 8)}
        </a>
      ))}
    </div>
  );
}

import { useEffect, useState } from 'react';
import { useLobbyUsers } from '../hooks/useLobbyUsers';

export default function LobbyRoom({ id }: { id: string }) {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [log, setLog] = useState<string[]>([]);
  const { data: users, refetch } = useLobbyUsers(id);

  useEffect(() => {
    const socket = new WebSocket(`ws://localhost:8000/lobbies/${id}/ws`);
    socket.onmessage = (ev) => setLog((prev) => [...prev, ev.data]);
    setWs(socket);
    return () => socket.close();
  }, [id]);

  const join = async () => {
    const username = localStorage.getItem('username');
    if (!username) return;
    await fetch(`http://localhost:8000/lobbies/${id}/join`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username }),
    });
    refetch();
  };

  return (
    <div className="p-4 space-y-4 max-w-md mx-auto">
      <a href="/" className="underline text-blue-600">
        ← Back
      </a>
      <h1 className="text-2xl font-semibold">Lobby {id.slice(0, 8)}</h1>
      <button
        className="bg-blue-600 text-white rounded px-3 py-1"
        onClick={join}
      >
        Join Lobby
      </button>
      <button
        className="bg-green-600 text-white rounded px-3 py-1"
        onClick={() => ws?.send('hello')}
      >
        Send “hello”
      </button>
      <pre className="bg-gray-800/80 p-3 rounded text-green-300">
        {log.join('\n')}
      </pre>
      <div>
        <h2 className="font-semibold">Players</h2>
        <ul className="list-disc ml-6">
          {users?.map((u) => <li key={u}>{u}</li>)}
        </ul>
      </div>
    </div>
  );
}

import { useEffect, useState } from 'react';

export default function LobbyRoom({ id }: { id: string }) {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [log, setLog] = useState<string[]>([]);

  useEffect(() => {
    const socket = new WebSocket(`ws://localhost:8000/lobbies/${id}/ws`);
    socket.onmessage = ev => setLog(prev => [...prev, ev.data]);
    setWs(socket);
    return () => socket.close();
  }, [id]);

  return (
    <div className="p-4 space-y-3">
      <button
        className="bg-green-600 text-white rounded px-3 py-1"
        onClick={() => ws?.send('hello')}
      >
        Send “hello”
      </button>
      <pre className="bg-gray-800/80 p-3 rounded text-green-300">
        {log.join('\n')}
      </pre>
    </div>
  );
}

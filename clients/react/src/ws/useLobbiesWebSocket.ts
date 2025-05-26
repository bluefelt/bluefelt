import type { Lobby } from '../api/lobbies';
import { useWebSocket } from './useWebSocket';

export function useLobbiesWebSocket(onLobbies: (lobbies: Lobby[]) => void) {
  const ws = useWebSocket('ws://localhost:8000/lobbies/ws', (data) => {
    try {
      const parsed = JSON.parse(data);
      if (Array.isArray(parsed)) {
        onLobbies(parsed as Lobby[]);
      }
    } catch {
      // ignore invalid JSON
    }
  });

  return { connected: ws.connected };
}

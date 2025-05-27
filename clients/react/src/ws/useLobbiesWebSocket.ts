import type { Lobby } from '../api/lobbies';
import { useReconnectingWebSocket } from './useReconnectingWebSocket';
import { WS_BASE_URL } from '../config';

export function useLobbiesWebSocket(onLobbies: (lobbies: Lobby[]) => void) {
  const ws = useReconnectingWebSocket(`${WS_BASE_URL}/lobbies/ws`, (data) => {
    try {
      const parsed = JSON.parse(data);
      if (Array.isArray(parsed)) {
        onLobbies(parsed as Lobby[]);
      }
    } catch {
      // ignore invalid JSON
    }
  });

  return { connected: ws.connected, state: ws.state };
}

import { useState, useEffect, useRef } from 'react';
import { applyPatch } from 'fast-json-patch';
import { useWebSocket, type WSMessage } from './useWebSocket';

export type LobbyState = {
  you?: string;
  meta?: any;
  state?: any;
  started?: boolean;
};

export function useLobbyWebSocket(
  lobbyId: string,
  playerId: string,
  autoJoin: boolean,
) {
  const [lobbyState, setLobbyState] = useState<LobbyState>({});
  const lastTickRef = useRef<number>(
    Number(localStorage.getItem(`lobby_${lobbyId}_lastTick`) || '0'),
  );
  const url = `ws://localhost:8000/lobbies/${lobbyId}/ws?player_id=${encodeURIComponent(
    playerId,
  )}&join=${autoJoin ? 1 : 0}&since=${lastTickRef.current}`;

  const { messages, sendMessage, connected } = useWebSocket(url, (dataStr) => {
    let data: any;
    try {
      data = JSON.parse(dataStr);
    } catch {
      return;
    }

    if (data.type === 'welcome') {
      setLobbyState({
        you: data.you,
        meta: data.meta,
        state: data.state,
        started: data.started,
      });
      if (typeof data.tick === 'number') lastTickRef.current = data.tick;
    } else if (data.type === 'diff' && Array.isArray(data.patch)) {
      setLobbyState((prev) => {
        const full = { meta: prev.meta, state: prev.state };
        const patched = applyPatch({ ...full }, data.patch, true, false)
          .newDocument as LobbyState;
        return { ...patched, you: prev.you, started: prev.started };
      });
      if (typeof data.tick === 'number') lastTickRef.current = data.tick;
    } else if (data.type === 'started') {
      setLobbyState((prev) => ({ ...prev, started: true }));
    }
  });

  useEffect(() => {
    return () => {
      localStorage.setItem(
        `lobby_${lobbyId}_lastTick`,
        String(lastTickRef.current),
      );
    };
  }, [lobbyId]);

  const joinLobby = () => sendMessage(JSON.stringify({ action: 'join' }));
  const leaveLobby = () => sendMessage(JSON.stringify({ action: 'leave' }));
  const startGame = () => sendMessage(JSON.stringify({ action: 'start_game' }));

  return {
    messages,
    sendMessage,
    connected,
    lobbyState,
    joinLobby,
    leaveLobby,
    startGame,
  };
}

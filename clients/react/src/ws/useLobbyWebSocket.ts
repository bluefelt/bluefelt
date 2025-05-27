import { useState, useEffect, useRef } from 'react';
import { applyPatch } from 'fast-json-patch';
import { useReconnectingWebSocket } from './useReconnectingWebSocket';
import { WS_BASE_URL } from '../config';

export type VerbOption = {
  zone: string;
  row: number;
  col: number;
};

export type GroupedVerb = {
  verb: string;
  direction: string;
  validOptions: VerbOption[];
};

export type LobbyState = {
  you?: string;
  meta?: {
    possibleVerbs?: Record<string, GroupedVerb[]>;
    players?: string[];
    gameStatus?: {
      state: string;
      winner?: string;
      tie?: boolean;
    };
  };
  state?: {
    turn?: string;
    players?: Array<{ id: string }>;
    zones?: Record<string, unknown[][]>;
  };
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
  const url = `${WS_BASE_URL}/lobbies/${lobbyId}/ws?player_id=${encodeURIComponent(
    playerId,
  )}&join=${autoJoin ? 1 : 0}&since=${lastTickRef.current}`;

  const { messages, sendMessage, connected, state } = useReconnectingWebSocket(url, (dataStr) => {
    let data: {
      type: string;
      you?: string;
      meta?: LobbyState['meta'];
      state?: LobbyState['state'];
      started?: boolean;
      tick?: number;
      patch?: unknown[];
      players?: string[];
    };
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
    } else if (data.type === 'playerUpdate') {
      setLobbyState((prev) => ({
        ...prev,
        meta: {
          ...prev.meta,
          players: data.players,
        },
      }));
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
    } else if (data.type === 'gameStarted') {
      setLobbyState((prev) => ({
        ...prev,
        you: data.you || prev.you,
        state: data.state,
        meta: data.meta,
        started: true,
      }));
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
    connectionState: state,
    lobbyState,
    joinLobby,
    leaveLobby,
    startGame,
  };
}

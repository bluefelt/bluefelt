import React, { useState, useEffect, useRef } from 'react';
import { applyPatch } from 'fast-json-patch';
import { useReconnectingWebSocket } from './useReconnectingWebSocket';
import { WS_BASE_URL } from '../config';
import type { ServerMessage, WelcomeMessage, PlayerUpdateMessage, DiffMessage, GameStartedMessage } from '../types/messages';

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

import type { EntityDefinition } from '../types/messages';

export type LobbyState = {
  you?: string;
  meta?: {
    possibleVerbs?: Record<string, GroupedVerb[]>;
    players?: string[];
    entities?: EntityDefinition[];
    gameStatus?: {
      state: string;
      winner?: string;
      tie?: boolean;
    };
  };
  state?: {
    turn?: string;
    players?: Array<{ id: string; mark?: string }>;
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
  // Create URL with initial lastTick value
  const url = React.useMemo(() => {
    const initialTick = Number(localStorage.getItem(`lobby_${lobbyId}_lastTick`) || '0');
    return `${WS_BASE_URL}/lobbies/${lobbyId}/ws?player_id=${encodeURIComponent(
      playerId,
    )}&join=${autoJoin ? 1 : 0}&since=${initialTick}`;
  }, [lobbyId, playerId, autoJoin]);

  // Message handlers map for better organization
  const messageHandlers = {
    welcome: (data: WelcomeMessage) => {
      setLobbyState({
        you: data.you,
        meta: data.meta,
        state: data.state,
        started: data.started,
      });
      if (typeof data.tick === 'number') lastTickRef.current = data.tick;
    },
    
    playerUpdate: (data: PlayerUpdateMessage) => {
      setLobbyState((prev) => ({
        ...prev,
        meta: {
          ...prev.meta,
          players: data.players,
        },
      }));
    },
    
    diff: (data: DiffMessage) => {
      console.log('[useLobbyWebSocket] Received diff with tick:', data.tick);
      if (Array.isArray(data.patch)) {
        setLobbyState((prev) => {
          const full = { meta: prev.meta, state: prev.state };
          const patched = applyPatch({ ...full }, data.patch as Parameters<typeof applyPatch>[1], true, false)
            .newDocument as LobbyState;
          return { ...patched, you: prev.you, started: prev.started };
        });
        if (typeof data.tick === 'number') lastTickRef.current = data.tick;
      }
    },
    
    started: () => {
      setLobbyState((prev) => ({ ...prev, started: true }));
    },
    
    gameStarted: (data: GameStartedMessage) => {
      setLobbyState((prev) => ({
        ...prev,
        you: data.you || prev.you,
        state: data.state,
        meta: data.meta,
        started: true,
      }));
    },
  };

  const { messages, sendMessage, connected, state } = useReconnectingWebSocket(url, (dataStr) => {
    try {
      const data = JSON.parse(dataStr) as ServerMessage;
      const handler = messageHandlers[data.type as keyof typeof messageHandlers];
      if (handler) {
        // @ts-expect-error - Union type requires type assertion
        handler(data);
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
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

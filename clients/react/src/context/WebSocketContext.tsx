import React, { createContext, useContext, type ReactNode } from 'react';
import { useReconnectingWebSocket } from '../ws/useReconnectingWebSocket';
import type { WebSocketState } from '../ws/useReconnectingWebSocket';
import { WS_BASE_URL } from '../config';
import type { Lobby } from '../api/lobbies';

interface WebSocketContextType {
  lobbiesWS: {
    connected: boolean;
    state: WebSocketState;
  };
  lobbies: Lobby[];
}

const WebSocketContext = createContext<WebSocketContextType | null>(null);

export function WebSocketProvider({ children }: { children: ReactNode }) {
  const [lobbies, setLobbies] = React.useState<Lobby[]>([]);

  const lobbiesWS = useReconnectingWebSocket(
    `${WS_BASE_URL}/lobbies/ws`,
    (data) => {
      try {
        const parsed = JSON.parse(data);
        if (Array.isArray(parsed)) {
          setLobbies(parsed as Lobby[]);
        }
      } catch {
        // ignore invalid JSON
      }
    },
    {
      reconnectAttempts: 10,
      reconnectInterval: 1000,
      maxReconnectInterval: 10000,
    }
  );

  return (
    <WebSocketContext.Provider
      value={{
        lobbiesWS: {
          connected: lobbiesWS.connected,
          state: lobbiesWS.state,
        },
        lobbies,
      }}
    >
      {children}
    </WebSocketContext.Provider>
  );
}

export function useWebSocketContext() {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocketContext must be used within WebSocketProvider');
  }
  return context;
}
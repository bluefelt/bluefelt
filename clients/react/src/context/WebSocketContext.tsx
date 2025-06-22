import React, { createContext, useContext, type ReactNode } from 'react';
import { useReconnectingWebSocket } from '../ws/useReconnectingWebSocket';
import type { WebSocketState } from '../ws/useReconnectingWebSocket';
import { API_BASE_URL } from '../config';

// Types
interface Lobby {
  id: string;
  name: string;
  created_at: string;
  members?: Array<{
    username: string;
    connected: boolean;
  }>;
  games?: Array<{
    id: string;
    type: string;
    status: 'preparing' | 'playing' | 'ended';
    players: Record<string, string>;
  }>;
}

interface WebSocketContextType {
  lobbies: Lobby[];
  fetchLobbies: () => Promise<void>;
  createLobby: (name: string) => Promise<Lobby>;
  deleteLobby: (id: string) => Promise<void>;
}

const WebSocketContext = createContext<WebSocketContextType | null>(null);

export function WebSocketProvider({ children }: { children: ReactNode }) {
  const [lobbies, setLobbies] = React.useState<Lobby[]>([]);

  // Fetch lobbies via HTTP
  const fetchLobbies = React.useCallback(async () => {
    try {
      const url = `${API_BASE_URL}/api/lobbies`;
      console.log('[WebSocketContext] Fetching lobbies from:', url);
      const response = await fetch(url);
      console.log('[WebSocketContext] Response status:', response.status);
      if (response.ok) {
        const data = await response.json();
        console.log('[WebSocketContext] Lobbies data:', data);
        setLobbies(data);
      } else {
        console.error('[WebSocketContext] Response not OK:', response.status, response.statusText);
      }
    } catch (error) {
      console.error('[WebSocketContext] Failed to fetch lobbies:', error);
    }
  }, []);

  // Create a new lobby
  const createLobby = React.useCallback(async (name: string): Promise<Lobby> => {
    const response = await fetch(`${API_BASE_URL}/api/lobbies`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    });
    
    if (!response.ok) {
      throw new Error('Failed to create lobby');
    }
    
    const lobby = await response.json();
    await fetchLobbies(); // Refresh lobby list
    return lobby;
  }, [fetchLobbies]);

  // Delete a lobby
  const deleteLobby = React.useCallback(async (id: string) => {
    const response = await fetch(`${API_BASE_URL}/api/lobbies/${id}`, {
      method: 'DELETE',
    });
    
    if (!response.ok) {
      throw new Error('Failed to delete lobby');
    }
    
    await fetchLobbies(); // Refresh lobby list
  }, [fetchLobbies]);

  // Auto-fetch lobbies on mount and periodically
  React.useEffect(() => {
    fetchLobbies();
    const interval = setInterval(fetchLobbies, 5000); // Poll every 5 seconds
    return () => clearInterval(interval);
  }, [fetchLobbies]);

  return (
    <WebSocketContext.Provider
      value={{
        lobbies,
        fetchLobbies,
        createLobby,
        deleteLobby,
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
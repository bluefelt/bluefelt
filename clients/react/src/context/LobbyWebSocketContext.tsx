import React, { createContext, useContext, type ReactNode } from 'react';
import { useLobbyWebSocket, type LobbyState } from '../ws/useLobbyWebSocket';

interface LobbyWebSocketContextType {
  lobbyState: LobbyState;
  connected: boolean;
  connectionState: string;
  sendGameAction: (gameId: string, actionData: any) => void;
  createTable: (bundleId: string, minPlayers?: number, maxPlayers?: number) => void;
  joinTable: (tableId: string) => void;
  claimSeat: (tableId: string, seatIndex: number) => void;
  releaseSeat: (tableId: string, seatIndex: number) => void;
  setReady: (tableId: string, ready: boolean) => void;
  sendChatMessage: (message: string, scope?: 'lobby' | 'table', tableId?: string) => void;
  clearTableError: () => void;
  joinLobby: () => void;
  leaveLobby: () => void;
  renameLobby: (name: string) => void;
}

const LobbyWebSocketContext = createContext<LobbyWebSocketContextType | null>(null);

interface LobbyWebSocketProviderProps {
  children: ReactNode;
  lobbyId: string;
  username: string;
}

export function LobbyWebSocketProvider({ children, lobbyId, username }: LobbyWebSocketProviderProps) {
  const lobbyWebSocket = useLobbyWebSocket(lobbyId, username);
  
  return (
    <LobbyWebSocketContext.Provider value={lobbyWebSocket}>
      {children}
    </LobbyWebSocketContext.Provider>
  );
}

export function useLobbyWebSocketContext() {
  const context = useContext(LobbyWebSocketContext);
  if (!context) {
    throw new Error('useLobbyWebSocketContext must be used within LobbyWebSocketProvider');
  }
  return context;
}
import React, { createContext, useContext, type ReactNode } from 'react';
import { useLobbyWebSocket } from '../ws/useLobbyWebSocket';
import type { 
  LobbyState, 
  GameInstance, 
  EntityUI 
} from '../types/game-types';

interface GameContextType {
  // Lobby state
  lobbyState: LobbyState | null;
  lobbyId: string;
  username: string;
  
  // Game states
  activeGames: Record<string, GameInstance>;
  currentGameId: string | null;
  currentGame: GameInstance | null;
  
  // Entity UI
  entityUI: Record<string, Record<string, Record<string, EntityUI>>>;
  currentEntityUI: Record<string, EntityUI> | null;
  
  // Connection
  connected: boolean;
  error: string | null;
  
  // Lobby actions
  createGame: (gameType: string) => void;
  joinGame: (gameId: string) => void;
  startGame: (gameId: string) => void;
  selectGame: (gameId: string | null) => void;
  
  // Game actions
  performEntityInteraction: (entityId: string, actionId: string, args?: any) => void;
  performZoneInteraction: (zoneId: string, actionId: string, position?: [number, number], args?: any) => void;
}

const GameContext = createContext<GameContextType | null>(null);

export function useGame() {
  const context = useContext(GameContext);
  if (!context) {
    throw new Error('useGame must be used within GameProvider');
  }
  return context;
}

interface GameProviderProps {
  lobbyId: string;
  username: string;
  children: ReactNode;
}

export function GameProvider({ lobbyId, username, children }: GameProviderProps) {
  const [currentGameId, setCurrentGameId] = React.useState<string | null>(null);
  
  const {
    lobbyState,
    activeGames,
    entityUI,
    connected,
    error,
    createGame,
    joinGame,
    startGame,
    sendEntityInteraction,
    sendZoneInteraction
  } = useLobbyWebSocket({
    lobbyId,
    username,
    onError: (err) => console.error('WebSocket error:', err)
  });
  
  // Get current game
  const currentGame = currentGameId ? activeGames[currentGameId] : null;
  const currentEntityUI = currentGame && entityUI[currentGameId] 
    ? entityUI[currentGameId][currentGame.you] || {}
    : null;
  
  // Game action helpers that use current game
  const performEntityInteraction = React.useCallback((
    entityId: string,
    actionId: string,
    args?: any
  ) => {
    if (!currentGameId) {
      console.error('No game selected');
      return;
    }
    sendEntityInteraction(currentGameId, entityId, actionId, args);
  }, [currentGameId, sendEntityInteraction]);
  
  const performZoneInteraction = React.useCallback((
    zoneId: string,
    actionId: string,
    position?: [number, number],
    args?: any
  ) => {
    if (!currentGameId) {
      console.error('No game selected');
      return;
    }
    sendZoneInteraction(currentGameId, zoneId, actionId, position, args);
  }, [currentGameId, sendZoneInteraction]);
  
  // Auto-select first active game
  React.useEffect(() => {
    const gameIds = Object.keys(activeGames);
    if (gameIds.length > 0 && !currentGameId) {
      setCurrentGameId(gameIds[0]);
    }
  }, [activeGames, currentGameId]);
  
  const value: GameContextType = {
    // Lobby
    lobbyState,
    lobbyId,
    username,
    
    // Games
    activeGames,
    currentGameId,
    currentGame,
    
    // Entity UI
    entityUI,
    currentEntityUI,
    
    // Connection
    connected,
    error,
    
    // Lobby actions
    createGame,
    joinGame,
    startGame,
    selectGame: setCurrentGameId,
    
    // Game actions
    performEntityInteraction,
    performZoneInteraction
  };
  
  return (
    <GameContext value={value}>
      {children}
    </GameContext>
  );
}
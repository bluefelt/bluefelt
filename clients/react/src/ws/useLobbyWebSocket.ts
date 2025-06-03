import React, { useState, useEffect, useRef } from 'react';
import { applyPatch } from 'fast-json-patch';
import { useReconnectingWebSocket } from './useReconnectingWebSocket';
import { WS_BASE_URL } from '../config';
import type { ServerMessage, WelcomeMessage, PlayerUpdateMessage, DiffMessage, GameStartedMessage, PatchOperation } from '../types/messages';

export type ActionOption = {
  zone: string;
  row?: number;
  col?: number;
  entity?: string;
};

export type GroupedAction = {
  action: string;
  direction: string;
  validOptions: ActionOption[];
};

import type { MetaState } from '../types/messages';

export type LobbyState = {
  you?: string;
  ui?: MetaState;
  game?: {
    turn?: number;
    currentPlayer?: string;
    tick?: number;
    gameStatus?: {
      state: string;
      winner?: string | null;
      tie?: boolean;
    };
    players?: Array<{ id: string; mark?: string }>;
    zones?: Record<string, unknown[][]>;
    phases?: Record<string, any>;
    selection?: any;
  };
  started?: boolean;
  error?: string;
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
    return `${WS_BASE_URL}/api/lobbies/${lobbyId}/ws?player_id=${encodeURIComponent(
      playerId,
    )}&join=${autoJoin ? 1 : 0}&since=${initialTick}`;
  }, [lobbyId, playerId, autoJoin]);

  // Message handlers map for better organization
  const messageHandlers = {
    welcome: (data: WelcomeMessage) => {
      setLobbyState({
        you: data.you,
        ui: data.ui || data.meta, // Support both new and old formats during transition
        game: data.game || data.state,
        started: data.started,
      });
      if (typeof data.tick === 'number') lastTickRef.current = data.tick;
    },
    
    playerUpdate: (data: PlayerUpdateMessage) => {
      setLobbyState((prev) => ({
        ...prev,
        ui: {
          ...prev.ui,
          players: data.players,
        },
      }));
    },
    
    diff: (data: DiffMessage) => {
      if (Array.isArray(data.patch)) {
        // Log all patches for debugging game end scenarios
        const hasGameStatus = data.patch.some((p: any) => p.path === '/ui/gameStatus' || p.path === '/game/gameStatus');
        const hasZonePatches = data.patch.some((p: any) => p.path?.startsWith('/game/zones/'));
        if (hasGameStatus || hasZonePatches) {
          // Special patches detected
        }
        
        setLobbyState((prev) => {
          // Ensure ui and game exist before applying patches
          const full = { 
            ui: prev.ui || {}, 
            game: prev.game || {}
          };
          
          // Pre-process patches to ensure parent paths exist
          const processedPatches: PatchOperation[] = [];
          for (const patch of data.patch as PatchOperation[]) {
            const processedPatch = { ...patch };
            
            // The full object has {ui: {...}, game: {...}}
            // Server sends patches with /game prefix for game state and /ui for UI metadata
            // We need to keep these as-is since they match our structure
            
            // Ensure actionMap exists for player-specific updates
            if (processedPatch.path.startsWith('/ui/actionMap/')) {
              if (!full.ui.actionMap) {
                full.ui.actionMap = {};
              }
              // Extract player ID from path like /ui/actionMap/p2
              const pathParts = processedPatch.path.split('/');
              if (pathParts.length >= 4) {
                const playerId = pathParts[3];
                if (!full.ui.actionMap[playerId]) {
                  full.ui.actionMap[playerId] = {};
                }
              }
            }
            
            if (processedPatch.path.startsWith('/game/phases')) {
              // Ensure phases exists
              if (!full.game.phases) {
                full.game.phases = {};
              }
            }
            
            // Debug log for game status patches
            if (processedPatch.path === '/ui/gameStatus' || processedPatch.path === '/game/gameStatus') {
              // Game status patch detected
            }
            
            // Debug log for zone patches
            if (processedPatch.path?.startsWith('/game/zones/')) {
              // Zone patch detected
            }
            
            // Debug log for ui patches in development only
            if (import.meta.env.DEV && processedPatch.path?.includes('/ui/')) {
              // UI patch detected
            }
            
            processedPatches.push(processedPatch);
          }
          
          try {
            if (import.meta.env.DEV) {
              // Development mode
            }
            
            // Apply patches one by one to handle partial failures
            let workingState = { ...full };
            // Track patches for debugging
            // let successfulPatches = 0;
            
            for (let i = 0; i < processedPatches.length; i++) {
              const patch = processedPatches[i];
              try {
                const patchResult = applyPatch(workingState, [patch], true, false);
                workingState = patchResult.newDocument;
                // successfulPatches++;
                // Only log successful patches in development for debugging
                if (import.meta.env.DEV) {
                  // Patch applied successfully
                }
              } catch (patchError: any) {
                // Special handling for operations on non-existent paths
                if (patchError?.name === 'OPERATION_PATH_UNRESOLVABLE') {
                  // Check if this is a legacy path that we can safely ignore
                  const legacyPaths = ['/meta/possibleActions/', '/ui/currentPhasePrompt'];
                  const isLegacyPath = legacyPaths.some(legacy => patch.path?.includes(legacy));
                  
                  if (patch.op === 'remove' || isLegacyPath) {
                    if (import.meta.env.DEV) {
                      // Legacy path ignored
                    }
                    // Count as successful since it's effectively a no-op
                  } else {
                    console.error(`[useLobbyWebSocket] Failed to apply patch ${i + 1}:`, patch, 'Error:', patchError);
                  }
                } else {
                  console.error(`[useLobbyWebSocket] Failed to apply patch ${i + 1}:`, patch, 'Error:', patchError);
                }
                // Continue with other patches even if one fails
              }
            }
            
            if (import.meta.env.DEV) {
              // Patch processing complete
            }
            
            const result = { ...workingState, you: prev.you, started: prev.started };
            
            if (import.meta.env.DEV) {
              // State updated
            }
            
            // Debug log for game status changes and board state
            if (result.game?.gameStatus?.state === 'ended') {
              // Game has ended
            }
            
            // Debug log for any zone changes
            const hasProcessedZonePatches = processedPatches.some((p: any) => p.path?.startsWith('/game/zones/'));
            if (hasProcessedZonePatches) {
              // Zone changes detected
            }
            
            return result;
          } catch (error) {
            console.error('[useLobbyWebSocket] Critical error in patch processing:', error);
            // Return previous state if something goes catastrophically wrong
            return prev;
          }
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
        game: data.game,
        ui: data.ui,
        started: true,
      }));
    },
    
    error: (data: { message: string }) => {
      console.error('Lobby error:', data.message);
      setLobbyState((prev) => ({ ...prev, error: data.message }));
      // If lobby doesn't exist, don't try to reconnect
      if (data.message === 'Lobby does not exist') {
        // Clear the stored lastTick for this lobby
        localStorage.removeItem(`lobby_${lobbyId}_lastTick`);
        setShouldReconnect(false);
        setHasReceivedError(true);
        disconnect();
      }
    },
  };

  const [shouldReconnect, setShouldReconnect] = useState(true);
  const [hasReceivedError, setHasReceivedError] = useState(false);
  
  const { messages, sendMessage, connected, state, disconnect } = useReconnectingWebSocket(url, (dataStr) => {
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
  }, { 
    shouldReconnect: shouldReconnect && !hasReceivedError,
    reconnectAttempts: 3
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
    disconnect,
  };
}

import React, { useState, useEffect, useRef } from 'react';
import { applyPatch } from 'fast-json-patch';
import { useReconnectingWebSocket } from './useReconnectingWebSocket';
import { WS_BASE_URL } from '../config';
import type { PatchOperation } from '../types/messages';
import { AnimationEngine, type AnimationEngineCallbacks } from '../animation/AnimationEngine';
import { useAnimation } from '../context/AnimationContext';

// State Types
export type LobbyState = {
  // Persistent lobby info
  id: string;
  name: string;
  owner?: string | null;
  archived?: boolean;
  inviteCode?: string;
  members: Array<{
    username: string;
    connected: boolean;
    currentGame?: string;
    activeTables?: string[];
  }>;
  games: Array<{
    id: string;
    type: string;
    status: 'preparing' | 'playing' | 'ended';
    players: Record<string, string>; // slot -> username mapping
  }>;
  
  // New table system
  tables: Array<{
    id: string;
    bundleId: string;
    name?: string;
    owner: string;
    status: string;
    seats: Array<{ playerId: string; username: string } | null>;
    readyStates: boolean[];
    minPlayers: number;
    maxPlayers: number;
    countdownEndsAt?: number;
  }>;
  
  // Chat messages
  recentChat: Array<{
    id: string;
    scope: 'lobby' | 'table';
    tableId?: string;
    sender: string;
    message: string;
    timestamp: number;
  }>;
  
  // Current game state (if in a game)
  game?: {
    id: string;
    tableId?: string; // The table this game is associated with
    gameId?: string;  // The game type (e.g., 'tic-tac-toe')
    state: any; // Game-specific state
    ui: {
      actionMap: Record<string, any>;
      gameLog: any[];
      zones: any[];
      entities: any;
      gameMetadata: any;
    };
  };
  
  // Client state
  you?: string;
  error?: string;
  tableError?: string;
  isJoined?: boolean; // Track if current user has joined the lobby
};

export function useLobbyWebSocket(
  lobbyId: string,
  username: string,
) {
  const [lobbyState, setLobbyState] = useState<LobbyState>({
    id: lobbyId,
    name: '',
    members: [],
    games: [],
    tables: [],
    recentChat: [],
  });
  const lobbyStateRef = useRef<LobbyState>(lobbyState);
  
  // Keep ref in sync with state
  useEffect(() => {
    lobbyStateRef.current = lobbyState;
  }, [lobbyState]);
  
  // Animation system integration
  const { state: animationState, addAnimation, removeAnimation } = useAnimation();
  const animationEngineRef = useRef<AnimationEngine | null>(null);
  
  // Initialize animation engine
  useEffect(() => {
    const callbacks: AnimationEngineCallbacks = {
      onAnimationStart: (animation) => {
        addAnimation(animation);
      },
      onAnimationComplete: (result) => {
        removeAnimation(result.animationId);
      },
      onQueueEmpty: () => {
        // Animation queue is empty
      }
    };
    
    animationEngineRef.current = new AnimationEngine(callbacks);
    animationEngineRef.current.updateAudioConfig(animationState.config);
    
    return () => {
      if (animationEngineRef.current) {
        animationEngineRef.current.cancelAllAnimations();
      }
    };
  }, [addAnimation, removeAnimation]);
  
  // Update audio config when it changes
  useEffect(() => {
    if (animationEngineRef.current) {
      animationEngineRef.current.updateAudioConfig(animationState.config);
    }
  }, [animationState.config]);

  // Create WebSocket URL - connect without joining by default
  const url = React.useMemo(() => {
    return `${WS_BASE_URL}/api/lobbies/${lobbyId}/ws?player=${encodeURIComponent(username)}`;
  }, [lobbyId, username]);

  // Apply patches to state
  const applyPatches = (patches: PatchOperation[]) => {
    if (!patches || patches.length === 0) return;
    
    // Process animations BEFORE applying patches
    if (animationEngineRef.current && animationState.config.enableAnimations && lobbyStateRef.current.game) {
      const preUpdateState = {
        game: lobbyStateRef.current.game.state,
        ui: lobbyStateRef.current.game.ui,
        you: lobbyStateRef.current.you
      };
      
      const gameId = lobbyStateRef.current.game.ui.gameMetadata?.gameId;
      
      patches.forEach(patch => {
        animationEngineRef.current!.processAnimatablePatch(
          patch,
          preUpdateState,
          animationState.config,
          preUpdateState.you,
          gameId
        ).catch(console.error);
      });
    }
    
    setLobbyState(prev => {
      try {
        const result = applyPatch(prev, patches, true, false);
        return result.newDocument as LobbyState;
      } catch (error) {
        console.error('[WebSocket] Patch failed:', error, patches);
        return prev;
      }
    });
  };

  // Handle messages
  const handleMessage = (messageData: any) => {
    const { type } = messageData;
    
    switch (type) {
      case 'lobbyView':
        // Initial connection - viewing lobby without joining
        setLobbyState(prev => ({
          ...prev,
          ...messageData.lobby,
          tables: messageData.lobby.tables || [],
          recentChat: messageData.lobby.recentChat || [],
          isJoined: false,
        }));
        break;
        
      case 'lobbyJoined':
        // User has joined the lobby
        setLobbyState(prev => ({
          ...prev,
          ...messageData.lobby,
          you: messageData.lobby.myId || prev.you,
          tables: messageData.lobby.tables || [],
          recentChat: messageData.lobby.recentChat || [],
          isJoined: true,
        }));
        break;
        
      case 'lobbyState':
        // Full lobby state update
        setLobbyState(prev => ({
          ...prev,
          ...messageData.lobby,
          you: messageData.you || prev.you,
        }));
        break;
        
      case 'gameCreated':
        // A new game was created in the lobby
        setLobbyState(prev => ({
          ...prev,
          games: [...(prev.games || []), messageData.game],
        }));
        break;
        
      case 'gameJoined':
        // Update game player list
        setLobbyState(prev => {
          const games = prev.games.map(g => 
            g.id === messageData.gameId 
              ? { ...g, players: messageData.players }
              : g
          );
          return { ...prev, games };
        });
        break;
        
      case 'gameStarted':
        // Game has started - update state and UI
        console.log('[WebSocket] Game started:', messageData);
        console.log('[WebSocket] You are:', messageData.you);
        setLobbyState(prev => {
          // Update tables status to Playing
          const tables = (prev.tables || []).map(t =>
            t.id === messageData.tableId
              ? { ...t, status: 'Playing' }
              : t
          );
          
          // Update legacy games for backward compatibility
          const games = prev.games.map(g => 
            g.id === messageData.gameInstanceId 
              ? { ...g, status: 'playing' as const }
              : g
          );
          
          const newState = {
            ...prev,
            games,
            tables,
            game: {
              id: messageData.gameInstanceId,
              tableId: messageData.tableId,  // Store the table ID here
              gameId: messageData.gameId,
              state: messageData.state,
              ui: messageData.ui || {
                actionMap: {},
                gameLog: [],
                zones: [],
                entities: {},
                gameMetadata: {
                  gameId: messageData.gameId
                }
              }
            },
            you: messageData.you || prev.you,
          };
          
          console.log('[WebSocket] Updated lobby state with game:', newState.game);
          console.log('[WebSocket] Your player ID is:', newState.you);
          return newState;
        });
        break;
        
      case 'gameUpdate':
        // Patches for current game
        if (messageData.patches && lobbyState.game?.id === messageData.gameId) {
          console.log('[WebSocket] Game update patches:', messageData.patches);
          // Apply patches to game.state
          setLobbyState(prev => {
            if (!prev.game || prev.game.id !== messageData.gameId) return prev;
            
            try {
              const gamePatches = messageData.patches.map((p: PatchOperation) => ({
                ...p,
                path: `/game/state${p.path}`
              }));
              
              const result = applyPatch(prev, gamePatches, true, false);
              return result.newDocument as LobbyState;
            } catch (error) {
              console.error('[WebSocket] Game patch failed:', error);
              return prev;
            }
          });
        }
        
        // Update UI if provided
        if (messageData.ui && lobbyState.game?.id === messageData.gameId) {
          setLobbyState(prev => ({
            ...prev,
            game: prev.game ? { ...prev.game, ui: messageData.ui } : prev.game
          }));
        }
        break;
        
      case 'gameEnded':
        // Game has ended
        setLobbyState(prev => {
          const games = prev.games.map(g => 
            g.id === messageData.gameId 
              ? { ...g, status: 'ended' as const }
              : g
          );
          
          // Clear current game if it's the one that ended
          const game = prev.game?.id === messageData.gameId ? undefined : prev.game;
          
          return { ...prev, games, game };
        });
        break;
        
      case 'memberJoined':
        // Update member list and check if it's the current user
        if (messageData.members) {
          setLobbyState(prev => {
            const isCurrentUserJoining = messageData.member === username;
            return {
              ...prev,
              members: messageData.members,
              isJoined: isCurrentUserJoining ? true : prev.isJoined,
            };
          });
        }
        break;
        
      case 'memberLeft':
      case 'memberUpdate':
        // Update member list
        if (messageData.members) {
          setLobbyState(prev => ({
            ...prev,
            members: messageData.members,
          }));
        }
        break;
        
      case 'tableJoined':
        // User successfully joined a table with auto-assigned seat
        console.log(`[WebSocket] Successfully joined table ${messageData.tableId} at seat ${messageData.seatIndex}`);
        break;
        
      // Table messages
      case 'tableCreated':
        setLobbyState(prev => ({
          ...prev,
          tables: [...(prev.tables || []), messageData.table],
        }));
        break;
        
      case 'tableUpdated':
        setLobbyState(prev => ({
          ...prev,
          tables: (prev.tables || []).map(t =>
            t.id === messageData.tableId
              ? {
                  ...t,
                  seats: messageData.seats,
                  readyStates: messageData.readyStates,
                  status: messageData.status,
                  countdownEndsAt: messageData.countdownEndsAt,
                }
              : t
          ),
        }));
        break;
        
      case 'countdownStarted':
        setLobbyState(prev => ({
          ...prev,
          tables: (prev.tables || []).map(t =>
            t.id === messageData.tableId
              ? { ...t, status: 'Countdown', countdownEndsAt: messageData.endsAt }
              : t
          ),
        }));
        break;
        
      case 'countdownCancelled':
        setLobbyState(prev => ({
          ...prev,
          tables: (prev.tables || []).map(t =>
            t.id === messageData.tableId
              ? { ...t, status: 'Open', countdownEndsAt: undefined }
              : t
          ),
        }));
        break;
        
      case 'chatMessage':
        setLobbyState(prev => ({
          ...prev,
          recentChat: [...(prev.recentChat || []), {
            id: messageData.id || `${Date.now()}`,
            scope: messageData.scope,
            tableId: messageData.tableId,
            sender: messageData.sender,
            message: messageData.message,
            timestamp: messageData.timestamp,
          }].slice(-100), // Keep last 100 messages
        }));
        break;
        
      case 'error':
        console.error('Lobby error:', messageData.message);
        // Check if it's a table-specific error
        if (messageData.context && messageData.context.includes('table')) {
          setLobbyState(prev => ({ ...prev, tableError: messageData.message }));
        } else {
          setLobbyState(prev => ({ ...prev, error: messageData.message }));
        }
        
        // If lobby doesn't exist, don't try to reconnect
        if (messageData.message === 'Lobby does not exist') {
          setShouldReconnect(false);
          setHasReceivedError(true);
          disconnect();
        }
        break;
        
      default:
        console.warn('Unknown message type:', type);
    }
  };

  const [shouldReconnect, setShouldReconnect] = useState(true);
  const [hasReceivedError, setHasReceivedError] = useState(false);
  
  const { messages, sendMessage, connected, state, disconnect } = useReconnectingWebSocket(url, (dataStr) => {
    try {
      const data = JSON.parse(dataStr);
      handleMessage(data);
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
    }
  }, { 
    shouldReconnect: shouldReconnect && !hasReceivedError,
    reconnectAttempts: 3
  });

  // Actions
  const createGame = (gameType: string) => {
    sendMessage(JSON.stringify({ action: 'createGame', gameType }));
  };
  
  const joinGame = (gameId: string) => {
    sendMessage(JSON.stringify({ action: 'joinGame', gameId }));
  };
  
  const leaveGame = (gameId: string) => {
    sendMessage(JSON.stringify({ action: 'leaveGame', gameId }));
  };
  
  const startGame = (gameId: string) => {
    console.log('[WebSocket] Starting game:', gameId);
    sendMessage(JSON.stringify({ action: 'startGame', gameId: gameId }));
  };
  
  const sendGameAction = (gameId: string, actionData: any) => {
    sendMessage(JSON.stringify({ 
      action: 'gameAction', 
      gameId,
      ...actionData 
    }));
  };
  
  const leaveLobby = () => {
    sendMessage(JSON.stringify({ action: 'leaveLobby' }));
  };
  
  const requestGameState = (tableId: string) => {
    console.log('[WebSocket] Requesting game state for table:', tableId);
    sendMessage(JSON.stringify({ action: 'requestGameState', tableId }));
  };
  
  const renameLobby = (name: string) => {
    sendMessage(JSON.stringify({ action: 'renameLobby', name }));
  };
  
  // Table actions
  const createTable = (bundleId: string, minPlayers?: number, maxPlayers?: number) => {
    sendMessage(JSON.stringify({ 
      action: 'createTable', 
      bundleId,
      minPlayers,
      maxPlayers,
    }));
  };
  
  const joinTable = (tableId: string) => {
    sendMessage(JSON.stringify({ 
      action: 'joinTable', 
      tableId 
    }));
  };
  
  const claimSeat = (tableId: string, seatIndex: number) => {
    sendMessage(JSON.stringify({ 
      action: 'claimSeat', 
      tableId, 
      seatIndex 
    }));
  };
  
  const releaseSeat = (tableId: string, seatIndex: number) => {
    sendMessage(JSON.stringify({ 
      action: 'releaseSeat', 
      tableId, 
      seatIndex 
    }));
  };
  
  const setReady = (tableId: string, ready: boolean) => {
    sendMessage(JSON.stringify({ 
      action: 'setReady', 
      tableId, 
      ready 
    }));
  };
  
  const sendChatMessage = (message: string, scope: 'lobby' | 'table' = 'lobby', tableId?: string) => {
    sendMessage(JSON.stringify({ 
      action: 'sendChatMessage', 
      message,
      scope,
      tableId,
    }));
  };

  const clearTableError = () => {
    setLobbyState(prev => ({ ...prev, tableError: undefined }));
  };

  // Join the lobby (separate from just viewing)
  const joinLobby = () => {
    sendMessage(JSON.stringify({ action: 'joinLobby' }));
  };

  return {
    messages,
    sendMessage,
    connected,
    connectionState: state,
    lobbyState,
    createGame,
    joinGame,
    leaveGame,
    startGame,
    sendGameAction,
    leaveLobby,
    renameLobby,
    disconnect,
    joinLobby,
    requestGameState,
    // Table actions
    createTable,
    joinTable,
    claimSeat,
    releaseSeat,
    setReady,
    sendChatMessage,
    clearTableError,
  };
}
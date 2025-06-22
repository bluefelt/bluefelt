import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useLobbyWebSocket } from '../ws/useLobbyWebSocket';
import type { DiffMessage, GameStartedMessage } from '../types/messages';

// Mock the underlying WebSocket hook
vi.mock('../ws/useReconnectingWebSocket');

// Mock the AnimationContext
vi.mock('../context/AnimationContext', () => ({
  useAnimationsEnabled: vi.fn(() => true),
  useAnimation: vi.fn(() => ({
    state: { isAnimating: false, config: { enableAnimations: true } },
    updateConfig: vi.fn(),
    addAnimation: vi.fn(),
    removeAnimation: vi.fn(),
    clearQueue: vi.fn(),
    isAnimating: false
  }))
}));

describe('State Synchronization Tests', () => {
  const mockSend = vi.fn();
  const mockConnect = vi.fn();
  const mockDisconnect = vi.fn();
  let messageHandler: (data: any) => void = () => {};

  const createMockWebSocket = (overrides = {}) => ({
    messages: [],
    sendMessage: mockSend,
    state: 'connected' as const,
    connected: true,
    disconnect: mockDisconnect,
    reconnect: mockConnect,
    ...overrides
  });

  beforeEach(async () => {
    vi.clearAllMocks();
    localStorage.clear();
    
    // Reset the message handler
    messageHandler = () => {};
    
    // Set up the mock to capture the message handler
    const { useReconnectingWebSocket } = await import('../ws/useReconnectingWebSocket');
    vi.mocked(useReconnectingWebSocket).mockImplementation((url, handler, options) => {
      // Store the message handler so tests can call it
      messageHandler = (data: any) => {
        // Call the actual handler with JSON string as the hook expects
        handler(JSON.stringify(data));
      };
      
      if (options?.onOpen) {
        // Simulate immediate connection
        setTimeout(() => options.onOpen?.(), 0);
      }
      return createMockWebSocket();
    });
  });

  describe('Game State Initialization', () => {
    it('should initialize selection object on gameStarted', () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'TestPlayer', false)
      );

      // Simulate gameStarted message
      const gameStartedMsg: GameStartedMessage = {
        type: 'gameStarted',
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          turn: 0,
          tick: 0,
          zones: {},
          players: [{ id: 'p1' }, { id: 'p2' }],
          phases: { game: 'play' },
          gameStatus: { state: 'playing', winner: null, tie: false },
          // Note: selection is missing in the message
        },
        ui: {
          actionMap: { p1: {} },
          entities: [],
          zones: [],
          players: ['Alice', 'Bob'],
        },
      };

      act(() => {
        messageHandler(gameStartedMsg);
      });

      // Verify selection was initialized
      expect(result.current.lobbyState.game?.selection).toBeDefined();
      expect(result.current.lobbyState.game?.selection).toEqual({});
    });
  });

  describe('Patch Application', () => {
    it('should apply patches without errors when selection exists', () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'TestPlayer', false)
      );

      // First, initialize game state
      const gameStartedMsg: GameStartedMessage = {
        type: 'gameStarted',
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          selection: {}, // Selection initialized
          zones: {},
          players: [],
          phases: {},
          gameStatus: { state: 'playing', winner: null, tie: false },
        },
        ui: { actionMap: {}, entities: [], zones: [], players: [] },
      };

      act(() => {
        messageHandler(gameStartedMsg);
      });

      // Apply patch to selection
      const diffMsg: DiffMessage = {
        type: 'diff',
        tick: 1,
        patch: [
          {
            op: 'add',
            path: '/game/selection/availableRanks',
            value: ['A', '2', '3', '4', '5'],
          },
        ],
      };

      // Should not throw error
      expect(() => {
        act(() => {
          messageHandler(diffMsg);
        });
      }).not.toThrow();

      // Verify patch was applied
      expect(result.current.lobbyState.game?.selection?.availableRanks)
        .toEqual(['A', '2', '3', '4', '5']);
    });

    it('should handle missing parent paths gracefully', () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'TestPlayer', false)
      );

      // Initialize without selection
      const gameStartedMsg: GameStartedMessage = {
        type: 'gameStarted',
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          zones: {},
          players: [],
          phases: {},
          gameStatus: { state: 'playing', winner: null, tie: false },
        },
        ui: { actionMap: {}, entities: [], zones: [], players: [] },
      };

      act(() => {
        messageHandler(gameStartedMsg);
      });

      // Try to patch selection that doesn't exist
      const diffMsg: DiffMessage = {
        type: 'diff',
        tick: 1,
        patch: [
          {
            op: 'add',
            path: '/game/selection/availableRanks',
            value: ['A', '2', '3'],
          },
        ],
      };

      // Should handle gracefully (with our fix, selection is auto-initialized)
      act(() => {
        messageHandler(diffMsg);
      });

      // Verify selection was created and patch applied
      expect(result.current.lobbyState.game?.selection).toBeDefined();
      expect(result.current.lobbyState.game?.selection?.availableRanks)
        .toEqual(['A', '2', '3']);
    });
  });

  describe('Action Map Synchronization', () => {
    it('should update action map for current player', () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'TestPlayer', false)
      );

      // Initialize game
      const gameStartedMsg: GameStartedMessage = {
        type: 'gameStarted',
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          zones: { board: { type: 'grid', cells: [[null, null, null]] } },
          players: [],
          phases: {},
          gameStatus: { state: 'playing', winner: null, tie: false },
        },
        ui: { 
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': {
                action: 'placeMarker',
                direction: 'Click to place',
              },
            },
          },
          entities: [], 
          zones: [], 
          players: [],
        },
      };

      act(() => {
        messageHandler(gameStartedMsg);
      });

      // Verify initial action map
      expect(result.current.lobbyState.ui?.actionMap?.p1).toBeDefined();
      expect(Object.keys(result.current.lobbyState.ui?.actionMap?.p1 || {}))
        .toHaveLength(1);

      // Update action map via patch
      const diffMsg: DiffMessage = {
        type: 'diff',
        tick: 1,
        patch: [
          {
            op: 'replace',
            path: '/ui/actionMap',
            value: {
              p2: {
                '/zones/board/cells/1/1': {
                  action: 'placeMarker',
                  direction: 'Your turn',
                },
              },
            },
          },
        ],
      };

      act(() => {
        messageHandler(diffMsg);
      });

      // Verify action map updated
      expect(result.current.lobbyState.ui?.actionMap?.p1).toBeUndefined();
      expect(result.current.lobbyState.ui?.actionMap?.p2).toBeDefined();
    });
  });

  describe('Turn Management', () => {
    it('should track current player correctly', () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'TestPlayer', false)
      );

      // Initialize as p1's turn
      const gameStartedMsg: GameStartedMessage = {
        type: 'gameStarted',
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          zones: {},
          players: [{ id: 'p1' }, { id: 'p2' }],
          phases: {},
          gameStatus: { state: 'playing', winner: null, tie: false },
        },
        ui: { actionMap: {}, entities: [], zones: [], players: [] },
      };

      act(() => {
        messageHandler(gameStartedMsg);
      });

      expect(result.current.lobbyState.game?.currentPlayer).toBe('p1');

      // Switch turn to p2
      const diffMsg: DiffMessage = {
        type: 'diff',
        tick: 1,
        patch: [
          {
            op: 'replace',
            path: '/game/currentPlayer',
            value: 'p2',
          },
          {
            op: 'add',
            path: '/game/turn',
            value: 1,
          },
        ],
      };

      act(() => {
        messageHandler(diffMsg);
      });

      expect(result.current.lobbyState.game?.currentPlayer).toBe('p2');
      expect(result.current.lobbyState.game?.turn).toBe(1);
    });
  });

  describe('Game End Handling', () => {
    it('should handle game end with winner', () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'TestPlayer', false)
      );

      // Initialize game
      const gameStartedMsg: GameStartedMessage = {
        type: 'gameStarted',
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          zones: {},
          players: [],
          phases: {},
          gameStatus: { state: 'playing', winner: null, tie: false },
        },
        ui: { actionMap: {}, entities: [], zones: [], players: [] },
      };

      act(() => {
        messageHandler(gameStartedMsg);
      });

      // End game with winner
      const diffMsg: DiffMessage = {
        type: 'diff',
        tick: 5,
        patch: [
          {
            op: 'replace',
            path: '/game/gameStatus',
            value: { state: 'ended', winner: 'p1', tie: false },
          },
        ],
      };

      act(() => {
        messageHandler(diffMsg);
      });

      expect(result.current.lobbyState.game?.gameStatus?.state).toBe('ended');
      expect(result.current.lobbyState.game?.gameStatus?.winner).toBe('p1');
      expect(result.current.lobbyState.game?.gameStatus?.tie).toBe(false);
    });

    it('should handle game end with tie', () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'TestPlayer', false)
      );

      // Initialize game
      const gameStartedMsg: GameStartedMessage = {
        type: 'gameStarted',
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          zones: {},
          players: [],
          phases: {},
          gameStatus: { state: 'playing', winner: null, tie: false },
        },
        ui: { actionMap: {}, entities: [], zones: [], players: [] },
      };

      act(() => {
        messageHandler(gameStartedMsg);
      });

      // End game with tie
      const diffMsg: DiffMessage = {
        type: 'diff',
        tick: 9,
        patch: [
          {
            op: 'replace',
            path: '/game/gameStatus',
            value: { state: 'ended', winner: null, tie: true },
          },
        ],
      };

      act(() => {
        messageHandler(diffMsg);
      });

      expect(result.current.lobbyState.game?.gameStatus?.state).toBe('ended');
      expect(result.current.lobbyState.game?.gameStatus?.winner).toBeNull();
      expect(result.current.lobbyState.game?.gameStatus?.tie).toBe(true);
    });
  });
});
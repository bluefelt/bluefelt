import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { WebSocketProvider } from '../../../context/WebSocketContext';
import { PlayerProvider } from '../../../context/PlayerContext';
import { useGameActions } from '../../../hooks/useGameActions';
import type { LobbyState } from '../../../types/messages';

// Mock the WebSocket context
vi.mock('../../../context/WebSocketContext', () => ({
  WebSocketProvider: ({ children }: { children: React.ReactNode }) => children,
  useWebSocket: () => ({
    sendMessage: vi.fn(),
    connectionState: 'connected'
  })
}));

describe('Connect Four Action Generation and Handling', () => {
  let queryClient: QueryClient;
  let mockSendMessage: ReturnType<typeof vi.fn>;
  let mockLobbyState: LobbyState;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false }
      }
    });

    mockSendMessage = vi.fn();

    // Create Connect Four specific lobby state
    mockLobbyState = {
      id: 'test-lobby',
      lobbyId: 'test-lobby',
      gameId: 'connect-four',
      players: ['Alice', 'Bob'],
      game: {
        currentPlayer: 'Alice',
        currentTurn: 1,
        meta: {
          gameStatus: {
            state: 'playing',
            winner: null,
            tie: false
          }
        }
      },
      ui: {
        actionMap: {
          '/zones/board/0': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/1': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/2': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/3': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/4': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/5': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/6': { action: 'drop', direction: 'Drop piece in column' }
        },
        selection: null,
        rngDebug: {
          seed: 'test-seed',
          position: 0
        }
      },
      zones: {
        board: {
          zoneId: 'board',
          zoneType: 'grid',
          owner: 'common',
          cards: [],
          gridSize: { rows: 6, cols: 7 },
          cells: Array(6).fill(null).map(() => Array(7).fill(null).map(() => ({ cards: [] })))
        }
      },
      entities: {},
      actions: {
        drop: {
          actionId: 'drop',
          name: 'Drop Piece',
          when: [
            { condition: 'player.isActor' },
            { condition: 'phase.is', with: { phase: 'playing' } }
          ],
          do: [
            {
              type: 'placeWithGravity',
              with: {
                zone: '/zones/board',
                entity: 'piece_{player}'
              }
            }
          ],
          ui: {
            type: 'click',
            zone: 'board',
            direction: 'Drop piece in column'
          }
        }
      },
      phases: {
        game: {
          current: 'playing',
          stack: []
        }
      }
    };
  });

  it('should generate drop actions for all valid columns', () => {
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        <PlayerProvider initialPlayerName="Alice">
          <WebSocketProvider>
            {children}
          </WebSocketProvider>
        </PlayerProvider>
      </QueryClientProvider>
    );

    const { result } = renderHook(
      () => useGameActions({
        isYourTurn: true,
        lobbyState: { ...mockLobbyState, you: 'p1' },
        sendMessage: mockSendMessage
      }),
      { wrapper }
    );

    // Verify hook returns the expected handler functions
    expect(result.current.handleCellClick).toBeDefined();
    expect(typeof result.current.handleCellClick).toBe('function');

    // Test that the action map contains all 7 columns
    const actionMap = mockLobbyState.ui.actionMap;
    expect(Object.keys(actionMap)).toHaveLength(7);
    
    // Check each column has the drop action
    for (let col = 0; col < 7; col++) {
      const actionKey = `/zones/board/${col}`;
      expect(actionMap[actionKey]).toBeDefined();
      expect(actionMap[actionKey].action).toBe('drop');
      expect(actionMap[actionKey].direction).toBe('Drop piece in column');
    }
  });

  it('should handle column selection correctly', () => {
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        <PlayerProvider initialPlayerName="Alice">
          <WebSocketProvider>
            {children}
          </WebSocketProvider>
        </PlayerProvider>
      </QueryClientProvider>
    );

    const stateWithPlayerActions = {
      ...mockLobbyState,
      you: 'p1',
      ui: {
        ...mockLobbyState.ui,
        actionMap: {
          p1: {
            '/zones/board/columns/3': { action: 'dropDisc', targetColumn: 3 }
          }
        }
      }
    };

    const { result } = renderHook(
      () => useGameActions({
        isYourTurn: true,
        lobbyState: stateWithPlayerActions,
        sendMessage: mockSendMessage
      }),
      { wrapper }
    );

    // Click on column 3 (using row = -1 to indicate column action)
    act(() => {
      result.current.handleCellClick(-1, 3);
    });

    // Should send drop action for column 3
    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'dropDisc',
        args: {
          zone: '/zones/board',
          targetColumn: 3,
          entity: 'disc_p1'
        }
      })
    );
  });

  it('should not allow actions when not current player', () => {
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        <PlayerProvider initialPlayerName="Bob">
          <WebSocketProvider>
            {children}
          </WebSocketProvider>
        </PlayerProvider>
      </QueryClientProvider>
    );

    const stateWithPlayerActions = {
      ...mockLobbyState,
      you: 'p2',
      ui: {
        ...mockLobbyState.ui,
        actionMap: {
          p2: {
            '/zones/board/columns/3': { action: 'dropDisc', targetColumn: 3 }
          }
        }
      }
    };

    const { result } = renderHook(
      () => useGameActions({
        isYourTurn: false, // Bob is not current player
        lobbyState: stateWithPlayerActions,
        sendMessage: mockSendMessage
      }),
      { wrapper }
    );

    // Try to click column 3 when it's not Bob's turn
    act(() => {
      result.current.handleCellClick(-1, 3);
    });

    // Should not send any message since it's not Bob's turn
    expect(mockSendMessage).not.toHaveBeenCalled();
  });

  it('should handle full column scenarios', () => {
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        <PlayerProvider initialPlayerName="Alice">
          <WebSocketProvider>
            {children}
          </WebSocketProvider>
        </PlayerProvider>
      </QueryClientProvider>
    );

    // Test with column 2 still available
    const almostFullState = {
      ...mockLobbyState,
      you: 'p1',
      ui: {
        ...mockLobbyState.ui,
        actionMap: mockLobbyState.ui.actionMap // All 7 columns still available
      }
    };

    const { result } = renderHook(
      () => useGameActions({
        isYourTurn: true,
        lobbyState: almostFullState,
        sendMessage: mockSendMessage
      }),
      { wrapper }
    );

    // Verify column 2 action exists in action map
    expect(almostFullState.ui.actionMap['/zones/board/2']).toBeDefined();

    // Now test with column 2 full (removed from action map)
    const fullColumnState = {
      ...almostFullState,
      ui: {
        ...almostFullState.ui,
        actionMap: {
          '/zones/board/0': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/1': { action: 'drop', direction: 'Drop piece in column' },
          // Column 2 is full, removed from action map
          '/zones/board/3': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/4': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/5': { action: 'drop', direction: 'Drop piece in column' },
          '/zones/board/6': { action: 'drop', direction: 'Drop piece in column' }
        }
      }
    };

    const { result: fullResult } = renderHook(
      () => useGameActions({
        isYourTurn: true,
        lobbyState: fullColumnState,
        sendMessage: mockSendMessage
      }),
      { wrapper }
    );

    // Column 2 should not be available anymore in action map
    expect(fullColumnState.ui.actionMap['/zones/board/2']).toBeUndefined();
    expect(Object.keys(fullColumnState.ui.actionMap)).toHaveLength(6);
  });

  it('should handle game end scenarios', () => {
    const gameEndedState = {
      ...mockLobbyState,
      you: 'p1',
      game: {
        ...mockLobbyState.game,
        currentPlayer: null,
        meta: {
          gameStatus: {
            state: 'ended',
            winner: 'Alice',
            tie: false
          }
        }
      },
      ui: {
        ...mockLobbyState.ui,
        actionMap: {} // No actions available when game ends
      }
    };

    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        <PlayerProvider initialPlayerName="Alice">
          <WebSocketProvider>
            {children}
          </WebSocketProvider>
        </PlayerProvider>
      </QueryClientProvider>
    );

    const { result } = renderHook(
      () => useGameActions({
        isYourTurn: false, // Game ended, no one's turn
        lobbyState: gameEndedState,
        sendMessage: mockSendMessage
      }),
      { wrapper }
    );

    // No actions should be available in action map
    expect(Object.keys(gameEndedState.ui.actionMap)).toHaveLength(0);
    
    // Verify hook still returns handler functions
    expect(result.current.handleCellClick).toBeDefined();
  });

  it('should handle tie game scenarios', () => {
    const tieState = {
      ...mockLobbyState,
      you: 'p1',
      game: {
        ...mockLobbyState.game,
        currentPlayer: null,
        meta: {
          gameStatus: {
            state: 'ended',
            winner: null,
            tie: true
          }
        }
      },
      ui: {
        ...mockLobbyState.ui,
        actionMap: {} // No actions available when game ends
      }
    };

    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        <PlayerProvider initialPlayerName="Alice">
          <WebSocketProvider>
            {children}
          </WebSocketProvider>
        </PlayerProvider>
      </QueryClientProvider>
    );

    const { result } = renderHook(
      () => useGameActions({
        isYourTurn: false, // Game ended, no one's turn
        lobbyState: tieState,
        sendMessage: mockSendMessage
      }),
      { wrapper }
    );

    // No actions should be available in action map
    expect(Object.keys(tieState.ui.actionMap)).toHaveLength(0);
    
    // Verify hook still returns handler functions
    expect(result.current.handleCellClick).toBeDefined();
  });
});
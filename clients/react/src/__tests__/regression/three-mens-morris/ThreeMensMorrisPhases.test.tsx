import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { WebSocketProvider } from '../../../context/WebSocketContext';
import { PlayerProvider } from '../../../context/PlayerContext';
import { useGameActions } from '../../../hooks/useGameActions';
import type { LobbyState } from '../../../types/messages';
import React from 'react';

// Mock WebSocket
const mockSendJsonMessage = vi.fn();
const mockWebSocketState = {
  sendJsonMessage: mockSendJsonMessage,
  lastJsonMessage: null,
  readyState: WebSocket.OPEN,
  lastMessage: null,
  sendMessage: vi.fn(),
  getWebSocket: vi.fn(() => null)
};

describe('Three Mens Morris Phase Transitions', () => {
  let queryClient: QueryClient;
  let mockSendMessage: ReturnType<typeof vi.fn>;
  let baseLobbyState: LobbyState;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false }
      }
    });

    mockSendMessage = vi.fn().mockReturnValue(true);

    // Base state for Three Men's Morris
    baseLobbyState = {
      you: 'p1',
      started: true,
      game: {
        currentPlayer: 'p1',
        turn: 0,
        tick: 1,
        players: [
          { id: 'p1' },
          { id: 'p2' }
        ],
        zones: {
          board: {
            type: 'grid',
            cells: [
              [null, null, null],
              [null, null, null],
              [null, null, null]
            ]
          },
          hand_p1: {
            type: 'list',
            items: [
              { entity: 'piece_p1' },
              { entity: 'piece_p1' },
              { entity: 'piece_p1' }
            ]
          },
          hand_p2: {
            type: 'list',
            items: [
              { entity: 'piece_p2' },
              { entity: 'piece_p2' },
              { entity: 'piece_p2' }
            ]
          }
        },
        gameStatus: {
          state: 'playing',
          winner: null,
          tie: false
        },
        phases: {
          game: 'placement'
        },
        selection: {}
      },
      ui: {
        actionMap: {
          p1: {
            '/zones/board/cells/0/0': { 
              action: 'placeToken',
              args: {
                target: '/zones/board/cells/0/0',
                location: '/zones/board/cells/0/0',
                entity: 'piece_p1',
                player: 'p1'
              }
            },
            '/zones/board/cells/0/1': { 
              action: 'placeToken',
              args: {
                target: '/zones/board/cells/0/1',
                location: '/zones/board/cells/0/1',
                entity: 'piece_p1',
                player: 'p1'
              }
            },
            '/zones/board/cells/0/2': { 
              action: 'placeToken',
              args: {
                target: '/zones/board/cells/0/2',
                location: '/zones/board/cells/0/2',
                entity: 'piece_p1',
                player: 'p1'
              }
            }
          }
        },
        players: ['p1', 'p2']
      }
    };
  });

  const renderHookWithProviders = (lobbyState: LobbyState) => {
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        <PlayerProvider initialPlayerName="p1">
          <WebSocketProvider 
            value={{
              ...mockWebSocketState,
              lastJsonMessage: { type: 'welcome', ...lobbyState }
            }}
          >
            {children}
          </WebSocketProvider>
        </PlayerProvider>
      </QueryClientProvider>
    );

    return renderHook(() => useGameActions({
      isYourTurn: lobbyState.game?.currentPlayer === lobbyState.you,
      lobbyState,
      sendMessage: mockSendMessage
    }), { wrapper });
  };

  it('should handle placement phase actions correctly', () => {
    const { result } = renderHookWithProviders(baseLobbyState);

    // Should be able to place a piece
    act(() => {
      result.current.handleCellClick(0, 0);
    });

    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'placeToken',
        args: {
          target: '/zones/board/cells/0/0',
          location: '/zones/board/cells/0/0',
          entity: 'piece_p1',
          player: 'p1'
        }
      })
    );

    // Verify game state
    expect(baseLobbyState.game?.phases?.game).toBe('placement');
    expect(baseLobbyState.game?.zones?.hand_p1?.items).toHaveLength(3);
  });

  it('should transition to movement phase correctly', () => {
    const movementPhaseState = {
      ...baseLobbyState,
      game: {
        ...baseLobbyState.game!,
        zones: {
          ...baseLobbyState.game!.zones,
          board: {
            type: 'grid',
            cells: [
              [{ entity: 'piece_p1' }, null, { entity: 'piece_p2' }],
              [{ entity: 'piece_p1' }, null, { entity: 'piece_p2' }],
              [{ entity: 'piece_p1' }, null, { entity: 'piece_p2' }]
            ]
          },
          hand_p1: {
            type: 'list',
            items: []
          },
          hand_p2: {
            type: 'list',
            items: []
          }
        },
        phases: {
          game: 'movement'
        }
      },
      ui: {
        ...baseLobbyState.ui!,
        actionMap: {
          p1: {
            '/zones/board/cells/0/0': { 
              action: 'selectPiece',
              args: {
                target: '/zones/board/cells/0/0',
                location: '/zones/board/cells/0/0',
                player: 'p1'
              }
            },
            '/zones/board/cells/1/0': { 
              action: 'selectPiece',
              args: {
                target: '/zones/board/cells/1/0',
                location: '/zones/board/cells/1/0',
                player: 'p1'
              }
            },
            '/zones/board/cells/2/0': { 
              action: 'selectPiece',
              args: {
                target: '/zones/board/cells/2/0',
                location: '/zones/board/cells/2/0',
                player: 'p1'
              }
            }
          }
        }
      }
    };

    const { result } = renderHookWithProviders(movementPhaseState);

    // Should be able to select a piece
    act(() => {
      result.current.handleCellClick(0, 0);
    });

    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'selectPiece',
        args: {
          target: '/zones/board/cells/0/0',
          location: '/zones/board/cells/0/0',
          player: 'p1'
        }
      })
    );

    // Verify phase
    expect(movementPhaseState.game?.phases?.game).toBe('movement');
  });

  it('should handle selection state in movement phase', () => {
    const selectedPieceState = {
      ...baseLobbyState,
      game: {
        ...baseLobbyState.game!,
        zones: {
          ...baseLobbyState.game!.zones,
          board: {
            type: 'grid',
            cells: [
              [{ entity: 'piece_p1' }, null, { entity: 'piece_p2' }],
              [{ entity: 'piece_p1' }, null, { entity: 'piece_p2' }],
              [{ entity: 'piece_p1' }, null, { entity: 'piece_p2' }]
            ]
          }
        },
        phases: {
          game: 'movement'
        },
        selection: {
          p1: {
            location: '/zones/board/cells/0/0',
            entity: { entity: 'piece_p1' }
          }
        }
      },
      ui: {
        ...baseLobbyState.ui!,
        actionMap: {
          p1: {
            '/zones/board/cells/0/1': { 
              action: 'moveSelectedPiece',
              args: {
                target: '/zones/board/cells/0/1',
                location: '/zones/board/cells/0/1',
                player: 'p1'
              }
            },
            '/zones/board/cells/1/1': { 
              action: 'moveSelectedPiece',
              args: {
                target: '/zones/board/cells/1/1',
                location: '/zones/board/cells/1/1',
                player: 'p1'
              }
            },
            '_global': { 
              action: 'clearSelection',
              args: {
                player: 'p1'
              }
            }
          }
        }
      }
    };

    const { result } = renderHookWithProviders(selectedPieceState);

    // Should be able to move to an adjacent empty cell
    act(() => {
      result.current.handleCellClick(0, 1);
    });

    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'moveSelectedPiece',
        args: {
          target: '/zones/board/cells/0/1',
          location: '/zones/board/cells/0/1',
          player: 'p1'
        }
      })
    );

    // Verify selection state
    expect(selectedPieceState.game?.selection?.p1?.location).toBe('/zones/board/cells/0/0');
  });

  it('should not allow actions when not current player', () => {
    const opponentTurnState = {
      ...baseLobbyState,
      game: {
        ...baseLobbyState.game!,
        currentPlayer: 'p2'
      }
    };

    const { result } = renderHookWithProviders(opponentTurnState);

    // Try to place a piece
    act(() => {
      result.current.handleCellClick(0, 0);
    });

    // Should not send any message
    expect(mockSendMessage).not.toHaveBeenCalled();
  });
});
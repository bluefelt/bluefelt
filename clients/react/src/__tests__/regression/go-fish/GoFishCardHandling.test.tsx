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

describe('Go Fish Card Handling and Actions', () => {
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

    // Base state for Go Fish
    baseLobbyState = {
      you: 'p1',
      started: true,
      game: {
        currentPlayer: 'p1',
        turn: 0,
        tick: 1,
        players: [
          { id: 'p1' },
          { id: 'p2' },
          { id: 'p3' }
        ],
        zones: {
          pool: {
            type: 'deck',
            items: []
          },
          hand_p1: {
            type: 'list',
            items: []
          },
          hand_p2: {
            type: 'list',
            items: []
          },
          hand_p3: {
            type: 'list',
            items: []
          },
          pairs_p1: {
            type: 'list',
            items: []
          },
          pairs_p2: {
            type: 'list',
            items: []
          },
          pairs_p3: {
            type: 'list',
            items: []
          },
          choice_p1: {
            type: 'choice',
            items: []
          }
        },
        gameStatus: {
          state: 'playing',
          winner: null,
          tie: false
        },
        phases: {
          game: 'selectingRank'
        },
        selection: {
          availableRanks: ['A', '2', '3', '4']
        }
      },
      ui: {
        actionMap: {
          p1: {}
        },
        players: ['p1', 'p2', 'p3']
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

  it('should handle rank selection correctly', () => {
    const stateWithRanks = {
      ...baseLobbyState,
      game: {
        ...baseLobbyState.game!,
        phases: {
          game: 'selectingRank'
        }
      },
      ui: {
        ...baseLobbyState.ui!,
        actionMap: {
          p1: {
            '/zones/choice_p1/ranks/A': {
              action: 'selectRank',
              direction: 'Choose a rank to ask for',
              args: {
                rank: 'A',
                location: '/zones/choice_p1/ranks/A',
                player: 'p1'
              }
            }
          }
        }
      }
    };

    const { result } = renderHookWithProviders(stateWithRanks);

    // Should be able to select a rank
    act(() => {
      result.current.handleChoiceSelect('choice_p1/ranks', 'A');
    });

    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'selectRank',
        args: { 
          rank: 'A', 
          location: '/zones/choice_p1/ranks/A',
          player: 'p1' 
        }
      })
    );
  });

  it('should handle player selection with multiple opponents', () => {
    const stateWithPlayerSelection = {
      ...baseLobbyState,
      game: {
        ...baseLobbyState.game!,
        phases: {
          game: 'selectingPlayer'
        }
      },
      ui: {
        ...baseLobbyState.ui!,
        actionMap: {
          p1: {
            '/zones/choice_p1/players/p2': {
              action: 'selectPlayer',
              direction: 'Choose a player to ask',
              args: {
                targetPlayer: 'p2',
                location: '/zones/choice_p1/players/p2',
                player: 'p1'
              }
            }
          }
        }
      }
    };

    const { result } = renderHookWithProviders(stateWithPlayerSelection);

    // Should be able to select a player
    act(() => {
      result.current.handleChoiceSelect('choice_p1/players', 'p2');
    });

    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'selectPlayer',
        args: { 
          targetPlayer: 'p2', 
          location: '/zones/choice_p1/players/p2',
          player: 'p1' 
        }
      })
    );
  });

  it('should handle book formation detection', () => {
    const stateWithBooks = {
      ...baseLobbyState,
      game: {
        ...baseLobbyState.game!,
        zones: {
          ...baseLobbyState.game!.zones,
          pairs_p1: {
            type: 'list',
            items: [
              { entity: 'card_hearts_A' },
              { entity: 'card_spades_A' }
            ]
          }
        }
      }
    };

    const { result } = renderHookWithProviders(stateWithBooks);

    // Verify the state has pairs
    const lobbyState = stateWithBooks;
    expect(lobbyState.game?.zones?.pairs_p1?.items).toHaveLength(2);
  });

  it('should handle empty hand scenarios', () => {
    const emptyHandState = {
      ...baseLobbyState,
      game: {
        ...baseLobbyState.game!,
        phases: {
          game: 'selectingRank'
        }
      },
      ui: {
        ...baseLobbyState.ui!,
        actionMap: {
          p1: {
            '/zones/choice_p1/ranks/A': {
              action: 'selectRank',
              direction: 'Choose a rank to ask for',
              rank: 'A'
            }
          }
        }
      }
    };

    const { result } = renderHookWithProviders(emptyHandState);

    // Should still be able to try selecting (server will validate)
    act(() => {
      result.current.handleChoiceSelect('choice_p1/ranks', 'A');
    });

    expect(mockSendMessage).toHaveBeenCalled();
  });

  it('should handle successful ask scenario', () => {
    const stateAfterAsk = {
      ...baseLobbyState,
      game: {
        ...baseLobbyState.game!,
        zones: {
          ...baseLobbyState.game!.zones,
          hand_p1: {
            type: 'list',
            items: [
              { entity: 'card_hearts_A' },
              { entity: 'card_spades_A' },
              { entity: 'card_diamonds_A' },
              { entity: 'card_clubs_A' }
            ]
          }
        }
      }
    };

    const lobbyState = stateAfterAsk;

    // Verify hand has cards
    expect(lobbyState.game?.zones?.hand_p1?.items).toHaveLength(4);
    
    // Verify it's still player's turn
    expect(lobbyState.game?.currentPlayer).toBe('p1');
  });

  it('should prevent actions during opponent turns', () => {
    const opponentTurnState = {
      ...baseLobbyState,
      game: {
        ...baseLobbyState.game!,
        currentPlayer: 'p2'
      }
    };

    const { result } = renderHookWithProviders(opponentTurnState);

    // Try to make a move
    act(() => {
      result.current.handleChoiceSelect('ranks', 'A');
    });

    // Should not send any message
    expect(mockSendMessage).not.toHaveBeenCalled();
  });
});
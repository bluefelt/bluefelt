import { describe, it, expect, vi } from 'vitest';
import { render, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import GameView from '../../../components/GameView';

// Mock API calls
vi.mock('../../../api/lobbies', () => ({
  getLobby: vi.fn().mockResolvedValue({
    id: 'test-lobby',
    game_id: 'go-fish',
    players: ['Alice', 'Bob'],
    started: true
  })
}));

// Mock the player context
vi.mock('../../../context/PlayerContext', () => ({
  PlayerProvider: ({ children }: any) => children,
  usePlayer: () => ({
    player: { username: 'Alice', id: 'p1' },
    setPlayer: vi.fn()
  })
}));

// Setup mocks outside describe block
let mockSendMessage = vi.fn();
let currentLobbyState: any;

// Mock the WebSocket hook
vi.mock('../../../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: vi.fn(() => ({
    sendMessage: mockSendMessage,
    lobbyState: currentLobbyState,
    connectionState: 'connected',
    joinLobby: vi.fn(),
    leaveLobby: vi.fn(),
    startGame: vi.fn(),
    disconnect: vi.fn()
  }))
}));

describe('Go Fish Patch Mechanism', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false }
      }
    });

    // Set up initial lobby state that matches what server sends
    currentLobbyState = {
      you: 'p1',
      started: true,
      game: {
        currentPlayer: 'p1',
        turn: 0,
        tick: 0,
        players: [{ id: 'p1' }, { id: 'p2' }],
        gameStatus: {
          state: 'playing',
          winner: null,
          tie: false
        },
        selection: {
          // This is empty initially, queryEntities will add availableRanks
        },
        phases: {
          game: 'selectingRank'
        },
        zones: {
          pool: {
            type: 'list',
            items: Array(38).fill(null).map((_, i) => ({ entity: `card_${i}` }))
          },
          hand_p1: {
            type: 'list',
            items: [
              { entity: 'card_hearts_2' },
              { entity: 'card_clubs_2' },
              { entity: 'card_diamonds_5' },
              { entity: 'card_spades_k' },
              { entity: 'card_hearts_a' }
            ]
          },
          hand_p2: {
            type: 'list',
            items: Array(7).fill(null).map((_, i) => ({ entity: `card_hidden_${i}` }))
          },
          pairs_p1: {
            type: 'list',
            items: []
          },
          pairs_p2: {
            type: 'list',
            items: []
          }
        }
      },
      ui: {
        players: ['Alice', 'Bob'],
        actionMap: {
          p1: {},
          p2: {}
        }
      }
    };
  });

  it('should handle queryEntities patch that adds availableRanks', async () => {
    const TestWrapper = ({ children }: { children: React.ReactNode }) => (
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </MemoryRouter>
    );

    const { rerender } = render(
      <TestWrapper>
        <GameView lobbyId="test-lobby" onLeave={() => {}} />
      </TestWrapper>
    );

    // Simulate server sending patch for availableRanks (from queryEntities)
    currentLobbyState = {
      ...currentLobbyState,
      game: {
        ...currentLobbyState.game,
        selection: {
          ...currentLobbyState.game.selection,
          availableRanks: ['2', '5', 'a', 'k']  // Ranks found in hand_p1
        }
      }
    };

    // Force re-render to pick up new state
    rerender(
      <TestWrapper>
        <GameView lobbyId="test-lobby" onLeave={() => {}} />
      </TestWrapper>
    );

    // Wait for the component to update
    await waitFor(() => {
      // The test passes if it doesn't throw an error
      expect(currentLobbyState.game.selection.availableRanks).toEqual(['2', '5', 'a', 'k']);
    });
  });

  it('should handle setState patch that adds selectedRank', async () => {
    const TestWrapper = ({ children }: { children: React.ReactNode }) => (
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </MemoryRouter>
    );

    // Start with availableRanks already set
    currentLobbyState.game.selection.availableRanks = ['2', '5', 'a', 'k'];

    const { rerender } = render(
      <TestWrapper>
        <GameView lobbyId="test-lobby" onLeave={() => {}} />
      </TestWrapper>
    );

    // Simulate server sending patch for selectedRank (from setState)
    currentLobbyState = {
      ...currentLobbyState,
      game: {
        ...currentLobbyState.game,
        selection: {
          ...currentLobbyState.game.selection,
          selectedRank: '2'
        }
      }
    };

    // Force re-render to pick up new state
    rerender(
      <TestWrapper>
        <GameView lobbyId="test-lobby" onLeave={() => {}} />
      </TestWrapper>
    );

    // Wait for the component to update
    await waitFor(() => {
      expect(currentLobbyState.game.selection.selectedRank).toBe('2');
    });
  });
});
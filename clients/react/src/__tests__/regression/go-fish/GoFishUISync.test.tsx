import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import { WebSocketProvider } from '../../../context/WebSocketContext';
import { PlayerProvider } from '../../../context/PlayerContext';
import GameView from '../../../components/GameView';
import type { LobbyState, Phase, Zone, Entity } from '../../../types/messages';

// Mock WebSocket
class MockWebSocket {
  url: string;
  readyState: number = 1;
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  constructor(url: string) {
    this.url = url;
    setTimeout(() => {
      if (this.onopen) {
        this.onopen(new Event('open'));
      }
    }, 0);
  }

  send(data: string) {
    const message = JSON.parse(data);
    console.log('Mock WebSocket sending:', message);
  }

  close() {
    this.readyState = 3;
    if (this.onclose) {
      this.onclose(new CloseEvent('close'));
    }
  }
}

global.WebSocket = MockWebSocket as any;

// Mock API calls
vi.mock('../../../api/lobbies', () => ({
  getLobby: vi.fn().mockResolvedValue({
    id: 'test-lobby',
    game_id: 'go-fish',
    players: ['Alice', 'Bob'],
    started: true
  })
}));

// Store mock return value for WebSocket hook
const mockLobbyWebSocketReturn = {
  sendMessage: vi.fn(),
  lobbyState: null as LobbyState | null,
  connectionState: 'connected' as const,
  joinLobby: vi.fn(),
  leaveLobby: vi.fn(),
  startGame: vi.fn(),
  disconnect: vi.fn(),
  isConnected: true,
  isJoined: true,
  playerName: 'Alice',
  lobbyInfo: {
    id: 'test-lobby',
    game_id: 'go-fish',
    players: ['Alice', 'Bob'],
    started: true
  }
};

// Mock the WebSocket hook
vi.mock('../../../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: vi.fn(() => mockLobbyWebSocketReturn)
}));

// Mock the player context
vi.mock('../../../context/PlayerContext', () => ({
  PlayerProvider: ({ children }: any) => children,
  usePlayer: () => ({
    player: { username: 'Alice', id: 'p1' },
    setPlayer: vi.fn()
  })
}));

describe('Go Fish UI Synchronization', () => {
  let queryClient: QueryClient;
  let mockLobbyState: LobbyState;

  beforeEach(() => {
    vi.clearAllMocks();
    mockLobbyWebSocketReturn.sendMessage = vi.fn();
    
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false }
      }
    });

    // Create Go Fish specific lobby state
    mockLobbyState = {
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
          availableRanks: ['2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K', 'A']
        },
        phases: {
          game: 'selectingRank'
        },
        zones: {}  // Will be filled below
      },
      ui: {
        manifest: {
          metadata: {
            name: 'Go Fish',
            description: 'Classic card game',
            players: { min: 2, max: 2 }
          }
        },
        players: ['Alice', 'Bob'],
        zones: [
          { id: 'pool', name: 'Fishing Pool', type: 'deck', visibility: 'count' },
          { id: 'hand_p1', name: 'Your Hand', type: 'list', visibility: 'owner' },
          { id: 'hand_p2', name: 'Your Hand', type: 'list', visibility: 'owner' },
          { id: 'pairs_p1', name: 'Player p1 Pairs', type: 'list', visibility: 'public' },
          { id: 'pairs_p2', name: 'Player p2 Pairs', type: 'list', visibility: 'public' },
          { id: 'choice_p1', name: 'Select Rank', type: 'choice', visibility: 'owner' },
          { id: 'choice_p2', name: 'Select Rank', type: 'choice', visibility: 'owner' }
        ],
        entities: [
          { id: 'card_hearts_2', name: '2 of Hearts', props: { rank: '2', suit: 'hearts' } },
          { id: 'card_clubs_2', name: '2 of Clubs', props: { rank: '2', suit: 'clubs' } },
          { id: 'card_diamonds_5', name: '5 of Diamonds', props: { rank: '5', suit: 'diamonds' } },
          { id: 'card_spades_k', name: 'King of Spades', props: { rank: 'K', suit: 'spades' } },
          { id: 'card_hearts_a', name: 'Ace of Hearts', props: { rank: 'A', suit: 'hearts' } },
          { id: 'card_hidden', name: 'Card Back', props: {} }
        ],
        actionMap: {
          p1: {
            '/zones/hand_p1/items/0': { action: 'selectRank', direction: 'Select a rank to ask for' },
            '/zones/hand_p1/items/1': { action: 'selectRank', direction: 'Select a rank to ask for' },
            '/zones/hand_p1/items/2': { action: 'selectRank', direction: 'Select a rank to ask for' },
            '/zones/hand_p1/items/3': { action: 'selectRank', direction: 'Select a rank to ask for' },
            '/zones/hand_p1/items/4': { action: 'selectRank', direction: 'Select a rank to ask for' }
          },
          p2: {}
        },
        gameStatus: {
          state: 'playing',
          winner: null,
          tie: false
        }
      }
    };
    
    // Set up zones in game state
    mockLobbyState.game.zones = {
      pool: {
        type: 'list',
        items: Array(38).fill(null).map((_, i) => ({
          entity: `card_hidden_${i}`
        }))
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
        items: Array(7).fill(null).map((_, i) => ({
          entity: `card_hidden_p2_${i}`
        }))
      },
      pairs_p1: {
        type: 'list',
        items: []
      },
      pairs_p2: {
        type: 'list',
        items: []
      },
      choice_p1: {
        type: 'single',
        contents: null
      },
      choice_p2: {
        type: 'single',
        contents: null
      }
    };
    
    // Set lobbyState in the mock return value
    mockLobbyWebSocketReturn.lobbyState = mockLobbyState;
  });

  it('should render Go Fish game with player hands and deck', async () => {
    const TestWrapper = ({ children }: { children: React.ReactNode }) => (
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            <WebSocketProvider>
              {children}
            </WebSocketProvider>
          </PlayerProvider>
        </QueryClientProvider>
      </MemoryRouter>
    );

    render(
      <TestWrapper>
        <GameView 
          lobbyId="test-lobby"
          onLeave={() => {}}
        />
      </TestWrapper>
    );

    // Wait for the game to load - look for game elements
    await waitFor(() => {
      expect(screen.getAllByText('Go Fish').length).toBeGreaterThan(0);
    });
    
    // Check that zones are rendered - look for zone names
    const yourHandZones = screen.getAllByText('Your Hand');
    expect(yourHandZones.length).toBe(2); // One for each player
    expect(screen.getByText('Fishing Pool')).toBeInTheDocument();
    expect(screen.getByText('Player p1 Pairs')).toBeInTheDocument();
    expect(screen.getByText('Player p2 Pairs')).toBeInTheDocument();

    // The pool zone should be visible
  });

  it('should display rank selection choice zone when it is player turn', async () => {
    // Update the action map to include choice zone actions
    mockLobbyState.ui.actionMap.p1 = {
      '/zones/choice_p1/ranks/2': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: '2' },
      '/zones/choice_p1/ranks/3': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: '3' },
      '/zones/choice_p1/ranks/4': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: '4' },
      '/zones/choice_p1/ranks/5': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: '5' },
      '/zones/choice_p1/ranks/6': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: '6' },
      '/zones/choice_p1/ranks/7': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: '7' },
      '/zones/choice_p1/ranks/8': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: '8' },
      '/zones/choice_p1/ranks/9': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: '9' },
      '/zones/choice_p1/ranks/10': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: '10' },
      '/zones/choice_p1/ranks/j': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: 'J' },
      '/zones/choice_p1/ranks/q': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: 'Q' },
      '/zones/choice_p1/ranks/k': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: 'K' },
      '/zones/choice_p1/ranks/a': { action: 'selectRank', direction: 'Choose a rank to ask for', rank: 'A' }
    };
    
    // Also update zone metadata to use 'shape' instead of 'type' for choice zones
    mockLobbyState.ui.zones[5] = { id: 'choice_p1', name: 'Select Rank', shape: 'choice', visibility: 'owner' };
    mockLobbyState.ui.zones[6] = { id: 'choice_p2', name: 'Select Rank', shape: 'choice', visibility: 'owner' };
    
    // Update the mock state
    mockLobbyWebSocketReturn.lobbyState = mockLobbyState;

    const TestWrapper = ({ children }: { children: React.ReactNode }) => (
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            <WebSocketProvider>
              {children}
            </WebSocketProvider>
          </PlayerProvider>
        </QueryClientProvider>
      </MemoryRouter>
    );

    render(
      <TestWrapper>
        <GameView 
          lobbyId="test-lobby"
          onLeave={() => {}}
        />
      </TestWrapper>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getAllByText('Go Fish').length).toBeGreaterThan(0);
    });
    
    // CRITICAL: The choice zone MUST be visible when it's the player's turn
    await waitFor(() => {
      const choiceZone = screen.getByTestId('choice-zone');
      expect(choiceZone).toBeInTheDocument();
      expect(choiceZone).toBeVisible();
    });
    
    // Verify that all rank options are displayed
    expect(screen.getByText('Rank 2')).toBeInTheDocument();
    expect(screen.getByText('Rank 3')).toBeInTheDocument();
    expect(screen.getByText('Rank 4')).toBeInTheDocument();
    expect(screen.getByText('Rank 5')).toBeInTheDocument();
    expect(screen.getByText('Rank 6')).toBeInTheDocument();
    expect(screen.getByText('Rank 7')).toBeInTheDocument();
    expect(screen.getByText('Rank 8')).toBeInTheDocument();
    expect(screen.getByText('Rank 9')).toBeInTheDocument();
    expect(screen.getByText('Rank 10')).toBeInTheDocument();
    expect(screen.getByText('Rank j')).toBeInTheDocument();
    expect(screen.getByText('Rank q')).toBeInTheDocument();
    expect(screen.getByText('Rank k')).toBeInTheDocument();
    expect(screen.getByText('Rank a')).toBeInTheDocument();
    
    // Test clicking a rank
    mockLobbyWebSocketReturn.sendMessage.mockClear();
    
    const rank5Button = screen.getByText('Rank 5');
    fireEvent.click(rank5Button);
    
    await waitFor(() => {
      expect(mockLobbyWebSocketReturn.sendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'selectRank',
          args: { rank: '5', player: 'p1' }
        })
      );
    });
  });

  it('should display player selection after rank selection', async () => {
    // State after rank selection
    const playerSelectionState: LobbyState = {
      ...mockLobbyState,
      ui: {
        ...mockLobbyState.ui,
        selection: '2', // Rank 2 was selected
        actionMap: {
          p1: {
            '/zones/choice_p1/players/p2': { action: 'selectPlayer', direction: 'Ask Bob for cards', args: { targetPlayer: 'p2' } }
          },
          p2: {}
        }
      },
      game: {
        ...mockLobbyState.game,
        zones: {
          ...mockLobbyState.game.zones,
          choice_p1: {},  // Choice zones are empty in game state
          choice_p2: {}
        },
        phases: {
          game: 'selectingPlayer'
        }
      }
    };
    
    // Update the mock state
    mockLobbyWebSocketReturn.lobbyState = playerSelectionState;

    const TestWrapper = ({ children }: { children: React.ReactNode }) => (
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            <WebSocketProvider>
              {children}
            </WebSocketProvider>
          </PlayerProvider>
        </QueryClientProvider>
      </MemoryRouter>
    );

    render(
      <TestWrapper>
        <GameView 
          lobbyId="test-lobby"
          onLeave={() => {}}
        />
      </TestWrapper>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getAllByText('Go Fish').length).toBeGreaterThan(0);
    });

    // Check that choice zone is displayed
    await waitFor(() => {
      expect(screen.getByTestId('choice-zone')).toBeInTheDocument();
    });
    
    // Check that Bob is shown as a choice
    expect(screen.getByText('Bob')).toBeInTheDocument();
  });


  it('should handle go fish scenario with deck drawing', async () => {
    // State showing "Go Fish!" message
    const goFishState: LobbyState = {
      ...mockLobbyState,
      ui: {
        ...mockLobbyState.ui,
        actionMap: {
          p1: {
            '/zones/pool': { action: 'drawCard', direction: 'Draw a card from the deck' }
          },
          p2: {}
        }
      },
      game: {
        ...mockLobbyState.game,
        phases: {
          game: 'drawingFromPool'
        }
      }
    };
    
    // Update the mock state
    mockLobbyWebSocketReturn.lobbyState = goFishState;

    const TestWrapper = ({ children }: { children: React.ReactNode }) => (
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            <WebSocketProvider>
              {children}
            </WebSocketProvider>
          </PlayerProvider>
        </QueryClientProvider>
      </MemoryRouter>
    );

    render(
      <TestWrapper>
        <GameView 
          lobbyId="test-lobby"
          onLeave={() => {}}
        />
      </TestWrapper>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getAllByText('Go Fish').length).toBeGreaterThan(0);
    });

    // Find and click on the deck zone
    const deckZone = screen.getByText(/Fishing Pool/).closest('[data-testid="card-zone"]');
    if (deckZone) {
      fireEvent.click(deckZone);
      
      await waitFor(() => {
        expect(mockLobbyWebSocketReturn.sendMessage).toHaveBeenCalledWith(
          JSON.stringify({
            action: 'drawCard',
            location: '/zones/pool'
          })
        );
      });
    }
  });

  it('should display game end with winner', async () => {
    const gameEndedState: LobbyState = {
      ...mockLobbyState,
      game: {
        ...mockLobbyState.game,
        currentPlayer: null,
        gameStatus: {
          state: 'ended',
          winner: 'p1',
          tie: false
        },
        zones: {
          ...mockLobbyState.game.zones,
          pairs_p1: {
            type: 'list',
            items: Array(7).fill(null).map((_, i) => ({
              entity: `book_${i}`
            }))
          },
          pairs_p2: {
            type: 'list',
            items: Array(6).fill(null).map((_, i) => ({
              entity: `book_b${i}`
            }))
          }
        }
      },
      ui: {
        ...mockLobbyState.ui,
        actionMap: {
          p1: {},
          p2: {}
        },
        gameStatus: {
          state: 'ended',
          winner: 'p1',
          tie: false
        }
      }
    };
    
    // Update the mock state
    mockLobbyWebSocketReturn.lobbyState = gameEndedState;

    const TestWrapper = ({ children }: { children: React.ReactNode }) => (
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            <WebSocketProvider>
              {children}
            </WebSocketProvider>
          </PlayerProvider>
        </QueryClientProvider>
      </MemoryRouter>
    );

    render(
      <TestWrapper>
        <GameView 
          lobbyId="test-lobby"
          onLeave={() => {}}
        />
      </TestWrapper>
    );

    // Wait for the game to load and check for winner display
    await waitFor(() => {
      expect(screen.getByText(/Alice wins!/i)).toBeInTheDocument();
    });
  });

});
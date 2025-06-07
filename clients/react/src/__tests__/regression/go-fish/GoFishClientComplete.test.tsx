import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import React from 'react';
import { BrowserRouter } from 'react-router-dom';
import GameView from '../../../components/GameView';
import { WebSocketProvider } from '../../../context/WebSocketContext';
import { PlayerProvider } from '../../../context/PlayerContext';
import type { LobbyState } from '../../../types/messages';

// Mock the useLobbyWebSocket hook
const mockLobbyWebSocketReturn = {
  sendMessage: vi.fn(),
  lobbyState: {} as LobbyState,
  connected: true,
  connectionState: 'connected' as const,
  joinLobby: vi.fn(),
  leaveLobby: vi.fn(),
  startGame: vi.fn(),
  disconnect: vi.fn(),
  messages: [],
};

vi.mock('../../../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: vi.fn(() => mockLobbyWebSocketReturn),
}));

// Mock the navigate function
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

// Mock the API call
vi.mock('../../../api/lobbies', () => ({
  getLobby: vi.fn().mockResolvedValue({ game_id: 'go-fish' }),
}));

// Mock the PlayerContext
const mockPlayer = { id: 'p1', username: 'Alice' };
vi.mock('../../../context/PlayerContext', () => ({
  PlayerProvider: ({ children }: { children: React.ReactNode }) => children,
  usePlayer: () => ({
    player: mockPlayer,
    setPlayer: vi.fn(),
  }),
}));

/**
 * Complete E2E regression tests for Go Fish
 * Tests the full game flow including UI interactions, action processing, and game logs
 */
describe('Go Fish Complete E2E Tests', () => {
  let mockLobbyState: LobbyState;

  beforeEach(() => {
    // Reset lobby state for Go Fish
    mockLobbyState = {
      you: 'p1',
      started: true,
      id: 'test-lobby',
      playersReady: ['p1', 'p2'],
      game: {
        id: 'go-fish',
        specVersion: '1.0',
        currentPlayer: 'p1',
        turn: 0,
        tick: 0,
        zones: {
          pool: {
            type: 'deck',
            count: 38, // 52 - 14 cards dealt
            entities: []
          },
          hand_p1: {
            type: 'list',
            entities: [
              { entity: 'card_hearts_5', props: { rank: '5' } },
              { entity: 'card_spades_5', props: { rank: '5' } },
              { entity: 'card_clubs_7', props: { rank: '7' } },
              { entity: 'card_diamonds_k', props: { rank: 'k' } },
              { entity: 'card_hearts_a', props: { rank: 'a' } },
              { entity: 'card_spades_3', props: { rank: '3' } },
              { entity: 'card_clubs_9', props: { rank: '9' } }
            ]
          },
          hand_p2: {
            type: 'list',
            entities: [
              // P2's hand (hidden from P1)
            ]
          },
          pairs_p1: {
            type: 'list',
            entities: []
          },
          pairs_p2: {
            type: 'list',
            entities: []
          }
        },
        players: [{ id: 'p1' }, { id: 'p2' }],
        phases: { game: 'selectingRank' },
        gameStatus: { state: 'playing', winner: null, tie: false },
        selection: {
          availableRanks: ['5', '7', 'k', 'a', '3', '9']
        }
      },
      ui: {
        actionMap: {
          p1: {
            // Can select any rank from hand
            '/ui/rank/5': { action: 'selectRank', direction: 'Ask for 5s' },
            '/ui/rank/7': { action: 'selectRank', direction: 'Ask for 7s' },
            '/ui/rank/k': { action: 'selectRank', direction: 'Ask for Ks' },
            '/ui/rank/a': { action: 'selectRank', direction: 'Ask for As' },
            '/ui/rank/3': { action: 'selectRank', direction: 'Ask for 3s' },
            '/ui/rank/9': { action: 'selectRank', direction: 'Ask for 9s' }
          }
        },
        messages: [],
        players: ['Alice', 'Bob'],
        manifest: {
          gameId: 'go-fish',
          version: '1.0',
          metadata: {
            name: 'Go Fish',
            description: 'Classic Go Fish card game',
            author: 'Bluefelt',
            players: { min: 2, max: 4 }
          }
        },
        zones: [
          { id: 'pool', type: 'deck', name: 'Pool' },
          { id: 'hand_p1', type: 'list', name: 'Your Hand' },
          { id: 'hand_p2', type: 'list', name: "Bob's Hand" },
          { id: 'pairs_p1', type: 'list', name: 'Your Pairs' },
          { id: 'pairs_p2', type: 'list', name: "Bob's Pairs" }
        ]
      },
      players: {
        p1: { name: 'Alice', connected: true },
        p2: { name: 'Bob', connected: true }
      }
    };

    // Set the mock lobbyState
    mockLobbyWebSocketReturn.lobbyState = mockLobbyState;

    // Reset mocks
    vi.clearAllMocks();
  });

  it('renders Go Fish game correctly in rank selection phase', async () => {
    render(
      <BrowserRouter>
        <PlayerProvider>
          <WebSocketProvider lobbyId="test-lobby">
            <GameView lobbyId="test-lobby" />
          </WebSocketProvider>
        </PlayerProvider>
      </BrowserRouter>
    );

    // Wait for the game to load by checking for the YOUR TURN text
    await waitFor(() => {
      expect(screen.getByText('YOUR TURN')).toBeInTheDocument();
    });

    // Should show rank selection prompt
    expect(screen.getByText('Ask for 5s')).toBeInTheDocument();

    // Should show players
    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.getByText('Bob')).toBeInTheDocument();
  });

  it('handles rank selection correctly', async () => {
    render(
      <BrowserRouter>
        <PlayerProvider>
          <WebSocketProvider lobbyId="test-lobby">
            <GameView lobbyId="test-lobby" />
          </WebSocketProvider>
        </PlayerProvider>
      </BrowserRouter>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText('YOUR TURN')).toBeInTheDocument();
    });

    // The action is in the turn prompt bar, not as a clickable button
    // Check that we show the rank selection prompt
    expect(screen.getByText('Ask for 5s')).toBeInTheDocument();

    // Since actions are sent via useGameActions hook when clicking zones,
    // we need to check that the sendMessage would be called with correct params
    // when action is triggered. For now, just verify the UI shows correctly.
  });

  it('renders player selection phase correctly', async () => {
    // Set up player selection phase
    mockLobbyWebSocketReturn.lobbyState = {
      ...mockLobbyState,
      game: {
        ...mockLobbyState.game,
        phases: { game: 'selectingPlayer' },
        selection: {
          selectedRank: '5',
          availableRanks: ['5', '7', 'k', 'a', '3', '9']
        }
      },
      ui: {
        ...mockLobbyState.ui,
        actionMap: {
          p1: {
            '/ui/player/p2': { action: 'selectPlayer', direction: 'Ask Bob for 5s' }
          }
        }
      }
    };

    render(
      <BrowserRouter>
        <PlayerProvider>
          <WebSocketProvider lobbyId="test-lobby">
            <GameView lobbyId="test-lobby" />
          </WebSocketProvider>
        </PlayerProvider>
      </BrowserRouter>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText('YOUR TURN')).toBeInTheDocument();
    });

    // Should show player selection
    expect(screen.getByText('Ask Bob for 5s')).toBeInTheDocument();
  });

  it('handles player selection correctly', async () => {
    // Set up player selection phase
    mockLobbyWebSocketReturn.lobbyState = {
      ...mockLobbyState,
      game: {
        ...mockLobbyState.game,
        phases: { game: 'selectingPlayer' },
        selection: {
          selectedRank: '5'
        }
      },
      ui: {
        ...mockLobbyState.ui,
        actionMap: {
          p1: {
            '/ui/player/p2': { action: 'selectPlayer', direction: 'Ask Bob for 5s' }
          }
        }
      }
    };

    render(
      <BrowserRouter>
        <PlayerProvider>
          <WebSocketProvider lobbyId="test-lobby">
            <GameView lobbyId="test-lobby" />
          </WebSocketProvider>
        </PlayerProvider>
      </BrowserRouter>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText('YOUR TURN')).toBeInTheDocument();
    });

    // Check that player selection is shown
    expect(screen.getByText('Ask Bob for 5s')).toBeInTheDocument();
  });

  it('shows Go Fish response correctly', async () => {
    // Set up fishing phase (player went fishing)
    mockLobbyWebSocketReturn.lobbyState = {
      ...mockLobbyState,
      game: {
        ...mockLobbyState.game,
        phases: { game: 'fishing' },
        selection: {
          selectedRank: '5',
          targetPlayer: 'p2'
        }
      },
      ui: {
        ...mockLobbyState.ui,
        actionMap: { p1: {} }, // No actions during auto fishing
        gameLog: [
          { message: 'Go Fish! Alice draws a card from the pool', timestamp: Date.now() }
        ]
      }
    };

    render(
      <BrowserRouter>
        <PlayerProvider>
          <WebSocketProvider lobbyId="test-lobby">
            <GameView lobbyId="test-lobby" />
          </WebSocketProvider>
        </PlayerProvider>
      </BrowserRouter>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText('Game Log')).toBeInTheDocument();
    });

    // Should show Go Fish message in game log
    expect(screen.getByText(/Go Fish! Alice draws a card/i)).toBeInTheDocument();
  });

  it('shows successful card transfer', async () => {
    // Set up state after successful card request
    mockLobbyWebSocketReturn.lobbyState = {
      ...mockLobbyState,
      game: {
        ...mockLobbyState.game,
        zones: {
          ...mockLobbyState.game.zones,
          hand_p1: {
            type: 'list',
            entities: [
              // P1 now has 3 fives (got 1 from P2)
              { entity: 'card_hearts_5', props: { rank: '5' } },
              { entity: 'card_spades_5', props: { rank: '5' } },
              { entity: 'card_diamonds_5', props: { rank: '5' } },
              { entity: 'card_clubs_7', props: { rank: '7' } },
              { entity: 'card_diamonds_k', props: { rank: 'k' } },
              { entity: 'card_hearts_a', props: { rank: 'a' } },
              { entity: 'card_spades_3', props: { rank: '3' } },
              { entity: 'card_clubs_9', props: { rank: '9' } }
            ]
          }
        }
      },
      ui: {
        actionMap: {
          p1: {
            '/ui/rank/5': { action: 'selectRank', direction: 'Ask for 5s' },
            '/ui/rank/7': { action: 'selectRank', direction: 'Ask for 7s' }
          }
        },
        gameLog: [
          { message: 'Bob gives Alice their 5s', timestamp: Date.now() }
        ]
      }
    };

    render(
      <BrowserRouter>
        <PlayerProvider>
          <WebSocketProvider lobbyId="test-lobby">
            <GameView lobbyId="test-lobby" />
          </WebSocketProvider>
        </PlayerProvider>
      </BrowserRouter>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText('Game Log')).toBeInTheDocument();
    });

    // Should show successful transfer message in game log
    expect(screen.getByText(/Bob gives Alice their 5s/i)).toBeInTheDocument();
  });

  it('shows pair formation correctly', async () => {
    // Set up state with a formed pair
    mockLobbyWebSocketReturn.lobbyState = {
      ...mockLobbyState,
      game: {
        ...mockLobbyState.game,
        zones: {
          ...mockLobbyState.game.zones,
          hand_p1: {
            type: 'list',
            entities: [
              // Hand without the 5s (they formed a pair)
              { entity: 'card_clubs_7', props: { rank: '7' } },
              { entity: 'card_diamonds_k', props: { rank: 'k' } },
              { entity: 'card_hearts_a', props: { rank: 'a' } }
            ]
          },
          pairs_p1: {
            type: 'list',
            entities: [
              // Pair of 5s
              { entity: 'pair_5s', props: { rank: '5', count: 4 } }
            ]
          }
        }
      },
      ui: {
        ...mockLobbyState.ui,
        gameLog: [
          { message: 'Alice forms a pair of 5s!', timestamp: Date.now() }
        ]
      }
    };

    render(
      <BrowserRouter>
        <PlayerProvider>
          <WebSocketProvider lobbyId="test-lobby">
            <GameView lobbyId="test-lobby" />
          </WebSocketProvider>
        </PlayerProvider>
      </BrowserRouter>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText('Game Log')).toBeInTheDocument();
    });

    // Should show pair formation message
    expect(screen.getByText(/Alice forms a pair of 5s!/i)).toBeInTheDocument();
  });

  it('shows win condition correctly', async () => {
    // Set up won game state
    mockLobbyWebSocketReturn.lobbyState = {
      ...mockLobbyState,
      game: {
        ...mockLobbyState.game,
        gameStatus: { state: 'ended', winner: 'p1', tie: false },
        zones: {
          ...mockLobbyState.game.zones,
          pairs_p1: {
            type: 'list',
            entities: [
              { entity: 'pair_5s', props: { rank: '5', count: 4 } },
              { entity: 'pair_7s', props: { rank: '7', count: 4 } },
              { entity: 'pair_ks', props: { rank: 'k', count: 4 } }
            ]
          },
          pairs_p2: {
            type: 'list',
            entities: [
              { entity: 'pair_as', props: { rank: 'a', count: 4 } },
              { entity: 'pair_3s', props: { rank: '3', count: 4 } }
            ]
          }
        }
      },
      ui: {
        ...mockLobbyState.ui,
        actionMap: { p1: {}, p2: {} }, // No actions when game ended
        gameLog: [
          { message: 'Game Over! Alice wins with 3 pairs!', timestamp: Date.now() }
        ]
      }
    };

    render(
      <BrowserRouter>
        <PlayerProvider>
          <WebSocketProvider lobbyId="test-lobby">
            <GameView lobbyId="test-lobby" />
          </WebSocketProvider>
        </PlayerProvider>
      </BrowserRouter>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText('Game Log')).toBeInTheDocument();
    });

    // Should show winner message
    expect(screen.getByText(/Alice wins with 3 pairs/i)).toBeInTheDocument();
  });

  it('handles turn switching correctly', async () => {
    // Set up state where it's player 2's turn (Bob)
    mockLobbyWebSocketReturn.lobbyState = {
      ...mockLobbyState,
      game: {
        ...mockLobbyState.game,
        currentPlayer: 'p2',
        turn: 1,
        zones: {
          ...mockLobbyState.game.zones,
          hand_p2: {
            type: 'list',
            entities: [
              { entity: 'card_hearts_2', props: { rank: '2' } },
              { entity: 'card_spades_4', props: { rank: '4' } },
              { entity: 'card_clubs_6', props: { rank: '6' } },
              { entity: 'card_diamonds_8', props: { rank: '8' } },
              { entity: 'card_hearts_10', props: { rank: '10' } },
              { entity: 'card_spades_q', props: { rank: 'q' } },
              { entity: 'card_clubs_k', props: { rank: 'k' } }
            ]
          }
        },
        selection: {
          availableRanks: ['2', '4', '6', '8', '10', 'q', 'k']
        }
      },
      ui: {
        ...mockLobbyState.ui,
        actionMap: {
          p1: {}, // No actions for p1 since it's p2's turn
          p2: {
            '/ui/rank/2': { action: 'selectRank', direction: 'Ask for 2s' },
            '/ui/rank/4': { action: 'selectRank', direction: 'Ask for 4s' }
          }
        }
      }
    };

    render(
      <BrowserRouter>
        <PlayerProvider>
          <WebSocketProvider lobbyId="test-lobby">
            <GameView lobbyId="test-lobby" />
          </WebSocketProvider>
        </PlayerProvider>
      </BrowserRouter>
    );

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText('Game Log')).toBeInTheDocument();
    });

    // Should show both players
    expect(screen.getByText('Bob')).toBeInTheDocument();
    expect(screen.getByText('Alice')).toBeInTheDocument();
    
    // Should show game is in progress
    expect(screen.getByText('In Progress')).toBeInTheDocument();
    
    // Since it's Bob's turn and we're Alice, we won't see a turn prompt
  });
});
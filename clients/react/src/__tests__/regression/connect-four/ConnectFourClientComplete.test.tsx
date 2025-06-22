import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import React from 'react';
import { BrowserRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import GameView from '../../../components/GameView';
import { WebSocketProvider } from '../../../context/WebSocketContext';
import { PlayerProvider } from '../../../context/PlayerContext';
import type { LobbyState } from '../../../types/messages';

// Mock the WebSocket and React Router
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => vi.fn()
  };
});

// Mock the WebSocket hook
let mockLobbyState: any = {};
const mockSendMessage = vi.fn();

vi.mock('../../../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: () => ({
    sendMessage: mockSendMessage,
    lobbyState: mockLobbyState,
    connectionState: 'connected',
    joinLobby: vi.fn(),
    leaveLobby: vi.fn(),
    startGame: vi.fn(),
    disconnect: vi.fn()
  })
}));

// Mock the API calls
vi.mock('../../../api/lobbies', () => ({
  getLobby: vi.fn().mockResolvedValue({
    id: 'test-lobby',
    game_id: 'connect-four',
    players: ['Alice', 'Bob'],
    started: true,
    manifest: {
      gameId: 'connect-four',
      version: '1.0',
      metadata: {
        name: 'Connect Four',
        description: 'Classic Connect Four game',
        author: 'Bluefelt',
        players: { min: 2, max: 2 }
      }
    }
  })
}));

// Mock the PlayerContext
vi.mock('../../../context/PlayerContext', () => ({
  PlayerProvider: ({ children }: { children: React.ReactNode }) => children,
  usePlayer: () => ({
    player: { username: 'Alice', color: '#FF0000' }
  })
}));

// Mock the AnimationContext
vi.mock('../../../context/AnimationContext', () => ({
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

// Mock the PlayerPreferencesContext
vi.mock('../../../context/PlayerPreferencesContext', () => ({
  usePlayerPreferences: vi.fn(() => ({
    preferences: { 
      username: 'Alice', 
      color: '#FF0000',
      tokenId: 'classic',
      showOpponentTokens: true,
      colorSchemeId: 'default',
      cardStyleId: 'classic'
    },
    isLoggedIn: true,
    login: vi.fn(),
    logout: vi.fn(),
    updatePreferences: vi.fn(),
    updateToken: vi.fn(),
    updateColorScheme: vi.fn(),
    updatePlayerColor: vi.fn(),
    updateShowOpponentTokens: vi.fn(),
    updateCardStyle: vi.fn(),
    updateColor: vi.fn()
  })),
  PlayerPreferencesProvider: ({ children }: { children: React.ReactNode }) => children,
  usePlayer: vi.fn(() => ({
    player: { username: 'Alice', color: '#FF0000' },
    login: vi.fn(),
    logout: vi.fn(),
    updateColor: vi.fn()
  }))
}));

// Helper to render game with all providers
function renderGame(lobbyState: LobbyState, lobbyId: string = 'test-lobby') {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false }
    }
  });
  
  // Update the mock's lobby state
  mockLobbyState = lobbyState;
  
  const utils = render(
    <BrowserRouter>
      <QueryClientProvider client={queryClient}>
        <PlayerProvider initialPlayer={{ username: 'Alice', color: '#ff0000' }}>
          <GameView lobbyId={lobbyId} onLeave={() => {}} />
        </PlayerProvider>
      </QueryClientProvider>
    </BrowserRouter>
  );
  
  return { ...utils };
}

/**
 * Client-side regression tests for Connect Four
 * Tests the client UI and state management for Connect Four gameplay
 */
describe('Connect Four Client Regression Tests', () => {
  let mockWebSocket: any;
  let onMessage: (event: MessageEvent) => void;
  let lobbyState: LobbyState;

  beforeEach(() => {
    // Reset mocks
    mockSendMessage.mockClear();
    
    // Reset lobby state for Connect Four
    lobbyState = {
      you: 'p1',
      started: true,
      game: {
        currentPlayer: 'p1',
        turn: 0,
        tick: 0,
        zones: {
          board: {
            type: 'grid',
            // 6x7 Connect Four board (6 rows, 7 columns)
            cells: Array(6).fill(null).map(() => Array(7).fill(null))
          }
        },
        players: [{ id: 'p1' }, { id: 'p2' }],
        phases: { game: 'play' },
        gameStatus: { state: 'playing', winner: null, tie: false },
        selection: {}
      },
      ui: {
        actionMap: {
          p1: {
            // Connect Four allows dropping in any column
            '/zones/board/columns/0': { action: 'dropDisc', direction: 'Drop disc in column 1', targetColumn: 0 },
            '/zones/board/columns/1': { action: 'dropDisc', direction: 'Drop disc in column 2', targetColumn: 1 },
            '/zones/board/columns/2': { action: 'dropDisc', direction: 'Drop disc in column 3', targetColumn: 2 },
            '/zones/board/columns/3': { action: 'dropDisc', direction: 'Drop disc in column 4', targetColumn: 3 },
            '/zones/board/columns/4': { action: 'dropDisc', direction: 'Drop disc in column 5', targetColumn: 4 },
            '/zones/board/columns/5': { action: 'dropDisc', direction: 'Drop disc in column 6', targetColumn: 5 },
            '/zones/board/columns/6': { action: 'dropDisc', direction: 'Drop disc in column 7', targetColumn: 6 }
          },
          p2: {}
        },
        zones: [
          { 
            id: 'board', 
            shape: 'grid',
            renderType: 'grid',
            name: 'Board', 
            visibility: 'all',
            gridProps: { rows: 6, cols: 7 }
          }
        ],
        entities: [
          { id: 'disc_p1', type: 'disc', props: { owner: 'p1' } },
          { id: 'disc_p2', type: 'disc', props: { owner: 'p2' } }
        ],
        players: ['Alice', 'Bob'],
        gameLog: [],
        manifest: {
          gameId: 'connect-four',
          version: '1.0',
          specVersion: '1.0',
          metadata: {
            name: 'Connect Four',
            description: 'Classic Connect Four game',
            author: 'Bluefelt',
            players: { min: 2, max: 2 }
          }
        }
      },
      players: {
        p1: { name: 'Alice', connected: true },
        p2: { name: 'Bob', connected: true }
      }
    };

    // Mock WebSocket
    mockWebSocket = {
      send: vi.fn(),
      close: vi.fn(),
      readyState: WebSocket.OPEN
    };

    // Mock WebSocket constructor
    global.WebSocket = vi.fn(() => mockWebSocket);

    // Set up message handler
    onMessage = vi.fn();
    
    // Set the mock lobby state
    mockLobbyState = lobbyState;
  });

  it('renders Connect Four game board correctly', async () => {
    await act(async () => {
      renderGame(lobbyState);
    });

    // Wait for game to render
    await waitFor(() => {
      expect(screen.getByText('Board')).toBeInTheDocument();
    });

    // Check that we have a board zone
    const boardZone = screen.getByTestId('board-zone');
    expect(boardZone).toBeInTheDocument();
    
    // Should show column drop zones with arrows
    const columnDropZones = screen.getAllByText('↓');
    expect(columnDropZones).toHaveLength(7); // 7 columns
  });

  it('handles disc drop action correctly', async () => {
    await act(async () => {
      renderGame(lobbyState);
    });

    // Wait for game to render
    await waitFor(() => {
      expect(screen.getByText('Board')).toBeInTheDocument();
    });

    // Get the board zone and find column drop zones
    const boardZone = screen.getByTestId('board-zone');
    const columnDropZones = screen.getAllByText('↓');
    expect(columnDropZones).toHaveLength(7);

    // Click on first column (index 0)
    await act(async () => {
      fireEvent.click(columnDropZones[0]);
    });

    // Should have sent a drop disc message
    expect(mockSendMessage).toHaveBeenCalledWith(
      expect.stringContaining('"action":"dropDisc"')
    );
  });

  it('shows win condition correctly', async () => {
    // Set up won game state
    const wonState = {
      ...lobbyState,
      game: {
        ...lobbyState.game,
        gameStatus: { state: 'ended', winner: 'p1', tie: false },
        zones: {
          board: {
            type: 'grid',
            // Board with 4 in a row for p1
            cells: [
              [null, null, null, null, null, null, null],
              [null, null, null, null, null, null, null],
              [null, null, null, null, null, null, null],
              ['disc_p1', 'disc_p1', 'disc_p1', 'disc_p1', null, null, null],
              ['disc_p2', 'disc_p2', 'disc_p2', null, null, null, null],
              ['disc_p1', 'disc_p2', 'disc_p1', null, null, null, null]
            ]
          }
        }
      },
      ui: {
        ...lobbyState.ui,
        actionMap: { p1: {}, p2: {} } // No actions when game ended
      }
    };

    await act(async () => {
      renderGame(wonState);
    });

    // Wait for the winner banner to appear
    await waitFor(() => {
      // Should show winner with "Player Alice wins!" format
      expect(screen.getByText(/Player Alice wins!/i)).toBeInTheDocument();
    });
  });

  it('handles turn switching correctly', async () => {
    // Set up state where it's player 2's turn
    const p2TurnState = {
      ...lobbyState,
      you: 'p1', // We're still player 1
      game: {
        ...lobbyState.game,
        currentPlayer: 'p2',
        turn: 1
      },
      ui: {
        ...lobbyState.ui,
        actionMap: {
          p1: {}, // p1 has no actions since it's not their turn
          p2: {
            '/zones/board/columns/0': { action: 'dropDisc', direction: 'Drop disc in column 1' },
            '/zones/board/columns/1': { action: 'dropDisc', direction: 'Drop disc in column 2' }
          }
        }
      }
    };

    await act(async () => {
      renderGame(p2TurnState);
    });

    // Wait for game header to render
    await waitFor(() => {
      // Should show it's Bob's turn in the turn banner
      expect(screen.getByText(/BOB'S TURN/i)).toBeInTheDocument();
    });

    // The board should still be visible but not interactive for player 1
    const boardZone = screen.getByTestId('board-zone');
    expect(boardZone).toBeInTheDocument();
    
    // Since it's player 2's turn and we're player 1, we shouldn't see any clickable arrows
    const arrows = screen.queryAllByText('↓');
    expect(arrows).toHaveLength(0);
  });

  it('handles board state updates from patches', async () => {
    // Initial render
    await act(async () => {
      renderGame(lobbyState);
    });

    // Wait for initial render
    await waitFor(() => {
      expect(screen.getByText('Board')).toBeInTheDocument();
    });

    // Simulate receiving a patch that places a disc
    const updatedState = {
      ...lobbyState,
      game: {
        ...lobbyState.game,
        zones: {
          board: {
            type: 'grid',
            cells: [
              [null, null, null, null, null, null, null],
              [null, null, null, null, null, null, null],
              [null, null, null, null, null, null, null],
              [null, null, null, null, null, null, null],
              [null, null, null, null, null, null, null],
              ['disc_p1', null, null, null, null, null, null] // Disc placed
            ]
          }
        },
        currentPlayer: 'p2',
        turn: 1
      },
      ui: {
        ...lobbyState.ui,
        actionMap: {
          p1: {}, // p1 has no actions since it's p2's turn
          p2: {
            '/zones/board/columns/1': { action: 'dropDisc', direction: 'Drop disc in column 2' },
            '/zones/board/columns/2': { action: 'dropDisc', direction: 'Drop disc in column 3' },
            '/zones/board/columns/3': { action: 'dropDisc', direction: 'Drop disc in column 4' },
            '/zones/board/columns/4': { action: 'dropDisc', direction: 'Drop disc in column 5' },
            '/zones/board/columns/5': { action: 'dropDisc', direction: 'Drop disc in column 6' },
            '/zones/board/columns/6': { action: 'dropDisc', direction: 'Drop disc in column 7' }
          }
        }
      }
    };

    // Update the mock and re-render
    mockLobbyState = updatedState;
    
    await act(async () => {
      renderGame(updatedState);
    });

    // Board should reflect the change - it's now Bob's turn
    await waitFor(() => {
      expect(screen.getByText(/BOB'S TURN/i)).toBeInTheDocument();
    });
  });
});
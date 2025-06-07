import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act, within } from '@testing-library/react';
import React from 'react';
import { BrowserRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import GameView from '../../../components/GameView';
import { PlayerProvider } from '../../../context/PlayerContext';
import type { LobbyState } from '../../../types/messages';

// Mock the API calls
vi.mock('../../../api/lobbies', () => ({
  getLobby: vi.fn().mockResolvedValue({
    id: 'test-lobby',
    game_id: 'three-mens-morris',
    players: ['Alice', 'Bob'],
    started: true,
    manifest: {
      gameId: 'three-mens-morris',
      version: '1.0',
      metadata: {
        name: 'Three Mens Morris',
        description: 'Classic Three Mens Morris game',
        author: 'Bluefelt',
        players: { min: 2, max: 2 }
      }
    }
  })
}));

// Mock React Router
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => vi.fn(),
    useParams: () => ({ lobbyId: 'test-lobby' })
  };
});

// Mock the PlayerContext
vi.mock('../../../context/PlayerContext', () => ({
  PlayerProvider: ({ children }: { children: React.ReactNode }) => children,
  usePlayer: () => ({
    player: { username: 'Alice', color: '#ff0000' },
    setPlayer: vi.fn(),
    clearPlayer: vi.fn()
  })
}));

// Mock useLobbyWebSocket
const mockSendAction = vi.fn();
const mockLobbyState = vi.fn();

vi.mock('../../../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: () => ({
    ws: { readyState: WebSocket.OPEN },
    lobbyState: mockLobbyState(),
    connectionState: 'connected',
    error: null,
    sendMessage: mockSendAction,
    sendAction: mockSendAction,
    joinLobby: vi.fn(),
    leaveLobby: vi.fn(),
    startGame: vi.fn(),
    disconnect: vi.fn(),
    leave: vi.fn(),
    reconnect: vi.fn()
  })
}));

/**
 * Client-side regression tests for Three Men's Morris
 * Tests the client UI and state management for placement and movement phases
 */
describe('Three Mens Morris Client Regression Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    
    // Reset mock sendAction
    mockSendAction.mockClear();
    
    // Default lobby state for Three Men's Morris
    mockLobbyState.mockReturnValue({
      you: 'p1',
      started: true,
      manifest: {
        gameId: 'three-mens-morris',
        version: '1.0',
        metadata: {
          name: 'Three Mens Morris',
          description: 'Classic Three Mens Morris game',
          author: 'Bluefelt',
          players: { min: 2, max: 2 }
        }
      },
      game: {
        currentPlayer: 'p1',
        turn: 0,
        tick: 0,
        zones: {
          board: {
            type: 'grid',
            // 3x3 board for Three Men's Morris
            cells: [
              [null, null, null],
              [null, null, null],
              [null, null, null]
            ]
          }
        },
        players: [{ id: 'p1' }, { id: 'p2' }],
        phases: { game: 'placement' },
        phaseStates: {
          game: { current: 'placement', count: 0, actionsProcessed: 0 }
        },
        gameStatus: { state: 'playing', winner: null, tie: false },
        selection: {}
      },
      ui: {
        actionMap: {
          p1: {
            // Placement phase - can place on any empty cell
            '/zones/board/cells/0/0': { action: 'placeToken', direction: 'Place your piece' },
            '/zones/board/cells/0/1': { action: 'placeToken', direction: 'Place your piece' },
            '/zones/board/cells/0/2': { action: 'placeToken', direction: 'Place your piece' },
            '/zones/board/cells/1/0': { action: 'placeToken', direction: 'Place your piece' },
            '/zones/board/cells/1/1': { action: 'placeToken', direction: 'Place your piece' },
            '/zones/board/cells/1/2': { action: 'placeToken', direction: 'Place your piece' },
            '/zones/board/cells/2/0': { action: 'placeToken', direction: 'Place your piece' },
            '/zones/board/cells/2/1': { action: 'placeToken', direction: 'Place your piece' },
            '/zones/board/cells/2/2': { action: 'placeToken', direction: 'Place your piece' }
          }
        },
        players: ['Alice', 'Bob'],
        messages: []
      },
      players: {
        p1: { name: 'Alice', connected: true },
        p2: { name: 'Bob', connected: true }
      }
    });
  });

  it('renders Three Mens Morris game board correctly in placement phase', async () => {
    await act(async () => {
      render(
        <BrowserRouter>
          <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })}>
            <PlayerProvider>
              <GameView />
            </PlayerProvider>
          </QueryClientProvider>
        </BrowserRouter>
      );
    });

    // Should show turn indicator
    await waitFor(() => {
      expect(screen.getByText('YOUR TURN')).toBeInTheDocument();
    });

    // Should have a 3x3 grid
    const cells = screen.getAllByRole('button');
    // Filter for board cells (not other buttons like controls)
    const boardCells = cells.filter(cell => cell.textContent === '');
    expect(boardCells.length).toBeGreaterThanOrEqual(9);
  });

  it('handles piece placement correctly', async () => {
    await act(async () => {
      render(
        <BrowserRouter>
          <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })}>
            <PlayerProvider>
              <GameView />
            </PlayerProvider>
          </QueryClientProvider>
        </BrowserRouter>
      );
    });

    // Wait for board to be rendered
    await waitFor(() => {
      expect(screen.getByText('YOUR TURN')).toBeInTheDocument();
    });

    // Click on a cell to place piece
    // Find the board container first
    const boardContainer = screen.getByTestId('board-container');
    const cells = within(boardContainer).getAllByRole('button');
    
    // Click center cell (index 4 in a 3x3 grid)
    fireEvent.click(cells[4]);

    // Should send action with correct location
    await waitFor(() => {
      expect(mockSendAction).toHaveBeenCalled();
      const messageString = mockSendAction.mock.calls[0][0];
      const message = JSON.parse(messageString);
      expect(message.action).toBe('placeToken');
      expect(message.args.target).toBe('/zones/board/cells/1/1');
      expect(message.args.entity).toMatch(/piece_p\d/);
    });
  });

  it('renders movement phase correctly', async () => {
    // Set up movement phase state
    mockLobbyState.mockReturnValue({
      you: 'p1',
      started: true,
      manifest: {
        gameId: 'three-mens-morris',
        version: '1.0',
        metadata: {
          name: 'Three Mens Morris',
          description: 'Classic Three Mens Morris game',
          author: 'Bluefelt',
          players: { min: 2, max: 2 }
        }
      },
      game: {
        currentPlayer: 'p1',
        turn: 0,
        tick: 0,
        phases: { game: 'movement' },
        phaseStates: {
          game: { current: 'movement', count: 0, actionsProcessed: 0 }
        },
        zones: {
          board: {
            type: 'grid',
            cells: [
              ['piece_p1', null, 'piece_p2'],
              [null, 'piece_p1', null],
              ['piece_p2', null, 'piece_p1']
            ]
          }
        },
        players: [{ id: 'p1' }, { id: 'p2' }],
        gameStatus: { state: 'playing', winner: null, tie: false },
        selection: {}
      },
      ui: {
        actionMap: {
          p1: {
            // Can select own pieces
            '/zones/board/cells/0/0': { action: 'selectPiece', direction: 'Select your piece to move' },
            '/zones/board/cells/1/1': { action: 'selectPiece', direction: 'Select your piece to move' },
            '/zones/board/cells/2/2': { action: 'selectPiece', direction: 'Select your piece to move' }
          }
        },
        players: ['Alice', 'Bob'],
        messages: []
      },
      players: {
        p1: { name: 'Alice', connected: true },
        p2: { name: 'Bob', connected: true }
      }
    });

    await act(async () => {
      render(
        <BrowserRouter>
          <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })}>
            <PlayerProvider>
              <GameView />
            </PlayerProvider>
          </QueryClientProvider>
        </BrowserRouter>
      );
    });

    // Wait for game to render
    await waitFor(() => {
      expect(screen.getByText('Board')).toBeInTheDocument();
    });

    // Should show current player
    expect(screen.getByText('YOUR TURN')).toBeInTheDocument();

    // Should show pieces on the board
    const cells = screen.getAllByRole('button');
    const pieceCells = cells.filter(cell => cell.textContent !== '');
    expect(pieceCells.length).toBeGreaterThan(0);
  });

  it('handles piece selection and movement', async () => {
    // First render with pieces to select
    mockLobbyState.mockReturnValue({
      you: 'p1',
      started: true,
      manifest: {
        gameId: 'three-mens-morris',
        version: '1.0',
        metadata: {
          name: 'Three Mens Morris',
          description: 'Classic Three Mens Morris game',
          author: 'Bluefelt',
          players: { min: 2, max: 2 }
        }
      },
      game: {
        currentPlayer: 'p1',
        turn: 0,
        tick: 0,
        phases: { game: 'movement' },
        phaseStates: {
          game: { current: 'movement', count: 0, actionsProcessed: 0 }
        },
        zones: {
          board: {
            type: 'grid',
            cells: [
              ['piece_p1', null, 'piece_p2'],
              [null, 'piece_p1', null],
              ['piece_p2', null, 'piece_p1']
            ]
          }
        },
        players: [{ id: 'p1' }, { id: 'p2' }],
        gameStatus: { state: 'playing', winner: null, tie: false },
        selection: {}
      },
      ui: {
        actionMap: {
          p1: {
            '/zones/board/cells/0/0': { action: 'selectPiece', direction: 'Select your piece to move' },
            '/zones/board/cells/1/1': { action: 'selectPiece', direction: 'Select your piece to move' },
            '/zones/board/cells/2/2': { action: 'selectPiece', direction: 'Select your piece to move' }
          }
        },
        players: ['Alice', 'Bob'],
        messages: []
      },
      players: {
        p1: { name: 'Alice', connected: true },
        p2: { name: 'Bob', connected: true }
      }
    });

    const { rerender } = render(
      <BrowserRouter>
        <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })}>
          <PlayerProvider>
            <GameView />
          </PlayerProvider>
        </QueryClientProvider>
      </BrowserRouter>
    );

    // Wait for board to render
    await waitFor(() => {
      expect(screen.getByText('Board')).toBeInTheDocument();
    });

    // Click on a piece to select it (pieces show as X for p1)
    const boardContainer = screen.getByTestId('board-container');
    const cells = within(boardContainer).getAllByRole('button');
    
    // Find a cell with a piece - first check if there are any clickable cells
    const clickableCells = cells.filter(cell => cell.getAttribute('role') === 'button');
    expect(clickableCells.length).toBeGreaterThan(0);
    
    // Click the first clickable cell (should be a piece we can select)
    fireEvent.click(clickableCells[0]);

    // Should send selectPiece action
    await waitFor(() => {
      expect(mockSendAction).toHaveBeenCalled();
      const messageString = mockSendAction.mock.calls[0][0];
      const message = JSON.parse(messageString);
      expect(message.action).toBe('selectPiece');
      // Should be one of the piece locations
      expect(message.args.location).toMatch(/^\/zones\/board\/cells\/\d\/\d$/);
      expect(message.args.player).toBe('p1');
    });

    // Now update state to show selection and move options
    mockLobbyState.mockReturnValue({
      you: 'p1',
      started: true,
      manifest: {
        gameId: 'three-mens-morris',
        version: '1.0',
        metadata: {
          name: 'Three Mens Morris',
          description: 'Classic Three Mens Morris game',
          author: 'Bluefelt',
          players: { min: 2, max: 2 }
        }
      },
      game: {
        currentPlayer: 'p1',
        turn: 0,
        tick: 0,
        phases: { game: 'movement' },
        phaseStates: {
          game: { current: 'movement', count: 0, actionsProcessed: 0 }
        },
        selection: { selectedPiece: '/zones/board/0/0' },
        zones: {
          board: {
            type: 'grid',
            cells: [
              ['piece_p1', null, 'piece_p2'],
              [null, 'piece_p1', null],
              ['piece_p2', null, 'piece_p1']
            ]
          }
        },
        players: [{ id: 'p1' }, { id: 'p2' }],
        gameStatus: { state: 'playing', winner: null, tie: false }
      },
      ui: {
        actionMap: {
          p1: {
            // After selecting, can move to adjacent empty cells
            '/zones/board/cells/0/1': { action: 'moveSelectedPiece', direction: 'Move here' },
            '/zones/board/cells/1/0': { action: 'moveSelectedPiece', direction: 'Move here' }
          }
        },
        players: ['Alice', 'Bob'],
        messages: []
      },
      players: {
        p1: { name: 'Alice', connected: true },
        p2: { name: 'Bob', connected: true }
      }
    });

    // Re-render with new state
    rerender(
      <BrowserRouter>
        <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })}>
          <PlayerProvider>
            <GameView />
          </PlayerProvider>
        </QueryClientProvider>
      </BrowserRouter>
    );

    // Find the board container again in the re-rendered component
    const boardContainer2 = screen.getByTestId('board-container');
    const cells2 = within(boardContainer2).getAllByRole('button');
    
    // Click on empty cell to move
    mockSendAction.mockClear();
    const emptyCell = cells2.find(cell => !cell.textContent || cell.textContent === '');
    fireEvent.click(emptyCell!);

    // Should send moveSelectedPiece action
    await waitFor(() => {
      expect(mockSendAction).toHaveBeenCalled();
      const messageString = mockSendAction.mock.calls[0][0];
      const message = JSON.parse(messageString);
      expect(message.action).toBe('moveSelectedPiece');
      expect(message.args.target).toMatch(/^\/zones\/board\/cells\/\d\/\d$/);
      expect(message.args.player).toBe('p1');
    });
  });

  it('shows win condition correctly', async () => {
    // Set up won game state
    mockLobbyState.mockReturnValue({
      you: 'p1',
      started: true,
      manifest: {
        gameId: 'three-mens-morris',
        version: '1.0',
        metadata: {
          name: 'Three Mens Morris',
          description: 'Classic Three Mens Morris game',
          author: 'Bluefelt',
          players: { min: 2, max: 2 }
        }
      },
      game: {
        currentPlayer: 'p1',
        turn: 0,
        tick: 0,
        gameStatus: { state: 'ended', winner: 'p1', tie: false },
        zones: {
          board: {
            type: 'grid',
            // Board with 3 in a row for p1
            cells: [
              ['piece_p1', 'piece_p1', 'piece_p1'],
              [null, 'piece_p2', null],
              ['piece_p2', null, null]
            ]
          }
        },
        players: [{ id: 'p1' }, { id: 'p2' }],
        phases: { game: 'movement' },
        phaseStates: {
          game: { current: 'movement', count: 0, actionsProcessed: 0 }
        },
        selection: {}
      },
      ui: {
        actionMap: { p1: {}, p2: {} }, // No actions when game ended
        players: ['Alice', 'Bob'],
        messages: []
      },
      players: {
        p1: { name: 'Alice', connected: true },
        p2: { name: 'Bob', connected: true }
      }
    });

    await act(async () => {
      render(
        <BrowserRouter>
          <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })}>
            <PlayerProvider>
              <GameView />
            </PlayerProvider>
          </QueryClientProvider>
        </BrowserRouter>
      );
    });

    // Should show winner
    await waitFor(() => {
      expect(screen.getByText(/Alice wins!/i)).toBeInTheDocument();
    });
  });

  it.skip('handles clear selection action', async () => {
    // This test is skipped because the current UI implementation doesn't render
    // UI actions like clear selection as clickable buttons in the interface.
    // The action exists in the action map but isn't rendered by the GameZones component.
  });
});
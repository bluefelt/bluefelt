import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within, act } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { PlayerProvider } from '../../context/PlayerContext';
import GameView from '../../components/GameView';
import type { LobbyState } from '../../ws/useLobbyWebSocket';

// Mock the PlayerContext with a fixed player
vi.mock('../../context/PlayerContext', () => ({
  usePlayer: () => ({
    player: { username: 'Alice', color: '#ff0000' },
    setPlayer: vi.fn(),
    clearPlayer: vi.fn()
  }),
  PlayerProvider: ({ children }: { children: React.ReactNode }) => children
}));

// Mock the AnimationContext
vi.mock('../../context/AnimationContext', () => ({
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
vi.mock('../../context/PlayerPreferencesContext', () => ({
  usePlayerPreferences: vi.fn(() => ({
    preferences: { 
      username: 'Alice', 
      color: '#ff0000',
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
    player: { username: 'Alice', color: '#ff0000' },
    login: vi.fn(),
    logout: vi.fn(),
    updateColor: vi.fn()
  }))
}));

// Mock WebSocket to track client messages
class MockWebSocket {
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  readyState: number = WebSocket.CONNECTING;
  
  constructor(public url: string) {
    setTimeout(() => {
      this.readyState = WebSocket.OPEN;
      this.onopen?.(new Event('open'));
    }, 0);
  }
  
  send(data: string) {
    const message = JSON.parse(data);
    (window as any).__sentMessages = (window as any).__sentMessages || [];
    (window as any).__sentMessages.push(message);
  }
  
  close() {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close'));
  }
}

// Helper to simulate server responses
function simulateServerMessage(ws: MockWebSocket, message: any) {
  ws.onmessage?.(new MessageEvent('message', {
    data: JSON.stringify(message)
  }));
}

describe('Three Men\'s Morris Client-Server Integration', () => {
  let mockWS: MockWebSocket;
  let queryClient: QueryClient;
  
  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false }
      }
    });
    
    // Clear sent messages
    (window as any).__sentMessages = [];
    
    // Mock WebSocket globally
    vi.stubGlobal('WebSocket', vi.fn().mockImplementation((url) => {
      mockWS = new MockWebSocket(url);
      return mockWS;
    }));
    
    // Mock fetch for lobby creation
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ id: 'test123', started: false })
    }));
  });
  
  afterEach(() => {
    vi.restoreAllMocks();
  });
  
  it.skip('should handle complete placement to movement phase flow with clicks', async () => {
    // Initial state: game just started, placement phase
    const initialState: LobbyState = {
      you: 'p1',
      started: true,
      game: {
        turn: 0,
        currentPlayer: 'p1',
        tick: 0,
        gameStatus: { state: 'playing', winner: null, tie: false },
        players: [{ id: 'p1' }, { id: 'p2' }],
        zones: {
          board: {
            cells: [
              [null, null, null],
              [null, null, null], 
              [null, null, null]
            ]
          }
        },
        phases: {
          game: { current: 'placement', count: 0, actionsProcessed: 0 }
        },
        selection: {}
      },
      ui: {
        actionMap: {
          p1: {
            '/zones/board/cells/0/0': { action: 'placeToken', args: { target: '/zones/board/cells/0/0', entity: 'piece_p1' }, direction: 'Place a piece' },
            '/zones/board/cells/0/1': { action: 'placeToken', args: { target: '/zones/board/cells/0/1', entity: 'piece_p1' }, direction: 'Place a piece' },
            '/zones/board/cells/0/2': { action: 'placeToken', args: { target: '/zones/board/cells/0/2', entity: 'piece_p1' }, direction: 'Place a piece' },
            '/zones/board/cells/1/0': { action: 'placeToken', args: { target: '/zones/board/cells/1/0', entity: 'piece_p1' }, direction: 'Place a piece' },
            '/zones/board/cells/1/1': { action: 'placeToken', args: { target: '/zones/board/cells/1/1', entity: 'piece_p1' }, direction: 'Place a piece' },
            '/zones/board/cells/1/2': { action: 'placeToken', args: { target: '/zones/board/cells/1/2', entity: 'piece_p1' }, direction: 'Place a piece' },
            '/zones/board/cells/2/0': { action: 'placeToken', args: { target: '/zones/board/cells/2/0', entity: 'piece_p1' }, direction: 'Place a piece' },
            '/zones/board/cells/2/1': { action: 'placeToken', args: { target: '/zones/board/cells/2/1', entity: 'piece_p1' }, direction: 'Place a piece' },
            '/zones/board/cells/2/2': { action: 'placeToken', args: { target: '/zones/board/cells/2/2', entity: 'piece_p1' }, direction: 'Place a piece' }
          },
          p2: {}
        },
        players: ['Alice', 'Bob'],
        entities: [
          { id: 'piece_p1', ui: { glyph: 'X' } },
          { id: 'piece_p2', ui: { glyph: 'O' } }
        ],
        zones: [],
        gameLog: [],
        manifest: {
          gameId: 'three-mens-morris',
          version: '1.0',
          specVersion: '1.0',
          metadata: {
            name: 'Three Men\'s Morris',
            description: 'Classic strategy game',
            author: 'Test',
            players: { min: 2, max: 2 }
          }
        }
      }
    };
    
    // Mock useLobbyWebSocket to return our state
    vi.doMock('../../ws/useLobbyWebSocket', () => ({
      useLobbyWebSocket: () => ({
        lobbyState: initialState,
        sendMessage: vi.fn((msg) => {
          const message = JSON.parse(msg);
          (window as any).__sentMessages = (window as any).__sentMessages || [];
          (window as any).__sentMessages.push(message);
          return true;
        }),
        connected: true,
        connectionState: 'connected',
        joinLobby: vi.fn(),
        leaveLobby: vi.fn(),
        startGame: vi.fn(),
        disconnect: vi.fn()
      })
    }));
    
    const TestWrapper = ({ children }: { children: React.ReactNode }) => (
      <BrowserRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            {children}
          </PlayerProvider>
        </QueryClientProvider>
      </BrowserRouter>
    );
    
    await act(async () => {
      render(
        <TestWrapper>
          <GameView lobbyId="test123" />
        </TestWrapper>
      );
    });
    
    // Wait for game to render
    await waitFor(() => {
      expect(screen.getByText('Three Men\'s Morris')).toBeInTheDocument();
    });
    
    // Test 1: Click on cell (0,0) for placement
    const cell00 = screen.getByTestId('cell-0-0');
    expect(cell00).toBeInTheDocument();
    
    await act(async () => {
      fireEvent.click(cell00);
    });
    
    // Verify client sent placeToken action
    const sentMessages = (window as any).__sentMessages || [];
    expect(sentMessages).toHaveLength(1);
    expect(sentMessages[0]).toEqual({
      action: 'placeToken',
      args: {
        target: '/zones/board/cells/0/0',
        entity: 'piece_p1'
      }
    });
    
    // Now simulate the state after placement phase is complete and we're in movement phase
    const movementState: LobbyState = {
      ...initialState,
      game: {
        ...initialState.game!,
        zones: {
          board: {
            cells: [
              [{ entity: 'piece_p1' }, { entity: 'piece_p1' }, null],
              [null, { entity: 'piece_p2' }, { entity: 'piece_p1' }], 
              [{ entity: 'piece_p2' }, null, { entity: 'piece_p2' }]
            ]
          }
        },
        phases: {
          game: { current: 'movement', count: 1, actionsProcessed: 6 }
        },
        currentPlayer: 'p1'
      },
      ui: {
        ...initialState.ui!,
        actionMap: {
          p1: {
            '/zones/board/cells/0/0': { action: 'selectPiece', args: { location: '/zones/board/cells/0/0', player: 'p1', target: '/zones/board/cells/0/0' }, direction: 'Select one of your pieces to move' },
            '/zones/board/cells/0/1': { action: 'selectPiece', args: { location: '/zones/board/cells/0/1', player: 'p1', target: '/zones/board/cells/0/1' }, direction: 'Select one of your pieces to move' },
            '/zones/board/cells/1/2': { action: 'selectPiece', args: { location: '/zones/board/cells/1/2', player: 'p1', target: '/zones/board/cells/1/2' }, direction: 'Select one of your pieces to move' }
          },
          p2: {}
        }
      }
    };
    
    // Re-render with movement phase state
    vi.doMock('../../ws/useLobbyWebSocket', () => ({
      useLobbyWebSocket: () => ({
        lobbyState: movementState,
        sendMessage: vi.fn((msg) => {
          const message = JSON.parse(msg);
          (window as any).__sentMessages = (window as any).__sentMessages || [];
          (window as any).__sentMessages.push(message);
          return true;
        }),
        connected: true,
        connectionState: 'connected',
        joinLobby: vi.fn(),
        leaveLobby: vi.fn(),
        startGame: vi.fn(),
        disconnect: vi.fn()
      })
    }));
    
    // Clear previous messages
    (window as any).__sentMessages = [];
    
    await act(async () => {
      render(
        <TestWrapper>
          <GameView lobbyId="test123" />
        </TestWrapper>
      );
    });
    
    // Test 2: Click on a piece to select it in movement phase
    const piece00 = screen.getByTestId('cell-0-0');
    
    await act(async () => {
      fireEvent.click(piece00);
    });
    
    // Verify client sent selectPiece action
    const movementMessages = (window as any).__sentMessages || [];
    expect(movementMessages).toHaveLength(1);
    expect(movementMessages[0]).toEqual({
      action: 'selectPiece',
      args: {
        location: '/zones/board/cells/0/0',
        player: 'p1',
        target: '/zones/board/cells/0/0'
      }
    });
    
    // Simulate state after piece selection (with move actions available)
    const selectedState: LobbyState = {
      ...movementState,
      game: {
        ...movementState.game!,
        selection: {
          p1: {
            location: '/zones/board/cells/0/0',
            entity: { entity: 'piece_p1' }
          }
        }
      },
      ui: {
        ...movementState.ui!,
        actionMap: {
          p1: {
            '/zones/board/cells/0/0': { action: 'selectPiece', args: { location: '/zones/board/cells/0/0', player: 'p1', target: '/zones/board/cells/0/0' }, direction: 'Select one of your pieces to move' },
            '/zones/board/cells/0/1': { action: 'selectPiece', args: { location: '/zones/board/cells/0/1', player: 'p1', target: '/zones/board/cells/0/1' }, direction: 'Select one of your pieces to move' },
            '/zones/board/cells/1/2': { action: 'selectPiece', args: { location: '/zones/board/cells/1/2', player: 'p1', target: '/zones/board/cells/1/2' }, direction: 'Select one of your pieces to move' },
            '/zones/board/cells/0/2': { action: 'moveSelectedPiece', args: { target: '/zones/board/cells/0/2', player: 'p1' }, direction: 'Move your selected piece to any empty location' },
            '/zones/board/cells/1/0': { action: 'moveSelectedPiece', args: { target: '/zones/board/cells/1/0', player: 'p1' }, direction: 'Move your selected piece to any empty location' },
            '/zones/board/cells/2/1': { action: 'moveSelectedPiece', args: { target: '/zones/board/cells/2/1', player: 'p1' }, direction: 'Move your selected piece to any empty location' },
            '_global': { action: 'clearSelection', args: { player: 'p1' }, direction: 'Cancel piece selection' }
          },
          p2: {}
        }
      }
    };
    
    // Re-render with selected state
    vi.doMock('../../ws/useLobbyWebSocket', () => ({
      useLobbyWebSocket: () => ({
        lobbyState: selectedState,
        sendMessage: vi.fn((msg) => {
          const message = JSON.parse(msg);
          (window as any).__sentMessages = (window as any).__sentMessages || [];
          (window as any).__sentMessages.push(message);
          return true;
        }),
        connected: true,
        connectionState: 'connected',
        joinLobby: vi.fn(),
        leaveLobby: vi.fn(),
        startGame: vi.fn(),
        disconnect: vi.fn()
      })
    }));
    
    // Clear previous messages
    (window as any).__sentMessages = [];
    
    await act(async () => {
      render(
        <TestWrapper>
          <GameView lobbyId="test123" />
        </TestWrapper>
      );
    });
    
    // Test 3: Click on empty cell to move piece
    const emptyCell02 = screen.getByTestId('cell-0-2');
    
    await act(async () => {
      fireEvent.click(emptyCell02);
    });
    
    // Verify client sent moveSelectedPiece action
    const moveMessages = (window as any).__sentMessages || [];
    expect(moveMessages).toHaveLength(1);
    expect(moveMessages[0]).toEqual({
      action: 'moveSelectedPiece',
      args: {
        target: '/zones/board/cells/0/2',
        player: 'p1'
      }
    });
  });
  
  it.skip('should prevent clicks when not your turn', async () => {
    const notYourTurnState: LobbyState = {
      you: 'p1',
      started: true,
      game: {
        currentPlayer: 'p2', // Not your turn!
        zones: {
          board: {
            cells: [
              [null, null, null],
              [null, null, null], 
              [null, null, null]
            ]
          }
        },
        phases: {
          game: { current: 'placement', count: 0, actionsProcessed: 0 }
        }
      },
      ui: {
        actionMap: {
          p1: {}, // No actions available
          p2: {
            '/zones/board/cells/0/0': { action: 'placeToken', args: { target: '/zones/board/cells/0/0', entity: 'piece_p2' }, direction: 'Place a piece' }
          }
        },
        players: ['Alice', 'Bob'],
        entities: [
          { id: 'piece_p1', ui: { glyph: 'X' } },
          { id: 'piece_p2', ui: { glyph: 'O' } }
        ]
      }
    };
    
    vi.doMock('../../ws/useLobbyWebSocket', () => ({
      useLobbyWebSocket: () => ({
        lobbyState: notYourTurnState,
        sendMessage: vi.fn((msg) => {
          const message = JSON.parse(msg);
          (window as any).__sentMessages = (window as any).__sentMessages || [];
          (window as any).__sentMessages.push(message);
          return true;
        }),
        connected: true,
        connectionState: 'connected',
        joinLobby: vi.fn(),
        leaveLobby: vi.fn(),
        startGame: vi.fn(),
        disconnect: vi.fn()
      })
    }));
    
    (window as any).__sentMessages = [];
    
    const TestWrapper = ({ children }: { children: React.ReactNode }) => (
      <BrowserRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            {children}
          </PlayerProvider>
        </QueryClientProvider>
      </BrowserRouter>
    );
    
    await act(async () => {
      render(
        <TestWrapper>
          <GameView lobbyId="test123" />
        </TestWrapper>
      );
    });
    
    // Try to click on a cell when it's not your turn
    const cell00 = screen.getByTestId('cell-0-0');
    
    await act(async () => {
      fireEvent.click(cell00);
    });
    
    // Should not send any messages
    const sentMessages = (window as any).__sentMessages || [];
    expect(sentMessages).toHaveLength(0);
  });
});
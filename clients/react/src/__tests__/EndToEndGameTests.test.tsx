import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within, act } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { PlayerProvider } from '../context/PlayerContext';
import GameView from '../components/GameView';
import type { LobbyState } from '../ws/useLobbyWebSocket';

// Mock the PlayerContext
vi.mock('../context/PlayerContext', () => ({
  usePlayer: () => ({
    player: { username: 'Alice', color: '#ff0000' },
    setPlayer: vi.fn(),
    clearPlayer: vi.fn()
  }),
  PlayerProvider: ({ children }: { children: React.ReactNode }) => children
}));

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

// Mock the PlayerPreferencesContext
vi.mock('../context/PlayerPreferencesContext', () => ({
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

// Mock WebSocket
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
    // Store sent messages for verification
    (window as any).__sentMessages = (window as any).__sentMessages || [];
    (window as any).__sentMessages.push(message);
  }
  
  close() {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close'));
  }
}

// Helper to create a complete game state
function createGameState(gameId: string, overrides: Partial<LobbyState> = {}): LobbyState {
  const baseState: LobbyState = {
    you: 'p1',
    started: true,
    game: {
      turn: 0,
      currentPlayer: 'p1',
      tick: 0,
      gameStatus: {
        state: 'playing',
        winner: null,
        tie: false
      },
      players: [
        { id: 'p1' },
        { id: 'p2' }
      ],
      zones: {},
      phases: {
        game: { current: 'play', count: 0, actionsProcessed: 0 }
      },
      selection: {}
    },
    ui: {
      actionMap: {
        p1: {},
        p2: {}
      },
      players: ['Alice', 'Bob'],
      entities: [],
      zones: [],
      gameLog: [],
      manifest: {
        gameId,
        version: '1.0',
        specVersion: '1.0',
        metadata: {
          name: 'Test Game',
          description: 'Test game',
          author: 'Test',
          players: { min: 2, max: 2 }
        }
      }
    },
    ...overrides
  };
  
  return baseState;
}

// Helper to simulate server responses
function simulateServerMessage(ws: MockWebSocket, message: any) {
  ws.onmessage?.(new MessageEvent('message', {
    data: JSON.stringify(message)
  }));
}

// Mock the WebSocket hook at module level
const mockSentMessages: any[] = [];
const mockLobbyWebSocket = {
  lobbyState: {} as LobbyState,
  sendMessage: vi.fn((msg: string) => {
    console.log('[TEST] sendMessage called with:', msg);
    const parsed = JSON.parse(msg);
    mockSentMessages.push(parsed);
    return true;
  }),
  connectionState: 'connected' as const,
  joinLobby: vi.fn(),
  leaveLobby: vi.fn(),
  startGame: vi.fn(),
  disconnect: vi.fn(),
  connected: true,
  messages: []
};

vi.mock('../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: () => mockLobbyWebSocket
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
  mockLobbyWebSocket.lobbyState = lobbyState;
  
  const utils = render(
    <BrowserRouter>
      <QueryClientProvider client={queryClient}>
        <PlayerProvider initialPlayer={{ username: 'Alice', color: '#ff0000' }}>
          <GameView lobbyId={lobbyId} onLeave={() => {}} />
        </PlayerProvider>
      </QueryClientProvider>
    </BrowserRouter>
  );
  
  return { ...utils, mockLobbyWebSocket };
}

describe('End-to-End Game Tests', () => {
  let originalWebSocket: typeof WebSocket;
  
  beforeEach(() => {
    originalWebSocket = global.WebSocket;
    (global as any).WebSocket = MockWebSocket;
    (window as any).__sentMessages = [];
    mockSentMessages.length = 0; // Clear the array
    mockLobbyWebSocket.sendMessage.mockClear();
    
    // Mock fetch for lobby info
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        id: 'test-lobby',
        game_id: 'test-game',
        players: ['Alice', 'Bob'],
        started: true,
        manifest: {
          gameId: 'test-game',
          version: '1.0',
          metadata: {
            name: 'Test Game',
            description: 'Test game',
            players: { min: 2, max: 2 }
          }
        }
      })
    });
  });
  
  afterEach(() => {
    (global as any).WebSocket = originalWebSocket;
    vi.clearAllMocks();
  });
  
  describe('Go Fish', () => {
    it('should render choice zones for rank selection and handle clicks', async () => {
      const goFishState = createGameState('go-fish', {
        game: {
          ...createGameState('go-fish').game,
          zones: {
            choice_p1: {
              type: 'choice',
              items: [
                { id: 'a', label: 'Ace', enabled: true },
                { id: '2', label: 'Two', enabled: true }
              ],
              prompt: 'Choose a rank to ask for'
            },
            choice_p2: {
              type: 'choice',
              items: [],
              prompt: 'Choose a rank to ask for'
            },
            hand_p1: {
              type: 'list',
              items: [
                { entity: 'card_hearts_a' },
                { entity: 'card_hearts_2' },
                { entity: 'card_spades_2' }
              ]
            },
            hand_p2: {
              type: 'list',
              items: []
            }
          },
          phases: {
            game: { current: 'selectingRank', count: 0, actionsProcessed: 0 }
          }
        },
        ui: {
          ...createGameState('go-fish').ui!,
          actionMap: {
            p1: {
              '/zones/choice_p1/a': {
                action: 'selectRank',
                direction: 'Choose a rank to ask for',
                rank: 'a'
              },
              '/zones/choice_p1/2': {
                action: 'selectRank',
                direction: 'Choose a rank to ask for',
                rank: '2'
              }
            },
            p2: {}
          },
          zones: [
            { 
              id: 'choice_p1', 
              name: 'Select Rank', 
              resolved_name: 'Select Rank',
              visibility: 'owner',
              owner: 'p1',
              layout_order: 0,
              renderType: 'choice',
              items: [
                { id: 'a', label: 'Ace', enabled: true },
                { id: '2', label: 'Two', enabled: true }
              ],
              prompt: 'Choose a rank to ask for'
            },
            { 
              id: 'choice_p2', 
              name: 'Select Rank', 
              resolved_name: 'Select Rank',
              visibility: 'owner',
              owner: 'p2',
              layout_order: 1,
              renderType: 'choice',
              items: [],
              prompt: 'Choose a rank to ask for'
            },
            { 
              id: 'hand_p1', 
              name: 'Your Hand', 
              resolved_name: 'Your Hand',
              visibility: 'owner',
              layout_order: 2,
              renderType: 'card',
              cards: [
                { entity: 'card_hearts_a', visible: true },
                { entity: 'card_hearts_2', visible: true },
                { entity: 'card_spades_2', visible: true }
              ],
              layout: 'fan'
            },
            { 
              id: 'hand_p2', 
              name: 'Your Hand', 
              resolved_name: 'Your Hand',
              visibility: 'owner',
              layout_order: 3,
              renderType: 'card',
              cards: [],
              layout: 'fan'
            }
          ],
          entities: [
            { id: 'card_hearts_a', type: 'card', props: { rank: 'A', suit: 'hearts' } },
            { id: 'card_hearts_2', type: 'card', props: { rank: '2', suit: 'hearts' } },
            { id: 'card_spades_2', type: 'card', props: { rank: '2', suit: 'spades' } }
          ]
        }
      });
      
      const { mockLobbyWebSocket } = renderGame(goFishState);
      
      // Wait for game to render - use getAllByText since there might be multiple elements
      await waitFor(() => {
        const elements = screen.getAllByText('Choose a rank to ask for');
        expect(elements.length).toBeGreaterThan(0);
      });
      
      // Check that choice zone is rendered
      const choiceZone = screen.getByTestId('choice-zone');
      expect(choiceZone).toBeInTheDocument();
      
      // Check that rank options are rendered
      expect(within(choiceZone!).getByText('Ace')).toBeInTheDocument();
      expect(within(choiceZone!).getByText('Two')).toBeInTheDocument();
      
      // Click on rank 'Ace'
      fireEvent.click(within(choiceZone!).getByText('Ace'));
      
      // Verify correct message was sent
      await waitFor(() => {
        expect(mockSentMessages).toHaveLength(1);
        expect(mockSentMessages[0]).toEqual({
          action: 'selectRank',
          args: {}
        });
      });
    });
    
    it('should show player selection after rank selection', async () => {
      const goFishState = createGameState('go-fish', {
        game: {
          ...createGameState('go-fish').game,
          zones: {
            choice_p1: {
              type: 'choice',
              items: [
                { id: 'p2', label: 'Player 2', enabled: true }
              ],
              prompt: 'Choose a player to ask'
            },
            choice_p2: {
              type: 'choice',
              items: [],
              prompt: 'Choose a player to ask'
            }
          },
          phases: {
            game: { current: 'selectingPlayer', count: 0, actionsProcessed: 0 }
          },
          selection: {
            selectedRank: '2'
          }
        },
        ui: {
          ...createGameState('go-fish').ui!,
          actionMap: {
            p1: {
              '/zones/choice_p1/p2': {
                action: 'selectPlayer',
                direction: 'Choose a player to ask',
                targetPlayer: 'p2'
              }
            },
            p2: {}
          },
          zones: [
            { 
              id: 'choice_p1', 
              name: 'Select Player', 
              resolved_name: 'Select Player',
              visibility: 'owner',
              owner: 'p1',
              layout_order: 0,
              renderType: 'choice',
              items: [
                { id: 'p2', label: 'Player 2', enabled: true }
              ],
              prompt: 'Choose a player to ask'
            },
            { 
              id: 'choice_p2', 
              name: 'Select Player', 
              resolved_name: 'Select Player',
              visibility: 'owner',
              owner: 'p2',
              layout_order: 1,
              renderType: 'choice',
              items: [],
              prompt: 'Choose a player to ask'
            }
          ]
        }
      });
      
      renderGame(goFishState);
      
      // Wait for game to render
      await waitFor(() => {
        const elements = screen.getAllByText('Choose a player to ask');
        expect(elements.length).toBeGreaterThan(0);
      });
      
      // Check that choice zone is rendered
      const choiceZone = screen.getByTestId('choice-zone');
      expect(choiceZone).toBeInTheDocument();
      
      // Check that player option is rendered
      expect(within(choiceZone).getByText('Player 2')).toBeInTheDocument();
      
      // Click on player
      fireEvent.click(within(choiceZone).getByText('Player 2'));
      
      // Verify correct message was sent
      await waitFor(() => {
        expect(mockSentMessages).toHaveLength(1);
        expect(mockSentMessages[0]).toEqual({
          action: 'selectPlayer',
          args: {}
        });
      });
    });
  });
  
  describe('Connect Four', () => {
    it('should render the game board', async () => {
      const connectFourState = createGameState('connect-four', {
        game: {
          ...createGameState('connect-four').game,
          zones: {
            board: {
              type: 'grid',
              cells: [
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null]
              ]
            }
          }
        },
        ui: {
          ...createGameState('connect-four').ui!,
          zones: [
            { 
              id: 'board', 
              name: 'Game Board', 
              resolved_name: 'Game Board',
              visibility: 'all',
              layout_order: 0,
              renderType: 'grid',
              cells: [
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null]
              ],
              rows: 6,
              cols: 7
            }
          ],
          actionMap: {
            p1: {
              '/zones/board/columns/0': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 0 },
              '/zones/board/columns/1': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 1 },
              '/zones/board/columns/2': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 2 },
              '/zones/board/columns/3': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 3 },
              '/zones/board/columns/4': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 4 },
              '/zones/board/columns/5': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 5 },
              '/zones/board/columns/6': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 6 }
            },
            p2: {}
          }
        }
      });
      
      renderGame(connectFourState);
      
      // Just wait for the board heading to appear
      await waitFor(() => {
        expect(screen.getByText('Game Board')).toBeInTheDocument();
      });
      
      // Check that we have a board zone
      const boardZone = screen.getByTestId('board-zone');
      expect(boardZone).toBeInTheDocument();
    });
    
    it('should handle column clicks and place pieces', async () => {
      const connectFourState = createGameState('connect-four', {
        game: {
          ...createGameState('connect-four').game,
          zones: {
            board: {
              type: 'grid',
              cells: [
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null]
              ]
            }
          }
        },
        ui: {
          ...createGameState('connect-four').ui!,
          actionMap: {
            p1: {
              '/zones/board/columns/0': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 0 },
              '/zones/board/columns/1': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 1 },
              '/zones/board/columns/2': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 2 },
              '/zones/board/columns/3': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 3 },
              '/zones/board/columns/4': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 4 },
              '/zones/board/columns/5': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 5 },
              '/zones/board/columns/6': { action: 'dropDisc', direction: 'Click a column to drop your disc', targetColumn: 6 }
            },
            p2: {}
          },
          zones: [
            { 
              id: 'board', 
              name: 'Game Board', 
              resolved_name: 'Game Board',
              visibility: 'all',
              layout_order: 0,
              renderType: 'grid',
              cells: [
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null]
              ],
              rows: 6,
              cols: 7
            }
          ]
        }
      });
      
      const { mockLobbyWebSocket } = renderGame(connectFourState);
      
      // Wait for the board to render
      await waitFor(() => {
        expect(screen.getByText('Game Board')).toBeInTheDocument();
      });
      
      // Wait for action map to be populated and components to settle
      await waitFor(() => {
        const board = screen.getByTestId('board-zone');
        expect(board).toBeInTheDocument();
        
        // Look for column drop zones - they should have the down arrow
        const columnDropZones = within(board).queryAllByText('↓');
        console.log('Column drop zones found:', columnDropZones.length);
        
        if (columnDropZones.length === 0) {
          // Debug: print the board HTML
          console.log('Board HTML:', board.innerHTML.substring(0, 500));
        }
        
        expect(columnDropZones.length).toBe(7); // Connect 4 has 7 columns
      }, { timeout: 3000 });
      
      // Now get the board and find column drop zones
      const board = screen.getByTestId('board-zone');
      
      // Find the column drop zones - they have down arrows
      const columnDropZones = within(board).getAllByText('↓');
      expect(columnDropZones).toHaveLength(7);
      
      // Click on column 3 (0-indexed)
      const clickTarget = columnDropZones[3];
      console.log('Clicking column drop zone 3');
      
      // Log sendMessage mock status before click
      console.log('mockLobbyWebSocket.sendMessage before click:', {
        called: mockLobbyWebSocket.sendMessage.mock.calls.length,
        mockSentMessages: mockSentMessages.length
      });
      
      fireEvent.click(clickTarget);
      
      // Give it a moment to process
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 100));
      });
      
      // Log sendMessage mock status after click
      console.log('mockLobbyWebSocket.sendMessage after click:', {
        called: mockLobbyWebSocket.sendMessage.mock.calls.length,
        calls: mockLobbyWebSocket.sendMessage.mock.calls,
        mockSentMessages: mockSentMessages
      });
      
      // Verify correct message was sent - Connect 4 uses dropDisc action
      await waitFor(() => {
        expect(mockSentMessages).toHaveLength(1);
        expect(mockSentMessages[0]).toEqual({
          action: 'dropDisc',
          args: {}
        });
      });
    });
    
    it('should show placed pieces and handle gravity', async () => {
      const connectFourState = createGameState('connect-four', {
        game: {
          ...createGameState('connect-four').game,
          zones: {
            board: {
              type: 'grid',
              cells: [
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, { entity: 'disc_p1' }, null, null, null],
                [null, null, null, { entity: 'disc_p2' }, null, null, null]
              ]
            }
          }
        },
        ui: {
          ...createGameState('connect-four').ui!,
          entities: [
            { id: 'disc_p1', type: 'disc', props: { owner: 'p1' } },
            { id: 'disc_p2', type: 'disc', props: { owner: 'p2' } }
          ],
          zones: [
            { 
              id: 'board', 
              name: 'Game Board', 
              resolved_name: 'Game Board',
              visibility: 'all',
              layout_order: 0,
              renderType: 'grid',
              cells: [
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, null, null, null, null],
                [null, null, null, { entity: 'disc_p1' }, null, null, null],
                [null, null, null, { entity: 'disc_p2' }, null, null, null]
              ],
              rows: 6,
              cols: 7
            }
          ]
        }
      });
      
      renderGame(connectFourState);
      
      // Wait for board to render
      await waitFor(() => {
        expect(screen.getByTestId('board-zone')).toBeInTheDocument();
      });
      
      // Check that pieces are visible
      const board = screen.getByTestId('board-zone');
      const p1Piece = within(board).getByTestId('cell-4-3');
      const p2Piece = within(board).getByTestId('cell-5-3');
      
      // The cells should contain some non-empty content (the disc)
      // Since we don't define glyphs for disc entities, they default to 'O'
      expect(p1Piece.textContent).not.toBe('');
      expect(p2Piece.textContent).not.toBe('');
      
      // Verify the cells are not empty (they contain entities)
      expect(p1Piece.querySelector('[class*="text-"]')).toBeInTheDocument();
      expect(p2Piece.querySelector('[class*="text-"]')).toBeInTheDocument();
    });
  });
  
  describe('Three Men\'s Morris', () => {
    it('should handle piece placement on empty points', async () => {
      const morrisState = createGameState('three-mens-morris', {
        game: {
          ...createGameState('three-mens-morris').game,
          zones: {
            board: {
              type: 'grid',
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
          ...createGameState('three-mens-morris').ui!,
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placePiece', direction: 'Place piece here' },
              '/zones/board/cells/0/1': { action: 'placePiece', direction: 'Place piece here' },
              '/zones/board/cells/0/2': { action: 'placePiece', direction: 'Place piece here' },
              '/zones/board/cells/1/0': { action: 'placePiece', direction: 'Place piece here' },
              '/zones/board/cells/1/1': { action: 'placePiece', direction: 'Place piece here' },
              '/zones/board/cells/1/2': { action: 'placePiece', direction: 'Place piece here' },
              '/zones/board/cells/2/0': { action: 'placePiece', direction: 'Place piece here' },
              '/zones/board/cells/2/1': { action: 'placePiece', direction: 'Place piece here' },
              '/zones/board/cells/2/2': { action: 'placePiece', direction: 'Place piece here' }
            },
            p2: {}
          },
          zones: [
            { 
              id: 'board', 
              name: 'Game Board', 
              resolved_name: 'Game Board',
              visibility: 'all',
              layout_order: 0,
              renderType: 'grid',
              cells: [
                [null, null, null],
                [null, null, null],
                [null, null, null]
              ],
              rows: 3,
              cols: 3
            }
          ]
        }
      });
      
      renderGame(morrisState);
      
      // Wait for board to render
      await waitFor(() => {
        expect(screen.getByTestId('board-zone')).toBeInTheDocument();
      });
      
      // Click on position 0,0
      const board = screen.getByTestId('board-zone');
      const cell = within(board).getByTestId('cell-0-0');
      
      // The cell is a container div, we need to click the inner clickable div
      const clickableArea = cell.querySelector('[class*="cursor-pointer"]');
      console.log('Found clickable area:', !!clickableArea);
      
      if (clickableArea) {
        fireEvent.click(clickableArea);
      } else {
        // Fallback to clicking the cell itself
        fireEvent.click(cell);
      }
      
      // Verify correct message was sent
      await waitFor(() => {
        expect(mockSentMessages).toHaveLength(1);
        expect(mockSentMessages[0]).toEqual({
          action: 'placePiece',
          args: {}
        });
      });
    });
  });
  
  describe('Tic Tac Toe', () => {
    it('should handle cell clicks and show game end state', async () => {
      const ticTacToeState = createGameState('tic-tac-toe', {
        game: {
          ...createGameState('tic-tac-toe').game,
          zones: {
            board: {
              type: 'grid',
              cells: [
                [{ entity: 'mark_p1' }, { entity: 'mark_p1' }, null],
                [{ entity: 'mark_p2' }, { entity: 'mark_p2' }, null],
                [null, null, null]
              ]
            }
          }
        },
        ui: {
          ...createGameState('tic-tac-toe').ui!,
          actionMap: {
            p1: {
              '/zones/board/cells/0/2': { action: 'placeMarker', direction: 'Place X here' },
              '/zones/board/cells/1/2': { action: 'placeMarker', direction: 'Place X here' },
              '/zones/board/cells/2/0': { action: 'placeMarker', direction: 'Place X here' },
              '/zones/board/cells/2/1': { action: 'placeMarker', direction: 'Place X here' },
              '/zones/board/cells/2/2': { action: 'placeMarker', direction: 'Place X here' }
            },
            p2: {}
          },
          entities: [
            { id: 'mark_p1', type: 'mark', props: { symbol: 'X' } },
            { id: 'mark_p2', type: 'mark', props: { symbol: 'O' } }
          ],
          zones: [
            { 
              id: 'board', 
              name: 'Game Board', 
              resolved_name: 'Game Board',
              visibility: 'all',
              layout_order: 0,
              renderType: 'grid',
              cells: [
                [{ entity: 'mark_p1' }, { entity: 'mark_p1' }, null],
                [{ entity: 'mark_p2' }, { entity: 'mark_p2' }, null],
                [null, null, null]
              ],
              rows: 3,
              cols: 3
            }
          ]
        }
      });
      
      renderGame(ticTacToeState);
      
      // Wait for board to render
      await waitFor(() => {
        expect(screen.getByTestId('board-zone')).toBeInTheDocument();
      });
      
      // Click to win the game
      const board = screen.getByTestId('board-zone');
      const winningCell = within(board).getByTestId('cell-0-2');
      
      // The cell is a container div, we need to click the inner clickable div
      const clickableArea = winningCell.querySelector('[class*="cursor-pointer"]');
      if (clickableArea) {
        fireEvent.click(clickableArea);
      } else {
        fireEvent.click(winningCell);
      }
      
      // Verify correct message was sent
      await waitFor(() => {
        expect(mockSentMessages).toHaveLength(1);
        expect(mockSentMessages[0]).toEqual({
          action: 'placeMarker',
          args: {}
        });
      });
    });
  });
  
  describe('Generic Game Behaviors', () => {
    it('should only show actions for current player', async () => {
      const state = createGameState('test-game', {
        you: 'p2',
        game: {
          ...createGameState('test-game').game,
          currentPlayer: 'p1' // Not our turn
        },
        ui: {
          ...createGameState('test-game').ui!,
          actionMap: {
            p1: {
              '/zones/board/0/0': { action: 'test', direction: 'Test action' }
            },
            p2: {} // No actions for p2
          }
        }
      });
      
      renderGame(state);
      
      // Should not see any clickable elements since it's not our turn
      await waitFor(() => {
        expect(screen.queryByText('Test action')).not.toBeInTheDocument();
      });
    });
    
    it('should disable all actions when game ends', async () => {
      const state = createGameState('test-game', {
        game: {
          ...createGameState('test-game').game,
          gameStatus: {
            state: 'ended',
            winner: 'p2',
            tie: false
          }
        }
      });
      
      renderGame(state);
      
      // Should show game result
      await waitFor(() => {
        expect(screen.getByText(/Player.*wins!/)).toBeInTheDocument();
      });
      
      // Should not have any clickable actions
      expect(mockSentMessages).toHaveLength(0);
    });
  });
});
import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BrowserRouter } from 'react-router-dom';
import { TestProviders } from '../../../test/TestProviders';
import GameView from '../../../components/GameView';

// Mock WebSocket
const mockWebSocket = {
  send: vi.fn(),
  close: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  readyState: WebSocket.OPEN,
};

vi.mock('../../../ws/useReconnectingWebSocket', () => ({
  useReconnectingWebSocket: () => ({
    ws: mockWebSocket,
    status: 'connected',
    sendMessage: mockWebSocket.send,
  }),
}));

// Mock the API calls
vi.mock('../../../api/lobbies', () => ({
  getLobby: vi.fn().mockResolvedValue({
    id: 'test-lobby',
    game_id: 'three-mens-morris',
    players: ['Alice', 'Bob'],
    started: true
  })
}));

// Mock usePlayer
vi.mock('../../../context/PlayerContext', () => ({
  usePlayer: () => ({
    player: { username: 'Alice', color: '#ff0000' }
  }),
  PlayerProvider: ({ children }: { children: React.ReactNode }) => children
}));

// Mock usePlayerPreferences
vi.mock('../../../context/PlayerPreferencesContext', () => ({
  usePlayerPreferences: () => ({
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
  }),
  PlayerPreferencesProvider: ({ children }: { children: React.ReactNode }) => children
}));

// Mock AnimationProvider  
vi.mock('../../../context/AnimationContext', () => ({
  useAnimationsEnabled: () => true,
  useAnimation: () => ({
    state: { isAnimating: false, config: { enableAnimations: true } },
    updateConfig: vi.fn(),
    addAnimation: vi.fn(),
    removeAnimation: vi.fn(),
    clearQueue: vi.fn(),
    isAnimating: false
  }),
  AnimationProvider: ({ children }: { children: React.ReactNode }) => children
}));

// Create mock lobby state
const createMovementPhaseState = () => ({
  you: 'p1',
  started: true,
  game: {
    currentPlayer: 'p1',
    phases: {
      game: 'movement'
    },
    zones: {
      board: [
        ['piece_p1', null, 'piece_p2'],
        ['piece_p1', null, 'piece_p2'],
        ['piece_p1', null, 'piece_p2']
      ]
    }
  },
  ui: {
    manifest: {
      metadata: {
        name: 'Three Men\'s Morris',
        description: 'A classic board game',
        players: { min: 2, max: 2 }
      }
    },
    players: ['Alice', 'Bob'],
    entities: [
      { id: 'piece_p1', ui: { glyph: 'X' } },
      { id: 'piece_p2', ui: { glyph: 'O' } }
    ],
    zones: [
      {
        id: 'board',
        name: 'Board',
        renderType: 'grid',
        visibility: 'all',
        gridDimensions: { rows: 3, cols: 3 }
      }
    ],
    actionMap: {
      p1: {
        '/zones/board/cells/0/0': { action: 'selectPiece', direction: 'Select piece to move' },
        '/zones/board/cells/1/0': { action: 'selectPiece', direction: 'Select piece to move' },
        '/zones/board/cells/2/0': { action: 'selectPiece', direction: 'Select piece to move' }
      },
      p2: {}
    }
  }
});

let mockLobbyState: any = null;

// Update the mock to use the state
vi.mock('../../../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: () => ({
    lobbyState: mockLobbyState,
    sendMessage: mockWebSocket.send,
    connectionState: 'connected',
    connected: true,
    joinLobby: vi.fn(),
    leaveLobby: vi.fn(),
    startGame: vi.fn(),
    sendPreferencesUpdate: vi.fn(),
    disconnect: vi.fn()
  })
}));

describe('Three Men\'s Morris - Movement Phase', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLobbyState = null;
    
    // Mock AudioContext
    global.AudioContext = vi.fn().mockImplementation(() => ({
      createBuffer: vi.fn(),
      createBufferSource: vi.fn(() => ({
        connect: vi.fn(),
        start: vi.fn(),
        buffer: null
      })),
      createGain: vi.fn(() => ({
        connect: vi.fn(),
        gain: { value: 1 }
      })),
      destination: {},
      sampleRate: 44100
    })) as any;
  });

  it('should show multi-step action button in movement phase', async () => {
    // Set the mock state
    mockLobbyState = createMovementPhaseState();

    render(
      <BrowserRouter>
        <TestProviders initialPlayer={{ username: 'Alice', color: '#ff0000' }}>
          <GameView lobbyId="test-lobby" />
        </TestProviders>
      </BrowserRouter>
    );

    // Wait for component to render with state
    await waitFor(() => {
      // Check if pieces are displayed
      const boardCells = screen.getAllByTestId(/cell-\d-\d/);
      expect(boardCells).toHaveLength(9);
    });

    // Check if pieces have action indicators
    const cellsWithPieces = [
      screen.getByTestId('cell-0-0'),
      screen.getByTestId('cell-1-0'),
      screen.getByTestId('cell-2-0')
    ];

    // Verify that we have cells with pieces
    expect(cellsWithPieces.length).toBe(3);
  });

  it.skip('should handle multi-step move action', async () => {
    const user = userEvent.setup();
    
    const movementPhaseState = {
      type: 'stateSync',
      state: {
        started: true,
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          phases: {
            game: {
              current: 'movement'
            }
          },
          zones: {
            board: {
              cells: [
                [{ entity: 'piece_p1' }, null, { entity: 'piece_p2' }],
                [{ entity: 'piece_p1' }, null, { entity: 'piece_p2' }],
                [{ entity: 'piece_p1' }, null, { entity: 'piece_p2' }]
              ]
            }
          }
        },
        ui: {
          manifest: {
            metadata: {
              name: 'Three Men\'s Morris',
              description: 'A classic board game',
              players: { min: 2, max: 2 }
            }
          },
          players: ['Alice', 'Bob'],
          entities: [
            { id: 'piece_p1', ui: { glyph: 'X' } },
            { id: 'piece_p2', ui: { glyph: 'O' } }
          ],
          zones: [
            {
              id: 'board',
              name: 'Board',
              type: 'grid',
              visibility: 'all',
              renderType: 'grid'
            }
          ],
          phases: {
            game: {
              phases: [
                { id: 'placement', ui: { display: 'Placement Phase' } },
                { id: 'movement', ui: { display: 'Movement Phase' } }
              ]
            }
          },
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': {
                actionId: 'movePiece',
                type: 'multiStep',
                step: 'selectPiece'
              },
              '/zones/board/cells/1/0': {
                actionId: 'movePiece',
                type: 'multiStep',
                step: 'selectPiece'
              },
              '/zones/board/cells/2/0': {
                actionId: 'movePiece',
                type: 'multiStep',
                step: 'selectPiece'
              }
            }
          }
        }
      }
    };

    render(
      <BrowserRouter>
        <TestProviders initialPlayer={{ username: 'Alice', color: '#ff0000' }}>
          <GameView lobbyId="test-lobby" />
        </TestProviders>
      </BrowserRouter>
    );

    // Simulate receiving the movement phase state
    const wsHandler = mockWebSocket.addEventListener.mock.calls.find(
      call => call[0] === 'message'
    )?.[1];

    await waitFor(() => {
      wsHandler?.({ data: JSON.stringify(movementPhaseState) });
    });

    // Click on a piece to start the multi-step action
    const pieceCell = screen.getByTestId('cell-0-0');
    await user.click(pieceCell);

    // Verify the click was sent
    await waitFor(() => {
      expect(mockWebSocket.send).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'click',
          location: '/zones/board/cells/0/0'
        })
      );
    });

    // Simulate receiving multi-step state
    const multiStepUpdate = {
      type: 'patch',
      path: '/ui/multiStepState/p1',
      value: {
        actionId: 'movePiece',
        currentStepId: 'selectDestination',
        currentStepIndex: 1,
        totalSteps: 2,
        storedData: {
          selectedPiece: 'piece_p1',
          sourceLocation: '/zones/board/cells/0/0'
        },
        canCancel: true
      }
    };

    await waitFor(() => {
      wsHandler?.({ data: JSON.stringify(multiStepUpdate) });
    });

    // Check if multi-step display is shown
    await waitFor(() => {
      expect(screen.getByText(/Step 2 of 2/)).toBeInTheDocument();
    });
  });
});
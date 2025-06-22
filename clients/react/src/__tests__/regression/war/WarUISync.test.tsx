import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import GameView from '../../../components/GameView';
import { AnimationProvider } from '../../../context/AnimationContext';
import { PlayerPreferencesProvider } from '../../../context/PlayerPreferencesContext';

// Mock React Router
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => vi.fn()
  };
});

// Mock PlayerContext
vi.mock('../../../context/PlayerContext', () => ({
  PlayerProvider: ({ children }: { children: React.ReactNode }) => children,
  usePlayer: () => ({
    player: { username: 'Alice', color: '#ff0000' },
    setPlayer: vi.fn(),
    clearPlayer: vi.fn()
  })
}));

// Mock AnimationContext
vi.mock('../../../context/AnimationContext', () => ({
  useAnimationsEnabled: vi.fn(() => true),
  useAnimation: vi.fn(() => ({
    state: { isAnimating: false, config: { enableAnimations: true } },
    updateConfig: vi.fn(),
    addAnimation: vi.fn(),
    removeAnimation: vi.fn(),
    clearQueue: vi.fn(),
    isAnimating: false
  })),
  AnimationProvider: ({ children }: { children: React.ReactNode }) => children
}));

// Mock PlayerPreferencesContext
vi.mock('../../../context/PlayerPreferencesContext', () => ({
  usePlayerPreferences: vi.fn(() => ({
    preferences: { 
      username: 'Alice', 
      color: '#ff0000',
      tokenId: 'classic',
      showOpponentTokens: true,
      colorSchemeId: 'default',
      cardStyleId: 'classic'
    },
    updatePreferences: vi.fn()
  })),
  PlayerPreferencesProvider: ({ children }: { children: React.ReactNode }) => children
}));

// Mock WebSocket
const mockSend = vi.fn();
const mockClose = vi.fn();

class MockWebSocket {
  url: string;
  readyState: number = WebSocket.OPEN;
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  constructor(url: string) {
    this.url = url;
    (window as any).mockWebSocket = this;
    setTimeout(() => {
      if (this.onopen) {
        this.onopen(new Event('open'));
      }
    }, 0);
  }

  send = mockSend;
  close = mockClose;
}

global.WebSocket = MockWebSocket as any;

// Mock the WebSocket hook
const mockSendMessage = vi.fn();
const mockLobbyWebSocketReturn = {
  lobbyState: null as any,
  sendMessage: mockSendMessage,
  connectionState: 'connected' as const,
  joinLobby: vi.fn(),
  leaveLobby: vi.fn(),
  startGame: vi.fn(),
  disconnect: vi.fn(),
  connected: true,
  messages: []
};

vi.mock('../../../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: () => mockLobbyWebSocketReturn
}));

describe('War - UI Synchronization Tests', () => {
  const lobbyId = 'test-lobby-123';
  const baseWarState = {
    you: 'p1',
    started: true,
    game: {
      currentPlayer: 'p1',
      turn: 0,
      tick: 0,
      zones: {},
      players: [{ id: 'p1' }, { id: 'p2' }],
      phases: { game: { current: 'ready', count: 0, actionsProcessed: 0 } },
      gameStatus: { state: 'playing', winner: null, tie: false },
      selection: {}
    },
    ui: {
      actionMap: {},
      entities: [],
      zones: [],
      players: ['Alice', 'Bob'],
      gameLog: [],
      manifest: {
        gameId: 'war',
        version: '1.0',
        specVersion: '1.0',
        metadata: {
          name: 'War',
          description: 'Classic card battle game',
          author: 'Bluefelt',
          players: { min: 2, max: 2 }
        }
      }
    }
  };
  
  beforeEach(() => {
    mockSend.mockClear();
    mockClose.mockClear();
    mockSendMessage.mockClear();
    mockLobbyWebSocketReturn.lobbyState = null;
  });

  it('should sync deck counts between players', async () => {
    // Initial state with full decks
    const initialState = {
      ...baseWarState,
      game: {
        ...baseWarState.game,
        zones: {
          deck_p1: {
            type: 'list',
            visibility: 'count',
            items: Array(26).fill({ entity: 'card_back' })
          },
          deck_p2: {
            type: 'list',
            visibility: 'count',
            items: Array(26).fill({ entity: 'card_back' })
          }
        }
      },
      ui: {
        ...baseWarState.ui,
        entities: [{ id: 'card_back', type: 'card', props: { rank: 'back', suit: 'none' } }],
        zones: [
          { id: 'deck_{player}', type: 'list', visibility: 'count', renderType: 'stack' }
        ]
      }
    };
    
    mockLobbyWebSocketReturn.lobbyState = initialState;

    const { container } = render(
      <BrowserRouter>
        <PlayerPreferencesProvider>
          <AnimationProvider>
            <GameView lobbyId={lobbyId} />
          </AnimationProvider>
        </PlayerPreferencesProvider>
      </BrowserRouter>
    );

    const ws = (window as any).mockWebSocket;

    if (ws?.onmessage) {
      ws.onmessage(new MessageEvent('message', { data: JSON.stringify(initialState) }));
    }

    // After a battle, deck counts change
    const battleResult = {
      patches: [
        {
          op: 'remove',
          path: '/game/zones/deck_p1/items/0'
        },
        {
          op: 'remove',
          path: '/game/zones/deck_p2/items/0'
        }
      ]
    };

    if (ws?.onmessage) {
      ws.onmessage(new MessageEvent('message', { data: JSON.stringify(battleResult) }));
    }

    await waitFor(() => {
      const zones = container.querySelectorAll('[data-testid^="zone-"]');
      expect(zones.length).toBeGreaterThan(0);
    });
  });

  it('should handle rapid state updates during war sequence', async () => {
    mockLobbyWebSocketReturn.lobbyState = baseWarState;
    
    render(
      <BrowserRouter>
        <PlayerPreferencesProvider>
          <AnimationProvider>
            <GameView lobbyId={lobbyId} />
          </AnimationProvider>
        </PlayerPreferencesProvider>
      </BrowserRouter>
    );

    const ws = (window as any).mockWebSocket;
    
    // Send initial state
    if (ws?.onmessage) {
      ws.onmessage(new MessageEvent('message', { data: JSON.stringify(baseWarState) }));
    }

    // Simulate rapid war sequence updates
    const warUpdates = [
      { patches: [{ op: 'replace', path: '/game/phases/game', value: { current: 'war', count: 0, actionsProcessed: 0 } }] },
      { patches: [{ op: 'add', path: '/ui/gameLog/-', value: { message: 'WAR!', timestamp: '12:00' } }] }
    ];

    // Send all updates rapidly
    warUpdates.forEach(update => {
      if (ws?.onmessage) {
        ws.onmessage(new MessageEvent('message', { data: JSON.stringify(update) }));
      }
    });

    await waitFor(() => {
      expect(screen.getByText(/WAR!/)).toBeInTheDocument();
    });
  });

  it('should maintain UI consistency during phase transitions', async () => {
    // Initial ready phase with action
    const readyState = {
      ...baseWarState,
      ui: {
        ...baseWarState.ui,
        actionMap: {
          player: [{ id: 'battleStart', source: { type: 'player' }, direction: 'Click to flip cards!' }]
        }
      }
    };
    
    mockLobbyWebSocketReturn.lobbyState = readyState;

    render(
      <BrowserRouter>
        <PlayerPreferencesProvider>
          <AnimationProvider>
            <GameView lobbyId={lobbyId} />
          </AnimationProvider>
        </PlayerPreferencesProvider>
      </BrowserRouter>
    );

    const ws = (window as any).mockWebSocket;
    
    if (ws?.onmessage) {
      ws.onmessage(new MessageEvent('message', { data: JSON.stringify(readyState) }));
    }

    // Should show battle start action
    expect(await screen.findByText(/Click to flip/i)).toBeInTheDocument();

    // Transition to battle phase
    const battlePhase = {
      patches: [
        { op: 'replace', path: '/game/phases/game', value: { current: 'battle', count: 0, actionsProcessed: 0 } },
        { op: 'replace', path: '/ui/actionMap', value: {} }
      ]
    };

    if (ws?.onmessage) {
      ws.onmessage(new MessageEvent('message', { data: JSON.stringify(battlePhase) }));
    }

    // Battle start action should be removed
    await waitFor(() => {
      expect(screen.queryByText(/Click to flip/i)).not.toBeInTheDocument();
    });
  });

  it('should properly display card visibility based on zone settings', async () => {
    const visibilityTest = {
      ...baseWarState,
      game: {
        ...baseWarState.game,
        zones: {
          deck_p1: {
            type: 'list',
            visibility: 'owner',
            items: [{ entity: 'card_hearts_a', rank: 'A', suit: 'hearts' }]
          },
          battle_p1: {
            type: 'list',
            visibility: 'all',
            items: [{ entity: 'card_spades_k', rank: 'K', suit: 'spades' }]
          }
        }
      },
      ui: {
        ...baseWarState.ui,
        entities: [
          { id: 'card_hearts_a', type: 'card', props: { rank: 'A', suit: 'hearts' } },
          { id: 'card_spades_k', type: 'card', props: { rank: 'K', suit: 'spades' } }
        ]
      }
    };
    
    mockLobbyWebSocketReturn.lobbyState = visibilityTest;

    const { container } = render(
      <BrowserRouter>
        <PlayerPreferencesProvider>
          <AnimationProvider>
            <GameView lobbyId={lobbyId} />
          </AnimationProvider>
        </PlayerPreferencesProvider>
      </BrowserRouter>
    );

    const ws = (window as any).mockWebSocket;
    
    if (ws?.onmessage) {
      ws.onmessage(new MessageEvent('message', { data: JSON.stringify(visibilityTest) }));
    }

    await waitFor(() => {
      // Cards should be rendered - check for card elements
      const cards = screen.getAllByTestId(/^card-/);
      expect(cards.length).toBeGreaterThan(0);
    });
  });
});
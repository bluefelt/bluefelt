import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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

describe('War - Complete Client Tests', () => {
  const lobbyId = 'test-lobby-123';
  const mockWarState = {
    you: 'p1',
    started: true,
    game: {
      currentPlayer: 'p1',
      turn: 0,
      tick: 0,
      zones: {
        deck_p1: {
          type: 'list',
          visibility: 'owner',
          items: Array(26).fill({ entity: 'card_back' })
        },
        deck_p2: {
          type: 'list',
          visibility: 'owner',
          items: Array(26).fill({ entity: 'card_back' })
        },
        battle_p1: {
          type: 'list',
          visibility: 'all',
          items: []
        },
        battle_p2: {
          type: 'list',
          visibility: 'all',
          items: []
        }
      },
      players: [{ id: 'p1' }, { id: 'p2' }],
      phases: {
        game: { 
          current: 'ready',
          count: 0,
          actionsProcessed: 0
        }
      },
      gameStatus: { state: 'playing', winner: null, tie: false },
      selection: {}
    },
    ui: {
      actionMap: {
        p1: {
          '/player': {
            action: 'battleStart',
            direction: 'Click to flip cards!'
          }
        },
        p2: {}
      },
      entities: [
        { id: 'card_back', type: 'card', props: { rank: 'back', suit: 'none' } }
      ],
      zones: [
        { id: 'deck_p1', name: 'Deck', resolved_name: 'Deck', visibility: 'owner', owner: 'p1', layout_order: 0, renderType: 'card', cards: [], layout: 'stack' },
        { id: 'deck_p2', name: 'Deck', resolved_name: 'Deck', visibility: 'owner', owner: 'p2', layout_order: 1, renderType: 'card', cards: [], layout: 'stack' },
        { id: 'battle_p1', name: 'Battle Zone', resolved_name: 'Battle Zone', visibility: 'all', owner: 'p1', layout_order: 2, renderType: 'card', cards: [], layout: 'stack' },
        { id: 'battle_p2', name: 'Battle Zone', resolved_name: 'Battle Zone', visibility: 'all', owner: 'p2', layout_order: 3, renderType: 'card', cards: [], layout: 'stack' }
      ],
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
    // Reset the mock lobby state
    mockLobbyWebSocketReturn.lobbyState = null;
  });

  it('should render initial game state correctly', async () => {
    // Set the lobby state in the mock
    mockLobbyWebSocketReturn.lobbyState = mockWarState;
    
    const { container } = render(
      <BrowserRouter>
        <PlayerPreferencesProvider>
          <AnimationProvider>
            <GameView lobbyId={lobbyId} />
          </AnimationProvider>
        </PlayerPreferencesProvider>
      </BrowserRouter>
    );

    await waitFor(() => {
      // Check that the game is rendered - look for players
      expect(screen.getByText(/Alice/)).toBeInTheDocument();
    });

    // Check players are displayed
    expect(screen.getByText(/Bob/)).toBeInTheDocument();
    
    // Check game log is present
    expect(screen.getByText('Game Log')).toBeInTheDocument();
  });

  it('should display deck zones', async () => {
    // Set the lobby state in the mock
    mockLobbyWebSocketReturn.lobbyState = mockWarState;
    
    const { container } = render(
      <BrowserRouter>
        <PlayerPreferencesProvider>
          <AnimationProvider>
            <GameView lobbyId={lobbyId} />
          </AnimationProvider>
        </PlayerPreferencesProvider>
      </BrowserRouter>
    );

    await waitFor(() => {
      // Check that zones are rendered
      const zones = container.querySelectorAll('[data-testid^="zone-"]');
      expect(zones.length).toBeGreaterThan(0);
    });
  });

  it('should handle player actions', async () => {
    const user = userEvent.setup();
    
    // Set the lobby state in the mock
    mockLobbyWebSocketReturn.lobbyState = mockWarState;
    
    render(
      <BrowserRouter>
        <PlayerPreferencesProvider>
          <AnimationProvider>
            <GameView lobbyId={lobbyId} />
          </AnimationProvider>
        </PlayerPreferencesProvider>
      </BrowserRouter>
    );

    // Wait for the game to be rendered
    await waitFor(() => {
      expect(screen.getByText(/Alice/)).toBeInTheDocument();
    });

    // Find player action button
    const actionButtons = screen.getAllByRole('button');
    expect(actionButtons.length).toBeGreaterThan(0);
  });

  it('should update game log with patches', async () => {
    // Set initial state
    mockLobbyWebSocketReturn.lobbyState = mockWarState;
    
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
    
    // Simulate game log update
    const logPatch = {
      patches: [
        {
          op: 'add',
          path: '/ui/gameLog/-',
          value: {
            message: 'Game started!',
            timestamp: '12:00'
          }
        }
      ]
    };

    if (ws?.onmessage) {
      ws.onmessage(new MessageEvent('message', { data: JSON.stringify(logPatch) }));
    }

    await waitFor(() => {
      expect(screen.getByText(/Game started!/)).toBeInTheDocument();
    });
  });

  it('should show game end state', async () => {
    // Set initial state
    mockLobbyWebSocketReturn.lobbyState = mockWarState;
    
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
    
    // Simulate game end
    const endState = {
      patches: [
        {
          op: 'replace',
          path: '/game/gameStatus',
          value: { state: 'ended', winner: 'p1', tie: false }
        }
      ]
    };

    if (ws?.onmessage) {
      ws.onmessage(new MessageEvent('message', { data: JSON.stringify(endState) }));
    }

    await waitFor(() => {
      // Game result banner should appear when game ends
      const resultBanner = screen.queryByText(/wins!/i);
      expect(resultBanner).toBeTruthy();
    });
  });
});
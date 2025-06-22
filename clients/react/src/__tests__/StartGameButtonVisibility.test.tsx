import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import GameView from '../components/GameView';
import { PlayerProvider } from '../context/PlayerContext';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const mockPlayer = { username: 'testplayer' };

// Mock the usePlayer hook
vi.mock('../context/PlayerContext', async () => {
  const actual = await vi.importActual('../context/PlayerContext');
  return {
    ...actual,
    usePlayer: () => ({ player: mockPlayer }),
  };
});

// Mock the getLobby API call
vi.mock('../api/lobbies', () => ({
  getLobby: vi.fn(() => Promise.resolve({
    id: 'test-lobby',
    gameId: 'go-fish',
    manifest: {
      gameId: 'go-fish',
      metadata: {
        name: 'Go Fish',
        players: { min: 2, max: 4 }
      }
    }
  }))
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
      username: 'testplayer', 
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
    player: { username: 'testplayer', color: '#FF0000' },
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
    // Store instance for test access
    (window as any).__mockWebSocket = this;
    
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
  
  simulateMessage(data: any) {
    this.onmessage?.(new MessageEvent('message', {
      data: JSON.stringify(data)
    }));
  }
}

describe('Start Game Button Visibility', () => {
  let queryClient: QueryClient;
  
  beforeEach(() => {
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
    // Replace global WebSocket with mock
    global.WebSocket = MockWebSocket as any;
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false }
      }
    });
    // Clear any previous messages
    (window as any).__sentMessages = [];
  });

  afterEach(() => {
    const mockWs = (window as any).__mockWebSocket;
    if (mockWs) {
      mockWs.close();
    }
    vi.clearAllMocks();
  });

  it('should show Start Game button when player is recognized and minimum players are present', async () => {
    const { act } = await import('@testing-library/react');
    
    render(
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            <GameView 
              lobbyId="test-lobby" 
              onLeave={() => {}}
            />
          </PlayerProvider>
        </QueryClientProvider>
      </MemoryRouter>
    );

    // Wait for WebSocket connection and ensure it's open
    await waitFor(() => {
      expect((window as any).__mockWebSocket).toBeDefined();
      expect((window as any).__mockWebSocket.readyState).toBe(WebSocket.OPEN);
    });

    const mockWs = (window as any).__mockWebSocket as MockWebSocket;

    // Send welcome message with all required fields
    await act(async () => {
      mockWs.simulateMessage({
        type: 'welcome',
        started: false,
        you: 'testplayer', // Player is recognized
        ui: {
          players: ['testplayer', 'otherplayer'], // 2 players (minimum for most games)
          manifest: {
            gameId: 'go-fish',
            metadata: {
              name: 'Go Fish',
              players: {
                min: 2,
                max: 4
              }
            }
          }
        },
        game: null,
        tick: 0
      });
    });

    // Start Game button should be visible
    await waitFor(() => {
      expect(screen.getByText('Start Game')).toBeInTheDocument();
    });
    
    // Join Game button should NOT be visible
    expect(screen.queryByText('Join Game')).not.toBeInTheDocument();
  });

  it('should show Join Game button when player is not recognized', async () => {
    const { act } = await import('@testing-library/react');
    
    render(
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            <GameView 
              lobbyId="test-lobby" 
              onLeave={() => {}}
            />
          </PlayerProvider>
        </QueryClientProvider>
      </MemoryRouter>
    );

    // Wait for WebSocket connection and ensure it's open
    await waitFor(() => {
      expect((window as any).__mockWebSocket).toBeDefined();
      expect((window as any).__mockWebSocket.readyState).toBe(WebSocket.OPEN);
    });

    const mockWs = (window as any).__mockWebSocket as MockWebSocket;

    // Send welcome message with you: null (not recognized)
    await act(async () => {
      mockWs.simulateMessage({
      type: 'welcome',
      started: false,
      you: null, // Player not recognized
      ui: {
        players: ['player1', 'player2'],
        manifest: {
          gameId: 'go-fish',
          metadata: {
            name: 'Go Fish',
            players: {
              min: 2,
              max: 4
            }
          }
        }
      },
      game: null,
      tick: 0
      });
    });

    // Join Game button should be visible
    await waitFor(() => {
      expect(screen.getByText('Join Game')).toBeInTheDocument();
    });
    
    // Start Game button should NOT be visible
    expect(screen.queryByText('Start Game')).not.toBeInTheDocument();
  });

  it('should not show Start Game button when insufficient players', async () => {
    const { act } = await import('@testing-library/react');
    
    render(
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            <GameView 
              lobbyId="test-lobby" 
              onLeave={() => {}}
            />
          </PlayerProvider>
        </QueryClientProvider>
      </MemoryRouter>
    );

    // Wait for WebSocket connection and ensure it's open
    await waitFor(() => {
      expect((window as any).__mockWebSocket).toBeDefined();
      expect((window as any).__mockWebSocket.readyState).toBe(WebSocket.OPEN);
    });

    const mockWs = (window as any).__mockWebSocket as MockWebSocket;

    // Send welcome message with only 1 player (below minimum)
    await act(async () => {
      mockWs.simulateMessage({
      type: 'welcome',
      started: false,
      you: 'testplayer',
      ui: {
        players: ['testplayer'], // Only 1 player
        manifest: {
          gameId: 'go-fish',
          metadata: {
            name: 'Go Fish',
            players: {
              min: 2, // Requires 2 players minimum
              max: 4
            }
          }
        }
      },
      game: null,
      tick: 0
      });
    });

    // Neither button should be visible with insufficient players
    await waitFor(() => {
      expect(screen.queryByText('Start Game')).not.toBeInTheDocument();
      expect(screen.queryByText('Join Game')).not.toBeInTheDocument();
    });
  });

  it('should handle missing manifest data gracefully', async () => {
    const { act } = await import('@testing-library/react');
    
    render(
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          <PlayerProvider>
            <GameView 
              lobbyId="test-lobby" 
              onLeave={() => {}}
            />
          </PlayerProvider>
        </QueryClientProvider>
      </MemoryRouter>
    );

    // Wait for WebSocket connection and ensure it's open
    await waitFor(() => {
      expect((window as any).__mockWebSocket).toBeDefined();
      expect((window as any).__mockWebSocket.readyState).toBe(WebSocket.OPEN);
    });

    const mockWs = (window as any).__mockWebSocket as MockWebSocket;

    // Send welcome message without manifest
    await act(async () => {
      mockWs.simulateMessage({
      type: 'welcome',
      started: false,
      you: 'testplayer',
      ui: {
        players: ['testplayer', 'otherplayer'],
        manifest: null // Missing manifest
      },
      game: null,
      tick: 0
      });
    });

    // Should not crash and should not show buttons without manifest
    await waitFor(() => {
      expect(screen.queryByText('Start Game')).not.toBeInTheDocument();
      expect(screen.queryByText('Join Game')).not.toBeInTheDocument();
    });
  });
});
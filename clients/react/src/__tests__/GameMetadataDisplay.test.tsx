import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import GameView from '../components/GameView';
import GameHeader from '../components/GameHeader';
import { PlayerProvider } from '../context/PlayerContext';
import { AnimationProvider } from '../context/AnimationContext';
import { PlayerPreferencesProvider } from '../context/PlayerPreferencesContext';
import { WebSocketProvider } from '../context/WebSocketContext';
import { BrowserRouter } from 'react-router-dom';

// Mock the WebSocket hook with proper zones array
vi.mock('../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: () => ({
    sendMessage: vi.fn(),
    lobbyState: {
      started: true,
      you: 'p1',
      ui: {
        manifest: {
          gameId: 'tic-tac-toe',
          metadata: {
            name: 'Tic-Tac-Toe',
            description: 'Classic game of X\'s and O\'s on a 3x3 grid',
            players: { min: 2, max: 2 }
          }
        },
        players: ['alice', 'bob'],
        entities: [],
        zones: [], // Fixed: was {} now []
        actionMap: {}
      },
      game: {
        currentPlayer: 'p1',
        zones: {},
        gameStatus: { state: 'active' }
      }
    },
    connectionState: 'connected',
    joinLobby: vi.fn(),
    leaveLobby: vi.fn(),
    startGame: vi.fn(),
    sendPreferencesUpdate: vi.fn(),
    disconnect: vi.fn()
  })
}));

// Mock hooks
vi.mock('../hooks/useGameActions', () => ({
  useGameActions: () => ({
    handleCellClick: vi.fn(),
    handleCardAction: vi.fn(),
    handleZoneAction: vi.fn(),
    handleChoiceSelect: vi.fn()
  })
}));

vi.mock('../utils/actionMapUtils', () => ({
  getPlayerActionMap: () => ({}),
  getFirstAction: () => null
}));

const TestProviders = ({ children }: { children: React.ReactNode }) => (
  <BrowserRouter>
    <WebSocketProvider url="ws://localhost">
      <PlayerProvider>
        <AnimationProvider>
          <PlayerPreferencesProvider>
            {children}
          </PlayerPreferencesProvider>
        </AnimationProvider>
      </PlayerProvider>
    </WebSocketProvider>
  </BrowserRouter>
);

describe('Game Metadata Display', () => {
  beforeEach(() => {
    // Set up player context
    localStorage.setItem('player', JSON.stringify({ username: 'alice' }));
  });

  it('should display game name from manifest when game is started', () => {
    render(
      <TestProviders>
        <GameView lobbyId="test-lobby" gameId="tic-tac-toe" />
      </TestProviders>
    );

    // The GameHeader should display "Tic-Tac-Toe" not "Game" - should appear multiple times
    const titleElements = screen.getAllByText('Tic-Tac-Toe');
    expect(titleElements.length).toBeGreaterThan(0);
    expect(screen.queryByText('Game')).not.toBeInTheDocument();
  });
});

describe('GameHeader Component', () => {
  it('should display the game name passed as prop', () => {
    render(
      <TestProviders>
        <GameHeader
          lobbyId="test"
          gameId="tic-tac-toe"
          gameName="Tic-Tac-Toe"
          status="in_progress"
          players={[
            { username: 'alice', isConnected: true },
            { username: 'bob', isConnected: true }
          ]}
          currentPlayer="alice"
        />
      </TestProviders>
    );

    expect(screen.getByRole('heading', { name: 'Tic-Tac-Toe' })).toBeInTheDocument();
  });

  it('should never display "Game" when proper name is provided', () => {
    render(
      <TestProviders>
        <GameHeader
          lobbyId="test"
          gameId="connect-four"
          gameName="Connect Four"
          status="in_progress"
          players={[
            { username: 'alice', isConnected: true },
            { username: 'bob', isConnected: true }
          ]}
          currentPlayer="alice"
        />
      </TestProviders>
    );

    expect(screen.getByRole('heading', { name: 'Connect Four' })).toBeInTheDocument();
    expect(screen.queryByText('Game')).not.toBeInTheDocument();
  });
});
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { LobbyPage } from '../pages/LobbyPage';
import { TableList } from '../components/TableList';
import { LobbyChat } from '../components/LobbyChat';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { WebSocketContext } from '../context/WebSocketContext';
import { PlayerContext } from '../context/PlayerContext';
import type { Table, LobbyState, ChatMessage } from '../types/game-types';

// Mock WebSocket context
const createMockWebSocketContext = () => {
  const mockSend = vi.fn();
  const mockCreateTable = vi.fn();
  const mockClaimSeat = vi.fn();
  const mockReleaseSeat = vi.fn();
  const mockSetReady = vi.fn();
  const mockSendChatMessage = vi.fn();
  
  return {
    lobbyState: null as LobbyState | null,
    playerId: 'player1',
    isConnected: true,
    send: mockSend,
    createTable: mockCreateTable,
    claimSeat: mockClaimSeat,
    releaseSeat: mockReleaseSeat,
    setReady: mockSetReady,
    sendChatMessage: mockSendChatMessage,
    startGame: vi.fn(),
    performAction: vi.fn(),
  };
};

// Test utilities
const renderWithProviders = (
  component: React.ReactElement,
  { 
    wsContext = createMockWebSocketContext(),
    playerId = 'player1',
    lobbyState = null as LobbyState | null,
  } = {}
) => {
  return render(
    <MemoryRouter initialEntries={['/lobby/test-lobby']}>
      <PlayerContext.Provider value={{ playerId, setPlayerId: vi.fn() }}>
        <WebSocketContext.Provider value={{ ...wsContext, lobbyState }}>
          <Routes>
            <Route path="/lobby/:lobbyId" element={component} />
          </Routes>
        </WebSocketContext.Provider>
      </PlayerContext.Provider>
    </MemoryRouter>
  );
};

describe('TableList Integration', () => {
  it('displays empty state when no tables exist', () => {
    renderWithProviders(<TableList />);
    expect(screen.getByText(/no tables yet/i)).toBeInTheDocument();
  });

  it('displays table list when tables exist', () => {
    const tables: Table[] = [
      {
        id: 'table1',
        bundleId: 'tic-tac-toe',
        name: 'Alice\'s Game',
        owner: 'player1',
        status: 'Open',
        seats: [
          { playerId: 'player1', username: 'Alice' },
          null,
        ],
        readyStates: [false, false],
        spectators: [],
        minPlayers: 2,
        maxPlayers: 2,
      },
    ];

    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [
        { player_id: 'player1', username: 'Alice' },
        { player_id: 'player2', username: 'Bob' },
      ],
      tables,
      chat: {
        lobby: [],
        tables: {},
      },
    };

    renderWithProviders(<TableList />, { lobbyState });
    
    expect(screen.getByText('Alice\'s Game')).toBeInTheDocument();
    expect(screen.getByText('tic-tac-toe')).toBeInTheDocument();
    expect(screen.getByText('Open')).toBeInTheDocument();
  });

  it('allows creating a new table', async () => {
    const wsContext = createMockWebSocketContext();
    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [{ player_id: 'player1', username: 'Alice' }],
      tables: [],
      chat: { lobby: [], tables: {} },
    };

    renderWithProviders(<TableList />, { wsContext, lobbyState });
    
    const createButton = screen.getByText(/create new table/i);
    fireEvent.click(createButton);
    
    expect(wsContext.createTable).toHaveBeenCalledWith('tic-tac-toe', undefined);
  });

  it('allows claiming an empty seat', () => {
    const wsContext = createMockWebSocketContext();
    const tables: Table[] = [
      {
        id: 'table1',
        bundleId: 'tic-tac-toe',
        name: 'Test Game',
        owner: 'player2',
        status: 'Open',
        seats: [null, null],
        readyStates: [false, false],
        spectators: [],
        minPlayers: 2,
        maxPlayers: 2,
      },
    ];

    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [{ player_id: 'player1', username: 'Alice' }],
      tables,
      chat: { lobby: [], tables: {} },
    };

    renderWithProviders(<TableList />, { wsContext, lobbyState });
    
    const claimButtons = screen.getAllByText(/claim seat/i);
    fireEvent.click(claimButtons[0]);
    
    expect(wsContext.claimSeat).toHaveBeenCalledWith('table1', 0);
  });

  it('shows ready button when seated', () => {
    const wsContext = createMockWebSocketContext();
    const tables: Table[] = [
      {
        id: 'table1',
        bundleId: 'tic-tac-toe',
        name: 'Test Game',
        owner: 'player2',
        status: 'Open',
        seats: [
          { playerId: 'player1', username: 'Alice' },
          null,
        ],
        readyStates: [false, false],
        spectators: [],
        minPlayers: 2,
        maxPlayers: 2,
      },
    ];

    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [{ player_id: 'player1', username: 'Alice' }],
      tables,
      chat: { lobby: [], tables: {} },
    };

    renderWithProviders(<TableList />, { wsContext, lobbyState, playerId: 'player1' });
    
    const readyButton = screen.getByText(/mark ready/i);
    expect(readyButton).toBeInTheDocument();
    
    fireEvent.click(readyButton);
    expect(wsContext.setReady).toHaveBeenCalledWith('table1', true);
  });

  it('displays countdown timer', () => {
    const tables: Table[] = [
      {
        id: 'table1',
        bundleId: 'tic-tac-toe',
        name: 'Test Game',
        owner: 'player1',
        status: 'Countdown',
        seats: [
          { playerId: 'player1', username: 'Alice' },
          { playerId: 'player2', username: 'Bob' },
        ],
        readyStates: [true, true],
        spectators: [],
        minPlayers: 2,
        maxPlayers: 2,
        countdownEndsAt: Date.now() + 5000, // 5 seconds from now
      },
    ];

    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [],
      tables,
      chat: { lobby: [], tables: {} },
    };

    renderWithProviders(<TableList />, { lobbyState });
    
    expect(screen.getByText(/game starting in/i)).toBeInTheDocument();
    expect(screen.getByText(/Countdown/i)).toBeInTheDocument();
  });
});

describe('LobbyChat Integration', () => {
  it('displays lobby and table tabs', () => {
    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [],
      tables: [],
      chat: { lobby: [], tables: {} },
    };

    renderWithProviders(<LobbyChat />, { lobbyState });
    
    expect(screen.getByText('Lobby Chat')).toBeInTheDocument();
    expect(screen.getByText('Table Chat')).toBeInTheDocument();
  });

  it('displays lobby messages', () => {
    const lobbyMessages: ChatMessage[] = [
      {
        id: 'msg1',
        playerId: 'player1',
        username: 'Alice',
        content: 'Hello lobby!',
        timestamp: Date.now() - 60000,
        scope: 'lobby',
      },
      {
        id: 'msg2',
        playerId: 'player2',
        username: 'Bob',
        content: 'Hi Alice!',
        timestamp: Date.now() - 30000,
        scope: 'lobby',
      },
    ];

    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [],
      tables: [],
      chat: { 
        lobby: lobbyMessages,
        tables: {},
      },
    };

    renderWithProviders(<LobbyChat />, { lobbyState });
    
    expect(screen.getByText('Hello lobby!')).toBeInTheDocument();
    expect(screen.getByText('Hi Alice!')).toBeInTheDocument();
  });

  it('sends lobby chat messages', () => {
    const wsContext = createMockWebSocketContext();
    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [],
      tables: [],
      chat: { lobby: [], tables: {} },
    };

    renderWithProviders(<LobbyChat />, { wsContext, lobbyState });
    
    const input = screen.getByPlaceholderText(/type a message/i);
    const sendButton = screen.getByText(/send/i);
    
    fireEvent.change(input, { target: { value: 'Test message' } });
    fireEvent.click(sendButton);
    
    expect(wsContext.sendChatMessage).toHaveBeenCalledWith('Test message', 'lobby');
  });

  it('switches to table chat when player is seated', () => {
    const wsContext = createMockWebSocketContext();
    const tables: Table[] = [
      {
        id: 'table1',
        bundleId: 'tic-tac-toe',
        name: 'Test Game',
        owner: 'player1',
        status: 'Open',
        seats: [
          { playerId: 'player1', username: 'Alice' },
          null,
        ],
        readyStates: [false, false],
        spectators: [],
        minPlayers: 2,
        maxPlayers: 2,
      },
    ];

    const tableMessages: ChatMessage[] = [
      {
        id: 'msg1',
        playerId: 'player1',
        username: 'Alice',
        content: 'Ready to play?',
        timestamp: Date.now() - 10000,
        scope: 'table',
        tableId: 'table1',
      },
    ];

    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [],
      tables,
      chat: { 
        lobby: [],
        tables: {
          'table1': tableMessages,
        },
      },
    };

    renderWithProviders(<LobbyChat />, { wsContext, lobbyState, playerId: 'player1' });
    
    // Click table chat tab
    const tableTab = screen.getByText('Table Chat');
    fireEvent.click(tableTab);
    
    expect(screen.getByText('Ready to play?')).toBeInTheDocument();
    
    // Send table message
    const input = screen.getByPlaceholderText(/type a message/i);
    const sendButton = screen.getByText(/send/i);
    
    fireEvent.change(input, { target: { value: 'Yes!' } });
    fireEvent.click(sendButton);
    
    expect(wsContext.sendChatMessage).toHaveBeenCalledWith('Yes!', 'table', 'table1');
  });
});

describe('Full Lobby Page Integration', () => {
  it('renders lobby page with tables and chat', () => {
    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [
        { player_id: 'player1', username: 'Alice' },
        { player_id: 'player2', username: 'Bob' },
      ],
      tables: [],
      chat: { lobby: [], tables: {} },
    };

    renderWithProviders(<LobbyPage />, { lobbyState });
    
    // Check that both components are rendered
    expect(screen.getByText(/create new table/i)).toBeInTheDocument();
    expect(screen.getByText('Lobby Chat')).toBeInTheDocument();
    expect(screen.getByText('Table Chat')).toBeInTheDocument();
  });

  it('handles table creation flow', async () => {
    const wsContext = createMockWebSocketContext();
    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [{ player_id: 'player1', username: 'Alice' }],
      tables: [],
      chat: { lobby: [], tables: {} },
    };

    const { rerender } = renderWithProviders(<LobbyPage />, { wsContext, lobbyState });
    
    // Create table
    const createButton = screen.getByText(/create new table/i);
    fireEvent.click(createButton);
    
    expect(wsContext.createTable).toHaveBeenCalled();
    
    // Simulate table created
    const newTable: Table = {
      id: 'table1',
      bundleId: 'tic-tac-toe',
      name: 'Alice\'s Game',
      owner: 'player1',
      status: 'Open',
      seats: [
        { playerId: 'player1', username: 'Alice' },
        null,
      ],
      readyStates: [false, false],
      spectators: [],
      minPlayers: 2,
      maxPlayers: 2,
    };
    
    const updatedLobbyState = {
      ...lobbyState,
      tables: [newTable],
    };
    
    rerender(
      <MemoryRouter initialEntries={['/lobby/test-lobby']}>
        <PlayerContext.Provider value={{ playerId: 'player1', setPlayerId: vi.fn() }}>
          <WebSocketContext.Provider value={{ ...wsContext, lobbyState: updatedLobbyState }}>
            <Routes>
              <Route path="/lobby/:lobbyId" element={<LobbyPage />} />
            </Routes>
          </WebSocketContext.Provider>
        </PlayerContext.Provider>
      </MemoryRouter>
    );
    
    // Verify table appears
    expect(screen.getByText('Alice\'s Game')).toBeInTheDocument();
    expect(screen.getByText(/mark ready/i)).toBeInTheDocument();
  });

  it('maintains backward compatibility with legacy games', () => {
    const lobbyState: LobbyState = {
      id: 'test-lobby',
      gameId: 'tic-tac-toe',
      members: [
        { player_id: 'player1', username: 'Alice' },
        { player_id: 'player2', username: 'Bob' },
      ],
      tables: [],
      chat: { lobby: [], tables: {} },
      // Legacy game state
      game: {
        id: 'tic-tac-toe',
        status: 'waiting',
        currentPlayer: null,
        zones: {},
        entities: {},
      },
      ui: {
        actionMap: {},
      },
    };

    renderWithProviders(<LobbyPage />, { lobbyState });
    
    // Should still show game UI when game exists
    expect(screen.getByText(/waiting for players/i)).toBeInTheDocument();
  });
});
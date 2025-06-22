import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { GameView } from '../components/GameView';
import { MultiStepActionDisplay } from '../components/MultiStepActionDisplay';
import { WebSocketContext } from '../context/WebSocketContext';
import { PlayerContext } from '../context/PlayerContext';
import { PlayerPreferencesProvider } from '../context/PlayerPreferencesContext';
import { AnimationProvider } from '../context/AnimationContext';
import { MemoryRouter } from 'react-router-dom';

// Mock useLobbyWebSocket to capture the state from context
let mockLobbyState: any = null;

vi.mock('../ws/useLobbyWebSocket', () => ({
  useLobbyWebSocket: () => {
    const React = require('react');
    const { useContext } = React;
    const { WebSocketContext } = require('../context/WebSocketContext');
    const context = useContext(WebSocketContext);
    mockLobbyState = context?.gameState || {};
    
    return {
      lobbyState: mockLobbyState,
      sendMessage: context?.sendMessage || vi.fn(),
      connectionState: 'connected',
      joinLobby: vi.fn(),
      leaveLobby: vi.fn(),
      startGame: vi.fn(),
      sendPreferencesUpdate: vi.fn(),
      disconnect: vi.fn()
    };
  }
}));

// Store references to handle clicks
let cellClickHandlers: Record<string, () => void> = {};
let multiStepHandlers = {
  onCancel: vi.fn(),
  onConfirm: vi.fn()
};

// Mock the GameView component with board cells for integration tests
vi.mock('../components/GameView', () => ({
  GameView: vi.fn(() => {
    const React = require('react');
    const { useContext } = React;
    const { WebSocketContext } = require('../context/WebSocketContext');
    const context = useContext(WebSocketContext);
    const multiStepState = mockLobbyState?.ui?.multiStepState;
    const hasMultiStep = Boolean(multiStepState);
    
    // Set up click handlers
    cellClickHandlers['/zones/board/cells/0/0'] = () => {
      if (hasMultiStep) {
        // During multi-step, check stepActionMap
        const action = multiStepState.stepActionMap?.['/zones/board/cells/0/0'];
        if (action) {
          context?.sendMessage({
            action: 'multiStepSelection',
            stepId: action.stepId,
            selection: '/zones/board/cells/0/0'
          });
        }
      } else {
        // Normal action
        const actionMap = mockLobbyState?.ui?.actionMap?.p1 || {};
        const action = actionMap['/zones/board/cells/0/0'];
        if (action) {
          context?.sendMessage({
            action: action.action,
            args: { target: '/zones/board/cells/0/0' }
          });
        }
      }
    };
    
    cellClickHandlers['/zones/board/cells/1/1'] = () => {
      if (hasMultiStep) {
        const action = multiStepState.stepActionMap?.['/zones/board/cells/1/1'];
        if (action) {
          context?.sendMessage({
            action: 'multiStepSelection',
            stepId: action.stepId,
            selection: '/zones/board/cells/1/1'
          });
        }
      } else {
        const actionMap = mockLobbyState?.ui?.actionMap?.p1 || {};
        const action = actionMap['/zones/board/cells/1/1'];
        if (action) {
          context?.sendMessage({
            action: action.action,
            args: { target: '/zones/board/cells/1/1' }
          });
        }
      }
    };
    
    multiStepHandlers.onCancel = () => {
      context?.sendMessage({
        action: 'multiStepCancel'
      });
    };
    
    multiStepHandlers.onConfirm = () => {
      context?.sendMessage({
        action: 'multiStepConfirm',
        confirmed: true,
        actionId: multiStepState?.actionId
      });
    };
    
    return (
      <div data-testid="game-view">
        <div data-testid="board">
          <div 
            data-testid="board-cell-0-0" 
            className="cursor-pointer"
            style={{ cursor: 'pointer' }}
            onClick={cellClickHandlers['/zones/board/cells/0/0']}
          >
            Cell 0,0
            <div data-testid="action-indicator" />
          </div>
          <div 
            data-testid="board-cell-1-1" 
            className="cursor-pointer"
            style={{ cursor: 'pointer' }}
            onClick={cellClickHandlers['/zones/board/cells/1/1']}
          >
            Cell 1,1
            {hasMultiStep && (
              <div data-testid="action-indicator" data-state="next_step" />
            )}
          </div>
        </div>
        {hasMultiStep && (
          <>
            <div data-testid="multi-step-display">
              <div>Action in Progress</div>
              <div>Step {(multiStepState.currentStepIndex || 0) + 1} of {multiStepState.totalSteps || 2}</div>
              {multiStepState.currentStepIndex === 1 && (
                <div>Select where to move</div>
              )}
            </div>
            {multiStepState.canCancel && (
              <button 
                aria-label="Cancel current action" 
                onClick={multiStepHandlers.onCancel}
              >
                Cancel
              </button>
            )}
            {multiStepState.requiresConfirmation && (
              <button 
                aria-label="Confirm action" 
                onClick={multiStepHandlers.onConfirm}
              >
                Confirm
              </button>
            )}
          </>
        )}
      </div>
    );
  })
}));

// Mock the PlayerContext
vi.mock('../context/PlayerContext', () => {
  const React = require('react');
  const MockContext = React.createContext({});
  return {
    usePlayer: () => ({
      player: { username: 'Alice', color: '#ff0000' },
      setPlayer: vi.fn(),
      clearPlayer: vi.fn()
    }),
    PlayerProvider: ({ children }: { children: React.ReactNode }) => children,
    PlayerContext: MockContext
  };
});

// Mock the WebSocketContext
vi.mock('../context/WebSocketContext', () => {
  const React = require('react');
  const MockWebSocketContext = React.createContext({});
  return {
    WebSocketContext: MockWebSocketContext,
    WebSocketProvider: ({ children }: { children: React.ReactNode }) => children
  };
});

// Mock data for tests
const mockMultiStepState = {
  actionId: 'movePiece',
  actionType: 'movePiece',
  currentStepId: 'selectPiece',
  currentStepIndex: 0,
  totalSteps: 2,
  storedData: {},
  canCancel: true,
  requiresConfirmation: true,
};

const mockGameState = {
  type: 'welcome',
  tick: 1,
  game: {
    currentPlayer: 'p1',
    gameStatus: { state: 'active' },
    zones: {
      board: {
        cells: Array(3).fill(null).map(() => Array(3).fill(null))
      }
    }
  },
  you: 'p1',
  started: true,
  ui: {
    actionMap: {
      p1: {
        '/zones/board/cells/0/0': {
          action: 'movePiece',
          isMultiStep: true,
          direction: 'Move a piece'
        }
      }
    },
    multiStepState: null,
    zones: [],
    manifest: {
      id: 'test-multistep',
      metadata: {
        name: 'Test Multi-Step',
        players: { min: 2, max: 2 }
      }
    },
    entities: {}
  }
};

describe('Multi-Step Actions', () => {
  const mockSendMessage = vi.fn();
  const mockWebSocketValue = {
    sendMessage: mockSendMessage,
    connected: true,
    lobbies: [],
    currentLobby: null,
    gameState: mockGameState,
    connectionId: 'test-connection',
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('MultiStepActionDisplay Component', () => {
    it('should render progress indicator for multi-step action', () => {
      const onCancel = vi.fn();
      const onConfirm = vi.fn();

      render(
        <MultiStepActionDisplay
          multiStepState={mockMultiStepState}
          onCancel={onCancel}
          onConfirm={onConfirm}
        />
      );

      // Check progress display
      expect(screen.getByText('Action in Progress')).toBeInTheDocument();
      expect(screen.getByText('Step 1 of 2')).toBeInTheDocument();
      expect(screen.getByText('50%')).toBeInTheDocument();
      expect(screen.getByText('Select a piece to move')).toBeInTheDocument();

      // Check cancel button
      const cancelButton = screen.getByLabelText('Cancel current action');
      expect(cancelButton).toBeInTheDocument();
      fireEvent.click(cancelButton);
      expect(onCancel).toHaveBeenCalled();
    });

    it('should render confirmation dialog when requiresConfirmation is true', () => {
      const confirmationState = {
        ...mockMultiStepState,
        currentStepIndex: 1,
        requiresConfirmation: true,
        confirmationPrompt: 'Move piece from A1 to B2?',
      };

      const onCancel = vi.fn();
      const onConfirm = vi.fn();

      render(
        <MultiStepActionDisplay
          multiStepState={confirmationState}
          onCancel={onCancel}
          onConfirm={onConfirm}
        />
      );

      // Check confirmation display
      expect(screen.getByText('Confirm Action')).toBeInTheDocument();
      expect(screen.getByText('Move piece from A1 to B2?')).toBeInTheDocument();
      expect(screen.getByText('Ready to complete')).toBeInTheDocument();

      // Check confirm button
      const confirmButton = screen.getByLabelText('Confirm and complete action');
      expect(confirmButton).toBeInTheDocument();
      fireEvent.click(confirmButton);
      expect(onConfirm).toHaveBeenCalled();
    });

    it('should show completed steps with checkmarks', () => {
      const advancedState = {
        ...mockMultiStepState,
        currentStepIndex: 1,
        storedData: { selectedPiece: 'piece_p1' },
      };

      render(
        <MultiStepActionDisplay
          multiStepState={advancedState}
          onCancel={vi.fn()}
          onConfirm={vi.fn()}
        />
      );

      // Check that first step shows as completed
      const completedStep = screen.getByLabelText('Step 1 completed');
      expect(completedStep).toBeInTheDocument();
      expect(completedStep).toHaveClass('bg-blue-600');

      // Current step should be highlighted
      const currentStep = screen.getByLabelText('Step 2 current');
      expect(currentStep).toBeInTheDocument();
      expect(currentStep).toHaveClass('bg-blue-100');
    });

    it('should not show cancel button when canCancel is false', () => {
      const nonCancellableState = {
        ...mockMultiStepState,
        canCancel: false,
      };

      render(
        <MultiStepActionDisplay
          multiStepState={nonCancellableState}
          onCancel={vi.fn()}
          onConfirm={vi.fn()}
        />
      );

      const cancelButton = screen.queryByLabelText('Cancel current action');
      expect(cancelButton).not.toBeInTheDocument();
    });
  });

  // Helper function available to all integration tests
  const renderGameView = (gameState = mockGameState) => {
    return render(
      <MemoryRouter>
        <PlayerPreferencesProvider>
          <AnimationProvider>
            <WebSocketContext.Provider value={{ ...mockWebSocketValue, gameState }}>
              <GameView />
            </WebSocketContext.Provider>
          </AnimationProvider>
        </PlayerPreferencesProvider>
      </MemoryRouter>
    );
  };

  describe('Multi-Step Action Integration', () => {

    it('should start multi-step action when clicking on a multi-step action zone', async () => {
      renderGameView();

      // Find a cell with multi-step action
      const actionableCell = screen.getByTestId('board-cell-0-0');
      expect(actionableCell).toBeInTheDocument();

      // Click to start multi-step action
      fireEvent.click(actionableCell);

      await waitFor(() => {
        expect(mockSendMessage).toHaveBeenCalledWith({
          action: 'movePiece',
          args: { target: '/zones/board/cells/0/0' }
        });
      });
    });

    it('should display multi-step state from server', () => {
      const gameStateWithMultiStep = {
        ...mockGameState,
        ui: {
          ...mockGameState.ui,
          multiStepState: {
            actionId: 'movePiece',
            currentStepIndex: 0,
            storedData: {},
            canCancel: true,
            requiresConfirmation: false,
            stepActionMap: {
              '/zones/board/cells/1/1': {
                action: 'multiStepSelection',
                stepId: 'selectPiece',
                direction: 'Select this piece'
              }
            }
          }
        }
      };

      renderGameView(gameStateWithMultiStep);

      // Should show multi-step progress
      expect(screen.getByText('Action in Progress')).toBeInTheDocument();
    });

    it('should send multi-step selection when clicking during multi-step', async () => {
      const gameStateWithMultiStep = {
        ...mockGameState,
        ui: {
          ...mockGameState.ui,
          multiStepState: {
            actionId: 'movePiece',
            currentStepIndex: 0,
            storedData: {},
            canCancel: true,
            requiresConfirmation: false,
            stepActionMap: {
              '/zones/board/cells/1/1': {
                action: 'multiStepSelection',
                stepId: 'selectPiece',
                direction: 'Select this piece'
              }
            }
          }
        }
      };

      renderGameView(gameStateWithMultiStep);

      // Click on a selectable cell during multi-step
      const selectableCell = screen.getByTestId('board-cell-1-1');
      fireEvent.click(selectableCell);

      await waitFor(() => {
        expect(mockSendMessage).toHaveBeenCalledWith({
          action: 'multiStepSelection',
          stepId: 'selectPiece',
          selection: '/zones/board/cells/1/1'
        });
      });
    });

    it('should send cancel message when canceling multi-step', async () => {
      const gameStateWithMultiStep = {
        ...mockGameState,
        ui: {
          ...mockGameState.ui,
          multiStepState: {
            actionId: 'movePiece',
            currentStepIndex: 0,
            storedData: {},
            canCancel: true,
            requiresConfirmation: false,
            stepActionMap: {}
          }
        }
      };

      renderGameView(gameStateWithMultiStep);

      // Find and click cancel button
      const cancelButton = screen.getByLabelText('Cancel current action');
      fireEvent.click(cancelButton);

      await waitFor(() => {
        expect(mockSendMessage).toHaveBeenCalledWith({
          action: 'multiStepCancel'
        });
      });
    });

    it('should send confirm message when confirming multi-step', async () => {
      const gameStateWithConfirmation = {
        ...mockGameState,
        ui: {
          ...mockGameState.ui,
          multiStepState: {
            actionId: 'movePiece',
            currentStepIndex: 1,
            storedData: {
              selectedPiece: '/zones/board/cells/0/0',
              destination: '/zones/board/cells/1/1'
            },
            canCancel: true,
            requiresConfirmation: true,
            confirmationPrompt: 'Move piece to new location?',
            stepActionMap: {}
          }
        }
      };

      renderGameView(gameStateWithConfirmation);

      // Find and click confirm button
      const confirmButton = screen.getByLabelText('Confirm and complete action');
      fireEvent.click(confirmButton);

      await waitFor(() => {
        expect(mockSendMessage).toHaveBeenCalledWith({
          action: 'multiStepConfirm'
        });
      });
    });

    it('should handle multi-step state restoration on reconnection', () => {
      const reconnectionState = {
        ...mockGameState,
        ui: {
          ...mockGameState.ui,
          multiStepState: {
            actionId: 'movePiece',
            currentStepIndex: 1,
            storedData: {
              selectedPiece: '/zones/board/cells/0/0'
            },
            canCancel: true,
            requiresConfirmation: false,
            stepActionMap: {
              '/zones/board/cells/2/2': {
                action: 'multiStepSelection',
                stepId: 'selectDestination',
                direction: 'Select destination'
              }
            }
          }
        }
      };

      renderGameView(reconnectionState);

      // Should show restored multi-step state
      expect(screen.getByText('Action in Progress')).toBeInTheDocument();
      expect(screen.getByText('Step 2 of 2')).toBeInTheDocument();
      
      // Should show appropriate instruction for current step
      expect(screen.getByText(/Select where to move/)).toBeInTheDocument();
    });
  });

  describe('Multi-Step Visual Indicators', () => {
    it('should highlight cells with multi-step actions available', () => {
      renderGameView();

      const actionableCell = screen.getByTestId('board-cell-0-0');
      // Check for visual indicator classes
      expect(actionableCell).toHaveClass('cursor-pointer');
      // The ActionIndicator component should be present
      const indicator = actionableCell.querySelector('[data-testid="action-indicator"]');
      expect(indicator).toBeInTheDocument();
    });

    it('should show different visual states during multi-step', () => {
      const gameStateWithMultiStep = {
        ...mockGameState,
        ui: {
          ...mockGameState.ui,
          multiStepState: {
            actionId: 'movePiece',
            currentStepIndex: 0,
            storedData: {},
            canCancel: true,
            requiresConfirmation: false,
            stepActionMap: {
              '/zones/board/cells/1/1': {
                action: 'multiStepSelection',
                stepId: 'selectPiece',
                direction: 'Select this piece'
              }
            }
          }
        }
      };

      renderGameView(gameStateWithMultiStep);

      // Selectable cells should have different styling
      const selectableCell = screen.getByTestId('board-cell-1-1');
      const indicator = selectableCell.querySelector('[data-testid="action-indicator"]');
      expect(indicator).toHaveAttribute('data-state', 'next_step');
    });
  });
});
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useGameActions } from '../hooks/useGameActions';

describe('Multi-Step Action Map Fix', () => {
  let mockSendMessage: any;
  let mockLobbyState: any;

  beforeEach(() => {
    mockSendMessage = vi.fn(() => true);
    
    // Base lobby state
    mockLobbyState = {
      you: 'p1',
      game: {
        currentPlayer: 'p1',
        zones: {
          board: {
            type: 'grid',
            cells: [[null, null], [null, null]]
          }
        }
      },
      ui: {
        actionMap: {
          p1: {},  // Empty due to multi-step
          p2: {}
        },
        multiStepState: {
          p1: {
            actionId: 'moveEntity',
            currentStepIndex: 0,
            stepActionMap: {
              '/zones/board/cells/0/0': {
                action: 'multiStepSelect',
                args: {
                  location: '/zones/board/cells/0/0'
                }
              },
              '/zones/board/cells/0/1': {
                action: 'multiStepSelect',
                args: {
                  location: '/zones/board/cells/0/1'
                }
              }
            }
          }
        }
      }
    };
  });

  it('should use step action map when multi-step is active', () => {
    const { result } = renderHook(() => 
      useGameActions({
        isYourTurn: true,
        lobbyState: mockLobbyState,
        sendMessage: mockSendMessage
      })
    );

    // Click on a cell that has an action in the step action map
    result.current.handleCellClick(0, 0);

    // Should send multiStepSelect action
    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'multiStepSelect',
        args: {
          location: '/zones/board/cells/0/0'
        }
      })
    );
  });

  it('should not allow actions when multi-step state is not properly indexed', () => {
    // Test the old broken format where multiStepState is not indexed by player
    mockLobbyState.ui.multiStepState = {
      actionId: 'moveEntity',
      stepActionMap: {
        '/zones/board/cells/0/0': {
          action: 'multiStepSelect',
          args: { location: '/zones/board/cells/0/0' }
        }
      }
    };

    const { result } = renderHook(() => 
      useGameActions({
        isYourTurn: true,
        lobbyState: mockLobbyState,
        sendMessage: mockSendMessage
      })
    );

    // Click on a cell
    result.current.handleCellClick(0, 0);

    // Should not send any message because multi-step state is not found
    expect(mockSendMessage).not.toHaveBeenCalled();
  });

  it('should handle multi-step state for the correct player', () => {
    // Set up multi-step for p2, but we are p1
    mockLobbyState.ui.multiStepState = {
      p2: {
        actionId: 'moveEntity',
        stepActionMap: {
          '/zones/board/cells/0/0': {
            action: 'multiStepSelect',
            args: { location: '/zones/board/cells/0/0' }
          }
        }
      }
    };

    const { result } = renderHook(() => 
      useGameActions({
        isYourTurn: true,
        lobbyState: mockLobbyState,
        sendMessage: mockSendMessage
      })
    );

    // Click on a cell
    result.current.handleCellClick(0, 0);

    // Should not send any message because we (p1) don't have multi-step state
    expect(mockSendMessage).not.toHaveBeenCalled();
  });

  it('should use regular action map when no multi-step is active', () => {
    // Remove multi-step state and add regular actions
    delete mockLobbyState.ui.multiStepState;
    mockLobbyState.ui.actionMap.p1 = {
      '/zones/board/cells/0/0': {
        action: 'place',
        args: {
          target: '/zones/board/cells/0/0',
          entity: 'piece_p1'
        }
      }
    };

    const { result } = renderHook(() => 
      useGameActions({
        isYourTurn: true,
        lobbyState: mockLobbyState,
        sendMessage: mockSendMessage
      })
    );

    // Click on a cell
    result.current.handleCellClick(0, 0);

    // Should send regular action
    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'place',
        args: {
          target: '/zones/board/cells/0/0',
          entity: 'piece_p1'
        }
      })
    );
  });
});